//! A fast, feature complete, protobuf implementation.

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
pub mod unknown_fields;

use crate::io::{LengthBuilder, CodedReader, ReaderResult, CodedWriter, WriterResult};
use std::fmt::Debug;
use std::hash::Hash;

pub use unknown_fields::UnknownFieldSet;

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