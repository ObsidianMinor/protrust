//! Defines the `UnknownFieldSet`, a struct used to contain unknown fields as they were read from coded readers.
//! 
//! As APIs are updated, certain fields may be removed or added from proto file definitions. If an old version of a message 
//! encounters fields it doesn't recognize can still read them to be returned again via unknown fields.
//! 
//! Unknown fields for unique field numbers can exist for multiple wire types at once to ensure that all data is properly returned.

use crate::{internal, FieldSet, FieldReadState};
use crate::io::{FieldNumber, WireType, Tag, LengthBuilder, CodedReader, ReaderResult, CodedWriter, WriterResult};
use crate::raw;

/// An unknown field in an [`UnknownFieldSet`](struct.UnknownFieldSet.html).
#[derive(Clone, Debug)]
pub enum UnknownField {
    /// A varint field value
    Varint(u64),
    /// A 64-bit field value
    Bit64(u64),
    /// A length delimited series of bytes
    LengthDelimited(Box<[u8]>),
    /// A group of other unknown fields
    Group(UnknownFieldSet),
    /// A 32-bit field value
    Bit32(u32)
}

/// A set of unknown fields encountered while parsing
#[derive(Default, Clone, Debug)]
pub struct UnknownFieldSet(hashbrown::HashMap<FieldNumber, Vec<UnknownField>>);

impl internal::Sealed for UnknownFieldSet { }
impl FieldSet for UnknownFieldSet {
    #[inline]
    fn try_add_field_from<'a, 'b>(&mut self, input: &'a mut CodedReader<'b>) -> ReaderResult<FieldReadState<'a, 'b>> {
        if input.skip_unknown_fields() || input.last_tag().map(Tag::wire_type) == Some(WireType::EndGroup) {
            Ok(FieldReadState::Yielded(input))
        } else {
            self.add_field_from(input)?;
            Ok(FieldReadState::Consumed)
        }
    }
    fn calculate_size(&self, mut builder: LengthBuilder) -> Option<LengthBuilder> {
        for (key, values) in &self.0 {
            for value in values {
                match value {
                    UnknownField::Varint(v) => {
                        builder = builder
                            .add_tag(Tag::new(*key, WireType::Varint))?
                            .add_value::<raw::Uint64>(v)?;
                    },
                    UnknownField::Bit64(v) => {
                        builder = builder
                            .add_tag(Tag::new(*key, WireType::Bit64))?
                            .add_value::<raw::Fixed64>(v)?;
                    },
                    UnknownField::LengthDelimited(v) => {
                        builder = builder
                            .add_tag(Tag::new(*key, WireType::LengthDelimited))?
                            .add_value::<raw::Bytes<_>>(v)?;
                    },
                    UnknownField::Group(v) => {
                        builder = builder
                            .add_tag(Tag::new(*key, WireType::StartGroup))?
                            .add_fields(v)?
                            .add_tag(Tag::new(*key, WireType::EndGroup))?;
                    },
                    UnknownField::Bit32(v) => {
                        builder = builder
                                .add_tag(Tag::new(*key, WireType::Bit32))?
                                .add_value::<raw::Fixed32>(v)?;
                    }
                }
            }
        }
        Some(builder)
    }
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult {
        for (key, values) in &self.0 {
            for value in values {
                match value {
                    UnknownField::Varint(v) => {
                        output.write_tag(Tag::new(*key, WireType::Varint))?;
                        output.write_varint64(*v)?;
                    },
                    UnknownField::Bit64(v) => {
                        output.write_tag(Tag::new(*key, WireType::Bit64))?;
                        output.write_bit64(*v)?;
                    },
                    UnknownField::LengthDelimited(v) => {
                        output.write_tag(Tag::new(*key, WireType::LengthDelimited))?;
                        output.write_length_delimited(v)?;
                    },
                    UnknownField::Group(v) => {
                        output.write_tag(Tag::new(*key, WireType::StartGroup))?;
                        output.write_fields(v)?;
                        output.write_tag(Tag::new(*key, WireType::EndGroup))?;
                    },
                    UnknownField::Bit32(v) => {
                        output.write_tag(Tag::new(*key, WireType::Bit32))?;
                        output.write_bit32(*v)?;
                    },
                }
            }
        }
        Ok(())
    }
    fn is_initialized(&self) -> bool { true }
    fn merge(&mut self, other: &Self) {
        for (key, values) in &other.0 {
            self.0.entry(*key).or_insert_with(Vec::new).extend(values.iter().cloned())
        }
    }
}

impl UnknownFieldSet {
    fn add_field_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        if let Some(last_tag) = input.last_tag() {
            match last_tag.wire_type() {
                WireType::Varint => self.add_value(last_tag.number(), UnknownField::Varint(dbg!(input.read_varint64()?))),
                WireType::Bit64 => self.add_value(last_tag.number(), UnknownField::Bit64(input.read_bit64()?)),
                WireType::LengthDelimited => self.add_value(last_tag.number(), UnknownField::LengthDelimited(input.read_length_delimited()?)),
                WireType::StartGroup => {
                    let mut group = UnknownFieldSet::new();
                    let end_tag = Tag::new(last_tag.number(), WireType::EndGroup);
                    while let Some(tag) = input.read_tag()? {
                        if tag != end_tag {
                            group.add_field_from(input)?;
                        } else {
                            break;
                        }
                    }
                    self.add_value(last_tag.number(), UnknownField::Group(group));
                },
                WireType::Bit32 => self.add_value(last_tag.number(), UnknownField::Bit32(input.read_bit32()?)),
                WireType::EndGroup => unreachable!()
            }
        }
        Ok(())
    }
    /// Creates a new unknown field set
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds an unknown field value to the set
    pub fn add_value(&mut self, num: FieldNumber, field: UnknownField) {
        self.0.entry(num).or_insert_with(Vec::new).push(field);
    }
    /// Gets the unknown fields present for the specified field number
    pub fn fields_for(&self, num: FieldNumber) -> &[UnknownField] {
        self.0.get(&num).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[cfg(test)]
mod test {
    use super::{UnknownFieldSet, UnknownField};
    use crate::io::{Tag, FieldNumber, WireType, Length, CodedWriter, CodedReader};
    use crate::raw;

    #[test]
    fn sizes() {
        let set = {
            let mut set = UnknownFieldSet::new();
            set.add_value(FieldNumber::new(1).unwrap(), UnknownField::Bit32(513));
            set
        };
        assert_eq!(Length::for_fields(&set).unwrap().get(), 5);

        let set = {
            let mut set = UnknownFieldSet::new();
            set.add_value(FieldNumber::new(1).unwrap(), UnknownField::Bit32(513));
            set.add_value(FieldNumber::new(16).unwrap(), UnknownField::Bit32(1));
            set
        };

        assert_eq!(Length::for_fields(&set).unwrap().get(), 11);

        let set = {
            let group = set;
            let mut set = UnknownFieldSet::new();
            set.add_value(FieldNumber::new(1).unwrap(), UnknownField::Group(group));
            set
        };

        assert_eq!(Length::for_fields(&set).unwrap().get(), 13);
    }
    #[test]
    fn read() {
        let mut set = UnknownFieldSet::new();
        set.add_value(FieldNumber::new(1).unwrap(), UnknownField::Varint(120));

        let number = FieldNumber::new(1).unwrap();
        let input = {
            let mut data = Vec::new();
            let mut output = CodedWriter::with_write(&mut data);

            output.write_value_with_number::<raw::Int32>(number, &10).unwrap();
            output.write_value_with_number::<raw::Int64>(number, &10).unwrap();
            output.write_value_with_number::<raw::Fixed32>(number, &10).unwrap();
            output.write_value_with_number::<raw::Fixed64>(number, &10).unwrap();

            output.write_tag(Tag::new(number, WireType::StartGroup)).unwrap();
            output.write_value_with_number::<raw::Int32>(FieldNumber::new(2).unwrap(), &10).unwrap();
            output.write_tag(Tag::new(number, WireType::EndGroup)).unwrap();

            data.into_boxed_slice()
        };

        dbg!(&input);

        let mut reader = CodedReader::with_slice(&input);

        dbg!(reader.read_tag().unwrap());
        reader.try_add_field_to(&mut set).unwrap().or_skip().unwrap();
        dbg!(reader.read_tag().unwrap());
        reader.try_add_field_to(&mut set).unwrap().or_skip().unwrap();
        dbg!(reader.read_tag().unwrap());
        reader.try_add_field_to(&mut set).unwrap().or_skip().unwrap();
        dbg!(reader.read_tag().unwrap());
        reader.try_add_field_to(&mut set).unwrap().or_skip().unwrap();
        dbg!(reader.read_tag().unwrap());
        reader.try_add_field_to(&mut set).unwrap().or_skip().unwrap();

        assert_eq!(set.fields_for(number).len(), 6); // existing field + 5 added fields. the group counts as one field
    }
}