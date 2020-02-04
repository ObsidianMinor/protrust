//! Contains types for protobuf values and traits for value operations.

use alloc::vec::Vec;
use core::convert::TryInto;
use crate::{internal::Sealed, Message as TraitMessage};
use crate::extend::ExtendableMessage;
use crate::io::{self, read, write, WireType, ByteString, Length, LengthBuilder, CodedReader, CodedWriter, Input, Output};

/// A protobuf value type paired with a Rust type used to represent that type in generated code.
/// 
/// Multiple value types may have the same Rust type representation.
pub trait ValueType: Sealed {
    /// The Rust type used to represent this protobuf type in code.
    type Inner;
}
/// A value capable of merging itself with an input value, writing itself to an output, calculating it's size, and checking it's initialization.
pub trait Value: ValueType {
    /// A value indicating the wire type of the value without packing.
    /// This can be used to indicate if a value is elegible for repeated field packing.
    const WIRE_TYPE: WireType;

    /// Calculates the size of the value as encoded on the wire
    fn calculate_size(this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder>;

    /// Merges the value with the [`CodedReader`](../io/read/struct.CodedReader.html)
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()>;

    /// Writes the value to the [`CodedWriter`](../io/write/struct.CodedWriter.html)
    fn write_to<T: Output>(this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result;

    /// Returns whether the value is initialized, that is, if all the required fields in the value are set.
    fn is_initialized(this: &Self::Inner) -> bool;

    /// Reads a new instance of the value
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner>;
}
/// A value with a constant size. This can be specialized over to enable certain optimizations with size caculations.
pub trait ConstSized: Value {
    /// The constant size of the value
    const SIZE: Length;
}
/// A trait indicating whether a value type can be packed or not. Used in conjunction with the [`Packed`](struct.Packed.html) struct.
pub trait Packable: Value { }

/// A packed value type used with packed repeated fields.
pub struct Packed<V: Packable>(V);
impl<V: Packable> Sealed for Packed<V> { }
impl<V: Packable> ValueType for Packed<V> {
    type Inner = V::Inner;
}

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
pub struct Int32(i32);
impl Sealed for Int32 { }
impl ValueType for Int32 {
    type Inner = i32;
}
impl Value for Int32 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        if this >= 0 {
            builder.add_bytes(io::raw_varint32_size(this as u32))
        } else {
            builder.add_bytes(MAX_VARINT64_SIZE)
        }
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        if this >= 0 {
            output.write_varint32(this as u32)
        } else {
            output.write_varint64(i64::from(this) as u64)
        }
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint32().map(|v| v as i32)
    }
}

/// A varint encoded 32-bit value. Can be at most 5 bytes.
pub struct Uint32;
impl Sealed for Uint32 { }
impl ValueType for Uint32 {
    type Inner = u32;
}
impl Value for Uint32 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint32_size(this))
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(this)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint32()
    }
}

/// A varint encoded 64-bit value. Can be at most 10 bytes.
pub struct Int64;
impl Sealed for Int64 { }
impl ValueType for Int64 {
    type Inner = i64;
}
impl Value for Int64 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(this as u64))
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(this as u64)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint64().map(|v| v as i64)
    }
}

/// A varint encoded 64-bit value. Can be at most 10 bytes.
pub struct Uint64;
impl Sealed for Uint64 { }
impl ValueType for Uint64 {
    type Inner = u64;
}
impl Value for Uint64 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(this))
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(this)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint64()
    }
}

/// A varint encoded 32-bit value. This is encoded using zig-zag encoding, 
/// which makes it more effecient at encoding negative values.
pub struct Sint32;
impl Sealed for Sint32 { }
impl ValueType for Sint32 {
    type Inner = i32;
}
impl Value for Sint32 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint32_size(((this << 1) ^ (this >> 31)) as u32))
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(((this << 1) ^ (this >> 31)) as u32)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint32().map(|v| ((v >> 1) ^ (v << 31)) as i32)
    }
}

/// A varint encoded 64-bit value. This encoded using zig-zag encoding,
/// which makes it more effecient at encoding negative values.
pub struct Sint64;
impl Sealed for Sint64 { }
impl ValueType for Sint64 {
    type Inner = i64;
}
impl Value for Sint64 {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(io::raw_varint64_size(((this << 1) ^ (this >> 63)) as u64))
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint64(((this << 1) ^ (this >> 63)) as u64)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint64().map(|v| ((v >> 1) ^ (v << 63)) as i64)
    }
}

/// A fixed size 32-bit value. This is encoded as 4 little endian bytes.
pub struct Fixed32;
impl Sealed for Fixed32 { }
impl ValueType for Fixed32 {
    type Inner = u32;
}
impl Value for Fixed32 {
    const WIRE_TYPE: WireType = WireType::Bit32;

    fn calculate_size(_this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit32(this)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_bit32()
    }
}
impl ConstSized for Fixed32 {
    const SIZE: Length = unsafe { Length::new_unchecked(4) };
}

/// A fixed size 64-bit value. This is encoded as 8 little endian bytes.
pub struct Fixed64;
impl Sealed for Fixed64 { }
impl ValueType for Fixed64 {
    type Inner = u64;
}
impl Value for Fixed64 {
    const WIRE_TYPE: WireType = WireType::Bit64;

    fn calculate_size(_this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit64(this)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_bit64()
    }
}
impl ConstSized for Fixed64 {
    const SIZE: Length = unsafe { Length::new_unchecked(8) };
}

/// A signed, fixed size 32-bit value. This is encoded as 4 little endian bytes.
pub struct Sfixed32;
impl Sealed for Sfixed32 { }
impl ValueType for Sfixed32 {
    type Inner = i32;
}
impl Value for Sfixed32 {
    const WIRE_TYPE: WireType = WireType::Bit32;

    fn calculate_size(_this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit32(this as u32)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_bit32().map(|v| v as i32)
    }
}
impl ConstSized for Sfixed32 {
    const SIZE: Length = unsafe { Length::new_unchecked(4) };
}

/// A signed, fixed size 64-bit value. This is encoded as 8 little endian bytes.
pub struct Sfixed64;
impl Sealed for Sfixed64 { }
impl ValueType for Sfixed64 {
    type Inner = i64;
}
impl Value for Sfixed64 {
    const WIRE_TYPE: WireType = WireType::Bit64;

    fn calculate_size(_this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_bit64(this as u64)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_bit64().map(|v| v as i64)
    }
}
impl ConstSized for Sfixed64 {
    const SIZE: Length = unsafe { Length::new_unchecked(8) };
}

/// A bool value. This is encoded as a varint value
pub struct Bool;
impl Sealed for Bool { }
impl ValueType for Bool {
    type Inner = bool;
}
impl Value for Bool {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(_this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(Self::SIZE)
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(&this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_varint32(this as u32)
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        input.read_varint64().map(|v| v != 0)
    }
}
impl ConstSized for Bool {
    const SIZE: Length = unsafe { Length::new_unchecked(1) };
}

/// A string value. This is encoded as a length-delimited series of bytes.
pub struct String;
impl Sealed for String { }
impl ValueType for String {
    type Inner = alloc::string::String;
}
impl Value for String {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = this.len().try_into().ok()?;
        builder
            .add_value::<Uint32>(&(len as u32))?
            .add_bytes(unsafe { Length::new_unchecked(len) })
    }
    fn merge_from<T: Input>(this: &mut Self::Inner, input: &mut CodedReader<T>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<T: Output>(this: &Self::Inner, output: &mut CodedWriter<T>) -> write::Result {
        output.write_length_delimited(this.as_bytes())
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<T: Input>(input: &mut CodedReader<T>) -> read::Result<Self::Inner> {
        alloc::string::String::from_utf8(input.read_value::<Bytes<Vec<_>>>()?)
            .map_err(io::read::Error::InvalidString)
    }
}

/// A bytes value. This is encoded as a length-delimited series of bytes.
pub struct Bytes<T>(T);
impl<T> Sealed for Bytes<T> { }
impl<T: ByteString> ValueType for Bytes<T> {
    type Inner = T;
}
impl<T: ByteString> Value for Bytes<T> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len: i32 = this.as_ref().len().try_into().ok()?;
        builder
            .add_value::<Uint32>(&(len as u32))?
            .add_bytes(unsafe { Length::new_unchecked(len) })
    }
    fn merge_from<U: Input>(this: &mut Self::Inner, input: &mut CodedReader<U>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<U: Output>(this: &Self::Inner, output: &mut CodedWriter<U>) -> write::Result {
        output.write_length_delimited(this.as_ref())
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self::Inner> {
        input.read_length_delimited::<T>()
    }
}

/// An enum value. This is encoded as a 32-bit varint value.
pub struct Enum<T>(T);
impl<T> Sealed for Enum<T> { }
impl<T: crate::Enum> ValueType for Enum<T> {
    type Inner = T;
}
impl<T: crate::Enum> Value for Enum<T> {
    const WIRE_TYPE: WireType = WireType::Varint;

    fn calculate_size(&this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_value::<Int32>(&this.into())
    }
    fn merge_from<U: Input>(this: &mut Self::Inner, input: &mut CodedReader<U>) -> read::Result<()> {
        Self::read_new(input).map(|v| *this = v)
    }
    fn write_to<U: Output>(&this: &Self::Inner, output: &mut CodedWriter<U>) -> write::Result {
        output.write_value::<Int32>(&this.into())
    }
    fn is_initialized(_this: &Self::Inner) -> bool { true }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self::Inner> {
        Int32::read_new(input).map(|v| v.into())
    }
}

/// A message value. This is encoded as a length-delimited series of bytes.
pub struct Message<T>(T);
impl<T> Sealed for Message<T> { }
impl<T: TraitMessage> ValueType for Message<T> {
    type Inner = T;
}
impl<T: TraitMessage> Value for Message<T> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    fn calculate_size(this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        let len = this.calculate_size()?;
        builder
            .add_value::<Uint32>(&(len.get() as u32))?
            .add_bytes(len)
    }
    fn merge_from<U: Input>(this: &mut Self::Inner, input: &mut CodedReader<U>) -> read::Result<()> {
        input.read_limit()?.then(|input| input.recurse(|input| this.merge_from(input)))
    }
    fn write_to<U: Output>(this: &Self::Inner, output: &mut CodedWriter<U>) -> write::Result {
        let length = this.calculate_size().ok_or(io::write::Error::ValueTooLarge)?;
        output.write_length(length)?;
        TraitMessage::write_to::<U>(this, output)?;
        Ok(())
    }
    fn is_initialized(this: &Self::Inner) -> bool {
        this.is_initialized()
    }
    default fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self::Inner> {
        input.recurse(|input| {
            let mut t = T::default();
            t.merge_from(input)?;
            Ok(t)
        })
    }
}
impl<T: TraitMessage + ExtendableMessage + 'static> Value for Message<T> {
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self::Inner> {
        let mut t = T::default();
        t.extensions_mut().replace_registry(input.registry());
        t.merge_from(input)?;
        Ok(t)
    }
}

/// A group value. This is encoded by putting a start and end tag between its encoded fields.
pub struct Group<T>(T);
impl<T> Sealed for Group<T> { }
impl<T: TraitMessage> ValueType for Group<T> {
    type Inner = T;
}
impl<T: TraitMessage> Value for Group<T> {
    const WIRE_TYPE: WireType = WireType::StartGroup;

    fn calculate_size(this: &Self::Inner, builder: LengthBuilder) -> Option<LengthBuilder> {
        builder.add_bytes(this.calculate_size()?)
    }
    fn merge_from<U: Input>(this: &mut Self::Inner, input: &mut CodedReader<U>) -> read::Result<()> {
        input.recurse(|input| this.merge_from(input))
    }
    fn write_to<U: Output>(this: &Self::Inner, output: &mut CodedWriter<U>) -> write::Result {
        this.write_to(output)
    }
    fn is_initialized(this: &Self::Inner) -> bool {
        this.is_initialized()
    }
    fn read_new<U: Input>(input: &mut CodedReader<U>) -> read::Result<Self::Inner> {
        let mut t = T::default();
        t.merge_from(input)?;
        Ok(t)
    }
}

#[cfg(test)]
mod test {
    macro_rules! test_cases {
        ($t:ty => {
            $($tt:ident: $id:ident => $tokens:tt),+ $(,)?
        }) => {
            $(
                case!($t => $tt: $id => $tokens);
            )*
        };
    }

    macro_rules! case {
        ($t:ty => write: $id:ident => {
            $($i:expr => $o:expr),+ $(,)?
        }) => {
            #[test]
            fn $id() {
                $({
                    use crate::io::CodedWriter;

                    let value = $i;
                    let expected = $o;
                    let mut output = alloc::vec![0; expected.len()].into_boxed_slice();
                    let mut writer = CodedWriter::with_slice(&mut output);

                    writer.write_value::<$t>(&value).expect("failed to write value");

                    let remaining = writer.into_inner();

                    assert!(remaining.is_empty());
                    assert_eq!(output.as_ref(), &expected);
                })+
            }
        };
        ($t:ty => size: $id:ident => {
            $($i:expr => $s:expr),+ $(,)?
        }) => {
            #[test]
            fn $id() {
                $({
                    let input = $i;
                    assert_eq!(crate::io::Length::of_value::<$t>(&input), $s);
                })+
            }
        };
        ($t:ty => read: $id:ident => {
            $($i:expr => $o:pat $(if $e:expr)?),+ $(,)?
        }) => {
            #[test]
            fn $id() {
                $({
                    use crate::io::CodedReader;

                    let input = $i;
                    let mut reader = CodedReader::with_slice(&input);

                    let result = reader.read_value::<$t>();

                    assert!(matches!(result, $o $(if $e)?))
                })+
            }
        };
    }

    mod int32 {
        use crate::io::Length;
        use crate::raw::Int32;

        test_cases! {
            Int32 => {
                write: write_int32 => {
                    0 => [0],
                    1 => [1],
                    127 => [127],
                    128 => [128, 1],
                    16383 => [255, 127],
                    16384 => [128, 128, 1],
                    2097151 => [255, 255, 127],
                    2097152 => [128, 128, 128, 1],
                    268435455 => [255, 255, 255, 127],
                    268435456 => [128, 128, 128, 128, 1],
                    -1 => [255, 255, 255, 255, 255, 255, 255, 255, 255, 1],
                    i32::min_value() => [128, 128, 128, 128, 248, 255, 255, 255, 255, 1],
                },
                size: calculate_int32_size => {
                    0                => Length::new(1),
                    1                => Length::new(1),

                    127              => Length::new(1),
                    128              => Length::new(2),

                    16383            => Length::new(2),
                    16384            => Length::new(3),

                    2097151          => Length::new(3),
                    2097152          => Length::new(4),

                    268435455        => Length::new(4),
                    268435456        => Length::new(5),

                    i32::max_value() => Length::new(5),

                    -1               => Length::new(10),
                    i32::min_value() => Length::new(10),
                },
                read: read_int32 => {
                    [0] => Ok(0),
                    [1] => Ok(1),
                    [127] => Ok(127),
                    [128, 1] => Ok(128),
                    [128, 128, 1] => Ok(16384),
                    [255, 255, 255, 255, 255, 255, 255, 255, 255, 1] => Ok(-1),
                },
            }
        }
    }
    mod uint32 {
        use crate::io::Length;
        use crate::raw::Uint32;

        test_cases! {
            Uint32 => {
                write: write_uint32 => {
                    0 => [0],
                    1 => [1],
                    127 => [127],
                    128 => [128, 1],
                    16383 => [255, 127],
                    16384 => [128, 128, 1],
                    2097151 => [255, 255, 127],
                    2097152 => [128, 128, 128, 1],
                    268435455 => [255, 255, 255, 127],
                    268435456 => [128, 128, 128, 128, 1],
                },
                size: calculate_uint32_size => {
                    0                => Length::new(1),
                    1                => Length::new(1),

                    127              => Length::new(1),
                    128              => Length::new(2),

                    16383            => Length::new(2),
                    16384            => Length::new(3),

                    2097151          => Length::new(3),
                    2097152          => Length::new(4),

                    268435455        => Length::new(4),
                    268435456        => Length::new(5),

                    u32::max_value() => Length::new(5),
                },
                read: read_uint32 => {
                    [0] => Ok(0),
                    [1] => Ok(1),
                    [127] => Ok(127),
                    [128, 1] => Ok(128),
                    [255, 127] => Ok(16383),
                    [128, 128, 1] => Ok(16384),
                    [255, 255, 127] => Ok(2097151),
                    [128, 128, 128, 1] => Ok(2097152),
                    [255, 255, 255, 127] => Ok(268435455),
                    [128, 128, 128, 128, 1] => Ok(268435456),
                }
            }
        }
    }
    mod int64 {

    }
    mod uint64 {

    }
    mod sint32 {

    }
    mod sint64 {

    }
    mod fixed32 {

    }
    mod fixed64 {

    }
    mod sfixed32 {

    }
    mod sfixed64 {

    }
    mod r#bool {
        use crate::raw::Bool;

        test_cases! {
            Bool => {
                write: write_bool => {
                    false => [0],
                    true  => [1],
                },
                read: read_bool => {
                    [0] => Ok(false),
                    [128, 128, 128, 128, 128, 128, 128, 128, 128, 0] => Ok(false),

                    [1] => Ok(true),
                    [128, 128, 128, 128, 128, 128, 128, 128, 128, 1] => Ok(true),
                },
            }
        }
    }
    mod string {

    }
    mod bytes {
        use alloc::vec;
        use alloc::vec::Vec;
        use crate::io::Length;
        use crate::raw::Bytes;

        test_cases! {
            Bytes<Vec<u8>> => {
                size: calculate_bytes_size => {
                    vec![]        => Length::new(1),
                    vec![1, 2, 3] => Length::new(4),
                    vec![0; 128]  => Length::new(130),
                },
                read: read_bytes => {
                    [0] => Ok(val) if val == [],
                    [3, 1, 2, 3] => Ok(val) if val == [1, 2, 3],
                },
                write: write_bytes => {
                    vec![] => [0],
                    vec![1, 2, 3] => [3, 1, 2, 3],
                }
            }
        }
    }
    mod r#enum {
        use core::fmt::{self, Debug, Formatter};
        use crate::io::Length;
        use crate::raw::Enum;

        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct FooBar(pub i32);

        impl FooBar {
            pub const NEGATIVE: FooBar = FooBar(-1);
            pub const DEFAULT: FooBar = FooBar(0);
            pub const XYZZY: FooBar = FooBar(1);
            pub const ALIAS: FooBar = FooBar(1);
        }

        impl From<i32> for FooBar {
            fn from(i: i32) -> Self {
                Self(i)
            }
        }

        impl From<FooBar> for i32 {
            fn from(f: FooBar) -> i32 {
                f.0
            }
        }

        impl Debug for FooBar {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match *self {
                    FooBar::NEGATIVE => write!(f, "NEGATIVE"),
                    FooBar::DEFAULT => write!(f, "DEFAULT"),
                    FooBar::XYZZY => write!(f, "XYZZY"),
                    FooBar(i) => i.fmt(f),
                }
            }
        }

        impl crate::Enum for FooBar { }

        test_cases! {
            Enum<FooBar> => {
                size: calculate_enum_size => {
                    FooBar::DEFAULT  => Length::new(1),
                    FooBar::XYZZY    => Length::new(1),
                    FooBar::ALIAS    => Length::new(1),
                    FooBar::NEGATIVE => Length::new(10),

                    FooBar(i32::max_value()) => Length::new(5),
                },
                read: read_enum => {
                    [255, 255, 255, 255, 255, 255, 255, 255, 255, 1] => Ok(FooBar::NEGATIVE),
                    [0] => Ok(FooBar::DEFAULT),
                    [1] => Ok(FooBar::XYZZY),
                    [1] => Ok(FooBar::ALIAS),

                    [127] => Ok(FooBar(127)),
                },
                write: write_enum => {
                    FooBar::NEGATIVE => [255, 255, 255, 255, 255, 255, 255, 255, 255, 1],
                    FooBar::DEFAULT => [0],
                    FooBar::XYZZY => [1],
                    FooBar::ALIAS => [1],
                    FooBar(127) => [127],
                }
            }
        }
    }
    mod message {

    }
    mod group {

    }
}