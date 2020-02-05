//! Defines collection types used by generated code for repeated and map fields

use crate::{Mergable, internal::Sealed};
use crate::io::{self, read, write, WireType, FieldNumber, Tag, LengthBuilder, Length, CodedReader, CodedWriter, Input, Output};
use crate::raw::{self, Value, Packable, Packed};
use core::convert::TryInto;
use core::hash::Hash;

pub mod unknown_fields;

/// A type of value that writes and reads repeated values on the wire, a common trait unifying repeated and map fields.
pub trait RepeatedValue<T>: Sealed {
    /// Gets the wire type of tags in this field.
    const WIRE_TYPE: WireType;

    /// Adds entries to the repeated field from the coded reader.
    fn add_entries_from<U: Input>(&mut self, input: &mut CodedReader<U>) -> read::Result<()>;
    /// Calculates the size of the repeated value.
    fn calculate_size(&self, builder: LengthBuilder, num: FieldNumber) -> Option<LengthBuilder>;
    /// Writes the value to the coded writer. This takes a field number to build the tag required for each field.
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>, num: FieldNumber) -> write::Result;
    /// Returns a bool indicating whether all the values in the field are initialized
    fn is_initialized(&self) -> bool;
}

/// A set of fields. This unifies unknown fields, extension fields, and any other future field set types
pub trait FieldSet: Sealed {
    /// Checks if the set can read the field from the input and reads it if it can. It returns a state indicating if the field was read.
    fn try_add_field_from<'a, T: Input>(&mut self, input: &'a mut CodedReader<T>) -> read::Result<TryRead<'a, T>>;
    /// Calculates the size of all the fields in this set
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
    /// Writes the fields in this set to the writer
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result;
    /// Returns if all the fields in this set are initialized
    fn is_initialized(&self) -> bool;
}

/// The result of trying to add a field to a field set
pub enum TryRead<'a, T: Input> {
    /// The set didn't read the field. Sets should return the
    /// borrowed reader to allow other sets to possibly read the field
    Yielded(&'a mut CodedReader<T>),
    /// The set read the field, consuming it
    Consumed,
}

impl<'a, T: Input> TryRead<'a, T> {
    /// Tries to read the field into the specified set. If the field has already been read, this does nothing.
    #[inline]
    pub fn or_try<S: FieldSet>(self, set: &mut S) -> read::Result<TryRead<'a, T>> {
        match self {
            TryRead::Yielded(input) => set.try_add_field_from(input),
            TryRead::Consumed => Ok(TryRead::Consumed),
        }
    }
    /// Skips the field if it hasn't already been read
    #[inline]
    pub fn or_skip(self) -> read::Result<()> {
        match self {
            TryRead::Yielded(input) => input.skip(),
            TryRead::Consumed => Ok(()),
        }
    }
}

/// The type used by generated code to represent a repeated field.
pub type RepeatedField<T> = alloc::vec::Vec<T>;

impl<T> Sealed for RepeatedField<T> { }
impl<V: Value> RepeatedValue<V> for RepeatedField<V::Inner> {
    const WIRE_TYPE: WireType = V::WIRE_TYPE;

    #[inline]
    fn add_entries_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_value::<V>().map(|v| self.push(v))
    }
    #[inline]
    fn calculate_size(&self, builder: LengthBuilder, num: FieldNumber) -> Option<LengthBuilder> {
        if self.is_empty() {
            return Some(builder);
        }

        let len: i32 = self.len().try_into().ok()?;

        let tag = Tag::new(num, V::WIRE_TYPE);
        let tag_len = io::raw_varint32_size(tag.get());
        let tags_len = unsafe { 
            Length::new_unchecked(
                if cfg!(feature = "checked_size") {
                    tag_len.get().checked_mul(len)?
                } else {
                    tag_len.get() * len
                }
            )
        };
        let builder = builder.add_bytes(tags_len)?;
        let builder = 
            // for groups we can add the tags length again for the end tags
            if V::WIRE_TYPE == WireType::StartGroup {
                builder.add_bytes(tags_len)?
            } else {
                builder
            };
        <Self as ValuesSize<V>>::calculate_size(self, builder)
    }
    #[inline]
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>, num: FieldNumber) -> write::Result {
        if self.is_empty() {
            return Ok(());
        }

        for value in self {
            output.write_field::<V>(num, value)?;
        }

        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.iter().all(V::is_initialized)
    }
}
impl<V: Value + Packable> RepeatedValue<Packed<V>> for RepeatedField<V::Inner> {
    const WIRE_TYPE: WireType = WireType::LengthDelimited;

    #[inline]
    fn add_entries_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        input.read_limit()?.for_all(|input| input.read_value::<V>().map(|v| self.push(v)))
    }
    #[inline]
    fn calculate_size(&self, builder: LengthBuilder, num: FieldNumber) -> Option<LengthBuilder> {
        if self.is_empty() {
            return Some(builder);
        }

        let len = <Self as ValuesSize<V>>::calculate_size(self, LengthBuilder::new())?.build();

        builder
            .add_tag(Tag::new(num, WireType::LengthDelimited))?
            .add_value::<raw::Uint32>(&(len.get() as u32))?
            .add_bytes(len)
    }
    #[inline]
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>, num: FieldNumber) -> write::Result {
        if self.is_empty() {
            return Ok(());
        }

        let len = 
            <Self as ValuesSize<V>>::calculate_size(self, LengthBuilder::new())
                .ok_or(write::Error::ValueTooLarge)?
                .build();

        output.write_tag(Tag::new(num, WireType::LengthDelimited))?;
        output.write_length(len)?;
        for value in self {
            output.write_value::<V>(value)?;
        }
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.iter().all(V::is_initialized)
    }
}
impl<V: Clone> Mergable for RepeatedField<V> {
    /// Merges two repeated fields by extending this field with the elements of the other
    fn merge(&mut self, other: &Self) {
        self.extend(other.iter().cloned())
    }
}

/// The type used by generated code to represent a map field.
pub type MapField<K, V> = hashbrown::HashMap<K, V>;

const KEY_FIELD: FieldNumber = unsafe { FieldNumber::new_unchecked(1) };
const VALUE_FIELD: FieldNumber = unsafe { FieldNumber::new_unchecked(2) };

impl<K, V> Sealed for MapField<K, V> { }
impl<K, V> RepeatedValue<(K, V)> for MapField<K::Inner, V::Inner>
    where 
        K: Value,
        K::Inner: Default + Eq + Hash,
        V: Value,
        V::Inner: Default
{
    const WIRE_TYPE: WireType = WireType::LengthDelimited;
    
    fn add_entries_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        let key_tag = Tag::new(KEY_FIELD, K::WIRE_TYPE);
        let value_tag = Tag::new(VALUE_FIELD, V::WIRE_TYPE);

        let mut key = None::<K::Inner>;
        let mut value = None::<V::Inner>;
        input.read_limit()?.then(|input| {
            while let Some(field) = input.read_field()? {
                match field.tag() {
                    k if k == key_tag.get() => field.and_then(key_tag, |input| input.read_value::<K>().map(|k| key = Some(k))),
                    v if v == value_tag.get() => field.and_then(value_tag, |input| input.read_value::<V>().map(|v| value = Some(v))),
                    _ => input.skip(),
                }?
            }
            Ok(())
        })?;
        self.insert(key.unwrap_or_default(), value.unwrap_or_default());

        Ok(())
    }
    fn calculate_size(&self, builder: LengthBuilder, num: FieldNumber) -> Option<LengthBuilder> {
        if self.is_empty() {
            return Some(builder);
        }

        let len: i32 = self.len().try_into().ok()?;
        let tag = Tag::new(num, WireType::LengthDelimited);
        let tag_len = io::raw_varint32_size(tag.get()).get();
        let start_len = // every size calculation starts with the size of all tags
            if cfg!(feature = "checked_size") {
                len.checked_mul(tag_len)?.checked_add(len.checked_mul(2)?)?
            } else {
                (len * tag_len) + (len * 2)
            };
        let mut builder = builder.add_bytes(Length::new(start_len)?)?;
        for (key, value) in self {
            let entry_len = 
                LengthBuilder::new()
                    .add_bytes(unsafe { Length::new_unchecked(2) })?
                    .add_value::<K>(key)?
                    .add_value::<V>(value)?
                    .build();
            builder = builder.add_value::<raw::Uint32>(&(entry_len.get() as u32))?.add_bytes(entry_len)?; // add the length size with the entry size
        }
        Some(builder)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>, num: FieldNumber) -> write::Result {
        if self.is_empty() {
            return Ok(());
        }

        let tag = Tag::new(num, WireType::LengthDelimited);
        for (key, value) in self {
            output.write_tag(tag)?;
            let length = 
                LengthBuilder::new()
                    .add_bytes(unsafe { Length::new_unchecked(2) }).ok_or(write::Error::ValueTooLarge)?
                    .add_value::<K>(key).ok_or(write::Error::ValueTooLarge)?
                    .add_value::<V>(value).ok_or(write::Error::ValueTooLarge)?
                    .build();
            output.write_length(length)?;
            output.write_tag(Tag::new(KEY_FIELD, K::WIRE_TYPE))?;
            output.write_value::<K>(key)?;
            output.write_tag(Tag::new(VALUE_FIELD, V::WIRE_TYPE))?;
            output.write_value::<V>(value)?;
        }

        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.values().all(V::is_initialized)
    }
}

impl<K, V> Mergable for hashbrown::HashMap<K, V>
    where
        K: Clone + Eq + Hash,
        V: Clone + Mergable
{
    fn merge(&mut self, other: &Self) {
        for (k, v) in other {
            self.raw_entry_mut() // use a raw entry so we can defer the cloning of the key until we need it
                .from_key(k)
                .and_modify(|_, e| e.merge(v))
                .or_insert_with(|| (k.clone(), v.clone()));
        }
    }
}

trait ValuesSize<T> {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
}

impl<V> ValuesSize<V> for RepeatedField<V::Inner>
    where V: Value
{
    default fn calculate_size(&self, mut builder: LengthBuilder) -> Option<LengthBuilder> {
        for value in self {
            builder = builder.add_value::<V>(value)?;
        }
        Some(builder)
    }
}

impl<V> ValuesSize<V> for RepeatedField<V::Inner>
    where V: raw::ConstSized
{
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let size = V::SIZE;
        let len: i32 = self.len().try_into().ok()?;

        builder.add_bytes(unsafe {
            if cfg!(feature = "checked_size") {
                Length::new_unchecked(len.checked_mul(size.get())?)
            } else {
                Length::new_unchecked(len * size.get())
            }
        })
    }
}