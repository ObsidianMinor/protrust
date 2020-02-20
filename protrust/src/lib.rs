//! A fast, feature complete, protobuf implementation.

#![feature(const_fn)]
#![feature(specialization)]
#![feature(const_if_match)]
#![feature(slice_ptr_range)]
#![feature(bool_to_option)]
#![feature(vec_drain_as_slice)]
#![feature(exact_size_is_empty)]
#![feature(result_copied)]
#![feature(read_initializer)]
#![feature(hash_raw_entry)]

#![warn(missing_docs)]

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("This library does not support 16-bit platforms");

#[doc(hidden)]
pub extern crate self as protrust;

mod internal {
    pub trait Sealed { }
}

#[doc(hidden)]
pub mod gen_prelude {
    pub use u8;
    pub use i32;
    pub use u32;
    pub use i64;
    pub use u64;
    pub use f32;
    pub use f64;
    pub use bool;
    pub use str;

    pub use ::protrust as p;
    pub use ::protrust::collections as pc;
    pub use ::protrust::extend as pe;
    pub use ::protrust::io as pio;
    pub use ::protrust::raw as pr;
    pub use ::protrust::reflect as prefl;

    pub use ::std::boxed::Box;
    pub use ::std::convert::{From, AsRef};
    pub use ::std::default::Default;
    pub use ::std::fmt::{self, Formatter, Debug};
    pub use ::std::option::Option;
    pub use ::std::option::Option::Some;
    pub use ::std::option::Option::None;
    pub use ::std::result::Result::Ok;
    pub use ::std::string::String;
    pub use ::std::vec::Vec;

    pub type ByteVec = ::std::vec::Vec<u8>;

    pub use ::protrust::{Message, Initializable, Enum, UnknownFieldSet};
    pub use ::protrust::collections::{RepeatedField, MapField};
    pub use ::protrust::extend::{ExtensionSet, ExtendableMessage, Extension, RepeatedExtension};
    pub use ::protrust::io::{Length, FieldNumber, Input, Output, CodedReader, CodedWriter, read, write};
}

#[doc(hidden)]
pub mod export {
    pub use protrust_macros as macros;
}

#[doc(hidden)]
pub mod gen;

/// The descriptor proto included with the library
pub use gen::google_protobuf_descriptor_proto as descriptor;

#[cfg(doctest)]
pub mod doctest;

pub mod collections;
pub mod extend;
pub mod io;
pub mod raw;
pub mod reflect;

use crate::io::{read, write, Length, CodedReader, CodedWriter, Input, Output};
use std::fmt::Debug;
use std::hash::Hash;

pub use collections::unknown_fields::UnknownFieldSet;

/// A message value.
/// 
/// This contains most operations required to work with a message and it's unknown fields.
/// This interface does not support reflection.
/// 
/// # Examples
/// 
/// ```ignore
/// # use protrust::doctest::timestamp::Timestamp;
/// use protrust::Message;
/// use protrust::io::{Length, CodedReader, CodedWriter};
/// 
/// let mut timestamp = Timestamp::new();
/// 
/// assert!(timestamp.is_initialized());
/// assert_eq!(Length::of_message(&timestamp), Length::new(0));
/// 
/// let input = [8, 5, 16, 100];
/// let mut reader = CodedReader::with_slice(&input);
/// 
/// timestamp.merge_from(&mut reader).expect("input is valid protobuf data");
/// 
/// assert_eq!(timestamp.seconds(), &5);
/// assert_eq!(timestamp.nanos(), &100);
/// assert_eq!(Length::of_message(&timestamp), Length::new(4));
/// 
/// *timestamp.nanos_mut() = 0;
/// 
/// assert_eq!(Length::of_message(&timestamp), Length::new(2));
/// 
/// let mut output = [0u8; 2];
/// let mut writer = CodedWriter::with_slice(&mut output);
/// 
/// timestamp.write_to(&mut writer).expect("size calculated ahead of time");
/// 
/// assert_eq!(output, [8, 5]);
/// ```
pub trait Message: Initializable + Default + Clone + PartialEq + Debug + Sized {
    /// Merges this message with data from the [`CodedReader`](io/read/struct.CodedReader.html).
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// # use protrust::doctest::timestamp::Timestamp;
    /// use protrust::Message;
    /// use protrust::io::{Length, CodedReader};
    /// 
    /// let mut timestamp = Timestamp::new();
    /// *timestamp.seconds_mut() = 10;
    /// 
    /// assert_eq!(timestamp.seconds(), &10);
    /// assert_eq!(timestamp.nanos(), &0);
    /// 
    /// let input = [8, 5, 16, 100];
    /// let mut reader = CodedReader::with_slice(&input);
    /// 
    /// timestamp.merge_from(&mut reader).expect("input is valid protobuf data");
    /// 
    /// assert_eq!(timestamp.seconds(), &5);
    /// assert_eq!(timestamp.nanos(), &100);
    /// ```
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()>;
    /// Calculates the size of this message, returning None if the size overflows an `i32`.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// # use protrust::doctest::timestamp::Timestamp;
    /// use protrust::Message;
    /// 
    /// let mut timestamp = Timestamp::new();
    /// assert_eq!(timestamp.calculate_size(), Length::new(0));
    /// 
    /// *timestamp.seconds_mut() = 10;
    /// assert_eq!(timestamp.calculate_size(), Length::new(2));
    /// ```
    fn calculate_size(&self) -> Option<Length>;
    /// Writes this message's data to the [`CodedWriter`](io/write/struct.CodedWriter.html).
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// # use protrust::doctest::timestamp::Timestamp;
    /// use protrust::Message;
    /// use protrust::io::CodedWriter;
    /// 
    /// let mut timestamp = Timestamp::new();
    /// *timestamp.seconds_mut() = 5;
    /// *timestamp.nanos_mut() = 100;
    /// 
    /// assert_eq!(timestamp.calculate_size(), Length::new(4));
    /// 
    /// let mut output = [0u8; 4];
    /// let mut writer = CodedWriter::with_slice(&mut output);
    /// 
    /// timestamp.write_to(&mut writer).expect("size is calculated ahead of time");
    /// ```
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result;

    /// Gets a shared reference to the unknown fields in this message.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// # use protrust::doctest::timestamp::Timestamp;
    /// use protrust::Message;
    /// use protrust::collections::unknown_fields::UnknownField;
    /// use protrust::io::{FieldNumber, CodedReader};
    /// 
    /// let mut timestamp = Timestamp::new();
    /// 
    /// let input = [32, 23];
    /// let mut reader = CodedReader::with_slice(&input);
    /// 
    /// timestamp.merge_from(&mut reader).expect("input is valid protobuf data");
    /// 
    /// let unknown_fields = timestamp.unknown_fields();
    /// 
    /// let values = unknown_fields.values(FieldNumber::new(3).unwrap());
    /// assert_eq!(values, &[UnknownField::Varint(23)]);
    /// ```
    fn unknown_fields(&self) -> &UnknownFieldSet;
    /// Gets a unique reference to the unknown fields in this message.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// # use protrust::doctest::timestamp::Timestamp;
    /// use protrust::Message;
    /// use protrust::collections::unknown_fields::UnknownField;
    /// use protrust::io::{FieldNumber, CodedReader};
    /// 
    /// let mut timestamp = Timestamp::new();
    /// 
    /// let input = [32, 23];
    /// let mut reader = CodedReader::with_slice(&input);
    /// 
    /// timestamp.merge_from(&mut reader).expect("input is valid protobuf data");
    /// 
    /// let unknown_fields = timestamp.unknown_fields_mut();
    /// 
    /// let values = unknown_fields.values(FieldNumber::new(3).unwrap());
    /// assert_eq!(values, &[UnknownField::Varint(23)]);
    /// 
    /// unknown_fields.clear();
    /// 
    /// assert!(unknown_fields.is_empty());
    /// ```
    fn unknown_fields_mut(&mut self) -> &mut UnknownFieldSet;
}

/// A marker trait used to mark enum types in generated code.
/// This defines all the main traits the enum types implement,
/// allowing code to refer to them easily.
/// 
/// To support name aliases, protobuf enums are not generated as
/// normal Rust enums, but as newtypes instead.
/// 
/// # Examples
/// 
/// For the enum Syntax in type.proto
/// ```text
/// // The syntax in which a protocol buffer element is defined.
/// enum Syntax {
///   // Syntax `proto2`.
///   SYNTAX_PROTO2 = 0;
///   // Syntax `proto3`.
///   SYNTAX_PROTO3 = 1;
/// }
/// ```
/// 
/// Similar code to the following would be generated:
/// ```
/// # use protrust::Enum;
/// # use std::fmt::{self, Debug, Formatter};
/// 
/// ##[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// pub struct Syntax(pub i32);
/// 
/// impl Syntax {
///     pub const PROTO2: Syntax = Syntax(0);
///     pub const PROTO3: Syntax = Syntax(1);
/// }
/// 
/// impl Default for Syntax {
///     fn default() -> Self {
///         Syntax(0)
///     }
/// }
/// 
/// impl From<i32> for Syntax {
///     fn from(x: i32) -> Self {
///         Self(x)
///     }
/// }
/// 
/// impl From<Syntax> for i32 {
///     fn from(x: Syntax) -> Self {
///         x.0
///     }
/// }
/// 
/// impl Debug for Syntax {
///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
///         match *self {
///             Syntax::PROTO2 => f.write_str("PROTO2"),
///             Syntax::PROTO3 => f.write_str("PROTO3"),
///             Syntax(x) => x.fmt(f),
///         }
///     }
/// }
/// 
/// impl Enum for Syntax { }
/// ```
/// 
/// Enums with aliases will use the first identifier listed for debug formatting.
/// 
/// # Examples
/// 
/// ```text
/// enum Aliased {
///   option allow_alias = true;
///   UNKNOWN = 0;
///   FOO = 1;
///   ALIAS = 1;
/// }
/// ```
/// 
/// Will format as:
/// ```
/// # use std::fmt::{self, Debug, Formatter};
/// # #[derive(PartialEq, Eq)]
/// # pub struct Aliased(pub i32);
/// # impl Aliased {
/// #   pub const UNKNOWN: Self = Self(0);
/// #   pub const FOO: Self = Self(1);
/// #   pub const ALIAS: Self = Self(1);
/// # }
/// # impl Debug for Aliased {
/// #   fn fmt(&self, f: &mut Formatter) -> fmt::Result {
/// #     match *self {
/// #       Aliased::UNKNOWN => f.write_str("UNKNOWN"),
/// #       Aliased::FOO => f.write_str("FOO"),
/// #       Aliased(x) => x.fmt(f),
/// #     }
/// #   }
/// # }
/// assert_eq!(format!("{:?}", Aliased::ALIAS), "FOO");
/// ```
pub trait Enum: From<i32> + Into<i32> + Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + Debug { }

/// A type that can be merged with one of `T`. Merge behavior is specific to each type.
/// 
/// Messages will merge fields present in `other` into self, while repeated fields will concat
/// and extend the field in `self` with entries from `other`.
/// 
/// # Examples
/// 
/// ```ignore
/// # use protrust::doctest::timestamp::Timestamp;
/// use protrust::{Mergable, Message};
/// 
/// let mut time = Timestamp::new();
/// *time.seconds_mut() = 5;
/// 
/// let mut other = Timestamp::new();
/// *time.nanos_mut() = 100;
/// 
/// time.merge(&other);
/// 
/// assert_eq!(time.seconds(), &5);
/// assert_eq!(time.nanos(), &100);
/// ```
pub trait Mergable<T = Self>: Sized {
    /// Merges another value into this one
    fn merge(&mut self, other: &T);
}

/// Merges two values together.
/// 
/// Internally uses an alias to `Mergable::merge`.
/// 
/// # Examples
/// 
/// ```ignore
/// # use protrust::doctest::timestamp::Timestamp;
/// use protrust::{Message, merge};
/// 
/// let mut time = Timestamp::new();
/// *time.seconds_mut() = 5;
/// 
/// let mut other = Timestamp::new();
/// *time.nanos_mut() = 100;
/// 
/// merge(&mut time, &other);
/// 
/// assert_eq!(time.seconds(), &5);
/// assert_eq!(time.nanos(), &100);
/// ```
pub fn merge<T: Mergable<V>, V>(value: &mut T, other: &V) {
    value.merge(other)
}

pub trait Initializable {
    fn is_initialized(&self) -> bool;
}

impl<T> Initializable for T {
    default fn is_initialized(&self) -> bool {
        true
    }
}

pub fn is_initialized<T: ?Sized + Initializable>(t: &T) -> bool {
    t.is_initialized()
}