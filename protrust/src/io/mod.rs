//! Contains types and traits for reading and writing protobuf coded data.

pub mod stream;
pub mod read;
pub mod write;

pub use read::CodedReader;
pub use write::CodedWriter;

use alloc::alloc::{Layout, Alloc, Global, handle_alloc_error};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp;
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::mem;
use core::num::NonZeroU32;
use core::ptr;
use core::slice;
use crate::{collections, raw};
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
/// This is used by [`CodedReader`](read/struct.CodedReader.html) to read length delimited byte values
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

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use crate::io::{CodedWriter, WriterResult, CodedReader, Tag, FieldNumber, WireType};
    use alloc::alloc::Global;
    use alloc::boxed::Box;

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