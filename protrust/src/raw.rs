//! Contains types for protobuf values and traits for value operations.

use alloc::vec::Vec;
use alloc::string as astring;
use core::convert::TryInto;
use crate::{internal::Sealed, Message as TraitMessage};
use crate::extend::ExtendableMessage;
use crate::io::{self, read, write, WireType, ByteString, Length, LengthBuilder, CodedReader, CodedWriter, Input, Output};

/// A protobuf value type represented.
pub trait ValueType: Sealed {
    /// The Rust representation of the value
    type Inner;

    /// A value indicating the wire type of the value.
    const WIRE_TYPE: WireType;
}
/// A value with the type `V` capable of merging itself with an input value, writing itself to an output, calculating it's size, and checking it's initialization.
pub trait Value<V: ValueType<Inner = Self>>: Sized {
    /// Calculates the size of the value as encoded on the wire
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;

    /// Merges the value with the [`CodedReader`](../io/read/struct.CodedReader.html)
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()>;

    /// Writes the value to the [`CodedWriter`](../io/write/struct.CodedWriter.html)
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result;

    /// Returns whether the value is initialized, that is, if all the required fields in the value are set.
    fn is_initialized(&self) -> bool;

    /// Reads a new instance of the value
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self>;
}
/// A value type with a constant size. This can be specialized over to enable certain optimizations with size caculations.
pub trait ConstSized: ValueType {
    /// The constant size of the value
    const SIZE: Length;
}
/// A trait indicating whether a value type can be packed or not. Used in conjunction with the [`Packed`](struct.Packed.html) struct.
pub trait Packable: ValueType { }

/// A packed value type used with packed repeated fields.
pub struct Packed<V>(V) where V: ValueType + Packable, V::Inner: Value<V>;
impl<V> ValueType for Packed<V>
    where
        V: ValueType + Packable,
        V::Inner: Value<V>,
{
    type Inner = V::Inner;

    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}

pub type DoublePacked<V> where V: ValueType + Packable, V::Inner: Value<V> = Packed<Packed<V>>;

macro_rules! packable {
    ($($t:ty),*) => {
        $(
            impl Packable for $t { }
        )*
    };
}

packable!(Int32, Uint32, Int64, Uint64, Sint32, Sint64, Fixed32, Fixed64, Sfixed32, Sfixed64, Bool);
impl<T: crate::Enum> Packable for Enum<T> { }

const MAX_VARINT64_SIZE: Length = unsafe { Length::new_unchecked(10) };

/// A varint encoded 32-bit value. Negative values are encoded as 10-byte varints.
pub struct Int32;
impl Sealed for Int32 { }
impl ValueType for Int32 {
    type Inner = i32;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl Value<Int32> for i32 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        if *self >= 0 {
            builder.add_bytes(io::raw_varint32_size(*self as u32))
        } else {
            builder.add_bytes(MAX_VARINT64_SIZE)
        }
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Int32>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        if *self >= 0 {
            output.write_varint32(*self as u32)
        } else {
            output.write_varint64(i64::from(*self) as u64)
        }
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint32().map(|v| v as i32)
    }
}

/// A varint encoded 32-bit value. Can be at most 5 bytes.
pub struct Uint32;
impl Sealed for Uint32 { }
impl ValueType for Uint32 {
    type Inner = u32;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl Value<Uint32> for u32 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint32_size(*self))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Uint32>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(*self)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint32()
    }
}

/// A varint encoded 64-bit value. Can be at most 10 bytes.
pub struct Int64;
impl Sealed for Int64 { }
impl ValueType for Int64 {
    type Inner = i64;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl Value<Int64> for i64 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(*self as u64))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Int64>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(*self as u64)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(|v| v as i64)
    }
}

/// A varint encoded 64-bit value. Can be at most 10 bytes.
pub struct Uint64;
impl Sealed for Uint64 { }
impl ValueType for Uint64 {
    type Inner = u64;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl Value<Uint64> for u64 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(*self))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Uint64>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(*self)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64()
    }
}

/// A varint encoded 32-bit value. This is encoded using zig-zag encoding, 
/// which makes it more effecient at encoding negative values.
pub struct Sint32;
impl Sealed for Sint32 { }
impl ValueType for Sint32 {
    type Inner = i32;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl Value<Sint32> for i32 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let n = *self as u32;
        builder.add_bytes(io::raw_varint32_size((n << 1) ^ (n >> 31)))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Sint32>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        let n = *self as u32;
        output.write_varint32((n << 1) ^ (n >> 31))
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint32().map(|v| ((v >> 1) ^ (v << 31)) as i32)
    }
}

/// A varint encoded 64-bit value. This encoded using zig-zag encoding,
/// which makes it more effecient at encoding negative values.
pub struct Sint64;
impl Sealed for Sint64 { }
impl ValueType for Sint64 {
    type Inner = i64;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl Value<Sint64> for i64 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let n = *self as u64;
        builder.add_bytes(io::raw_varint64_size((n << 1) ^ (n >> 63)))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Sint64>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        let n = *self as u64;
        output.write_varint64((n << 1) ^ (n >> 63))
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(|v| ((v >> 1) ^ (v << 63)) as i64)
    }
}

/// A fixed size 32-bit value. This is encoded as 4 little endian bytes.
pub struct Fixed32;
impl Sealed for Fixed32 { }
impl ValueType for Fixed32 {
    type Inner = u32;
    const WIRE_TYPE: WireType = WireType::Bit32;
}
impl ConstSized for Fixed32 {
    const SIZE: Length = unsafe { Length::new_unchecked(4) };
}
impl Value<Fixed32> for u32 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Fixed32::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Fixed32>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit32(*self)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit32()
    }
}

/// A fixed size 64-bit value. This is encoded as 8 little endian bytes.
pub struct Fixed64;
impl Sealed for Fixed64 { }
impl ValueType for Fixed64 {
    type Inner = u64;
    const WIRE_TYPE: WireType = WireType::Bit64;
}
impl ConstSized for Fixed64 {
    const SIZE: Length = unsafe { Length::new_unchecked(8) };
}
impl Value<Fixed64> for u64 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Fixed64::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Fixed64>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit64(*self)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit64()
    }
}

/// A signed, fixed size 32-bit value. This is encoded as 4 little endian bytes.
pub struct Sfixed32;
impl Sealed for Sfixed32 { }
impl ValueType for Sfixed32 {
    type Inner = i32;
    const WIRE_TYPE: WireType = WireType::Bit32;
}
impl ConstSized for Sfixed32 {
    const SIZE: Length = unsafe { Length::new_unchecked(4) };
}
impl Value<Sfixed32> for i32 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Sfixed32::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Sfixed32>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit32(*self as u32)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit32().map(|v| v as i32)
    }
}

/// A signed, fixed size 64-bit value. This is encoded as 8 little endian bytes.
pub struct Sfixed64;
impl Sealed for Sfixed64 { }
impl ValueType for Sfixed64 {
    type Inner = i64;
    const WIRE_TYPE: WireType = WireType::Bit64;
}
impl ConstSized for Sfixed64 {
    const SIZE: Length = unsafe { Length::new_unchecked(8) };
}
impl Value<Sfixed64> for i64 {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Sfixed64::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Sfixed64>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit64(*self as u64)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit64().map(|v| v as i64)
    }
}

/// A bool value. This is encoded as a varint value
pub struct Bool;
impl Sealed for Bool { }
impl ValueType for Bool {
    type Inner = bool;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl ConstSized for Bool {
    const SIZE: Length = unsafe { Length::new_unchecked(1) };
}
impl Value<Bool> for bool {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Bool::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<Bool>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(*self as u32)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(|v| v != 0)
    }
}

/// A string value. This is encoded as a length-delimited series of bytes.
pub struct String;
impl Sealed for String { }
impl ValueType for String {
    type Inner = astring::String;
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}
impl Value<String> for astring::String {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = self.len().try_into().ok()?;
        builder.add_bytes(Length::new(len)?)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<String>().map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_length_delimited(self.as_bytes())
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        alloc::string::String::from_utf8(input.read_value::<Bytes<Vec<_>>>()?)
            .map_err(io::read::Error::InvalidString)
    }
}

/// A bytes value. This is encoded as a length-delimited series of bytes.
pub struct Bytes<T>(T);
impl<T> ValueType for Bytes<T> {
    type Inner = T;
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}
impl<T> Sealed for Bytes<T> { }
impl<T: ByteString> Value<Bytes<T>> for T {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = self.as_ref().len().try_into().ok()?;
        builder.add_bytes(Length::new(len)?)
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        output.write_length_delimited(self.as_ref())
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        input.read_length_delimited::<T>()
    }
}

/// An enum value. This is encoded as a 32-bit varint value.
pub struct Enum<T>(T);
impl<T> Sealed for Enum<T> { }
impl<T: crate::Enum> ValueType for Enum<T> {
    type Inner = T;
    const WIRE_TYPE: WireType = WireType::Varint;
}
impl<T: crate::Enum> Value<Enum<T>> for T {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_value::<Int32>(&(*self).into())
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        input.read_value::<Enum<T>>().map(|v| *self = v)
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        output.write_value::<Int32>(&(*self).into())
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        input.read_value::<Int32>().map(|v| v.into())
    }
}

/// A message value. This is encoded as a length-delimited series of bytes.
pub struct Message<T>(T);
impl<T> Sealed for Message<T> { }
impl<T: TraitMessage> ValueType for Message<T> {
    type Inner = T;
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
}
impl<T: TraitMessage> Value<Message<T>> for T {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len = self.calculate_size()?;
        builder
            .add_value::<Uint32>(&(len.get() as u32))?
            .add_bytes(len)
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        input.read_limit()?.then(|input| self.merge_from(input))
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        let length = self.calculate_size().ok_or(io::write::Error::ValueTooLarge)?;
        output.write_length(length)?;
        self.write_to(output)?;
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }
    default fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        let mut t = T::new();
        t.merge_from(input)?;
        Ok(t)
    }
}
impl<T: TraitMessage + ExtendableMessage + 'static> Value<Message<T>> for T {
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        let mut t = T::new();
        t.extensions_mut().replace_registry(input.registry());
        input.merge_value::<Message<T>>(&mut t)?;
        Ok(t)
    }
}

/// A group value. This is encoded by putting a start and end tag between its encoded fields.
pub struct Group<T>(T);
impl<T> Sealed for Group<T> { }
impl<T: TraitMessage> ValueType for Group<T> {
    type Inner = T;
    const WIRE_TYPE: WireType = WireType::StartGroup;
}
impl<T: TraitMessage> Value<Group<T>> for T {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(self.calculate_size()?)
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        self.merge_from(input)
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        self.write_to(output)
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        let mut t = T::new();
        t.merge_from(input)?;
        Ok(t)
    }
}