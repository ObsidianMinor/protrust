//! Defines the `UnknownFieldSet`, a struct used to contain unknown fields as they were read from coded readers.
//! 
//! As APIs are updated, certain fields may be removed or added from proto file definitions. If an old version of a message 
//! encounters fields it doesn't recognize can still read them to be returned again via unknown fields.
//! 
//! Unknown fields for unique field numbers can exist for multiple wire types at once to ensure that all data is properly returned.

use crate::{internal::Sealed, Mergable};
use crate::io::{read, write, FieldNumber, WireType, Tag, LengthBuilder, CodedReader, CodedWriter, Input, Output};
use crate::raw;
use std::collections::{HashMap, hash_map};
use std::fmt::{self, Formatter, Debug};
use std::iter::FusedIterator;
use std::ops::RangeBounds;
use std::vec;
use super::{FieldSet, TryRead};

/// An unknown field in an [`UnknownFieldSet`](struct.UnknownFieldSet.html).
#[derive(PartialEq, Clone, Debug)]
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
#[derive(PartialEq, Default, Clone, Debug)]
pub struct UnknownFieldSet {
    inner: HashMap<FieldNumber, Vec<UnknownField>>,
}

impl Sealed for UnknownFieldSet { }
impl Mergable for UnknownFieldSet {
    fn merge(&mut self, other: &Self) {
        for (&key, values) in &other.inner {
            self.inner.entry(key).or_insert_with(Vec::new).extend(values.clone())
        }
    }
}
impl FieldSet for UnknownFieldSet {
    #[inline]
    fn try_add_field_from<'a, T: Input>(&mut self, input: &'a mut CodedReader<T>) -> read::Result<TryRead<'a, T>> {
        if input.unknown_field_handling().skip() || input.last_tag().map(Tag::wire_type) == Some(WireType::EndGroup) {
            Ok(TryRead::Yielded(input))
        } else {
            self.add_field_from(input)?;
            Ok(TryRead::Consumed)
        }
    }
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        self.inner
            .iter()
            .try_fold(builder, |builder, (&key, values)| 
                values
                    .iter()
                    .try_fold(builder, |builder, value| {
                        match value {
                            UnknownField::Varint(v) => {
                                builder
                                    .add_tag(Tag::new(key, WireType::Varint))?
                                    .add_value::<raw::Uint64>(v)
                            },
                            UnknownField::Bit64(v) => {
                                builder
                                    .add_tag(Tag::new(key, WireType::Bit64))?
                                    .add_value::<raw::Fixed64>(v)
                            },
                            UnknownField::LengthDelimited(v) => {
                                builder
                                    .add_tag(Tag::new(key, WireType::LengthDelimited))?
                                    .add_value::<raw::Bytes<_>>(v)
                            },
                            UnknownField::Group(v) => {
                                builder
                                    .add_tag(Tag::new(key, WireType::StartGroup))?
                                    .add_fields(v)?
                                    .add_tag(Tag::new(key, WireType::EndGroup))
                            },
                            UnknownField::Bit32(v) => {
                                builder
                                    .add_tag(Tag::new(key, WireType::Bit32))?
                                    .add_value::<raw::Fixed32>(v)
                            }
                        }
                })
            )
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        for (key, values) in &self.inner {
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
}
impl UnknownFieldSet {
    fn add_field_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        if let Some(last_tag) = input.last_tag() {
            match last_tag.wire_type() {
                WireType::Varint => self.push_value(last_tag.field(), UnknownField::Varint(input.read_varint64()?)),
                WireType::Bit64 => self.push_value(last_tag.field(), UnknownField::Bit64(input.read_bit64()?)),
                WireType::LengthDelimited => self.push_value(last_tag.field(), UnknownField::LengthDelimited(input.read_length_delimited()?)),
                WireType::StartGroup => {
                    let mut group = UnknownFieldSet::new();
                    let end_tag = Tag::new(last_tag.field(), WireType::EndGroup);
                    while let Some(tag) = input.read_tag()? {
                        if tag != end_tag {
                            input.recurse(|input| group.add_field_from(input))?;
                        } else {
                            break;
                        }
                    }
                    self.push_value(last_tag.field(), UnknownField::Group(group));
                },
                WireType::Bit32 => self.push_value(last_tag.field(), UnknownField::Bit32(input.read_bit32()?)),
                WireType::EndGroup => return Err(read::Error::InvalidTag(last_tag.get()))
            }
        }
        Ok(())
    }
}
impl UnknownFieldSet {
    /// Creates a new unknown field set in the specified allocator
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    /// Gets the number of fields present in this set
    #[inline]
    pub fn field_len(&self) -> usize {
        self.inner.len()
    }
    /// Returns whether any fields are present in this set
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }
    /// Returns a slice of values for a field
    pub fn values(&self, num: FieldNumber) -> &[UnknownField] {
        self.inner.get(&num).map(Vec::as_slice).unwrap_or(&[])
    }
    /// Returns a mutable slice of values for a field
    pub fn values_mut(&mut self, num: FieldNumber) -> &mut [UnknownField] {
        self.inner.get_mut(&num).map(Vec::as_mut_slice).unwrap_or(&mut [])
    }
    /// Pushes an new value to the field
    pub fn push_value(&mut self, num: FieldNumber, value: UnknownField) {
        self.inner.entry(num).or_insert_with(Vec::new).push(value)
    }
    /// Pops the last value added for the specified field
    pub fn pop_value(&mut self, num: FieldNumber) -> Option<UnknownField> {
        self.inner.get_mut(&num).and_then(Vec::pop)
    }
    /// Returns an iterator of all of the fields in the set
    pub fn fields(&self) -> Iter {
        Iter(self.inner.iter())
    }
    /// Returns a mutable iterator of all the fields in the set
    pub fn fields_mut(&mut self) -> IterMut {
        IterMut(self.inner.iter_mut())
    }
    /// Clears the set, removing all fields
    pub fn clear(&mut self) {
        self.inner.clear()
    }
    /// Clears the field, removing all values
    pub fn clear_field(&mut self, num: FieldNumber) {
        self.inner.remove(&num);
    }
    /// Gets an iterator of all fields by their field number
    pub fn field_numbers(&self) -> FieldNumbers {
        FieldNumbers(self.inner.keys())
    }
    /// Clears the set, returning the owned field values
    pub fn drain(&mut self) -> Drain {
        Drain(self.inner.drain())
    }
    /// Drains a range of values from a field
    pub fn drain_values<R: RangeBounds<usize>>(&mut self, num: FieldNumber, range: R) -> FieldDrain {
        FieldDrain(self.inner.get_mut(&num).map(|v| v.drain(range)))
    }
}

/// An iterator over the fields of an unknown field set.
#[derive(Clone, Debug)]
pub struct Iter<'a>(hash_map::Iter<'a, FieldNumber, Vec<UnknownField>>);

impl<'a> Iterator for Iter<'a> {
    type Item = (FieldNumber, &'a [UnknownField]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(&n, v)| (n, v.as_slice()))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl FusedIterator for Iter<'_> { }

/// A mutable iterator over the fields of an unknown field set.
#[derive(Debug)]
pub struct IterMut<'a>(hash_map::IterMut<'a, FieldNumber, Vec<UnknownField>>);

impl<'a> Iterator for IterMut<'a> {
    type Item = (FieldNumber, &'a mut [UnknownField]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(&n, v)| (n, v.as_mut_slice()))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for IterMut<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl FusedIterator for IterMut<'_> { }

/// An iterator over the field numbers present in this set.
/// 
/// This `struct` is created by the [`field_numbers`] method on [`UnknownFieldSet`].
/// See its documentation for more.
/// 
/// [`field_numbers`]: struct.UnknownFieldSet.html#method.field_numbers
/// [`UnknownFieldSet`]: struct.UnknownFieldSet.html
#[derive(Debug, Clone)]
pub struct FieldNumbers<'a>(hash_map::Keys<'a, FieldNumber, Vec<UnknownField>>);

impl Iterator for FieldNumbers<'_> {
    type Item = FieldNumber;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for FieldNumbers<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl FusedIterator for FieldNumbers<'_> { }

/// A draining iterator that returns each field along with a boxed slice of unknown fields.
/// 
/// This `struct` is created by the [`drain`] method on [`UnknownFieldSet`].
/// See its documentation for more.
/// 
/// [`drain`]: struct.UnknownFieldSet.html#method.drain
/// [`UnknownFieldSet`]: struct.UnknownFieldSet.html
#[derive(Debug)]
pub struct Drain<'a>(hash_map::Drain<'a, FieldNumber, Vec<UnknownField>>);

impl Iterator for Drain<'_> {
    type Item = (FieldNumber, Box<[UnknownField]>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(n, v)| (n, v.into_boxed_slice()))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for Drain<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl FusedIterator for Drain<'_> { }

/// A draining iterator that returns the unknown fields for a single field.
/// 
/// This `struct` is created by the [`drain_field`] method on [`UnknownFieldSet`].
/// See its documentation for more.
/// 
/// [`drain_field`]: struct.UnknownFieldSet.html#method.drain_field
/// [`UnknownFieldSet`]: struct.UnknownFieldSet.html
pub struct FieldDrain<'a>(Option<vec::Drain<'a, UnknownField>>);

impl Debug for FieldDrain<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.0 {
            Some(drain) => {
                f.debug_tuple("FieldDrain")
                 .field(&drain.as_slice())
                 .finish()
            },
            None => {
                f.debug_tuple("FieldDrain")
                 .field(&[] as &[UnknownField; 0])
                 .finish()
            }
        }
    }
}

impl Iterator for FieldDrain<'_> {
    type Item = UnknownField;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(<_>::next)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.as_ref().map_or((0, Some(0)), <_>::size_hint)
    }
}
impl DoubleEndedIterator for FieldDrain<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<UnknownField> {
        self.0.as_mut().and_then(<_>::next_back)
    }
}
impl ExactSizeIterator for FieldDrain<'_> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.0.as_ref().map_or(true, <_>::is_empty)
    }
}
impl FusedIterator for FieldDrain<'_> { }

#[cfg(test)]
mod test {
    
}