//! Defines collection types used by generated code for repeated and map fields

use crate::{Mergable, internal::Sealed};
use crate::io::{WireType, FieldNumber, Tag, LengthBuilder, CodedReader, ReaderResult, CodedWriter, WriterResult, WriterError};
use crate::raw::{self, Heaping, Primitive, Value};
use std::convert::TryInto;
use std::hash::Hash;
use trapper::Wrapper;

/// A type of value that writes and reads repeated values on the wire, a common trait unifying repeated and map fields.
pub trait RepeatedValue<T>: Sealed {
    /// Calculates the size of the repeated value. This takes a corresponding tag to indicate the packedness of the field if required.
    fn calculate_size(&self, builder: LengthBuilder, tag: Tag) -> Option<LengthBuilder>;
    /// Writes the value to the coded writer. This takes a corresponding tag to indicate the packedness of the field if required.
    fn write_to(&self, output: &mut CodedWriter, tag: Tag) -> WriterResult;
    /// Returns a bool indicating whether all the values in the field are initialized
    fn is_initialized(&self) -> bool;
}
/// A type of sized value that doesn't allocate any dynamic memory.
pub trait RepeatedPrimitiveValue<T>: RepeatedValue<T> {
    /// Adds entries to the repeated field from the coded reader. This doesn't take a corresponding tag as 
    /// inputs should be able to handle packed or unpacked values if their type supports it, even if the 
    /// field doesn't match with the encoded value's packedness.
    fn add_entries_from(&mut self, input: &mut CodedReader) -> ReaderResult<()>;
}
/// A type of value that, when adding entries from an input, requires an allocator to allocate dynamic memory into.
pub trait RepeatedHeapingValue<T: Heaping>: RepeatedValue<T> {
    /// Adds entries to the repeated field from the coded reader. This doesn't take a corresponding tag as 
    /// inputs should be able to handle packed or unpacked values if their type supports it, even if the 
    /// field doesn't match with the encoded value's packedness. This may allocate memory into the specified allocator.
    fn add_entries_from(&mut self, input: &mut CodedReader, a: T::Alloc) -> ReaderResult<()>;
}

/// A repeated field. This is the type used by generated code to represent a repeated field value (if it isn't a map).
pub type RepeatedField<T> = Vec<T>;
/// A map field. This is the type used by generated code to represent a map field value.
pub type MapField<T, V> = hashbrown::HashMap<T, V, hashbrown::hash_map::DefaultHashBuilder>;

impl<T> Sealed for RepeatedField<T> { }
impl<T: Value + Wrapper> RepeatedValue<T> for RepeatedField<T::Inner> {
    #[inline]
    fn calculate_size(&self, builder: LengthBuilder, tag: Tag) -> Option<LengthBuilder> {
        if self.is_empty() {
            return Some(builder);
        }

        let len: i32 = self.len().try_into().ok()?;

        let tag_len = raw::raw_varint32_size(tag.get()).get();
        let builder =
            if WireType::is_packable(T::WIRE_TYPE) && tag.wire_type() == WireType::LengthDelimited {
                builder.add_bytes(tag_len)
            } else {
                builder.add_bytes({
                    #[cfg(feature = "checked_size")]
                    { tag_len.checked_mul(len)? }
                    #[cfg(not(feature = "checked_size"))]
                    { tag_len * len }
                })
            }?;
        <Self as ValuesSize<T>>::calculate_size(self, builder)
    }
    #[inline]
    fn write_to(&self, output: &mut CodedWriter, tag: Tag) -> WriterResult {
        if self.is_empty() {
            return Ok(());
        }

        if WireType::is_packable(T::WIRE_TYPE) && tag.wire_type() == WireType::LengthDelimited {
            let len = <Self as ValuesSize<T>>::calculate_size(self, LengthBuilder::new()).ok_or(WriterError::ValueTooLarge)?.build();
            output.write_length(len)?;
            for value in self {
                output.write_value::<T>(value)?;
            }
        } else {
            for value in self {
                output.write_tag(tag)?;
                output.write_value::<T>(value)?;
            }
        }
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.iter().map(T::wrap_ref).all(T::is_initialized)
    }
}
impl<T> RepeatedPrimitiveValue<T> for RepeatedField<T::Inner>
    where
        T: Primitive + Wrapper
{
    #[inline]
    fn add_entries_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        if let Some(last_tag) = input.last_tag() {
            if WireType::is_packable(T::WIRE_TYPE) && last_tag.wire_type() == WireType::LengthDelimited {
                let old = input.read_and_push_length()?;
                while !input.reached_limit() {
                    self.push(input.read_value::<T>()?);
                }
                input.pop_length(old);
            } else {
                self.push(input.read_value::<T>()?);
            }
        }
        Ok(())
    }
}
impl<T> RepeatedHeapingValue<T> for RepeatedField<T::Inner>
    where
        T: Heaping + Wrapper,
        T::Alloc: Clone
{
    #[inline]
    fn add_entries_from(&mut self, input: &mut CodedReader, a: T::Alloc) -> ReaderResult<()> {
        if let Some(last_tag) = input.last_tag() {
            if WireType::is_packable(T::WIRE_TYPE) && last_tag.wire_type() == WireType::LengthDelimited {
                let old = input.read_and_push_length()?;
                while !input.reached_limit() {
                    self.push(input.read_value_in::<T>(a.clone())?);
                }
                input.pop_length(old);
            } else {
                self.push(input.read_value_in::<T>(a.clone())?);
            }
        }
        Ok(())
    }
}
impl<T: Clone> Mergable for RepeatedField<T> {
    /// Merges two repeated fields by extending this field with the elements of the other
    fn merge(&mut self, other: &Self) {
        self.extend(other.iter().cloned())
    }
}

const KEY_FIELD: FieldNumber = unsafe { FieldNumber::new_unchecked(1) };
const VALUE_FIELD: FieldNumber = unsafe { FieldNumber::new_unchecked(2) };

impl<K, V> Sealed for MapField<K, V> { }
impl<K, V> RepeatedValue<(K, V)> for MapField<K::Inner, V::Inner>
    where 
        K: Value + Wrapper,
        V: Value + Wrapper,
{
    fn calculate_size(&self, builder: LengthBuilder, tag: Tag) -> Option<LengthBuilder> {
        if self.is_empty() {
            return Some(builder);
        }

        let len: i32 = self.len().try_into().ok()?;
        let tag_len = raw::raw_varint32_size(tag.get()).get();
        let start_len = { // every size calculation starts with the size of all tags
            #[cfg(feature = "checked_size")]
            { len.checked_mul(tag_len)?.checked_add(len.checked_mul(2)?)? }
            #[cfg(not(feature = "checked_size"))]
            { (len * tag_len) + (len * 2) }
        };
        let mut builder = builder.add_bytes(start_len)?;
        for (key, value) in self {
            let entry_len = LengthBuilder(2).add_value::<K>(key)?.add_value::<V>(value)?.build().get(); // calculate the length of each entry
            builder = builder.add_value::<raw::Uint32>(&(entry_len as u32))?.add_bytes(entry_len)?; // add the length size with the entry size
        }
        Some(builder)
    }
    fn write_to(&self, output: &mut CodedWriter, tag: Tag) -> WriterResult {
        if self.is_empty() {
            return Ok(());
        }

        for (key, value) in self {
            output.write_tag(tag)?;
            let length = 
                LengthBuilder(2)
                    .add_value::<K>(key)
                    .and_then(|b| 
                        b.add_value::<V>(value))
                    .map(|b| b.build())
                    .ok_or(WriterError::ValueTooLarge)?;
            output.write_length(length)?;
            output.write_tag(Tag::new(KEY_FIELD, K::WIRE_TYPE))?;
            output.write_value::<K>(key)?;
            output.write_tag(Tag::new(VALUE_FIELD, V::WIRE_TYPE))?;
            output.write_value::<V>(value)?;
        }

        Ok(())
    }
    fn is_initialized(&self) -> bool {
        self.values().map(V::wrap_ref).all(V::is_initialized)
    }
}
impl<K, V> RepeatedPrimitiveValue<(K, V)> for MapField<K::Inner, V::Inner>
    where
        K: Primitive + Wrapper,
        V: Primitive + Wrapper,
        K::Inner: Eq + Hash + Default,
        V::Inner: Default
{
    fn add_entries_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        let key_tag = Tag::new(KEY_FIELD, K::WIRE_TYPE);
        let value_tag = Tag::new(VALUE_FIELD, V::WIRE_TYPE);

        let old = input.read_and_push_length()?;
        let mut key = K::Inner::default();
        let mut value = V::Inner::default();
        while let Some(tag) = input.read_tag()? {
            match tag {
                t if t == key_tag => key = input.read_value::<K>()?,
                v if v == value_tag => value = input.read_value::<V>()?,
                _ => input.skip()?,
            }
        }
        input.pop_length(old);
        self.insert(key, value);

        Ok(())
    }
}
impl<K, V> Mergable for MapField<K, V>
    where
        K: Clone + Eq + Hash,
        V: Clone + Mergable
{
    fn merge(&mut self, other: &Self) {
        for (k, v) in other {
            match self.get_mut(k) {
                Some(ev) => ev.merge(v),
                None => { self.insert(k.clone(), v.clone()); }
            }
        }
    }
}

trait ValuesSize<T> {
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
}

impl<T> ValuesSize<T> for RepeatedField<T::Inner>
    where T: Value + Wrapper
{
    default fn calculate_size(&self, mut builder: LengthBuilder) -> Option<LengthBuilder> {
        for value in self {
            builder = builder.add_value::<T>(value)?;
        }
        Some(builder)
    }
}

impl<T> ValuesSize<T> for RepeatedField<T::Inner>
    where T: raw::ConstSized + Wrapper
{
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        let size = T::SIZE;
        let len: i32 = self.len().try_into().ok()?;

        #[cfg(feature = "checked_size")]
        return builder.add_bytes(len.checked_mul(size)?);

        #[cfg(not(feature = "checked_size"))]
        return builder.add_bytes(len * size);
    }
}