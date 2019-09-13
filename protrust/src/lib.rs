//! # protrust

#![feature(new_uninit)]
#![feature(read_initializer)]
#![feature(const_fn)]
#![feature(specialization)]

#![warn(missing_docs)]

mod internal {
    pub trait Sealed { }
}
pub mod collections;
pub mod io;
pub mod raw;

use crate::io::{FieldNumber, WireType, Tag, LengthBuilder, CodedReader, ReaderResult, CodedWriter, WriterResult};
use std::fmt::Debug;
use std::hash::Hash;

/// An object-safe message value that can merge from an input, calculate its size, write to an output, and get its initialization state
pub trait CodableMessage {
    /// Merges this message with data from the specified [`CodedReader`](io/struct.CodedReader.html)
    fn merge_from(&mut self, input: &mut CodedReader) -> ReaderResult<()>;
    /// Adds the size of the data in the message to the [`LengthBuilder`](io/struct.LengthBuilder.html)
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
    /// Writes this message's data to the specified [`CodedWriter`](io/struct.CodedWriter.html)
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult;
    /// Returns if the message value is initialized
    fn is_initialized(&self) -> bool;
}

/// A LITE message. 
pub trait LiteMessage: CodableMessage + Clone + Default + Debug {
    /// Gets a shared reference to the unknown fields in this message
    fn unknown_fields(&self) -> &UnknownFieldSet;
    /// Gets a unique reference to the unknown fields in this message
    fn unknown_fields_mut(&mut self) -> &mut UnknownFieldSet;

    /// Merges another instance of this message into this one
    fn merge(&mut self, other: &Self);

    /// Creates a new instance of the message
    fn new() -> Self {
        Self::default()
    }
    /// Reads a new instance of the message from a [`CodedReader`](io/struct.CodedReader.html)
    fn new_from(input: &mut CodedReader) -> ReaderResult<Self> {
        let mut instance = Self::new();
        instance.merge_from(input)?;
        Ok(instance)
    }
}

/// A marker trait used to mark enum types in generated code.
/// This defines all the main traits the enum types implement,
/// allowing code to refer to them easily.
pub trait Enum: From<i32> + Into<i32> + Clone + Copy + Debug + Hash { }

/// The result of trying to add a field to a field set
pub enum FieldReadState<'a, 'b> {
    /// The set didn't read the field. Sets should return the
    /// borrowed reader to allow other sets to possibly read the field
    Yielded(&'a mut CodedReader<'b>),
    /// The set read the field, consuming it
    Consumed,
}

impl<'a, 'b> FieldReadState<'a, 'b> {
    /// Tries to read the field into the specified set. If the field has already been read, this does nothing.
    #[inline]
    pub fn or_try(self, set: &mut impl FieldSet) -> ReaderResult<FieldReadState<'a, 'b>> {
        match self {
            FieldReadState::Yielded(input) => input.try_add_field_to(set),
            FieldReadState::Consumed => Ok(FieldReadState::Consumed),
        }
    }
    /// Skips the field if it hasn't already been read
    #[inline]
    pub fn or_skip(self) -> ReaderResult<()> {
        match self {
            FieldReadState::Yielded(input) => input.skip(),
            FieldReadState::Consumed => Ok(()),
        }
    }
}

/// A set of fields. This unifies unknown fields, extension fields, and any other future field set types
pub trait FieldSet: internal::Sealed {
    /// Checks if the set can read the field from the input and reads it if it can. It returns a state indicating if the field was read.
    fn try_add_field_from<'a, 'b>(&mut self, input: &'a mut CodedReader<'b>) -> ReaderResult<FieldReadState<'a, 'b>>;
    /// Calculates the size of all the fields in this set
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
    /// Writes the fields in this set to the writer
    fn write_to(&self, output: &mut CodedWriter) -> WriterResult;
    /// Returns if all the fields in this set are initialized
    fn is_initialized(&self) -> bool;
    /// Merges another set of fields into this one
    fn merge(&mut self, other: &Self);
}

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
                WireType::Varint => self.add_value(last_tag.number(), UnknownField::Varint(input.read_varint64()?)),
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
}

#[cfg(test)]
mod test {
    mod unknown_field_set {
        use crate::{UnknownFieldSet, UnknownField};
        use crate::io::{FieldNumber, Length};

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
    }
}