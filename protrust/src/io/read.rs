//! Defines the `CodedReader`, a reader for reading values from a protobuf encoded byte stream.

use alloc::string::FromUtf8Error;
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::mem::ManuallyDrop;
use core::num::{NonZeroU32, NonZeroUsize};
use core::ops;
use core::ptr::NonNull;
use core::result;
use crate::collections::{RepeatedValue, FieldSet, TryRead};
use crate::internal::Sealed;
use crate::io::{Tag, WireType, Length, ByteString, stream::{self, Read}};
use crate::raw::{self, Value};
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error;

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

mod internal {
    use alloc::boxed::Box;
    use core::marker::PhantomData;
    use core::num::NonZeroU32;
    use core::ptr::NonNull;
    use core::result;
    use core::slice;
    use crate::io::{Tag, Length, ByteString, stream::{self, Read}, read::{Result, Any}};

    pub trait Array: AsRef<[u8]> {
        const LENGTH: usize;
    }

    macro_rules! fva {
        ($($len:literal),*) => {
            $(
                impl Array for [u8; $len] {
                    const LENGTH: usize = $len;
                }
            )*
        };
    }

    fva!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

    pub trait Reader {
        fn push_limit(&mut self, limit: Length) -> result::Result<Option<Length>, stream::Error>;
        fn pop_limit(&mut self, old: Option<Length>);
        fn reached_limit(&self) -> bool;

        fn read_tag(&mut self) -> Result<Option<u32>>;
        fn read_varint32(&mut self) -> Result<u32>;
        fn read_varint64(&mut self) -> Result<u64>;
        fn read_bit32(&mut self) -> Result<u32>;
        fn read_bit64(&mut self) -> Result<u64>;
        fn read_length_delimited<B: ByteString>(&mut self) -> Result<B>;

        fn skip_varint(&mut self) -> Result<()>;
        fn skip_bit32(&mut self) -> Result<()>;
        fn skip_bit64(&mut self) -> Result<()>;
        fn skip_length_delimited(&mut self) -> Result<()>;

        fn into_any<'a>(&'a mut self) -> Any<'a>;
        fn from_any<'a>(&'a mut self, any: Any<'a>);
    }

    pub struct FlatBuffer<'a> {
        pub a: PhantomData<&'a [u8]>,
        pub start: *const u8,
        pub limit: *const u8,
        pub end: *const u8
    }

    impl<'a> FlatBuffer<'a> {
        pub fn new(value: &'a [u8]) -> Self {
            let end = unsafe { value.as_ptr().add(value.len()) };
            FlatBuffer {
                a: PhantomData,
                start: value.as_ptr(),
                end,
                limit: end
            }
        }
        pub fn is_empty_limited(&self) -> bool {
            self.start >= self.limit
        }
        pub fn len_limited(&self) -> usize {
            usize::wrapping_sub(self.limit as _, self.start as _)
        }
        pub fn len(&self) -> usize {
            usize::wrapping_sub(self.end as _, self.start as _)
        }
        pub fn try_limited_as_array<A: Array>(&self) -> Option<&'a A> {
            if self.len_limited() >= A::LENGTH {
                unsafe { Some(&*(self.start as *const A)) }
            } else {
                None
            }
        }
        pub fn try_as_array<A: Array>(&self) -> Option<&'a A> {
            if self.len() >= A::LENGTH {
                unsafe { Some(&*(self.start as *const A)) }
            } else {
                None
            }
        }
        pub unsafe fn copy_limited_into(&self, slice: &mut [u8]) {
            debug_assert!(self.len_limited() >= slice.len());

            core::ptr::copy_nonoverlapping(self.start, slice.as_mut_ptr(), slice.len());
        }
        pub unsafe fn advance(&mut self, amnt: usize) {
            let new_pos = self.start.add(amnt);

            debug_assert!(new_pos < self.limit, "advanced past end of limit");
            debug_assert!(new_pos < self.end, "advanced past end of buffer");

            self.start = new_pos;
        }
        pub fn as_limited_slice(&self) -> &'a [u8] {
            unsafe { slice::from_raw_parts(self.start, self.len_limited()) }
        }
    }

    impl Reader for FlatBuffer<'_> {
        fn push_limit(&mut self, limit: Length) -> result::Result<Option<Length>, stream::Error> {
            unimplemented!()
        }
        fn pop_limit(&mut self, old: Option<Length>) {
            unimplemented!()
        }
        fn reached_limit(&self) -> bool {
            unimplemented!()
        }

        fn read_tag(&mut self) -> Result<Option<u32>> {
            unimplemented!()
        }
        fn read_varint32(&mut self) -> Result<u32> {
            unimplemented!()
        }
        fn read_varint64(&mut self) -> Result<u64> {
            unimplemented!()
        }
        fn read_bit32(&mut self) -> Result<u32> {
            unimplemented!()
        }
        fn read_bit64(&mut self) -> Result<u64> {
            unimplemented!()
        }
        fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
            unimplemented!()
        }

        fn skip_varint(&mut self) -> Result<()> {
            unimplemented!()
        }
        fn skip_bit32(&mut self) -> Result<()> {
            unimplemented!()
        }
        fn skip_bit64(&mut self) -> Result<()> {
            unimplemented!()
        }
        fn skip_length_delimited(&mut self) -> Result<()> {
            unimplemented!()
        }

        fn into_any<'a>(&'a mut self) -> Any<'a> {
            unimplemented!()
        }
        fn from_any<'a>(&'a mut self, any: Any<'a>) {
            unimplemented!()
        }
    }

    pub struct StreamBuffer<T> {
        input: T,
        buf: Box<[u8]>,
        start: Option<NonNull<u8>>,
        limit: Option<NonNull<u8>>,
    }

    impl<T> StreamBuffer<T> {
        pub fn new(input: T, cap: usize) -> Self {
            unimplemented!()
        }
    }

    impl<T: Read> Reader for StreamBuffer<T> {
        fn push_limit(&mut self, limit: Length) -> result::Result<Option<Length>, stream::Error> {
            unimplemented!()
        }
        fn pop_limit(&mut self, old: Option<Length>) {
            unimplemented!()
        }
        fn reached_limit(&self) -> bool {
            unimplemented!()
        }

        fn read_tag(&mut self) -> Result<Option<u32>> {
            unimplemented!()
        }
        fn read_varint32(&mut self) -> Result<u32> {
            unimplemented!()
        }
        fn read_varint64(&mut self) -> Result<u64> {
            unimplemented!()
        }
        fn read_bit32(&mut self) -> Result<u32> {
            unimplemented!()
        }
        fn read_bit64(&mut self) -> Result<u64> {
            unimplemented!()
        }
        fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
            unimplemented!()
        }

        fn skip_varint(&mut self) -> Result<()> {
            unimplemented!()
        }
        fn skip_bit32(&mut self) -> Result<()> {
            unimplemented!()
        }
        fn skip_bit64(&mut self) -> Result<()> {
            unimplemented!()
        }
        fn skip_length_delimited(&mut self) -> Result<()> {
            unimplemented!()
        }

        fn into_any<'a>(&'a mut self) -> Any<'a> {
            unimplemented!()
        }
        fn from_any<'a>(&'a mut self, any: Any<'a>) {
            unimplemented!()
        }
    }

    #[cfg(test)]
    mod test {
        use super::{FlatBuffer, StreamBuffer};

        #[test]
        fn flat_buffer() {
            let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            let mut buf = FlatBuffer::new(&data);
        }
    }
}

use internal::Reader;

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
pub type Result<T> = result::Result<T, Error>;

/// An input type that can be used to create a `Reader` for a [`CodedReader`] instance.
/// 
/// [`CodedReader`]: struct.CodedReader.html
pub trait Input: Sealed {
    /// The reader type used by the [`CodedReader`](struct.CodedReader.html) to read data
    type Reader: internal::Reader;
}

/// A type used for a [`CodedReader`] reading from a [`slice`] input.
pub struct Slice<'a>(&'a [u8]);
impl Sealed for Slice<'_> { }
impl<'a> Input for Slice<'a> {
    type Reader = internal::FlatBuffer<'a>;
}

/// A type used for a [`CodedReader`] reading from a [`Read`] input. This input type buffers the stream's data.
pub struct Stream<T>(T);
impl<T> Sealed for Stream<T> { }
impl<T: Read> Input for Stream<T> {
    type Reader = internal::StreamBuffer<T>;
}

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
    ///         .with_buffer(&data);
    /// ```
    #[inline]
    pub fn with_slice<'a>(&self, inner: &'a [u8]) -> CodedReader<Slice<'a>> {
        CodedReader {
            inner: internal::FlatBuffer::new(inner),
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
    pub fn with_stream<T: Read>(&self, inner: T) -> CodedReader<Stream<T>> {
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
    pub fn with_capacity<T: Read>(&self, capacity: usize, inner: T) -> CodedReader<Stream<T>> {
        CodedReader {
            inner: internal::StreamBuffer::new(inner, capacity),
            last_tag: None,
            options: self.options.clone()
        }
    }
}

/// Represents any input type for a CodedReader. This is slower than a
/// generic stream input or slice, but is more flexible and can be used 
/// in cases where the input or message type is unknown.
pub struct Any<'a> {
    input: Option<&'a mut dyn Read>,
    buf: Option<&'a mut [u8]>,
    /// Values < 0 indicate no limit
    remaining_limit: isize,

    start: NonNull<u8>,
    /// With no limit, this is equal to end
    limit: NonNull<u8>,
    end: NonNull<u8>,
}

impl Any<'_> {
    fn reached_end(&self) -> bool {
        self.start > self.end
    }
    fn reached_limit(&self) -> bool {
        self.remaining_limit >= 0 && self.start > self.limit
    }
    #[inline]
    fn next_byte(&self) -> Option<u8> {
        if !self.reached_limit() {
            unsafe { Some(*self.start.as_ptr()) }
        } else {
            None
        }
    }
    #[inline]
    fn try_read_byte(&mut self) -> Option<u8> {
        if !self.reached_limit() {
            let result = unsafe { Some(*self.start.as_ptr()) };
            self.advance(1);
            result
        } else {
            None
        }
    }
    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        if !self.reached_limit() {
            let result = unsafe { Ok(*self.start.as_ptr()) };
            self.advance(1);
            result
        } else {
            unimplemented!()
        }
    }
    fn is_stream(&self) -> bool {
        self.input.is_some()
    }
    fn len(&self) -> usize {
        usize::wrapping_sub(self.limit.as_ptr() as _, self.start.as_ptr() as _)
    }
    fn refresh(&mut self) -> Result<Option<NonZeroUsize>> {
        match (&mut self.input, &mut self.buf) {
            (Some(input), Some(buf)) => 
                input.read(buf)
                    .map(NonZeroUsize::new)
                    .map_err(Into::into),
            _ => Ok(None)
        }
    }

    fn advance(&mut self, amnt: usize) {
        unsafe {
            NonNull::new_unchecked(self.start.as_ptr().add(amnt));
        }
    }
}

impl Sealed for Any<'_> { }
impl Input for Any<'_> {
    type Reader = Self;
}
impl internal::Reader for Any<'_> {
    fn push_limit(&mut self, limit: Length) -> result::Result<Option<Length>, stream::Error> {
        unimplemented!()
    }
    fn pop_limit(&mut self, old: Option<Length>) {
        unimplemented!()
    }
    fn reached_limit(&self) -> bool {
        unimplemented!()
    }

    #[inline]
    fn read_tag(&mut self) -> Result<Option<u32>> {
        if self.reached_limit() {
            Ok(None)
        } else
        if !self.reached_limit() && self.reached_end() {
            match self.refresh()? {
                None => Ok(None),
                _ => self.read_varint32().map(Some)
            }
        } else {
            self.read_varint32().map(Some)
        }
    }
    fn read_varint32(&mut self) -> Result<u32> {
        unimplemented!()
    }
    fn read_varint64(&mut self) -> Result<u64> {
        unimplemented!()
    }
    fn read_bit32(&mut self) -> Result<u32> {
        unimplemented!()
    }
    fn read_bit64(&mut self) -> Result<u64> {
        unimplemented!()
    }
    fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        unimplemented!()
    }

    fn skip_varint(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn skip_bit32(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn skip_bit64(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn skip_length_delimited(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn into_any<'a>(&'a mut self) -> Any<'a> {
        let reborrowed_input = self.input.as_mut().map::<&'a mut dyn Read, _>(|v| &mut **v);
        let reborrowed_buf = self.buf.as_mut().map::<&'a mut [u8], _>(|v| &mut **v);

        Any {
            input: reborrowed_input,
            buf: reborrowed_buf,
            remaining_limit: self.remaining_limit,
            start: self.start,
            limit: self.limit,
            end: self.end
        }
    }
    fn from_any<'a>(&'a mut self, any: Any<'a>) {
        self.remaining_limit = any.remaining_limit;
        self.start = any.start;
        self.limit = any.limit;
        self.end = any.end;
    }
}

unsafe impl Send for Any<'_> { }
unsafe impl Sync for Any<'_> { }

/// Provides a bridge for a generic [`CodedReader`] to be converted
/// to a [`CodedReader`]`<`[`Any`]`>` and vice versa.
/// 
/// This allows certain code to bridge gaps where not all merge functions
/// can be generic over an input like extension or reflection contexts.
/// 
/// [`CodedReader`]: struct.CodedReader.html
/// [`Any`]: struct.Any.html
pub struct AnyConverter<'a, T: Input + 'a> {
    src: NonNull<CodedReader<T>>,
    brdg: ManuallyDrop<CodedReader<Any<'a>>>
}

impl<'a, T: Input> AnyConverter<'a, T> {
    fn new(src: &'a mut CodedReader<T>) -> Self {
        let src_ptr = unsafe { NonNull::new_unchecked(src) }; // don't use from since the borrow moves into the from call
        let brdg = 
            CodedReader {
                inner: src.inner.into_any(),
                last_tag: src.last_tag,
                options: src.options.clone()
            };
        Self {
            src: src_ptr,
            brdg: ManuallyDrop::new(brdg)
        }
    }
}

impl<'a, T: Input> ops::Deref for AnyConverter<'a, T> {
    type Target = CodedReader<Any<'a>>;

    fn deref(&self) -> &CodedReader<Any<'a>> {
        &self.brdg
    }
}

impl<'a, T: Input> ops::DerefMut for AnyConverter<'a, T> {
    fn deref_mut(&mut self) -> &mut CodedReader<Any<'a>> {
        &mut self.brdg
    }
}

impl<'a, T: Input> Drop for AnyConverter<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let src: &'a mut CodedReader<T> = &mut *self.src.as_ptr();

            src.last_tag = self.brdg.last_tag;
            src.inner.from_any(ManuallyDrop::take(&mut self.brdg).inner);
        }
    }
}

/// A reader used by generated code to quickly parse field values without tag
/// wire type and field number checking.
/// 
/// This structure defers tag checking, making it faster to read fields when matching
/// on an existing field tag value.
pub struct FieldReader<'a, T: Input + 'a> {
    inner: &'a mut CodedReader<T>,
    tag: u32,
}

impl<'a, T: Input + 'a> FieldReader<'a, T> {
    #[inline]
    pub fn tag(&self) -> u32 {
        self.tag
    }
    #[inline]
    pub fn read_value<F: FnOnce(&'a mut CodedReader<T>) -> Result<()>>(self, tag: Tag, f: F) -> Result<()> {
        debug_assert_eq!(self.tag, tag.get(), "Provided tag does not match read tag value");
        self.inner.last_tag = Some(tag);

        f(self.inner)
    }

    #[inline]
    pub fn check_and_read_value<F: FnOnce(&'a mut CodedReader<T>) -> Result<()>>(self, f: F) -> Result<()> {
        let tag = Tag::try_from(self.tag).map_err(|_| Error::InvalidTag(self.tag))?;
        self.inner.last_tag = Some(tag);

        f(self.inner)
    }
}

/// A coded input reader that reads from a specified input.
pub struct CodedReader<T: Input> {
    inner: T::Reader,
    last_tag: Option<Tag>,
    options: ReaderOptions,
}

impl<T: Read> CodedReader<Stream<T>> {
    /// Creates a new [`CodedReader`] in the default configuration
    /// over the specified [`Read`] with the default buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
    pub fn with_stream(inner: T) -> Self {
        Builder::new().with_stream(inner)
    }
    /// Creates a new [`CodedReader`] in the default configuration
    /// over the specified [`Read`] with the specified buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: streams/trait.Read.html
    pub fn with_capacity(capacity: usize, inner: T) -> Self {
        Builder::new().with_capacity(capacity, inner)
    }
}

impl<'a> CodedReader<Slice<'a>> {
    /// Creates a new [`CodedReader`] over the borrowed [`slice`]
    /// in the default configuration. This is optimized to read directly
    /// from the slice, making it faster than reading from a [`Read`] object.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`slice`]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    /// [`Read`]: streams/trait.Read.html
    pub fn with_slice(inner: &'a [u8]) -> Self {
        Builder::new().with_slice(inner)
    }
}

impl<T: Input> CodedReader<T> {
    pub fn skip_unknown_fields(&self) -> bool {
        self.options.skip_unknown_fields
    }
    pub fn last_tag(&self) -> Option<Tag> {
        self.last_tag
    }
    pub fn as_any<'a>(&'a mut self) -> AnyConverter<'a, T> {
        AnyConverter::new(self)
    }

    /// Reads a length value from the input.
    /// 
    /// # Errors
    /// 
    /// If a negative length is read, this returns a `NegativeSize` error.
    pub fn read_limit(&mut self) -> Result<Length> {
        self.read_value::<raw::Int32>().and_then(|i| Length::new(i).ok_or(Error::NegativeSize))
    }
    /// Pushes a new limit to the reader.
    /// 
    /// The previous limit returned by this function must be returned back to the input 
    /// via [`pop_limit`](#method.pop_limit). Failure to do so may pre-emptively end the stream.
    /// 
    /// # Errors
    /// 
    /// Certain inputs will perform a check and return a [`stream::Error`](../stream/struct.Error.html)
    /// if a limit extends beyond the end of the input.
    pub fn push_limit(&mut self, limit: Length) -> result::Result<Option<Length>, stream::Error> {
        self.inner.push_limit(limit)
    }
    /// Pops the last limit off the stack for the reader. The consumer must return the previous limit returned by [`push_limit`](#method.push_limit).
    /// 
    /// # Safety
    /// 
    /// This must be the limit previously last returned by [`push_limit`](#method.push_limit). Any other values are undefined behavior.
    pub unsafe fn pop_limit(&mut self, old_limit: Option<Length>) {
        self.inner.pop_limit(old_limit)
    }
    /// Returns whether this coded reader has reached the current length limit
    pub fn reached_limit(&self) -> bool {
        self.inner.reached_limit()
    }

    /// Reads a field tag from the input
    pub fn read_tag(&mut self) -> Result<Option<Tag>> {
        self.last_tag = 
            self.inner.read_tag()?
                .map(|v| Tag::try_from(v).map_err(|_| Error::InvalidTag(v)))
                .transpose()?;
        Ok(self.last_tag)
    }
    /// Reads a 32-bit varint field value. This is funactionally similar to [`read_varint64`](#method.read_varint64),
    /// but is optimised for 32-bit values and will discard any top bits from 64-bit values.
    pub fn read_varint32(&mut self) -> Result<u32> {
        self.inner.read_varint32()
    }
    /// Reads a 64-bit varint field value.
    pub fn read_varint64(&mut self) -> Result<u64> {
        self.inner.read_varint64()
    }
    /// Reads a 4-byte little endian value
    pub fn read_bit32(&mut self) -> Result<u32> {
        self.inner.read_bit32()
    }
    /// Reads a 8-byte little endian value
    pub fn read_bit64(&mut self) -> Result<u64> {
        self.inner.read_bit64()
    }
    /// Reads a length delimited string of bytes.
    pub fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        self.inner.read_length_delimited()
    }
    /// Skips the last field read from the input
    pub fn skip(&mut self) -> Result<()> {
        if let Some(last_tag) = self.last_tag() {
            match last_tag.wire_type() {
                WireType::Varint => self.inner.skip_varint()?,
                WireType::Bit64 => self.inner.skip_bit64()?,
                WireType::LengthDelimited => self.inner.skip_length_delimited()?,
                WireType::StartGroup => {
                    let end = Tag::new(last_tag.number(), WireType::EndGroup);
                    loop {
                        match self.read_tag()? {
                            Some(tag) if tag == end => break,
                            Some(_) => self.skip()?,
                            None => return Err(Error::StreamError(stream::Error))
                        }
                    }
                },
                WireType::EndGroup => { },
                WireType::Bit32 => self.inner.skip_bit32()?,
            }
        }

        Ok(())
    }

    #[inline]
    pub fn read_field<'a>(&'a mut self) -> Result<Option<FieldReader<'a, T>>> {
        self.inner.read_tag().map(move |t| t.map(move |t| FieldReader { inner: self, tag: t }))
    }
    #[inline]
    pub fn read_value<V: Value + Wrapper>(&mut self) -> Result<V::Inner> {
        V::read_new(self).map(V::unwrap)
    }
    #[inline]
    pub fn merge_value<V: Value + Wrapper>(&mut self, value: &mut V::Inner) -> Result<()> {
        V::wrap_mut(value).merge_from(self)
    }
    #[inline]
    pub fn add_entries_to<U: RepeatedValue<V> + Wrapper, V>(&mut self, value: &mut U::Inner) -> Result<()> {
        U::wrap_mut(value).add_entries_from(self)
    }
    #[inline]
    pub fn try_add_field_to<'a, U: FieldSet>(&'a mut self, value: &mut U) -> Result<TryRead<'a, T>> {
        value.try_add_field_from(self)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use crate::io::{Tag, FieldNumber, WireType, CodedReader, read::{Result, Error}};

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

        assert_matches!(reader.read_varint32(), Err(Error::MalformedVarint));

        let mut read = data.as_ref();
        let mut reader = CodedReader::with_read(&mut read);

        assert_matches!(reader.read_varint32(), Err(Error::MalformedVarint));

        let mut reader = CodedReader::with_slice(&data);

        assert_matches!(reader.read_varint64(), Err(Error::MalformedVarint));

        let mut read = data.as_ref();
        let mut reader = CodedReader::with_read(&mut read);

        assert_matches!(reader.read_varint64(), Err(Error::MalformedVarint));
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

        assert_matches!(reader.read_tag(), Err(Error::InvalidTag(0)));
        assert_eq!(reader.last_tag(), None);

        let mut bytes = data.as_ref();
        let mut reader = CodedReader::with_read(&mut bytes);

        assert_matches!(reader.read_tag(), Err(Error::InvalidTag(0)));
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