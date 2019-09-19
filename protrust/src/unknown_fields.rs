//! Defines the `UnknownFieldSet`, a struct used to contain unknown fields as they were read from coded readers.
//! 
//! As APIs are updated, certain fields may be removed or added from proto file definitions. If an old version of a message 
//! encounters fields it doesn't recognize can still read them to be returned again via unknown fields.
//! 
//! Unknown fields for unique field numbers can exist for multiple wire types at once to ensure that all data is properly returned.

use crate::{internal, Mergable, FieldSet, FieldReadState};
use crate::io::{FieldNumber, WireType, Tag, LengthBuilder, CodedReader, ReaderResult, CodedWriter, WriterResult};
use crate::raw;
use hashbrown::{HashMap, hash_map};
use std::alloc::{Alloc, Global};
use std::ops::RangeBounds;
use std::vec;

/// An unknown field in an [`UnknownFieldSet`](struct.UnknownFieldSet.html).
#[derive(Clone, Debug)]
pub enum UnknownField<A: Alloc> {
    /// A varint field value
    Varint(u64),
    /// A 64-bit field value
    Bit64(u64),
    /// A length delimited series of bytes
    LengthDelimited(Box<[u8]>),
    /// A group of other unknown fields
    Group(UnknownFieldSet<A>),
    /// A 32-bit field value
    Bit32(u32)
}

/// A set of unknown fields encountered while parsing
#[derive(Default, Clone, Debug)]
pub struct UnknownFieldSet<A: Alloc> {
    inner: HashMap<FieldNumber, Vec<UnknownField<A>>>,
    alloc: A
}

impl<A: Alloc> internal::Sealed for UnknownFieldSet<A> { }
impl<A: Alloc + Clone> Mergable for UnknownFieldSet<A> {
    fn merge(&mut self, other: &Self) {
        for (key, values) in &other.inner {
            self.inner.entry(*key).or_insert_with(Vec::new).extend(values.iter().cloned())
        }
    }
}
impl FieldSet for UnknownFieldSet<Global> {
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
        for (key, values) in &self.inner {
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
impl UnknownFieldSet<Global> {
    fn add_field_from(&mut self, input: &mut CodedReader) -> ReaderResult<()> {
        if let Some(last_tag) = input.last_tag() {
            match last_tag.wire_type() {
                WireType::Varint => self.push_value(last_tag.number(), UnknownField::Varint(dbg!(input.read_varint64()?))),
                WireType::Bit64 => self.push_value(last_tag.number(), UnknownField::Bit64(input.read_bit64()?)),
                WireType::LengthDelimited => self.push_value(last_tag.number(), UnknownField::LengthDelimited(input.read_length_delimited::<_>(self.alloc.clone())?)),
                WireType::StartGroup => {
                    let mut group = UnknownFieldSet::new(self.alloc.clone());
                    let end_tag = Tag::new(last_tag.number(), WireType::EndGroup);
                    while let Some(tag) = input.read_tag()? {
                        if tag != end_tag {
                            group.add_field_from(input)?;
                        } else {
                            break;
                        }
                    }
                    self.push_value(last_tag.number(), UnknownField::Group(group));
                },
                WireType::Bit32 => self.push_value(last_tag.number(), UnknownField::Bit32(input.read_bit32()?)),
                WireType::EndGroup => unreachable!()
            }
        }
        Ok(())
    }
}
impl<A: Alloc> UnknownFieldSet<A> {
    /// Creates a new unknown field set in the specified allocator
    pub fn new(a: A) -> Self {
        Self {
            inner: Default::default(),
            alloc: a
        }
    }
    /// Gets the number of fields present in this set
    pub fn field_len(&self) -> usize {
        self.inner.len()
    }
    /// Returns a slice of values for a field
    pub fn values(&self, num: FieldNumber) -> &[UnknownField<A>] {
        self.inner.get(&num).map(Vec::as_slice).unwrap_or(&[])
    }
    /// Returns a mutable slice of values for a field
    pub fn values_mut(&mut self, num: FieldNumber) -> &mut [UnknownField<A>] {
        self.inner.get_mut(&num).map(Vec::as_mut_slice).unwrap_or(&mut [])
    }
    /// Pushes an new value to the field
    pub fn push_value(&mut self, num: FieldNumber, value: UnknownField<A>) {
        self.inner.entry(num).or_insert_with(Vec::new).push(value)
    }
    /// Pops the last value added for the specified field
    pub fn pop_value(&mut self, num: FieldNumber) -> Option<UnknownField<A>> {
        self.inner.get_mut(&num).and_then(Vec::pop)
    }
    /// Returns an iterator of all of the fields in the set
    pub fn fields<'a>(&'a self) -> Iter<'a, A> {
        Iter(self.inner.iter())
    }
    /// Returns a mutable iterator of all the fields in the set
    pub fn fields_mut<'a>(&'a mut self) -> IterMut<'a, A> {
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
    pub fn field_numbers<'a>(&'a self) -> FieldNumbers<'a, A> {
        FieldNumbers(self.inner.keys())
    }
    /// Clears the set, returning the owned field values
    pub fn drain<'a>(&'a mut self) -> Drain<'a, A> {
        Drain(self.inner.drain())
    }
    /// Drains a range of values from a field
    pub fn drain_values<'a, R: RangeBounds<usize>>(&'a mut self, num: FieldNumber, range: R) -> FieldDrain<'a, A> {
        FieldDrain(self.inner.get_mut(&num).map(|v| v.drain(range)))
    }
}

/// An iterator over the fields of an unknown field set.
pub struct Iter<'a, A: Alloc>(hash_map::Iter<'a, FieldNumber, Vec<UnknownField<A>>>);

/// A mutable iterator over the fields of an unknown field set.
pub struct IterMut<'a, A: Alloc>(hash_map::IterMut<'a, FieldNumber, Vec<UnknownField<A>>>);

/// An iterator over the field numbers present in this set.
/// 
/// This `struct` is created by the [`field_numbers`] method on [`UnknownFieldSet`].
/// See its documentation for more.
/// 
/// [`field_numbers`]: struct.UnknownFieldSet.html#method.field_numbers
/// [`UnknownFieldSet`]: struct.UnknownFieldSet.html
pub struct FieldNumbers<'a, A: Alloc>(hash_map::Keys<'a, FieldNumber, Vec<UnknownField<A>>>);

/// A draining iterator that returns each field along with a boxed slice of unknown fields.
/// 
/// This `struct` is created by the [`drain`] method on [`UnknownFieldSet`].
/// See its documentation for more.
/// 
/// [`drain`]: struct.UnknownFieldSet.html#method.drain
/// [`UnknownFieldSet`]: struct.UnknownFieldSet.html
pub struct Drain<'a, A: Alloc>(hash_map::Drain<'a, FieldNumber, Vec<UnknownField<A>>>);

/// A draining iterator that returns the unknown fields for a single field.
/// 
/// This `struct` is created by the [`drain_field`] method on [`UnknownFieldSet`].
/// See its documentation for more.
/// 
/// [`drain_field`]: struct.UnknownFieldSet.html#method.drain_field
/// [`UnknownFieldSet`]: struct.UnknownFieldSet.html
pub struct FieldDrain<'a, A: Alloc>(Option<vec::Drain<'a, UnknownField<A>>>);

#[cfg(test)]
mod test {
    use super::{UnknownFieldSet, UnknownField};
    use crate::io::{Tag, FieldNumber, WireType, Length, CodedWriter, CodedReader};
    use crate::raw;
    use std::alloc::Global;

    #[test]
    fn sizes() {
        let set = {
            let mut set = UnknownFieldSet::new(Global);
            set.push_value(FieldNumber::new(1).unwrap(), UnknownField::Bit32(513));
            set
        };
        assert_eq!(Length::for_fields(&set).unwrap().get(), 5);

        let set = {
            let mut set = UnknownFieldSet::new(Global);
            set.push_value(FieldNumber::new(1).unwrap(), UnknownField::Bit32(513));
            set.push_value(FieldNumber::new(1).unwrap(), UnknownField::Bit32(1));
            set
        };

        assert_eq!(Length::for_fields(&set).unwrap().get(), 11);

        let set = {
            let group = set;
            let mut set = UnknownFieldSet::new(Global);
            set.push_value(FieldNumber::new(1).unwrap(), UnknownField::Group(group));
            set
        };

        assert_eq!(Length::for_fields(&set).unwrap().get(), 13);
    }
    #[test]
    fn read() {
        let mut set = UnknownFieldSet::new(Global);
        set.push_value(FieldNumber::new(1).unwrap(), UnknownField::Varint(120));

        let number = FieldNumber::new(1).unwrap();
        let input = {
            let mut data = Vec::new();
            let mut output = CodedWriter::with_write(&mut data);

            output.write_tag(Tag::new(number, WireType::Varint)).unwrap();
            output.write_value::<raw::Int32>(&10).unwrap();
            output.write_tag(Tag::new(number, WireType::Varint)).unwrap();
            output.write_value::<raw::Int64>(&10).unwrap();
            output.write_tag(Tag::new(number, WireType::Bit32)).unwrap();
            output.write_value::<raw::Fixed32>(&10).unwrap();
            output.write_tag(Tag::new(number, WireType::Bit64)).unwrap();
            output.write_value::<raw::Fixed64>(&10).unwrap();

            output.write_tag(Tag::new(number, WireType::StartGroup)).unwrap();
            output.write_tag(Tag::new(FieldNumber::new(2).unwrap(), WireType::Varint)).unwrap();
            output.write_value::<raw::Int32>(&10).unwrap();
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

        assert_eq!(set.values(number).len(), 6); // existing field + 5 added fields. the group counts as one field
    }
}