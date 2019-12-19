//! Contains types for protobuf values and traits for value operations.

use alloc::vec::Vec;
use core::convert::TryInto;
use crate::{internal::Sealed, Message as TraitMessage};
use crate::io::{self, read, write, WireType, ByteString, Length, LengthBuilder, CodedReader, CodedWriter, Input, Output};
use trapper::{newtype, Wrapper};

/// A value capable of merging itself with an input value, writing itself to an output, calculating it's size, and checking it's initialization.
pub trait Value: Sized + Sealed {
    /// A value indicating the wire type of the value without packing.
    /// This can be used to indicate if a value is elegible for repeated field packing.
    const WIRE_TYPE: WireType;

    /// Calculates the size of the value as encoded on the wire
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;

    /// Merges the value with the [`CodedRead`](../io/read/struct.CodedRead.html)
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()>;

    /// Writes the value to the [`CodedWrite`](../io/write/struct.CodedWrite.html)
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result;

    /// Returns if the value is initialized, that is, if all the required fields in the value are set.
    fn is_initialized(&self) -> bool;

    /// Reads a new instance of the value
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self>;
}
/// A value with a constant size. This can be specialized over to enable certain optimizations with size caculations.
pub trait ConstSized: Value {
    /// The constant size of the value
    const SIZE: Length;
}

newtype! {
    /// A varint encoded 32-bit value. Negative values are encoded as 10-byte varints.
    pub type Int32(i32);
}

impl Sealed for Int32 { }
impl Value for Int32 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        if self.0 >= 0 {
            builder.add_bytes(io::raw_varint32_size(self.0 as u32))
        } else {
            builder.add_bytes(unsafe { Length::new_unchecked(10) })
        }
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        if self.0 >= 0 {
            output.write_varint32(self.0 as u32)
        } else {
            output.write_varint64(i64::from(self.0) as u64)
        }
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint32().map(|v| Self(v as i32))
    }
}

newtype! {
    /// A varint encoded 32-bit value. Can be at most 5 bytes.
    pub type Uint32(u32);
}

impl Sealed for Uint32 { }
impl Value for Uint32 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint32_size(self.0))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(self.0)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint32().map(Self)
    }
}

newtype! {
    /// A varint encoded 64-bit value. Can be at most 10 bytes.
    pub type Int64(i64);
}

impl Sealed for Int64 { }
impl Value for Int64 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(self.0 as u64))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(self.0 as u64)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(|v| Self(v as i64))
    }
}

newtype! {
    /// A varint encoded 64-bit value. Can be at most 10 bytes.
    pub type Uint64(u64);
}

impl Sealed for Uint64 { }
impl Value for Uint64 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(self.0))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(self.0)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(Self)
    }
}

newtype! {
    /// A varint encoded 32-bit value. This is encoded using zig-zag encoding, 
    /// which makes it more effecient at encoding negative values.
    pub type Sint32(i32);
}

impl Sealed for Sint32 { }
impl Value for Sint32 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let n = self.0 as u32;
        builder.add_bytes(io::raw_varint32_size((n << 1) ^ (n >> 31)))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        let n = self.0 as u32;
        output.write_varint32((n << 1) ^ (n >> 31))
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint32().map(|v| Self(((v >> 1) ^ (v << 31)) as i32))
    }
}

newtype! {
    /// A varint encoded 64-bit value. This encoded using zig-zag encoding,
    /// which makes it more effecient at encoding negative values.
    pub type Sint64(i64);
}

impl Sealed for Sint64 { }
impl Value for Sint64 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let n = self.0 as u64;
        builder.add_bytes(io::raw_varint64_size((n << 1) ^ (n >> 63)))
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        let n = self.0 as u64;
        output.write_varint64((n << 1) ^ (n >> 63))
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(|v| Self(((v >> 1) ^ (v << 63)) as i64))
    }
}

newtype! {
    /// A fixed size 32-bit value. This is encoded as 4 little endian bytes.
    pub type Fixed32(u32);
}

impl Sealed for Fixed32 { }
impl Value for Fixed32 {
    const WIRE_TYPE: WireType = WireType::Bit32;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit32(self.0)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit32().map(Self)
    }
}
impl ConstSized for Fixed32 {
    const SIZE: Length = unsafe { Length::new_unchecked(4) };
}

newtype! {
    /// A fixed size 64-bit value. This is encoded as 8 little endian bytes.
    pub type Fixed64(u64);
}

impl Sealed for Fixed64 { }
impl Value for Fixed64 {
    const WIRE_TYPE: WireType = WireType::Bit64;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit64(self.0)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit64().map(Self)
    }
}
impl ConstSized for Fixed64 {
    const SIZE: Length = unsafe { Length::new_unchecked(8) };
}

newtype! {
    /// A signed, fixed size 32-bit value. This is encoded as 4 little endian bytes.
    pub type Sfixed32(i32);
}

impl Sealed for Sfixed32 { }
impl Value for Sfixed32 {
    const WIRE_TYPE: WireType = WireType::Bit32;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit32(self.0 as u32)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit32().map(|v| Self(v as i32))
    }
}
impl ConstSized for Sfixed32 {
    const SIZE: Length = unsafe { Length::new_unchecked(4) };
}

newtype! {
    /// A signed, fixed size 64-bit value. This is encoded as 8 little endian bytes.
    pub type Sfixed64(i64);
}

impl Sealed for Sfixed64 { }
impl Value for Sfixed64 {
    const WIRE_TYPE: WireType = WireType::Bit64;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit64(self.0 as u64)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_bit64().map(|v| Self(v as i64))
    }
}
impl ConstSized for Sfixed64 {
    const SIZE: Length = unsafe { Length::new_unchecked(8) };
}

newtype! {
    /// A bool value. This is encoded as a varint value
    pub type Bool(bool);
}

impl Sealed for Bool { }
impl Value for Bool {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(self.0 as u32)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        input.read_varint64().map(|v| Self(v != 0))
    }
}
impl ConstSized for Bool {
    const SIZE: Length = unsafe { Length::new_unchecked(1) };
}

newtype! {
    /// A string value. This is encoded as a length-delimited series of bytes.
    pub type String(alloc::string::String);
}

impl Sealed for String { }
impl Value for String {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = self.0.len().try_into().ok()?;
        builder.add_bytes(Length::new(len)?)
    }
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        output.write_length_delimited(self.0.as_bytes())
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self> {
        alloc::string::String::from_utf8(input.read_value::<Bytes<Vec<_>>>()?)
            .map_err(io::read::Error::InvalidString)
            .map(Self)
    }
}

newtype! {
    /// A bytes value. This is encoded as a length-delimited series of bytes.
    pub type Bytes<T>(T);
}

impl<T> Sealed for Bytes<T> { }
impl<T: ByteString> Value for Bytes<T> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = self.0.as_ref().len().try_into().ok()?;
        builder.add_bytes(Length::new(len)?)
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        output.write_length_delimited(self.0.as_ref())
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        input.read_length_delimited::<T>().map(Self)
    }
}

newtype! {
    /// An enum value. This is encoded as a 32-bit varint value.
    pub type Enum<T>(T);
}

impl<T> Sealed for Enum<T> { }
impl<T: crate::Enum> Value for Enum<T> {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_value::<Int32>(&self.0.into())
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        Int32(self.0.into()).write_to(output)
    }
    fn is_initialized(&self) -> bool { true }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        Int32::read_new(input).map(|v| Self(v.0.into()))
    }
}

newtype! {
    /// A message value. This is encoded as a length-delimited series of bytes.
    pub type Message<T>(T);
}

impl<T> Sealed for Message<T> { }
impl<T: TraitMessage> Value for Message<T> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        self.0.calculate_size(builder)
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        input.read_limit()?.then(|input| self.0.merge_from(input))
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        let length = Length::of_value::<Self>(&self.0).ok_or(io::write::Error::ValueTooLarge)?;
        output.write_length(length)?;
        TraitMessage::write_to::<U>(&self.0, output)?;
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        let mut t = Self::wrap(T::new());
        t.merge_from(input)?;
        Ok(t)
    }
}

newtype! {
    /// A group value. This is encoded by putting a start and end tag between its encoded fields.
    pub type Group<T>(T);
}

impl<T> Sealed for Group<T> { }
impl<T: TraitMessage> Value for Group<T> {
    const WIRE_TYPE: WireType = WireType::StartGroup;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        self.0.calculate_size(builder)
    }
    fn merge_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()> {
        self.0.merge_from(input)
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        self.0.write_to(output)
    }
    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self> {
        let mut t = T::new();
        t.merge_from(input)?;
        Ok(Self(t))
    }
}