//! Contains types and traits for reading and writing protobuf coded data.

pub mod stream;
pub mod read;
pub mod write;

pub use read::{Input, CodedReader};
pub use write::{Output, CodedWriter};

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::num::NonZeroU32;
use crate::collections::{RepeatedValue, FieldSet};
use crate::raw::Value;

#[cfg(feature = "std")]
use std::error::Error;

mod internal {
    pub trait Array: AsRef<[u8]> + AsMut<[u8]> {
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
}

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

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
        (self as u8 == WireType::Varint as u8) ||
        (self as u8 == WireType::Bit64 as u8) ||
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
        match WireType::try_from((self.get() & 0b0111) as u8) {
            Ok(wt) => wt,
            // we can only reach this through unsafe code
            Err(_) => unsafe { core::hint::unreachable_unchecked() }
        }
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
    pub fn field(self) -> FieldNumber {
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

    /// Returns the length of the value in the specified form
    pub fn of_value<V: Value>(value: &V::Inner) -> Option<Length> {
        LengthBuilder::new().add_value::<V>(value).map(LengthBuilder::build)
    }

    /// Returns the length of the set of values with the specified tag
    pub fn of_values<T: RepeatedValue<V>, V>(value: &T, num: FieldNumber) -> Option<Length> {
        LengthBuilder::new().add_values::<T, V>(value, num).map(LengthBuilder::build)
    }

    /// Returns the length of the field set
    pub fn of_fields<T: FieldSet>(value: &T) -> Option<Length> {
        LengthBuilder::new().add_fields::<T>(value).map(LengthBuilder::build)
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
    pub const fn new() -> LengthBuilder {
        Self(0)
    }

    /// Adds an arbitrary number of bytes to the length
    #[inline]
    #[must_use = "this returns the builder to chain and does not mutate it in place"]
    pub fn add_bytes(self, value: Length) -> Option<Self> {
        #[cfg(feature = "checked_size")]
        return self.0.checked_add(value.get()).map(LengthBuilder);

        #[cfg(not(feature = "checked_size"))]
        return Some(LengthBuilder(self.0 + value.get()));
    }

    /// Adds a tag's size to the length
    #[inline]
    #[must_use = "this returns the builder to chain and does not mutate it in place"]
    pub fn add_tag(self, tag: Tag) -> Option<Self> {
        self.add_bytes(raw_varint32_size(tag.get()))
    }

    /// Adds a value's length to this instance
    #[inline]
    #[must_use = "this returns the builder to chain and does not mutate it in place"]
    pub fn add_value<V: Value>(self, value: &V::Inner) -> Option<Self> {
        V::calculate_size(value, self)
    }
    /// Adds a field's length to this instance using the specified field number
    #[inline]
    #[must_use = "this returns the builder to chain and does not mutate it in place"]
    pub fn add_field<V: Value>(self, num: FieldNumber, value: &V::Inner) -> Option<Self> {
        let temp = 
            self.add_tag(Tag::new(num, V::WIRE_TYPE))?
                .add_value::<V>(value)?;

        if V::WIRE_TYPE == WireType::StartGroup {
            temp.add_tag(Tag::new(num, WireType::EndGroup))
        } else {
            Some(temp)
        }
    }

    /// Adds a value collection's length to this instance with the specified tag
    #[inline]
    #[must_use = "this returns the builder to chain and does not mutate it in place"]
    pub fn add_values<T: RepeatedValue<V>, V>(self, value: &T, num: FieldNumber) -> Option<Self> {
        value.calculate_size(self, num)
    }

    /// Adds the length of the fields in the set to this instance
    #[inline]
    #[must_use = "this returns the builder to chain and does not mutate it in place"]
    pub fn add_fields<T: FieldSet>(self, value: &T) -> Option<Self> {
        value.calculate_size(self)
    }

    /// Consumes the builder, returning a [`Length`](struct.Length.html) for writing to an output
    #[inline]
    pub const fn build(self) -> Length {
        Length(self.0)
    }
}

/// A generic string of bytes.
/// This is used by [`CodedReader`](read/struct.CodedReader.html) to read length delimited byte values
/// into various kinds of byte collections.
pub trait ByteString: AsRef<[u8]> + AsMut<[u8]> {
    /// Creates a new instance of the byte string. This value does not need to be zeroed.
    fn new(len: usize) -> Self;
}

impl ByteString for Box<[u8]> {
    fn new(len: usize) -> Self {
        <Vec<u8> as ByteString>::new(len).into_boxed_slice()
    }
}

impl ByteString for Vec<u8> {
    fn new(len: usize) -> Self {
        alloc::vec![0; len]
    }
}

#[inline]
pub(crate) const fn raw_varint32_size(value: u32) -> Length {
    unsafe { Length::new_unchecked((((31 ^ (value | 1).leading_zeros()) * 9 + 73) / 64) as i32) }
}

#[inline]
pub(crate) const fn raw_varint64_size(value: u64) -> Length {
    unsafe { Length::new_unchecked((((63 ^ (value | 1).leading_zeros()) * 9 + 73) / 64) as i32) }
}

#[cfg(test)]
mod test {
    
}