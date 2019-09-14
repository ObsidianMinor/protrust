//! Contains types and traits for reading and writing protobuf coded data.

use crate::{collections, raw};
use either::{Either, Left, Right};
use std::convert::{TryInto, TryFrom};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read, BufReader, Write};
use std::mem;
use std::num::NonZeroU32;
use std::ptr;
use std::slice;
use std::string::FromUtf8Error;
use trapper::Wrapper;

/// The wire type of a protobuf value.
///
/// A wire type is paired with a field number between 1 and 536,870,911 to create a tag,
/// a unique identifier for a field on the wire.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum WireType {
    /// A value read a variable length integer.
    ///
    /// See the protobuf docs for more information on this encoding: https://developers.google.com/protocol-buffers/docs/encoding#varints
    Varint = 0,
    /// A 64-bit value encoded as 8 little endian bytes
    Bit64 = 1,
    /// A length delimited value. The length is encoded as a varint
    LengthDelimited = 2,
    /// A start group tag, deprecated in proto3.
    StartGroup = 3,
    /// An end group tag, deprecated in proto3.
    EndGroup = 4,
    /// A 32-bit value encoded as 4 little endian bytes
    Bit32 = 5,
}

/// The error struct used when trying to convert from an byte to a wire type
#[derive(Debug)]
pub struct InvalidWireType;

impl WireType {
    /// Gets whether a wire type is eligible for repeated field packing.
    /// The valid packable wire types are Bit32, Bit64, and Varint.
    pub const fn is_packable(self) -> bool {
        (self as u8 == WireType::Varint as u8) |
        (self as u8 == WireType::Bit64 as u8) |
        (self as u8 == WireType::Bit32 as u8)
    }
}

impl TryFrom<u8> for WireType {
    type Error = InvalidWireType;

    fn try_from(value: u8) -> Result<WireType, InvalidWireType> {
        match value & 0b111 {
            0 => Ok(WireType::Varint),
            1 => Ok(WireType::Bit64),
            2 => Ok(WireType::LengthDelimited),
            3 => Ok(WireType::StartGroup),
            4 => Ok(WireType::EndGroup),
            5 => Ok(WireType::Bit32),
            _ => Err(InvalidWireType),
        }
    }
}

/// A protobuf field number. Its value is known to be less than or equal to 536870911 and not 0.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldNumber(NonZeroU32);

impl FieldNumber {
    /// The max value of a field number as a u32
    pub const MAX_VALUE: u32 = 536_870_911;

    /// The max value of a field number
    pub const MAX: FieldNumber = unsafe { FieldNumber::new_unchecked(FieldNumber::MAX_VALUE) };

    /// Create a field number without checking the value.
    ///
    /// # Safety
    ///
    /// The value must be a valid field number
    #[inline]
    pub const unsafe fn new_unchecked(n: u32) -> FieldNumber {
        FieldNumber(NonZeroU32::new_unchecked(n))
    }

    /// Creates a field number if the given value is not zero or more than 536870911
    #[inline]
    pub fn new(n: u32) -> Option<FieldNumber> {
        if n != 0 && n <= Self::MAX_VALUE {
            unsafe { Some(FieldNumber(NonZeroU32::new_unchecked(n))) }
        } else {
            None
        }
    }

    /// Returns the value as a [`u32`](https://doc.rust-lang.org/nightly/std/primitive.u32.html)
    #[inline]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl Display for FieldNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A tag containing a wire type and field number. Its value is known to not be 0, and both field number and wire type are valid values
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(NonZeroU32);

impl Tag {
    /// Create a tag without checking the value.
    ///
    /// # Safety
    ///
    /// The value must be a valid tag
    #[inline]
    pub const unsafe fn new_unchecked(n: u32) -> Tag {
        Tag(NonZeroU32::new_unchecked(n))
    }

    /// Creates a new tag if the value is not zero and has a valid field number and wire type
    ///
    /// # Examples
    ///
    /// ```
    /// use protrust::io::Tag;
    ///
    /// assert!(Tag::new_from(1).is_none());
    /// assert!(Tag::new_from(8).is_some());
    /// assert!(Tag::new_from(16).is_some());
    /// assert!(Tag::new_from(14).is_none());
    /// ```
    #[inline]
    pub fn new_from(n: u32) -> Option<Tag> {
        match (n & 0b111, n >> 3) {
            // (wire type, field number)
            (6, _) | (7, _) | (_, 0) => None,
            _ => unsafe { Some(Tag(NonZeroU32::new_unchecked(n))) },
        }
    }

    /// Creates a new tag value
    #[inline]
    pub const fn new(f: FieldNumber, wt: WireType) -> Tag {
        unsafe { Tag(NonZeroU32::new_unchecked((f.get() << 3) | wt as u32)) }
    }

    /// Gets the wire type from this tag
    /// 
    /// # Examples
    /// 
    /// ```
    /// use protrust::io::{Tag, WireType};
    /// 
    /// assert_eq!(Tag::new_from(8).unwrap().wire_type(), WireType::Varint);
    /// assert_eq!(Tag::new_from(17).unwrap().wire_type(), WireType::Bit64);
    /// ```
    #[inline]
    pub fn wire_type(self) -> WireType {
        WireType::try_from((self.get() & 0b111) as u8).expect("invalid wire type")
    }

    /// Gets the field number from this tag
    /// 
    /// # Examples
    /// 
    /// ```
    /// use protrust::io::Tag;
    /// 
    /// assert_eq!(Tag::new_from(8).unwrap().number().get(), 1);
    /// assert_eq!(Tag::new_from(17).unwrap().number().get(), 2);
    /// ```
    #[inline]
    pub fn number(self) -> FieldNumber {
        unsafe { FieldNumber::new_unchecked(self.get() >> 3) }
    }

    /// Returns the value as a [`u32`](https://doc.rust-lang.org/nightly/std/primitive.u32.html)
    #[inline]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// An opaque type that represents the length of a delimited value
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length(pub(crate) i32);

impl Display for Length {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        self.get().fmt(fmt)
    }
}

impl Length {
    /// Returns the value as a [`i32`](https://doc.rust-lang.org/nightly/std/primitive.i32.html)
    #[inline]
    pub const fn get(self) -> i32 {
        self.0
    }

    /// Calculates the length of the value
    #[inline]
    pub fn for_value<T: raw::Value + Wrapper>(value: &T::Inner) -> Option<Length> {
        T::wrap_ref(value).calculate_size(LengthBuilder::new()).map(LengthBuilder::build)
    }

    /// Calculates the length of a collection of values
    #[inline]
    pub fn for_values<T>(value: &impl collections::RepeatedValue<T>, codec: raw::TaggedValue<T>) -> Option<Length> {
        value.calculate_size(LengthBuilder::new(), codec).map(LengthBuilder::build)
    }

    /// Calculates the length of a set of fields
    #[inline]
    pub fn for_fields(value: &impl crate::FieldSet) -> Option<Length> {
        value.calculate_size(LengthBuilder::new()).map(LengthBuilder::build)
    }
}

/// An opaque type for building a length for writing to an output.
/// 
/// This exists to make creating checked lengths easier in generated code.
pub struct LengthBuilder(pub(crate) i32);

impl LengthBuilder {
    /// Creates a new length builder
    #[inline]
    pub const fn new() -> LengthBuilder {
        Self(0)
    }

    /// Adds an arbitrary number of bytes to the length
    #[inline]
    pub const fn add_bytes(self, value: i32) -> Option<Self> {
        #[cfg(feature = "checked_size")]
        return self.0.checked_add(value).map(LengthBuilder);

        #[cfg(not(feature = "checked_size"))]
        return Some(LengthBuilder(self.0 + value));
    }

    /// Adds a tag to the output
    #[inline]
    pub const fn add_tag(self, tag: Tag) -> Option<Self> {
        self.add_bytes(raw::raw_varint32_size(tag.get()).get())
    }

    /// Adds a value's size to the length, returning None if the value's size is invalid
    #[inline]
    pub fn add_value<T: raw::Value + Wrapper>(self, value: &T::Inner) -> Option<Self> {
        T::wrap_ref(value).calculate_size(self)
    }

    /// Adds a value's size to the length if it exists, otherwise does nothing
    #[inline]
    pub fn add_optional_value<T: raw::Value + Wrapper>(self, value: Option<&T::Inner>) -> Option<Self> {
        match value {
            Some(value) => self.add_value::<T>(value),
            None => Some(self)
        }
    }

    /// Adds a repeated value's size to the length
    #[inline]
    pub fn add_values<V>(self, value: &impl collections::RepeatedValue<V>, codec: raw::TaggedValue<V>) -> Option<Self> {
        value.calculate_size(self, codec)
    }

    /// Adds a set of fields to the length
    #[inline]
    pub fn add_fields(self, value: &impl crate::FieldSet) -> Option<Self> {
        value.calculate_size(self)
    }

    /// Consumes the builder, returning a [`Length`](struct.Length.html) for writing to an output
    #[inline]
    pub const fn build(self) -> Length {
        Length(self.0)
    }
}

/// The error type for [`CodedReader`](struct.CodedReader.html)
#[derive(Debug)]
pub enum ReaderError {
    /// The input contained a malformed variable length integer
    MalformedVarint,
    /// The input contained a length delimited value which reported it had a negative size
    NegativeSize,
    /// The input contained an invalid tag (zero or the tag had an invalid wire format)
    InvalidTag(u32),
    /// An error occured while reading from the underlying `Read` object
    IoError(io::Error),
    /// The input contained an invalid UTF8 string
    InvalidString(FromUtf8Error),
}

impl From<io::Error> for ReaderError {
    fn from(value: io::Error) -> ReaderError {
        ReaderError::IoError(value)
    }
}

impl From<FromUtf8Error> for ReaderError {
    fn from(value: FromUtf8Error) -> ReaderError {
        ReaderError::InvalidString(value)
    }
}

impl Display for ReaderError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            ReaderError::MalformedVarint => write!(fmt, "the input contained an invalid variable length integer"),
            ReaderError::NegativeSize => write!(fmt, "the input contained a length delimited value which reported it had a negative size"),
            ReaderError::InvalidTag(val) => write!(fmt, "the input contained an tag that was either invalid or was unexpected at this point in the input: {}", val),
            ReaderError::IoError(_) => write!(fmt, "an error occured in the underlying input"),
            ReaderError::InvalidString(_) => write!(fmt, "the input contained an invalid UTF8 string")
        }
    }
}

impl Error for ReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReaderError::IoError(ref e) => Some(e),
            ReaderError::InvalidString(ref e) => Some(e),
            _ => None,
        }
    }
}

/// A result for a [`CodedReader`](struct.CodedReader.html) read operation
pub type ReaderResult<T> = std::result::Result<T, ReaderError>;

#[derive(Copy, Clone, Debug)]
/// A set of options that can be used to modify the behavior of [`CodedReader`](struct.CodedReader.html)
pub struct ReaderOptions {
    /// Indicates if unknown field sets should skip reading fields
    pub skip_unknown_fields: bool,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        ReaderOptions {
            skip_unknown_fields: false
        }
    }
}

/// A coded input reader that reads from a borrowed [`BufRead`].
/// 
/// [`BufRead`]: https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html
pub struct CodedReader<'a> {
    inner: Either<BufReader<&'a mut dyn Read>, &'a [u8]>,
    limit: Option<i32>,
    last_tag: Option<Tag>,
    options: ReaderOptions,
}

impl<'a> CodedReader<'a> {
    /// Creates a new [`CodedReader`] over the borrowed [`Read`].
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
    #[inline]
    pub fn with_read(inner: &'a mut dyn Read) -> Self {
        Self { 
            inner: Left(BufReader::new(inner)),
            limit: None,
            last_tag: None,
            options: Default::default()
        }
    }
    /// Creates a new [`CodedReader`] over the borrowed [`Read`] with a specified inner buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
    #[inline]
    pub fn with_capacity(cap: usize, inner: &'a mut dyn Read) -> Self {
        Self { 
            inner: Left(BufReader::with_capacity(cap, inner)),
            limit: None,
            last_tag: None,
            options: Default::default()
        }
    }
    /// Creates a new [`CodedReader`] over the borrowed [`slice`].
    /// This is optimized to read directly from the slice, making it faster than reading from a [`BufRead`] object.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`slice`]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    /// [`BufRead`]: https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html
    #[inline]
    pub fn with_slice(inner: &'a [u8]) -> Self {
        Self { 
            inner: Right(inner),
            limit: None,
            last_tag: None,
            options: Default::default()
        }
    }

    /// Sets options in use by the reader
    pub fn with_options(mut self, options: ReaderOptions) -> Self {
        self.options = options;
        self
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> ReaderResult<()> {
        if let Some(limit) = self.limit {
            if buf.len() > limit as usize {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
            } else {
                self.limit = Some(limit - buf.len() as i32);
            }
        }
        match self.inner {
            Left(ref mut read) => read.read_exact(buf)?,
            Right(ref mut slice) => {
                if slice.len() < buf.len() {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
                }
                unsafe {
                    ptr::copy_nonoverlapping(slice.as_ptr(), buf.as_mut_ptr(), buf.len());
                    *slice = slice.get_unchecked(buf.len()..);
                }
            }
        }
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
    pub fn read_length(&mut self) -> ReaderResult<Length> {
        let value = self.read_value::<raw::Int32>()?;
        if value < 0 {
            Err(ReaderError::NegativeSize)
        } else {
            Ok(Length(value))
        }
    }

    /// Reads a length from the input and pushes it, returning the old length to return when the input has reached it's limit.
    /// If an error occurs while reading the length, this does not push a length.
    #[inline]
    pub fn read_and_push_length(&mut self) -> ReaderResult<Option<Length>> {
        let length = self.read_length()?;
        Ok(self.push_length(length))
    }

    /// Pushes a new length to the reader, limiting the amount of data read from the input by the 
    /// specified amount.
    #[inline]
    pub fn push_length(&mut self, length: Length) -> Option<Length> {
        std::mem::replace(&mut self.limit, Some(length.get())).map(Length)
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
    pub fn read_tag(&mut self) -> ReaderResult<Option<Tag>> {
        let tag = self.read_varint32().map(Tag::new_from)?;
        self.last_tag = tag;
        Ok(tag)
    }

    /// Reads a 32-bit varint from the input. This is optimized for 32-bit varint values and will discard 
    /// the top 32 bits of a 64-bit varint value.
    #[inline]
    pub fn read_varint32(&mut self) -> ReaderResult<u32> {
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
        Err(ReaderError::MalformedVarint)
    }

    /// Reads a 64-bit varint from the input.
    #[inline]
    pub fn read_varint64(&mut self) -> ReaderResult<u64> {
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
        Err(ReaderError::MalformedVarint)
    }

    /// Reads a 32-bit little endian value from the input
    #[inline]
    pub fn read_bit32(&mut self) -> ReaderResult<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Reads a 64-bit little endian value from the input
    #[inline]
    pub fn read_bit64(&mut self) -> ReaderResult<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Reads a length delimited value from the input prefixed by a length
    #[inline]
    pub fn read_length_delimited(&mut self) -> ReaderResult<Box<[u8]>> {
        let length = self.read_length()?;
        let mut data;
        unsafe {
            data = Box::new_uninit_slice(length.get() as usize).assume_init();
            if let Left(ref r) = self.inner {
                r.initializer().initialize(data.as_mut());
            }
        }
        self.read_exact(data.as_mut())?;
        Ok(data)
    }

    #[inline]
    /// Skips the last value based on the tag read from the input. If no tag has been read, this does nothing
    pub fn skip(&mut self) -> ReaderResult<()> {
        if let Some(tag) = self.last_tag {
            match tag.wire_type() {
                WireType::Varint => { self.read_varint64()?; },
                WireType::Bit64 => { self.read_bit64()?; },
                WireType::LengthDelimited => { self.read_length_delimited()?; },
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

    /// Reads a new instance of the value from the input
    #[inline]
    pub fn read_value<T: raw::SizedValue + Wrapper>(&mut self) -> ReaderResult<T::Inner> {
        T::read_new(self).map(T::unwrap)
    }

    /// Merges an existing instance of a value with a value from the input
    #[inline]
    pub fn merge_value<T: raw::Value + Wrapper>(&mut self, value: &mut T::Inner) -> ReaderResult<()> {
        T::wrap_mut(value).merge_from(self)
    }

    /// Adds values from the input to the repeated value
    #[inline]
    pub fn add_values_to<T>(&mut self, value: &mut impl collections::RepeatedValue<T>) -> ReaderResult<()> {
        value.add_entries_from(self)
    }

    /// Tries to add the field to the set, possibly adding the field or yielding control to another set
    #[inline]
    pub fn try_add_field_to<'b>(&'b mut self, value: &mut impl crate::FieldSet) -> ReaderResult<crate::FieldReadState<'b, 'a>> {
        value.try_add_field_from(self)
    }
}

/// The error type for [`CodedWriter`](struct.CodedWriter.html)
#[derive(Debug)]
pub enum WriterError {
    /// An error used by consumer code used to indicate a value was 
    /// provided that was too large to write to an output
    ValueTooLarge,
    /// An error occured while writing data to the output
    IoError(io::Error)
}

impl From<io::Error> for WriterError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

/// A result for a [`CodedWriter`](struct.CodedWriter.html) read operation
pub type WriterResult = std::result::Result<(), WriterError>;

/// A coded input writer that writes to a borrowed [`Write`].
/// 
/// [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html
pub struct CodedWriter<'a> {
    inner: Either<&'a mut dyn Write, &'a mut [u8]>
}

impl<'a> CodedWriter<'a> {
    /// Creates a new [`CodedWriter`] over the borrowed [`Write`].
    /// 
    /// [`CodedWriter`]: struct.CodedWriter.html
    /// [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html
    #[inline]
    pub fn with_write(inner: &'a mut dyn Write) -> Self {
        Self { inner: Left(inner) }
    }
    /// Creates a new [`CodedWriter`] over the borrowed [`slice`].
    /// 
    /// [`CodedWriter`]: struct.CodedWriter.html
    /// [`slice`]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    #[inline]
    pub fn with_slice(inner: &'a mut [u8]) -> Self {
        Self { inner: Right(inner) }
    }

    /// Writes a tag to the output.
    #[inline]
    pub fn write_tag(&mut self, tag: Tag) -> WriterResult {
        self.write_value::<raw::Uint32>(&tag.get())
    }

    /// Writes a length to the output.
    #[inline]
    pub fn write_length(&mut self, length: Length) -> WriterResult {
        self.write_value::<raw::Uint32>(&(length.get() as u32))
    }

    /// Writes a 32-bit varint to the output. This is the same as upcasting 
    /// the value to a u64 and writing that, however this is more optimized 
    /// for writing 32-bit values.
    #[inline]
    pub fn write_varint32(&mut self, mut value: u32) -> WriterResult {
        match self.inner {
            Left(ref mut write) => {
                let mut buf = [0u8; 5];
                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            let part = buf.get_unchecked(0..i);
                            write.write_all(part)?;
                            return Ok(())
                        }
                    }
                }
            },
            Right(ref mut buf) => {
                if raw::raw_varint32_size(value).get() as usize > buf.len() {
                    return Err(io::Error::from(io::ErrorKind::WriteZero).into());
                }

                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            return Ok(())
                        }
                    }
                }
            }
        }
    }

    /// Writes a 64-bit varint to the output.
    #[inline]
    pub fn write_varint64(&mut self, mut value: u64) -> WriterResult {
        match self.inner {
            Left(ref mut write) => {
                let mut buf = [0u8; 10];
                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            let part = buf.get_unchecked(0..i);
                            write.write_all(part)?;
                            return Ok(())
                        }
                    }
                }
            },
            Right(ref mut buf) => {
                if raw::raw_varint64_size(value).get() as usize > buf.len() {
                    return Err(io::Error::from(io::ErrorKind::WriteZero).into());
                }

                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            return Ok(())
                        }
                    }
                }
            }
        }
    }

    /// Writes a 32-bit little endian integer to the output.
    #[inline]
    pub fn write_bit32(&mut self, value: u32) -> WriterResult {
        let value = value.to_le_bytes();
        const SIZE: usize = mem::size_of::<u32>();

        match self.inner {
            Left(ref mut i) => i.write_all(&value)?,
            Right(ref mut buf) => {
                if buf.len() < SIZE {
                    return Err(io::Error::from(io::ErrorKind::WriteZero).into());
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), SIZE);
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(SIZE), buf.len() - SIZE);
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes a 64-bit little endian integer to the output.
    #[inline]
    pub fn write_bit64(&mut self, value: u64) -> WriterResult {
        let value = value.to_le_bytes();
        const SIZE: usize = mem::size_of::<u64>();
        match self.inner {
            Left(ref mut i) => i.write_all(&value)?,
            Right(ref mut buf) => {
                if buf.len() >= SIZE {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), SIZE);
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(SIZE), buf.len() - SIZE);
                    }
                } else {
                    return Err(io::Error::from(io::ErrorKind::WriteZero).into());
                }
            }
        }
        Ok(())
    }

    /// Writes raw bytes to the output. This should be used carefully as to not corrupt the coded output.
    #[inline]
    pub fn write_bytes(&mut self, value: &[u8]) -> WriterResult {
        match &mut self.inner {
            Left(writer) => writer.write_all(value)?,
            Right(buf) => {
                if value.len() <= buf.len() {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), value.len());
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(value.len()), buf.len() - value.len());
                    }
                } else {
                    return Err(io::Error::from(io::ErrorKind::WriteZero).into())
                }
            }
        }
        Ok(())
    }

    /// Writes a length delimited set of bytes to the output.
    #[inline]
    pub fn write_length_delimited(&mut self, value: &[u8]) -> WriterResult {
        let len: i32 = value.len().try_into().map_err(|_| WriterError::ValueTooLarge)?;
        self.write_length(Length(len))?;
        self.write_bytes(value)
    }

    /// Writes a generic value to the output.
    #[inline]
    pub fn write_value<T: raw::Value + Wrapper>(&mut self, value: &T::Inner) -> WriterResult {
        T::wrap_ref(value).write_to(self)
    }

    /// Writes a generic value with a tag to the output based on the provided field number.
    #[inline]
    pub fn write_value_with_number<T: raw::SizedValue + Wrapper>(&mut self, num: FieldNumber, value: &T::Inner) -> WriterResult {
        self.write_tag(Tag::new(num, T::WIRE_TYPE))?;
        self.write_value::<T>(value)?;
        if T::WIRE_TYPE == WireType::StartGroup {
            self.write_tag(Tag::new(num, WireType::EndGroup))?;
        }
        Ok(())
    }

    /// Writes a collection of values to the output.
    #[inline]
    pub fn write_values<T>(&mut self, value: &impl collections::RepeatedValue<T>, codec: raw::TaggedValue<T>) -> WriterResult {
        value.write_to(self, codec)
    }

    /// Writes a collection of fields to the output.
    #[inline]
    pub fn write_fields(&mut self, value: &impl crate::FieldSet) -> WriterResult {
        value.write_to(self)
    }
}

#[cfg(test)]
mod test {
    mod coded_writer {
        use crate::io::{CodedWriter, Length};
        use crate::raw::{Uint32, Uint64};

        #[test]
        fn varint32_encode() {
            fn try_encode(value: u32, bytes: &[u8]) {
                let mut output = [0u8; 5];
                let mut writer = CodedWriter::with_slice(&mut output);
                writer.write_varint32(value).unwrap();

                let len = Length::for_value::<Uint32>(&value).unwrap().get() as usize;
                let slice = &output[0..len];

                assert_eq!(slice, bytes);
            }

            try_encode(0, &[0x80]);
            try_encode(127, &[0xFF]);
            try_encode(16_383, &[0x7F, 0xFF]);
            try_encode(2_097_151, &[0x7F, 0x7F, 0xFF]);
            try_encode(268_435_455, &[0x7F, 0x7F, 0x7F, 0xFF]);
            try_encode(u32::max_value(), &[0x7F, 0x7F, 0x7F, 0x7F, 0x8F]);
        }
        #[test]
        fn varint64_encode() {
            fn try_encode(value: u64, bytes: &[u8]) {
                let mut output = [0u8; 10];
                let mut writer = CodedWriter::with_slice(&mut output);
                writer.write_varint64(value).unwrap();

                let len = Length::for_value::<Uint64>(&value).unwrap().get() as usize;
                let slice = &output[0..len];

                assert_eq!(slice, bytes);
            }

            try_encode(0, &[0x80]);
            try_encode(127, &[0xFF]);
            try_encode(16_383, &[0x7F, 0xFF]);
            try_encode(2_097_151, &[0x7F, 0x7F, 0xFF]);
            try_encode(268_435_455, &[0x7F, 0x7F, 0x7F, 0xFF]);
            try_encode(u32::max_value() as u64, &[0x7F, 0x7F, 0x7F, 0x7F, 0x8F]);
            try_encode(u64::max_value(), &[0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x81]);
        }
        #[test]
        fn bit32_encode() {
            let value = 25;

            let mut output = [0u8; 4];
            let mut writer = CodedWriter::with_slice(&mut output);
            writer.write_bit32(value).unwrap();

            assert_eq!(output, value.to_le_bytes());

            let mut output = [0u8; 4];
            let mut write = output.as_mut();
            let mut writer = CodedWriter::with_write(&mut write);
            writer.write_bit32(value).unwrap();

            assert_eq!(output, value.to_le_bytes());
        }
        #[test]
        fn bit64_encode() {
            let value = 25;

            let mut output = [0u8; 8];
            let mut writer = CodedWriter::with_slice(&mut output);
            writer.write_bit64(value).unwrap();

            assert_eq!(output, value.to_le_bytes());

            let mut output = [0u8; 8];
            let mut write = output.as_mut();
            let mut writer = CodedWriter::with_write(&mut write);
            writer.write_bit64(value).unwrap();

            assert_eq!(output, value.to_le_bytes());
        }
        #[test]
        fn raw_bytes_encode() {
            let mut output = [0u8];
            let mut writer = CodedWriter::with_slice(&mut output);
            writer.write_bytes(&[1]).unwrap();

            assert_eq!(output, [1]);

            let mut output = [0u8];
            let mut write = output.as_mut();
            let mut writer = CodedWriter::with_write(&mut write);
            writer.write_bytes(&[1]).unwrap();

            assert_eq!(output, [1]);
        }

        // test that writing a value to less than the value's required space remaining returns an error
        macro_rules! fail_write_int {
            ($n:ident, $f:ident) => {
                #[test]
                fn $f() {
                    let mut empty = [].as_mut();

                    let mut writer = CodedWriter::with_slice(empty);
                    assert_eq!(writer.$f(10).ok(), None);

                    let mut writer = CodedWriter::with_write(&mut empty);
                    assert_eq!(writer.$f(10).ok(), None);
                }
            };
        }

        fail_write_int!(fail_write_varint32, write_varint32);
        fail_write_int!(fail_write_varint64, write_varint64);
        fail_write_int!(fail_write_bit32, write_bit32);
        fail_write_int!(fail_write_bit64, write_bit64);

        #[test]
        fn fail_write_bytes() {
            let mut empty = [].as_mut();

            let mut writer = CodedWriter::with_slice(empty);
            assert_eq!(writer.write_bytes(&[1]).ok(), None);

            let mut writer = CodedWriter::with_write(&mut empty);
            assert_eq!(writer.write_bytes(&[1]).ok(), None);
        }
    }
    mod coded_reader {
        use crate::io::CodedReader;

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
    }
}