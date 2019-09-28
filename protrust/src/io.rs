//! Contains types and traits for reading and writing protobuf coded data.

pub mod stream;

use alloc::alloc::{Layout, Alloc, Global, handle_alloc_error};
use alloc::boxed::Box;
use alloc::string::FromUtf8Error;
use alloc::vec::Vec;
use core::cmp;
use core::convert::{TryInto, TryFrom};
use core::fmt::{self, Display, Formatter};
use core::mem;
use core::num::NonZeroU32;
use core::ptr;
use core::slice;
use crate::{collections, raw};
use either::{Either, Left, Right};
use self::stream::{Read, Write};
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error::Error;

/// The wire type of a protobuf value.
///
/// A wire type is paired with a field number between 1 and 536,870,911 to create a tag,
/// a unique identifier for a field on the wire.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum WireType {
    /// A value read as a variable length integer.
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
    /// 
    /// # Examples
    /// 
    /// ```
    /// use protrust::io::FieldNumber;
    /// 
    /// assert_eq!(FieldNumber::new(0), None);
    /// assert_eq!(FieldNumber::new(1).map(Into::into), Some(1));
    /// assert_eq!(FieldNumber::new(FieldNumber::MAX_VALUE), Some(FieldNumber::MAX));
    /// assert_eq!(FieldNumber::new(FieldNumber::MAX_VALUE + 1), None);
    /// ```
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

impl From<FieldNumber> for u32 {
    fn from(x: FieldNumber) -> u32 {
        x.get()
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
    /// # use std::convert::TryFrom;
    /// 
    /// assert_eq!(Tag::try_from(8).unwrap().wire_type(), WireType::Varint);
    /// assert_eq!(Tag::try_from(17).unwrap().wire_type(), WireType::Bit64);
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
    /// # use std::convert::TryFrom;
    /// 
    /// assert_eq!(Tag::try_from(8).unwrap().number().get(), 1);
    /// assert_eq!(Tag::try_from(17).unwrap().number().get(), 2);
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

impl From<Tag> for u32 {
    fn from(x: Tag) -> u32 {
        x.get()
    }
}

/// The error returned when an attempt to convert a 32-bit value to a tag fails due to an invalid field number or wire type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TryTagFromRawError(());

impl Display for TryTagFromRawError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "invalid tag; this could be caused by an invalid wire type or a 0 field number")
    }
}

#[cfg(feature = "std")]
impl Error for TryTagFromRawError { }

impl TryFrom<u32> for Tag {
    type Error = TryTagFromRawError;

    /// Creates a new tag if the value is not zero and has a valid field number and wire type
    ///
    /// # Examples
    ///
    /// ```
    /// use protrust::io::Tag;
    /// # use std::convert::TryFrom;
    ///
    /// assert!(Tag::try_from(1).is_err());
    /// assert!(Tag::try_from(8).is_ok());
    /// assert!(Tag::try_from(16).is_ok());
    /// assert!(Tag::try_from(14).is_err());
    /// ```
    #[inline]
    fn try_from(n: u32) -> Result<Tag, TryTagFromRawError> {
        match (n & 0b111, n >> 3) {
            // (wire type, field number)
            (6, _) | (7, _) | (_, 0) => Err(TryTagFromRawError(())),
            _ => unsafe { Ok(Tag(NonZeroU32::new_unchecked(n))) },
        }
    }
}

/// An opaque type that represents the length of a delimited value
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length(i32);

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

    /// Makes a new length from the specified [`i32`], returning [`None`] if the value is negative
    /// 
    /// [`i32`]: https://doc.rust-lang.org/nightly/std/primitive.i32.html
    /// [`None`]: https://doc.rust-lang.org/nightly/std/option/enum.Option.html#variant.None
    pub fn new(x: i32) -> Option<Length> {
        if x < 0 {
            None
        } else {
            unsafe { Some(Length::new_unchecked(x)) }
        }
    }

    /// Makes a new length from the specified [`i32`], without checking if the value is negative.
    /// 
    /// # Safety
    /// 
    /// Providing a negative value may cause 
    pub const unsafe fn new_unchecked(x: i32) -> Length {
        Length(x)
    }

    /// Calculates the length of the value
    #[inline]
    pub fn for_value<T: raw::Value + Wrapper>(value: &T::Inner) -> Option<Length> {
        T::wrap_ref(value).calculate_size(LengthBuilder::new()).map(LengthBuilder::build)
    }

    /// Calculates the length of a collection of values
    #[inline]
    pub fn for_values<T>(value: &impl collections::RepeatedValue<T>, tag: Tag) -> Option<Length> {
        value.calculate_size(LengthBuilder::new(), tag).map(LengthBuilder::build)
    }

    /// Calculates the length of a set of fields
    #[inline]
    pub fn for_fields(value: &impl crate::FieldSet) -> Option<Length> {
        value.calculate_size(LengthBuilder::new()).map(LengthBuilder::build)
    }
}

impl From<Length> for i32 {
    fn from(x: Length) -> i32 {
        x.get()
    }
}

/// An opaque type for building a length for writing to an output.
/// 
/// This exists to make creating checked lengths easier in generated code.
pub struct LengthBuilder(i32);

impl LengthBuilder {
    /// Creates a new length builder
    #[inline]
    pub fn new() -> LengthBuilder {
        Self(0)
    }

    /// Adds an arbitrary number of bytes to the length
    #[inline]
    pub fn add_bytes(self, value: i32) -> Option<Self> {
        #[cfg(feature = "checked_size")]
        return self.0.checked_add(value).map(LengthBuilder);

        #[cfg(not(feature = "checked_size"))]
        return Some(LengthBuilder(self.0 + value));
    }

    /// Adds a tag to the output
    #[inline]
    pub fn add_tag(self, tag: Tag) -> Option<Self> {
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
    pub fn add_values<V>(self, value: &impl collections::RepeatedValue<V>, tag: Tag) -> Option<Self> {
        value.calculate_size(self, tag)
    }

    /// Adds a set of fields to the length
    #[inline]
    pub fn add_fields(self, value: &impl crate::FieldSet) -> Option<Self> {
        value.calculate_size(self)
    }

    /// Consumes the builder, returning a [`Length`](struct.Length.html) for writing to an output
    #[inline]
    pub fn build(self) -> Length {
        Length(self.0)
    }
}

/// A string of bytes that can be allocated into a provided allocator.
/// This is used by [`CodedReader`](struct.CodedReader.html) to read length delimited byte values
/// into various kinds of byte collections.
pub trait ByteString: AsRef<[u8]> + AsMut<[u8]> {
    /// The allocator to allocate the byte string into
    type Alloc: Alloc;

    /// Creates a new instance of the byte string with the specified allocator. This value must be zeroed.
    fn new(len: usize, a: Self::Alloc) -> Self;

    /// Resizes the byte string, reusing an existing allocator. This value must be zeroed.
    fn resize(&mut self, new_len: usize);
}

impl ByteString for Box<[u8]> {
    type Alloc = Global;

    fn new(len: usize, mut a: Global) -> Self {
        unsafe {
            let layout = Layout::from_size_align_unchecked(len, mem::align_of::<u8>());
            let value = a.alloc_zeroed(layout).unwrap_or_else(|_| handle_alloc_error(layout));
            let slice = slice::from_raw_parts_mut(value.as_ptr(), len);
            Box::from_raw(slice)
        }
    }
    fn resize(&mut self, new_len: usize) {
        let mut a = Global;
        match (self.len(), new_len) {
            (0, 0) => { /* do nothing, since there's nothing here */ },
            (0, new_len) => { // we don't need to deallocate and instead we can just allocate and write
                *self = ByteString::new(new_len, a);
            },
            (_, 0) => { // we just need to deallocate and write an empty slice
                *self = Box::new([]);
            },
            (old_len, new_len) => unsafe {
                if old_len == new_len {
                    ptr::write_bytes(self.as_mut_ptr(), 0, new_len);
                } else {
                    let old = mem::replace(self, Box::new([]));
                    let layout = Layout::for_value::<[u8]>(&old);
                    let ptr = Box::into_raw_non_null(old).cast::<u8>();
                    let result =
                        if old_len > new_len {
                            a.shrink_in_place(ptr, layout, new_len)
                        } else {
                            a.grow_in_place(ptr, layout, new_len)
                        };
                    if let Ok(()) = result {
                        ptr::write_bytes(ptr.as_ptr(), 0, new_len); // no guarantee that the newly available memory is zeroed
                        *self = Box::from_raw(slice::from_raw_parts_mut(ptr.as_ptr(), new_len));
                    } else {
                        a.dealloc(ptr, layout);
                        *self = ByteString::new(new_len, a);
                    }
                }
            }
        }
    }
}

impl ByteString for Vec<u8> {
    type Alloc = Global;

    fn new(len: usize, a: Global) -> Self {
        <Box<[u8]> as ByteString>::new(len, a).into_vec()
    }
    fn resize(&mut self, new_len: usize) {
        let old_len = self.len();
        self.resize(new_len, 0);
        let old_data = &mut self[0..cmp::min(old_len, new_len)];
        unsafe { ptr::write_bytes(old_data.as_mut_ptr(), 0, old_data.len()); }
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
    StreamError(stream::Error),
    /// The input contained an invalid UTF8 string
    InvalidString(FromUtf8Error),
}

impl From<stream::Error> for ReaderError {
    fn from(value: stream::Error) -> ReaderError {
        ReaderError::StreamError(value)
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
            ReaderError::StreamError(_) => write!(fmt, "an error occured in the underlying input"),
            ReaderError::InvalidString(_) => write!(fmt, "the input contained an invalid UTF8 string")
        }
    }
}

#[cfg(feature = "std")]
impl Error for ReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReaderError::StreamError(ref e) => Some(e),
            ReaderError::InvalidString(ref e) => Some(e),
            _ => None,
        }
    }
}

/// A result for a [`CodedReader`](struct.CodedReader.html) read operation
pub type ReaderResult<T> = core::result::Result<T, ReaderError>;

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
pub struct ReaderBuilder {
    options: ReaderOptions
}

impl ReaderBuilder {
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
    fn read_exact(&mut self, buf: &mut [u8]) -> ReaderResult<()> {
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
        ReaderBuilder::new().with_read(inner)
    }
    /// Creates a new [`CodedReader`] in the default configuration
    /// over the borrowed [`Read`] with the specified buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: streams/trait.Read.html
    #[inline]
    pub fn with_capacity(capacity: usize, inner: &'a mut dyn Read) -> Self {
        ReaderBuilder::new().with_capacity(capacity, inner)
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
        ReaderBuilder::new().with_slice(inner)
    }

    #[inline]
    fn consume_limit(&mut self, len: i32) {
        if let Some(limit) = self.limit.as_mut() {
            *limit -= len;
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> ReaderResult<()> {
        match self.inner {
            Left(ref mut read) => {
                read.read_exact(buf)?;
                read.consume(buf.len() as i32);
            },
            Right(ref mut curs) => {
                let slice = 
                    apply_limit(curs.get(), self.limit)
                        .get(..buf.len())
                        .ok_or(ReaderError::from(stream::Error))?;
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
    pub fn read_tag(&mut self) -> ReaderResult<Option<Tag>> {
        unimplemented!()
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
    pub fn read_length_delimited<T: ByteString>(&mut self, a: T::Alloc) -> ReaderResult<T> {
        let length = self.read_length()?.get();
        let mut value = T::new(length as usize, a);
        debug_assert!(value.as_ref().len() == length as usize);

        self.read_exact(value.as_mut())?;
        Ok(value)
    }

    /// Merges a length delimited value from the input prefixed by the length. This may reallocate
    pub fn merge_length_delimited<T: ByteString>(&mut self, value: &mut T) -> ReaderResult<()> {
        let length = self.read_length()?.get();
        value.resize(length as usize);

        debug_assert!(value.as_ref().len() == length as usize);

        self.read_exact(value.as_mut())?;
        Ok(())
    }

    #[inline]
    /// Skips the last value based on the tag read from the input. If no tag has been read, this does nothing
    pub fn skip(&mut self) -> ReaderResult<()> {
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
    pub fn read_value<T: raw::Primitive + Wrapper>(&mut self) -> ReaderResult<T::Inner> {
        T::read_new(self).map(T::unwrap)
    }

    /// Reads a heaping values in the specified allocator instance
    #[inline]
    pub fn read_value_in<T: raw::Heaping + Wrapper>(&mut self, a: T::Alloc) -> ReaderResult<T::Inner> {
        T::read_new(self, a).map(T::unwrap)
    }

    /// Merges an existing instance of a value with a value from the input
    #[inline]
    pub fn merge_value<T: raw::Value + Wrapper>(&mut self, value: &mut T::Inner) -> ReaderResult<()> {
        T::wrap_mut(value).merge_from(self)
    }

    /// Adds values from the input to the repeated value
    #[inline]
    pub fn add_values_to<T: raw::Primitive>(&mut self, value: &mut impl collections::RepeatedPrimitiveValue<T>) -> ReaderResult<()> {
        value.add_entries_from(self)
    }

    /// Adds values from the input to the 
    #[inline]
    pub fn add_values_to_in<T: raw::Heaping>(&mut self, value: &mut impl collections::RepeatedHeapingValue<T>, a: T::Alloc) -> ReaderResult<()> {
        value.add_entries_from(self, a)
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
    /// An error used to indicate a value was provided that was 
    /// too large to write to an output.
    ValueTooLarge,
    /// An error occured while writing data to the output.
    /// For slice outputs, this is used to indicate if
    /// not all data could be written to the slice.
    IoError(stream::Error)
}

impl Display for WriterError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            WriterError::ValueTooLarge => write!(f, "the value was too large to write to the output"),
            WriterError::IoError(_) => write!(f, "an error occured while writing to the output")
        }
    }
}

#[cfg(feature = "std")]
impl Error for WriterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WriterError::IoError(e) => Some(e),
            _ => None
        }
    }
}

impl From<stream::Error> for WriterError {
    fn from(e: stream::Error) -> Self {
        Self::IoError(e)
    }
}

/// A result for a [`CodedWriter`](struct.CodedWriter.html) read operation
pub type WriterResult = core::result::Result<(), WriterError>;

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
                            write.write(part)?;
                            return Ok(())
                        }
                    }
                }
            },
            Right(ref mut buf) => {
                if raw::raw_varint32_size(value).get() as usize > buf.len() {
                    return Err(stream::Error.into());
                }

                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(i), buf.len() - i);
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
                            write.write(part)?;
                            return Ok(())
                        }
                    }
                }
            },
            Right(ref mut buf) => {
                if raw::raw_varint64_size(value).get() as usize > buf.len() {
                    return Err(stream::Error.into());
                }

                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(i), buf.len() - i);
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
            Left(ref mut i) => i.write(&value)?,
            Right(ref mut buf) => {
                if buf.len() < SIZE {
                    return Err(stream::Error.into());
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
            Left(ref mut i) => i.write(&value)?,
            Right(ref mut buf) => {
                if buf.len() >= SIZE {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), SIZE);
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(SIZE), buf.len() - SIZE);
                    }
                } else {
                    return Err(stream::Error.into());
                }
            }
        }
        Ok(())
    }

    /// Writes raw bytes to the output. This should be used carefully as to not corrupt the coded output.
    #[inline]
    pub fn write_bytes(&mut self, value: &[u8]) -> WriterResult {
        match &mut self.inner {
            Left(writer) => writer.write(value)?,
            Right(buf) => {
                if value.len() <= buf.len() {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), value.len());
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(value.len()), buf.len() - value.len());
                    }
                } else {
                    return Err(stream::Error.into());
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

    /// Writes a collection of values to the output.
    #[inline]
    pub fn write_values<T>(&mut self, value: &impl collections::RepeatedValue<T>, tag: Tag) -> WriterResult {
        value.write_to(self, tag)
    }

    /// Writes a collection of fields to the output.
    #[inline]
    pub fn write_fields(&mut self, value: &impl crate::FieldSet) -> WriterResult {
        value.write_to(self)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use crate::io::{CodedWriter, WriterResult, CodedReader, Tag, FieldNumber, WireType};
    use alloc::alloc::Global;
    use alloc::boxed::Box;

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
                    assert!(writer.$f(10).is_err());

                    let mut writer = CodedWriter::with_write(&mut empty);
                    assert!(writer.$f(10).is_err());
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
            assert!(writer.write_bytes(&[1]).is_err());

            let mut writer = CodedWriter::with_write(&mut empty);
            assert!(writer.write_bytes(&[1]).is_err());
        }
    }
    mod coded_reader {
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

    #[test]
    fn roundtrip_many_values() {
        fn write(output: &mut CodedWriter) -> WriterResult {
            output.write_tag(Tag::new(FieldNumber::new(1).unwrap(), WireType::Varint))?;
            output.write_varint32(1)?;
            output.write_tag(Tag::new(FieldNumber::new(2).unwrap(), WireType::Varint))?;
            output.write_varint64(u64::max_value())?;
            output.write_tag(Tag::new(FieldNumber::new(3).unwrap(), WireType::Bit32))?;
            output.write_bit32(123)?;
            output.write_tag(Tag::new(FieldNumber::new(4).unwrap(), WireType::Bit64))?;
            output.write_bit64(1234567890)?;
            output.write_tag(Tag::new(FieldNumber::new(5).unwrap(), WireType::LengthDelimited))?;
            output.write_length_delimited(&[12])?;
            Ok(())
        }
        fn read(input: &mut CodedReader) {
            assert_matches!(input.read_tag(), Ok(Some(tag)) => assert_eq!(tag, Tag::new(FieldNumber::new(1).unwrap(), WireType::Varint)));
            assert_matches!(input.read_varint32(), Ok(1));
            assert_matches!(input.read_tag(), Ok(Some(tag)) => assert_eq!(tag, Tag::new(FieldNumber::new(2).unwrap(), WireType::Varint)));
            assert_matches!(input.read_varint64(), Ok(core::u64::MAX));
            assert_matches!(input.read_tag(), Ok(Some(tag)) => assert_eq!(tag, Tag::new(FieldNumber::new(3).unwrap(), WireType::Bit32)));
            assert_matches!(input.read_bit32(), Ok(123));
            assert_matches!(input.read_tag(), Ok(Some(tag)) => assert_eq!(tag, Tag::new(FieldNumber::new(4).unwrap(), WireType::Bit64)));
            assert_matches!(input.read_bit64(), Ok(1234567890));
            assert_matches!(input.read_tag(), Ok(Some(tag)) => assert_eq!(tag, Tag::new(FieldNumber::new(5).unwrap(), WireType::LengthDelimited)));
            assert_matches!(input.read_length_delimited::<Box<_>>(Global), Ok(ref value) if value.as_ref().eq(&[12]));
        }

        // (5 * 1 byte) tags + 1 1 byte varint + 1 10 byte varint + 1 32-bit fixed + 1 64-bit fixed + 1 1 byte length delimited (2 bytes)

        let mut data = [0u8; 30];
        let mut writer = CodedWriter::with_slice(&mut data);
        write(&mut writer).unwrap();

        let mut input = CodedReader::with_slice(&data);
        read(&mut input);

        let mut data = [0u8; 30];
        let mut bytes = data.as_mut();
        let mut writer = CodedWriter::with_write(&mut bytes);
        write(&mut writer).unwrap();

        let mut bytes = data.as_ref();
        let mut input = CodedReader::with_read(&mut bytes);
        read(&mut input);
    }
}