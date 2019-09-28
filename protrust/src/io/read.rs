//! Contains the `CodedReader`, a reader for reading values from a protobuf encoded byte stream

use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::string::FromUtf8Error;
use alloc::vec::Vec;
use core::cmp;
use core::fmt::{self, Display, Formatter};
use crate::collections;
use crate::io::{Tag, Length, WireType, ByteString, stream::{self, Read}};
use crate::raw;
use either::{Either, Left, Right};
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error;

/// The error type for [`CodedReader`](struct.CodedReader.html)
#[derive(Debug)]
pub enum Error {
    /// The input contained a malformed variable length integer
    MalformedVarint,
    /// The input contained a length delimited value which reported it had a negative size
    NegativeSize,
    /// The input contained an invalid tag (zero or the tag had an invalid wire format)
    InvalidTag(u32),
    /// An error occured while reading from the underlying `Read` object
    StreamError(stream::Error),
    /// The input contained an invalid UTF8 string
    InvalidString(FromUtf8Error),
}

impl From<stream::Error> for Error {
    fn from(value: stream::Error) -> Error {
        Error::StreamError(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Error {
        Error::InvalidString(value)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Error::MalformedVarint => write!(fmt, "the input contained an invalid variable length integer"),
            Error::NegativeSize => write!(fmt, "the input contained a length delimited value which reported it had a negative size"),
            Error::InvalidTag(val) => write!(fmt, "the input contained an tag that was either invalid or was unexpected at this point in the input: {}", val),
            Error::StreamError(_) => write!(fmt, "an error occured in the underlying input"),
            Error::InvalidString(_) => write!(fmt, "the input contained an invalid UTF8 string")
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::StreamError(ref e) => Some(e),
            Error::InvalidString(ref e) => Some(e),
            _ => None,
        }
    }
}

/// A result for a [`CodedReader`](struct.CodedReader.html) read operation
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug)]
struct ReaderOptions {
    skip_unknown_fields: bool,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        ReaderOptions {
            skip_unknown_fields: false
        }
    }
}

/// A builder used to construct [`CodedReader`](struct.CodedReader.html) instances
#[derive(Clone, Debug, Default)]
pub struct Builder {
    options: ReaderOptions
}

impl Builder {
    /// Creates a new builder with the default configuration
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets whether unknown field sets should skip unknown fields
    #[inline]
    pub fn skip_unknown_fields(mut self, value: bool) -> Self {
        self.options.skip_unknown_fields = value;
        self
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and 
    /// the specified slice of bytes
    /// 
    /// # Examples
    /// 
    /// ```
    /// use protrust::io::{CodedReader, ReaderBuilder};
    /// 
    /// let data = [8, 15];
    /// let mut reader =
    ///     ReaderBuilder::new()
    ///         .skip_unknown_fields(true)
    ///         .with_slice(&data);
    /// ```
    #[inline]
    pub fn with_slice<'a>(&self, inner: &'a [u8]) -> CodedReader<'a> {
        CodedReader {
            inner: Right(Cursor::new(inner)),
            limit: None,
            last_tag: None,
            options: self.options.clone()
        }
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and 
    /// the specified [`Read`](stream/trait.Read.html) object with the default buffer capacity
    /// 
    /// # Examples
    /// 
    /// ```
    /// use protrust::io::{CodedReader, ReaderBuilder};
    /// 
    /// let data = [8, 15];
    /// let mut reader =
    ///     ReaderBuilder::new()
    ///         .skip_unknown_fields(true)
    ///         .with_slice(&data);
    /// ```
    #[inline]
    pub fn with_read<'a>(&self, inner: &'a mut dyn Read) -> CodedReader<'a> {
        self.with_capacity(DEFAULT_BUF_SIZE, inner)
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and
    /// the specified [`Read`](stream/trait.Read.html) object with the specified buffer capacity
    /// 
    /// # Examples
    /// 
    /// ```
    /// use protrust::io::{CodedReader, ReaderBuilder};
    /// 
    /// let data = [8, 15];
    /// let mut reader =
    ///     ReaderBuilder::new()
    ///         .skip_unknown_fields(true)
    ///         .with_slice(&data);
    /// ```
    #[inline]
    pub fn with_capacity<'a>(&self, capacity: usize, inner: &'a mut dyn Read) -> CodedReader<'a> {
        CodedReader {
            inner: Left(ReadWithBuffer::new(inner, capacity)),
            limit: None,
            last_tag: None,
            options: self.options.clone()
        }
    }
}

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

struct ReadWithBuffer<'a> {
    inner: &'a mut dyn Read,
    buf: Box<[u8]>,
    cap: usize,
    pos: usize
}

impl<'a> ReadWithBuffer<'a> {
    #[inline]
    fn new(inner: &'a mut dyn Read, cap: usize) -> ReadWithBuffer<'a> {
        let buf = unsafe {
            let mut buf = Vec::with_capacity(cap);
            buf.set_len(cap);
            buf.into_boxed_slice()
        };
        ReadWithBuffer { inner, buf, cap: 0, pos: 0 }
    }
    fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }
    fn consume(&mut self, len: i32) {
        self.pos += len as usize;
        if self.pos >= self.cap {
            self.pos = 0;
            self.cap = 0;
        }
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let buffer = self.buffer();
        if buffer.len() >= buf.len() {
            buf.copy_from_slice(&buffer[..buf.len()]);
        } else {
            let (copyable, mut remainder) = buf.split_at_mut(buffer.len());
            copyable.copy_from_slice(buffer);
            while !remainder.is_empty() {
                let amnt = self.inner.read(remainder)?;
                if amnt == 0 {
                    return Err(stream::Error.into());
                }
                remainder = &mut remainder[amnt..];
            }
        }

        Ok(())
    }
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize
}

impl<'a> Cursor<'a> {
    #[inline]
    fn new(buf: &'a [u8]) -> Cursor<'a> {
        Cursor { buf, pos: 0 }
    }
    #[inline]
    fn get(&self) -> &[u8] {
        unsafe { self.buf.get_unchecked(self.pos..) }
    }
    #[inline]
    fn consume(&mut self, amnt: i32) {
        debug_assert!(self.pos + amnt as usize <= self.buf.len());

        self.pos += amnt as usize;
    }
}

#[inline]
fn apply_limit(buf: &[u8], limit: Option<i32>) -> &[u8] {
    if let Some(limit) = limit {
        &buf[..cmp::min(buf.len(), limit as usize)]
    } else {
        buf
    }
}

/// A coded input reader that reads from a borrowed [`BufRead`].
/// 
/// [`BufRead`]: https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html
pub struct CodedReader<'a> {
    inner: Either<ReadWithBuffer<'a>, Cursor<'a>>,
    limit: Option<i32>,
    last_tag: Option<Tag>,
    options: ReaderOptions,
}

impl<'a> CodedReader<'a> {
    /// Creates a new [`CodedReader`] in the default configuration
    ///  over the borrowed [`Read`] with the default buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
    #[inline]
    pub fn with_read(inner: &'a mut dyn Read) -> Self {
        Builder::new().with_read(inner)
    }
    /// Creates a new [`CodedReader`] in the default configuration
    /// over the borrowed [`Read`] with the specified buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: streams/trait.Read.html
    #[inline]
    pub fn with_capacity(capacity: usize, inner: &'a mut dyn Read) -> Self {
        Builder::new().with_capacity(capacity, inner)
    }
    /// Creates a new [`CodedReader`] over the borrowed [`slice`]
    /// in the default configuration. This is optimized to read directly
    /// from the slice, making it faster than reading from a [`Read`] object.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`slice`]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    /// [`Read`]: streams/trait.Read.html
    #[inline]
    pub fn with_slice(inner: &'a [u8]) -> Self {
        Builder::new().with_slice(inner)
    }

    #[inline]
    fn consume_limit(&mut self, len: i32) {
        if let Some(limit) = self.limit.as_mut() {
            *limit -= len;
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        match self.inner {
            Left(ref mut read) => {
                read.read_exact(buf)?;
                read.consume(buf.len() as i32);
            },
            Right(ref mut curs) => {
                let slice = 
                    apply_limit(curs.get(), self.limit)
                        .get(..buf.len())
                        .ok_or(Error::from(stream::Error))?;
                buf.copy_from_slice(slice);
                curs.consume(buf.len() as i32);
            }
        }
        self.consume_limit(buf.len() as i32);
        Ok(())
    }

    /// Returns if unknown field sets should skip any unknown fields when merging
    #[inline]
    pub fn skip_unknown_fields(&self) -> bool {
        self.options.skip_unknown_fields
    }

    /// Gets the last tag read from the input.
    #[inline]
    pub fn last_tag(&self) -> Option<Tag> {
        self.last_tag
    }

    /// Reads a length delimited value's length from the input.
    /// This returns [`InputError::NegativeSize`](enum.InputError.html#variant.NegativeSize) if the length is invalid.
    #[inline]
    pub fn read_length(&mut self) -> Result<Length> {
        let value = self.read_value::<raw::Int32>()?;
        Length::new(value).ok_or(Error::NegativeSize)
    }

    /// Reads a length from the input and pushes it, returning the old length to return when the input has reached it's limit.
    /// If an error occurs while reading the length, this does not push a length.
    #[inline]
    pub fn read_and_push_length(&mut self) -> Result<Option<Length>> {
        let length = self.read_length()?;
        Ok(self.push_length(length))
    }

    /// Pushes a new length to the reader, limiting the amount of data read from the input by the 
    /// specified amount.
    #[inline]
    pub fn push_length(&mut self, length: Length) -> Option<Length> {
        core::mem::replace(&mut self.limit, Some(length.get())).map(Length)
    }

    /// Returns an old length to the reader.
    /// 
    /// This should only be used after the current length has been read to completion. Using this 
    /// before doing so can cause odd behavior.
    #[inline]
    pub fn pop_length(&mut self, old: Option<Length>) {
        self.limit = old.map(Length::get)
    }

    /// Returns if the length's limit has been reached. The returned value of this is unknown if it's 
    /// called when no length has been pushed.
    #[inline]
    pub fn reached_limit(&self) -> bool {
        self.limit == Some(0)
    }

    /// Reads a tag from the input, returning none if there is no more data available in the input.
    #[inline]
    pub fn read_tag(&mut self) -> Result<Option<Tag>> {
        unimplemented!()
    }

    /// Reads a 32-bit varint from the input. This is optimized for 32-bit varint values and will discard 
    /// the top 32 bits of a 64-bit varint value.
    #[inline]
    pub fn read_varint32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 1];
        let mut value = 0u32;
        for i in 0..5 {
            self.read_exact(&mut buf)?;
            let b = buf[0] as u32;
            value |= (b & 0x7F) << (7 * i);
            if b >= 0x80 {
                return Ok(value);
            }
        }
        for _ in 0..5 {
            self.read_exact(&mut buf)?;
            let b = buf[0] as u32;
            if b >= 0x80 {
                return Ok(value);
            }
        }
        Err(Error::MalformedVarint)
    }

    /// Reads a 64-bit varint from the input.
    #[inline]
    pub fn read_varint64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 1];
        let mut value = 0u64;
        for i in 0..10 {
            self.read_exact(&mut buf)?;
            let b = buf[0] as u64;
            value |= (b & 0x7F) << (7 * i);
            if b >= 0x80 {
                return Ok(value);
            }
        }
        Err(Error::MalformedVarint)
    }

    /// Reads a 32-bit little endian value from the input
    #[inline]
    pub fn read_bit32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Reads a 64-bit little endian value from the input
    #[inline]
    pub fn read_bit64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Reads a length delimited value from the input prefixed by a length
    #[inline]
    pub fn read_length_delimited<T: ByteString>(&mut self, a: T::Alloc) -> Result<T> {
        let length = self.read_length()?.get();
        let mut value = T::new(length as usize, a);
        debug_assert!(value.as_ref().len() == length as usize);

        self.read_exact(value.as_mut())?;
        Ok(value)
    }

    /// Merges a length delimited value from the input prefixed by the length. This may reallocate
    pub fn merge_length_delimited<T: ByteString>(&mut self, value: &mut T) -> Result<()> {
        let length = self.read_length()?.get();
        value.resize(length as usize);

        debug_assert!(value.as_ref().len() == length as usize);

        self.read_exact(value.as_mut())?;
        Ok(())
    }

    #[inline]
    /// Skips the last value based on the tag read from the input. If no tag has been read, this does nothing
    pub fn skip(&mut self) -> Result<()> {
        if let Some(tag) = self.last_tag {
            match tag.wire_type() {
                WireType::Varint => { self.read_varint64()?; },
                WireType::Bit64 => { self.read_bit64()?; },
                WireType::LengthDelimited => { self.read_length_delimited::<Box<_>>(Global)?; },
                WireType::StartGroup => {
                    let end_tag = Tag::new(tag.number(), WireType::EndGroup);
                    while let Some(tag) = self.read_tag()? {
                        if tag != end_tag {
                            self.skip()?;
                        } else {
                            break;
                        }
                    }
                },
                WireType::EndGroup => { },
                WireType::Bit32 => { self.read_bit64()?; }
            }
        }
        Ok(())
    }

    /// Reads an instance of the specified value
    #[inline]
    pub fn read_value<T: raw::Primitive + Wrapper>(&mut self) -> Result<T::Inner> {
        T::read_new(self).map(T::unwrap)
    }

    /// Reads a heaping values in the specified allocator instance
    #[inline]
    pub fn read_value_in<T: raw::Heaping + Wrapper>(&mut self, a: T::Alloc) -> Result<T::Inner> {
        T::read_new(self, a).map(T::unwrap)
    }

    /// Merges an existing instance of a value with a value from the input
    #[inline]
    pub fn merge_value<T: raw::Value + Wrapper>(&mut self, value: &mut T::Inner) -> Result<()> {
        T::wrap_mut(value).merge_from(self)
    }

    /// Adds values from the input to the repeated value
    #[inline]
    pub fn add_values_to<T: raw::Primitive>(&mut self, value: &mut impl collections::RepeatedPrimitiveValue<T>) -> Result<()> {
        value.add_entries_from(self)
    }

    /// Adds values from the input to the 
    #[inline]
    pub fn add_values_to_in<T: raw::Heaping>(&mut self, value: &mut impl collections::RepeatedHeapingValue<T>, a: T::Alloc) -> Result<()> {
        value.add_entries_from(self, a)
    }

    /// Tries to add the field to the set, possibly adding the field or yielding control to another set
    #[inline]
    pub fn try_add_field_to<'b>(&'b mut self, value: &mut impl crate::FieldSet) -> Result<crate::TryRead<'b, 'a>> {
        value.try_add_field_from(self)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use crate::io::{Tag, FieldNumber, WireType, CodedReader, ReaderError};

    #[test]
    fn varint32_decode() {
        fn try_decode(bytes: &[u8], expected: u32) {
            let mut reader = CodedReader::with_slice(bytes);
            let value = reader.read_varint32().unwrap();

            assert_eq!(expected, value);

            let mut bytes = bytes;
            let mut reader = CodedReader::with_read(&mut bytes);
            let value = reader.read_varint32().unwrap();

            assert_eq!(expected, value);
        }

        try_decode(&[0x80], 0);
        try_decode(&[0xFF], 127);
        try_decode(&[0x7F, 0xFF], 16_383);
        try_decode(&[0x7F, 0x7F, 0xFF], 2_097_151);
        try_decode(&[0x7F, 0x7F, 0x7F, 0xFF], 268_435_455);
        try_decode(&[0x7F, 0x7F, 0x7F, 0x7F, 0x8F], u32::max_value());
        try_decode(&[0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x81], u32::max_value()); // test that we discard the top 32 bits
    }
    #[test]
    fn varint64_decode() {
        fn try_decode(bytes: &[u8], expected: u64) {
            let mut reader = CodedReader::with_slice(bytes);
            let value = reader.read_varint64().unwrap();

            assert_eq!(expected, value);

            let mut bytes = bytes;
            let mut reader = CodedReader::with_read(&mut bytes);
            let value = reader.read_varint64().unwrap();

            assert_eq!(expected, value);
        }

        try_decode(&[0x80], 0);
        try_decode(&[0xFF], 127);
        try_decode(&[0x7F, 0xFF], 16_383);
        try_decode(&[0x7F, 0x7F, 0xFF], 2_097_151);
        try_decode(&[0x7F, 0x7F, 0x7F, 0xFF], 268_435_455);
        try_decode(&[0x7F, 0x7F, 0x7F, 0x7F, 0x8F], u32::max_value() as u64);
        try_decode(&[0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x81], u64::max_value());
    }
    #[test]
    fn malformed_varint() {
        let data = [0u8; 11];
        let mut reader = CodedReader::with_slice(&data);

        assert_matches!(reader.read_varint32(), Err(ReaderError::MalformedVarint));

        let mut read = data.as_ref();
        let mut reader = CodedReader::with_read(&mut read);

        assert_matches!(reader.read_varint32(), Err(ReaderError::MalformedVarint));

        let mut reader = CodedReader::with_slice(&data);

        assert_matches!(reader.read_varint64(), Err(ReaderError::MalformedVarint));

        let mut read = data.as_ref();
        let mut reader = CodedReader::with_read(&mut read);

        assert_matches!(reader.read_varint64(), Err(ReaderError::MalformedVarint));
    }
    #[test]
    fn tag_decode() {
        // decoding a tag should read it and set the last tag
        let expected_tag = Tag::new(FieldNumber::new(1).unwrap(), WireType::Varint);
        let data = [136];
        let mut reader = CodedReader::with_slice(&data);

        assert_matches!(reader.read_tag(), Ok(Some(tag)) if tag == expected_tag);
        assert_eq!(reader.last_tag(), Some(expected_tag));

        let mut bytes = data.as_ref();
        let mut reader = CodedReader::with_read(&mut bytes);

        assert_matches!(reader.read_tag(), Ok(Some(tag)) if tag == expected_tag);
        assert_eq!(reader.last_tag(), Some(expected_tag));
    }
    #[test]
    fn fail_tag_decode() {
        // decoding an invalid tag should return the InvalidTag error
        let data = [128]; // a zero tag
        let mut reader = CodedReader::with_slice(&data);

        assert_matches!(reader.read_tag(), Err(ReaderError::InvalidTag(0)));
        assert_eq!(reader.last_tag(), None);

        let mut bytes = data.as_ref();
        let mut reader = CodedReader::with_read(&mut bytes);

        assert_matches!(reader.read_tag(), Err(ReaderError::InvalidTag(0)));
        assert_eq!(reader.last_tag(), None);
    }
    #[test]
    fn none_tag_marks_eof() {
        let data = [];
        let mut reader = CodedReader::with_slice(&data);

        assert_matches!(reader.read_tag(), Ok(None));
        assert_eq!(reader.last_tag(), None);

        let mut bytes = data.as_ref();
        let mut reader = CodedReader::with_read(&mut bytes);

        assert_matches!(reader.read_tag(), Ok(None));
        assert_eq!(reader.last_tag(), None);
    }
    #[test]
    fn push_pop_length_stack() {
        fn check(reader: &mut CodedReader) {
            let len = reader.read_length().unwrap();
            assert_eq!(len.get(), 0);

            let old = reader.push_length(len);
            assert!(reader.reached_limit());

            reader.pop_length(old);
        }

        let data = [128u8];
        check(&mut CodedReader::with_slice(&data));

        let mut read = data.as_ref();
        check(&mut CodedReader::with_read(&mut read));
    }
    #[test]
    fn nested_lengths() {
        fn check(reader: &mut CodedReader) {
            let len = reader.read_length().unwrap();
            assert_eq!(len.get(), 1);

            let old = reader.push_length(len);
            assert!(!reader.reached_limit());

            let nested_len = reader.read_length().unwrap();
            assert_eq!(nested_len.get(), 0);

            let nested_old = reader.push_length(nested_len);
            assert!(reader.reached_limit());

            reader.pop_length(nested_old);

            assert!(reader.reached_limit());

            reader.pop_length(old);
        }

        let data = [129u8, 128];
        check(&mut CodedReader::with_slice(&data));

        let mut read = data.as_ref();
        check(&mut CodedReader::with_read(&mut read));
    }
    #[test]
    fn decode_bit32() {
        let bytes = [123, 0, 0, 0];
        let mut reader = CodedReader::with_slice(&bytes);

        assert_eq!(reader.read_bit32().unwrap(), 123);

        let mut read = bytes.as_ref();
        let mut reader = CodedReader::with_read(&mut read);

        assert_eq!(reader.read_bit32().unwrap(), 123);
    }
    #[test]
    fn decode_bit64() {
        let bytes = [123, 0, 0, 0, 0, 0, 0, 0];
        let mut reader = CodedReader::with_slice(&bytes);

        assert_eq!(reader.read_bit64().unwrap(), 123);

        let mut read = bytes.as_ref();
        let mut reader = CodedReader::with_read(&mut read);

        assert_eq!(reader.read_bit64().unwrap(), 123);
    }
}