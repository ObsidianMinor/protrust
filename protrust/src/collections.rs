//! Defines collection types used by generated code for repeated and map fields

use crate::{Mergable, internal::Sealed};
use crate::io::{read, write, WireType, FieldNumber, Tag, LengthBuilder, CodedReader, CodedWriter};
use crate::raw::{self, Value};
use core::convert::TryInto;
use core::hash::Hash;
use trapper::{newtype, Wrapper};

/// A type of value that writes and reads repeated values on the wire, a common trait unifying repeated and map fields.
pub trait RepeatedValue<T>: Sealed {
    /// Adds entries to the repeated field from the coded reader. This doesn't take a corresponding tag as 
    /// inputs should be able to handle packed or unpacked values if their type supports it, even if the 
    /// field doesn't match with the encoded value's packedness.
    fn add_entries_from(&mut self, input: &mut CodedReader) -> read::Result<()>;
    /// Calculates the size of the repeated value. This takes a corresponding tag to indicate the packedness of the field if required.
    fn calculate_size(&self, builder: LengthBuilder, tag: Tag) -> Option<LengthBuilder>;
    /// Writes the value to the coded writer. This takes a corresponding tag to indicate the packedness of the field if required.
    fn write_to(&self, output: &mut CodedWriter, tag: Tag) -> write::Result;
    /// Returns a bool indicating whether all the values in the field are initialized
    fn is_initialized(&self) -> bool;
}

newtype! {
    /// A repeated field. This is the type used by generated code to represent a repeated field value (if it isn't a map).
    pub type RepeatedField<T>(T);
}

impl<T> Sealed for RepeatedField<T> { }
impl<V: Value + Wrapper> RepeatedValue<V> for RepeatedField<alloc::vec::Vec<V::Inner>> {
    #[inline]
    fn add_entries_from(&mut self, input: &mut CodedReader) -> read::Result<()> {
        if let Some(last_tag) = input.last_tag() {
            if WireType::is_packable(V::WIRE_TYPE) && last_tag.wire_type() == WireType::LengthDelimited {
                let old = input.read_and_push_length()?;
                while !input.reached_limit() {
                    self.0.push(input.read_value::<V>()?);
                }
                input.pop_length(old);
            } else {
                self.0.push(input.read_value::<V>()?);
            }
        }
        Ok(())
    }
    #[inline]
    fn calculate_size(&self, builder: LengthBuilder, tag: Tag) -> Option<LengthBuilder> {
        if self.0.is_empty() {
            return Some(builder);
        }

        let len: i32 = self.0.len().try_into().ok()?;

        let tag_len = raw::raw_varint32_size(tag.get()).get();
        let builder =
            if WireType::is_packable(V::WIRE_TYPE) && tag.wire_type() == WireType::LengthDelimited {
                builder.add_bytes(tag_len)
            } else {
                builder.add_bytes({
                    #[cfg(feature = "checked_size")]
                    { tag_len.checked_mul(len)? }
                    #[cfg(not(feature = "checked_size"))]
                    { tag_len * len }
                })
            }?;
        <Self as ValuesSize<V>>::calculate_size(self, builder)
    }
    #[inline]
    fn write_to(&self, output: &mut CodedWriter, tag: Tag) -> write::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        if WireType::is_packable(V::WIRE_TYPE) && tag.wire_type() == WireType::LengthDelimited {
            let len = <Self as ValuesSize<V>>::calculate_size(self, LengthBuilder::new()).ok_or(write::Error::ValueTooLarge)?.build();
            output.write_length(len)?;
            for value in &self.0 {
                output.write_value::<V>(value)?;
            }
        } else {
            for value in &self.0 {
                output.write_tag(tag)?;
                output.write_value::<V>(value)?;
            }
        }
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.0.iter().map(V::wrap_ref).all(V::is_initialized)
    }
}
impl<V: Clone> Mergable for alloc::vec::Vec<V> {
    /// Merges two repeated fields by extending this field with the elements of the other
    fn merge(&mut self, other: &Self) {
        self.extend(other.iter().cloned())
    }
}

newtype! {
    /// A map field. This is the type used by generated code to represent a map field value.
    pub type MapField<T>(T);
}

impl<T> MapField<T> {
    const KEY_FIELD: FieldNumber = unsafe { FieldNumber::new_unchecked(1) };
    const VALUE_FIELD: FieldNumber = unsafe { FieldNumber::new_unchecked(2) };
}

impl<T> Sealed for MapField<T> { }
impl<K, V> RepeatedValue<(K, V)> for MapField<hashbrown::HashMap<K::Inner, V::Inner>>
    where 
        K: Value + Wrapper,
        K::Inner: Default + Eq + Hash,
        V: Value + Wrapper,
        V::Inner: Default
{
    fn add_entries_from(&mut self, input: &mut CodedReader) -> read::Result<()> {
        let key_tag = Tag::new(Self::KEY_FIELD, K::WIRE_TYPE);
        let value_tag = Tag::new(Self::VALUE_FIELD, V::WIRE_TYPE);

        let old = input.read_and_push_length()?;
        let mut key = None;
        let mut value = None;
        while let Some(tag) = input.read_tag()? {
            match tag {
                t if t == key_tag => key = Some(input.read_value::<K>()?),
                v if v == value_tag => value = Some(input.read_value::<V>()?),
                _ => input.skip()?,
            }
        }
        input.pop_length(old);
        self.0.insert(key.unwrap_or_default(), value.unwrap_or_default());

        Ok(())
    }
    fn calculate_size(&self, builder: LengthBuilder, tag: Tag) -> Option<LengthBuilder> {
        if self.0.is_empty() {
            return Some(builder);
        }

        let len: i32 = self.0.len().try_into().ok()?;
        let tag_len = raw::raw_varint32_size(tag.get()).get();
        let start_len = { // every size calculation starts with the size of all tags
            #[cfg(feature = "checked_size")]
            { len.checked_mul(tag_len)?.checked_add(len.checked_mul(2)?)? }
            #[cfg(not(feature = "checked_size"))]
            { (len * tag_len) + (len * 2) }
        };
        let mut builder = builder.add_bytes(start_len)?;
        for (key, value) in &self.0 {
            let entry_len = 
                LengthBuilder::new()
                    .add_bytes(2).unwrap()
                    .add_value::<K>(key)?
                    .add_value::<V>(value)?
                    .build()
                    .get(); // calculate the length of each entry
            builder = builder.add_value::<raw::Uint32>(&(entry_len as u32))?.add_bytes(entry_len)?; // add the length size with the entry size
        }
        Some(builder)
    }
    fn write_to(&self, output: &mut CodedWriter, tag: Tag) -> write::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        for (key, value) in &self.0 {
            output.write_tag(tag)?;
            let length = 
                LengthBuilder::new()
                    .add_bytes(2).unwrap()
                    .add_value::<K>(key)
                    .and_then(|b| 
                        b.add_value::<V>(value))
                    .map(|b| b.build())
                    .ok_or(write::Error::ValueTooLarge)?;
            output.write_length(length)?;
            output.write_tag(Tag::new(Self::KEY_FIELD, K::WIRE_TYPE))?;
            output.write_value::<K>(key)?;
            output.write_tag(Tag::new(Self::VALUE_FIELD, V::WIRE_TYPE))?;
            output.write_value::<V>(value)?;
        }

        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.0.values().map(V::wrap_ref).all(V::is_initialized)
    }
}

impl<K, V> Mergable for hashbrown::HashMap<K, V>
    where
        K: Clone + Eq + Hash,
        V: Clone + Mergable
{
    fn merge(&mut self, other: &Self) {
        for (k, v) in other {
            self.raw_entry_mut().from_key(k).and_modify(|_, e| e.merge(v)).or_insert_with(|| (k.clone(), v.clone()));
        }
    }
}

trait ValuesSize<T> {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
}

impl<V> ValuesSize<V> for RepeatedField<alloc::vec::Vec<V::Inner>>
    where V: Value + Wrapper
{
    default fn calculate_size(&self, mut builder: LengthBuilder) -> Option<LengthBuilder> {
        for value in &self.0 {
            builder = builder.add_value::<V>(value)?;
        }
        Some(builder)
    }
}

impl<V> ValuesSize<V> for RepeatedField<alloc::vec::Vec<V::Inner>>
    where V: raw::ConstSized + Wrapper
{
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let size = V::SIZE;
        let len: i32 = self.0.len().try_into().ok()?;

        #[cfg(feature = "checked_size")]
        return builder.add_bytes(len.checked_mul(size)?);

        #[cfg(not(feature = "checked_size"))]
        return builder.add_bytes(len * size);
    }
}