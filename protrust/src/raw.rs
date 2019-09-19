//! Contains types for protobuf values and traits for value operations. 
//! Each value with specific serialization or deserialization, a specific 

use crate::{internal::Sealed, CodableMessage, LiteMessage};
use crate::io::{self, WireType, ByteString, Length, LengthBuilder, CodedReader, ReaderResult, CodedWriter, WriterResult};
use std::alloc::Global;
use std::convert::TryInto;
use trapper::{newtype, Wrapper};

/// A value capable of merging itself with an input value, writing itself to an output, calculating it's size, and checking it's initialization.
pub trait Value: Sized + Sealed {
    /// A value indicating the wire type of the value without packing.
    /// This can be used to indicate if a value is elegible for repeated field packing.
    const WIRE_TYPE: WireType;

    /// Calculates the size of the value as encoded on the wire
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;

    /// Merges the value with the [`CodedReader`](../io/struct.CodedReader.html)
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()>;

    /// Writes the value to the [`CodedWriter`](../io/struct.CodedWriter.html)
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult;

    /// Returns if the value is initialized, that is, if all the required fields in the value are set.
    fn is_initialized(&self) -> bool;
}
/// A value which does not allocate any dynamic memory and can be read without providing an allocator.
pub trait Primitive: Value {
    /// Reads a new instance of the value
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self>;
}
/// A value which may allocate dynamic memory into a specified allocator.
pub trait Heaping: Value {
    /// The allocator type associated with this value
    type Alloc;

    /// Reads a new instance of this value from the input. This may allocate data into the provided allocator.
    fn read_new(input: &mut CodedReader, a: Self::Alloc) -> ReaderResult<Self>;
}
/// A value with a constant size. This can be specialized over to enable certain optimizations with size caculations.
pub trait ConstSized: Value {
    /// The constant size of the value
    const SIZE: i32;
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
            builder.add_bytes(raw_varint32_size(self.0 as u32).get())
        } else {
            builder.add_bytes(10)
        }
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        if self.0 >= 0 {
            output.write_varint32(self.0 as u32)
        } else {
            output.write_varint64(i64::from(self.0) as u64)
        }
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Int32 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
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
        builder.add_bytes(raw_varint32_size(self.0).get())
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_varint32(self.0)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Uint32 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
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
        builder.add_bytes(raw_varint64_size(self.0 as u64).get())
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_varint64(self.0 as u64)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Int64 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
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
        builder.add_bytes(raw_varint64_size(self.0).get())
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_varint64(self.0)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Uint64 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
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
        builder.add_bytes(raw_varint32_size((n << 1) ^ (n >> 31)).get())
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        let n = self.0 as u32;
        output.write_varint32((n << 1) ^ (n >> 31))
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Sint32 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
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
        builder.add_bytes(raw_varint64_size((n << 1) ^ (n >> 63)).get())
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        let n = self.0 as u64;
        output.write_varint64((n << 1) ^ (n >> 63))
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Sint64 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
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
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_bit32(self.0)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Fixed32 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
        input.read_bit32().map(Self)
    }
}
impl ConstSized for Fixed32 {
    const SIZE: i32 = 4;
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
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_bit64(self.0)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Fixed64 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
        input.read_bit64().map(Self)
    }
}
impl ConstSized for Fixed64 {
    const SIZE: i32 = 8;
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
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_bit32(self.0 as u32)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Sfixed32 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
        input.read_bit32().map(|v| Self(v as i32))
    }
}
impl ConstSized for Sfixed32 {
    const SIZE: i32 = 4;
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
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_bit64(self.0 as u64)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Sfixed64 {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
        input.read_bit64().map(|v| Self(v as i64))
    }
}
impl ConstSized for Sfixed64 {
    const SIZE: i32 = 8;
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
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_varint32(self.0 as u32)
    }
    fn is_initialized(&self) -> bool { true }
}
impl Primitive for Bool {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
        input.read_varint32().map(|v| Self(v != 0))
    }
}
impl ConstSized for Bool {
    const SIZE: i32 = 1;
}

newtype! {
    /// A string value. This is encoded as a length-delimited series of bytes.
    pub type String(std::string::String);
}

impl Sealed for String { }
impl Value for String {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = self.0.len().try_into().ok()?;
        builder.add_bytes(len)
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input, Global).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_length_delimited(self.0.as_bytes())
    }
    fn is_initialized(&self) -> bool { true }
}
impl Heaping for String {
    type Alloc = Global;

    fn read_new(input: &mut CodedReader, a: Global) -> ReaderResult<Self> {
        std::string::String::from_utf8(input.read_value_in::<Bytes<_>>(a)?)
            .map_err(io::ReaderError::InvalidString)
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
        builder.add_bytes(len)
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        input.merge_length_delimited(&mut self.0)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        output.write_length_delimited(self.0.as_ref())
    }
    fn is_initialized(&self) -> bool { true }
}
impl<T: ByteString> Heaping for Bytes<T> {
    type Alloc = T::Alloc;

    fn read_new(input: &mut CodedReader, a: T::Alloc) -> ReaderResult<Self> {
        input.read_length_delimited::<T>(a).map(|v| Self(v.into()))
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
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        Self::read_new(input).map(|v| *self = v)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        Int32(self.0.into()).write_to(output)
    }
    fn is_initialized(&self) -> bool { true }
}
impl<T: crate::Enum> Primitive for Enum<T> {
    fn read_new(input: &mut CodedReader) -> ReaderResult<Self> {
        Int32::read_new(input).map(|v| Self(v.0.into()))
    }
}

newtype! {
    /// A message value. This is encoded as a length-delimited series of bytes.
    pub type Message<T>(T);
}

impl<T> Sealed for Message<T> { }
impl<T: CodableMessage> Value for Message<T> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        self.0.calculate_size(builder)
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        let old = input.read_and_push_length()?;
        self.0.merge_from(input)?;
        input.pop_length(old);
        Ok(())
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        let length = Length::for_value::<Self>(&self.0).ok_or(io::WriterError::ValueTooLarge)?;
        output.write_length(length)?;
        self.0.write_to(output)?;
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
}
impl<T: LiteMessage> Heaping for Message<T> {
    type Alloc = T::Alloc;

    fn read_new(input: &mut CodedReader, a: T::Alloc) -> ReaderResult<Self> {
        let mut t = Self::wrap(T::new(a));
        t.merge_from(input)?;
        Ok(t)
    }
}

newtype! {
    /// A group value. This is encoded by putting a start and end tag between its encoded fields.
    pub type Group<T>(T);
}

impl<T> Sealed for Group<T> { }
impl<T: CodableMessage> Value for Group<T> {
    const WIRE_TYPE: WireType = WireType::StartGroup;

    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        self.0.calculate_size(builder)
    }
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        self.0.merge_from(input)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        self.0.write_to(output)
    }
    fn is_initialized(&self) -> bool {
        self.0.is_initialized()
    }
}
impl<T: LiteMessage> Heaping for Group<T> {
    type Alloc = T::Alloc;

    fn read_new(input: &mut CodedReader, a: T::Alloc) -> ReaderResult<Self> {
        T::new_from(input, a).map(Self)
    }
}

#[inline]
pub(crate) const fn raw_varint32_size(value: u32) -> Length {
    Length((((31 ^ (value | 1).leading_zeros()) * 9 + 73) / 64) as i32)
}

#[inline]
pub(crate) const fn raw_varint64_size(value: u64) -> Length {
    Length((((63 ^ (value | 1).leading_zeros()) * 9 + 73) / 64) as i32)
}