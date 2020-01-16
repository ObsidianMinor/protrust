//! A fast, feature complete, protobuf implementation.

#![feature(const_fn)]
#![feature(read_initializer)]
#![feature(specialization)]
#![feature(box_into_raw_non_null)]
#![feature(new_uninit)]
#![feature(manually_drop_take)]
#![feature(matches_macro)]
#![feature(const_if_match)]
#![feature(slice_ptr_range)]
#![feature(bool_to_option)]
#![feature(vec_drain_as_slice)]
#![feature(exact_size_is_empty)]

#![warn(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("This library does not support 16-bit platforms");

extern crate alloc;

mod internal {
    pub trait Sealed { }
}
pub mod collections;
pub mod extend;
pub mod io;
pub mod raw;

use core::fmt::Debug;
use core::hash::Hash;
use crate::io::{read, write, LengthBuilder, CodedReader, CodedWriter, Input, Output};

pub use collections::unknown_fields::UnknownFieldSet;

/// A message value.
pub trait Message: Sized {
    /// Merges this message with data from the [`CodedReader`](io/read/struct.CodedReader.html) of the specified type.
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()>;
    /// Adds the size of the data in the message to the [`LengthBuilder`](io/struct.LengthBuilder.html)
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
    /// Writes this message's data to the [`CodedWriter`](io/write/struct.CodedWriter.html) of the specified type.
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result;
    /// Returns whether the message value is initialized
    fn is_initialized(&self) -> bool;

    /// Gets a shared reference to the unknown fields in this message
    fn unknown_fields(&self) -> &UnknownFieldSet;
    /// Gets a unique reference to the unknown fields in this message
    fn unknown_fields_mut(&mut self) -> &mut UnknownFieldSet;

    /// Creates a new instance of the message
    fn new() -> Self;
}

/// A marker trait used to mark enum types in generated code.
/// This defines all the main traits the enum types implement,
/// allowing code to refer to them easily.
pub trait Enum: From<i32> + Into<i32> + Clone + Copy + Debug + Hash { }

/// A type that can be merged with one of `T`.
/// Merge behavior is specific to each type, the default behavior for clonable types clones from the other value.
pub trait Mergable<T = Self>: Sized {
    /// Merges another value into this one
    fn merge(&mut self, other: &T);
}

default impl<T: Clone> Mergable for T {
    fn merge(&mut self, other: &T) {
        self.clone_from(other)
    }
}

/// Merges two values together.
/// 
/// Internally uses an alias to `Mergable::merge`
pub fn merge<T: Mergable<V>, V>(value: &mut T, other: &V) {
    value.merge(other)
}