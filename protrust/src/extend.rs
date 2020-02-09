//! Types and traits for working with proto2 extensions

use crate::Mergable;
use crate::collections::{RepeatedField, FieldSet, TryRead};
use crate::internal::Sealed;
use crate::io::{read::{self, Input}, write::{self, Output}, FieldNumber, WireType, Tag, LengthBuilder, CodedReader, CodedWriter};
use crate::raw::{ValueType, Value, Packable, Packed};
use std::any::TypeId;
use std::borrow::{Borrow, Cow, ToOwned};
use std::collections::{HashMap, hash_map};
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::mem;

mod internal {
    use crate::{Mergable, merge};
    use crate::collections::{RepeatedField, RepeatedValue};
    use crate::io::{read, write, FieldNumber, WireType, Tag, LengthBuilder, CodedReader, CodedWriter};
    use crate::raw::{ValueType, Value, Packable, Packed};
    use std::any::{Any, TypeId};
    use std::fmt::{self, Debug, Formatter};
    use super::ExtendableMessage;

    pub trait ExtensionIdentifier: Sync {
        fn field_number(&self) -> FieldNumber;
        fn message_type(&self) -> TypeId;

        fn try_read_value(&self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<Box<dyn AnyExtension>>>;
    }

    pub enum TryReadValue<U> {
        Yielded,
        Consumed(U)
    }

    pub trait ExtensionType: Sized + ExtensionIdentifier {
        type Entry: AnyExtension + AsRef<Self::Value> + AsMut<Self::Value>;
        type Extended: ExtendableMessage;
        type Value;

        fn new_entry(&self, value: Self::Value) -> Self::Entry;
        fn entry_value(entry: Self::Entry) -> Self::Value;
    }

    pub trait AnyExtension: Any + Debug + Send + Sync {
        fn clone_into_box(&self) -> Box<dyn AnyExtension>;
        fn merge(&mut self, other: &dyn AnyExtension);
        fn eq(&self, other: &dyn AnyExtension) -> bool;
        fn field_number(&self) -> FieldNumber;

        fn try_merge_from(&mut self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<()>>;
        fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder>;
        fn write_to(&self, output: &mut CodedWriter<write::Any>) -> write::Result;
        fn is_initialized(&self) -> bool;
    }

    pub struct ExtensionValue<V: ValueType> {
        pub value: V::Inner,
        pub num: FieldNumber,
    }

    impl<V: ValueType> AsRef<V::Inner> for ExtensionValue<V> {
        fn as_ref(&self) -> &V::Inner {
            &self.value
        }
    }

    impl<V: ValueType> AsMut<V::Inner> for ExtensionValue<V> {
        fn as_mut(&mut self) -> &mut V::Inner {
            &mut self.value
        }
    }

    impl<V> AnyExtension for ExtensionValue<V>
        where
            V: Value + 'static,
            V::Inner: Clone + Mergable + PartialEq + Debug + Send + Sync
    {
        fn clone_into_box(&self) -> Box<dyn AnyExtension> {
            Box::new(
                Self {
                    value: self.value.clone(),
                    num: self.num
                }
            )
        }
        fn merge(&mut self, other: &dyn AnyExtension) {
            assert_eq!(TypeId::of::<Self>(), other.type_id());

            let other: &Self = unsafe { &*(other as *const dyn AnyExtension as *const Self) };
            merge(&mut self.value, &other.value);
        }
        fn eq(&self, other: &dyn AnyExtension) -> bool {
            assert_eq!(TypeId::of::<Self>(), other.type_id());

            let other: &Self = unsafe { &*(other as *const dyn AnyExtension as *const Self) };
            self.value.eq(&other.value)
        }
        fn field_number(&self) -> FieldNumber { self.num }

        fn try_merge_from(&mut self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<()>> {
            if Some(Tag::new(self.num, V::WIRE_TYPE)) == input.last_tag() {
                input.merge_value::<V>(&mut self.value).map(TryReadValue::Consumed)
            } else {
                Ok(TryReadValue::Yielded)
            }
        }
        fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
            builder.add_field::<V>(self.num, &self.value)
        }
        fn write_to(&self, output: &mut CodedWriter<write::Any>) -> write::Result {
            output.write_field::<V>(self.num, &self.value)
        }
        fn is_initialized(&self) -> bool {
            V::is_initialized(&self.value)
        }
    }

    impl<V> Debug for ExtensionValue<V>
        where
            V: ValueType + 'static,
            V::Inner: Debug
    {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            self.value.fmt(f)
        }
    }

    pub struct RepeatedExtensionValue<V: ValueType> {
        pub value: RepeatedField<V::Inner>,
        pub num: FieldNumber,
    }

    impl<V: ValueType> AsRef<RepeatedField<V::Inner>> for RepeatedExtensionValue<V> {
        fn as_ref(&self) -> &RepeatedField<V::Inner> {
            &self.value
        }
    }

    impl<V: ValueType> AsMut<RepeatedField<V::Inner>> for RepeatedExtensionValue<V> {
        fn as_mut(&mut self) -> &mut RepeatedField<V::Inner> {
            &mut self.value
        }
    }

    impl<V> AnyExtension for RepeatedExtensionValue<V>
        where
            V: Value + 'static,
            V::Inner: Clone + PartialEq + Debug + Send + Sync
    {
        fn clone_into_box(&self) -> Box<dyn AnyExtension> {
            Box::new(
                Self {
                    value: self.value.clone(),
                    num: self.num,
                }
            )
        }
        fn merge(&mut self, other: &dyn AnyExtension) {
            assert_eq!(TypeId::of::<Self>(), other.type_id());

            #[allow(clippy::cast_ptr_alignment)]
            let other: &Self = unsafe { &*(other as *const dyn AnyExtension as *const Self) };
            merge(&mut self.value, &other.value);
        }
        fn eq(&self, other: &dyn AnyExtension) -> bool {
            assert_eq!(TypeId::of::<Self>(), other.type_id());

            #[allow(clippy::cast_ptr_alignment)]
            let other: &Self = unsafe { &*(other as *const dyn AnyExtension as *const Self) };
            self.value.eq(&other.value)
        }
        fn field_number(&self) -> FieldNumber { self.num }

        fn try_merge_from(&mut self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<()>> {
            let tag = input.last_tag().unwrap();
            let num = tag.field();
            let wt = tag.wire_type();
            if self.num == num && (wt == V::WIRE_TYPE || (V::WIRE_TYPE.is_packable() && wt == WireType::LengthDelimited)) {
                input.add_entries_to::<_, V>(&mut self.value).map(TryReadValue::Consumed)
            } else {
                Ok(TryReadValue::Yielded)
            }
        }
        fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
            builder.add_values::<_, V>(&self.value, self.num)
        }
        fn write_to(&self, output: &mut CodedWriter<write::Any>) -> write::Result {
            output.write_values::<_, V>(&self.value, self.num)
        }
        fn is_initialized(&self) -> bool {
            RepeatedValue::<V>::is_initialized(&self.value)
        }
    }

    impl<V> AnyExtension for RepeatedExtensionValue<Packed<V>>
        where
            V: Value + Packable + 'static,
            V::Inner: Clone + PartialEq + Debug + Send + Sync
    {
        fn clone_into_box(&self) -> Box<dyn AnyExtension> {
            Box::new(
                Self {
                    value: self.value.clone(),
                    num: self.num,
                }
            )
        }
        fn merge(&mut self, other: &dyn AnyExtension) {
            assert_eq!(TypeId::of::<Self>(), other.type_id());

            #[allow(clippy::cast_ptr_alignment)]
            let other: &Self = unsafe { &*(other as *const dyn AnyExtension as *const Self) };
            merge(&mut self.value, &other.value);
        }
        fn eq(&self, other: &dyn AnyExtension) -> bool {
            assert_eq!(TypeId::of::<Self>(), other.type_id());

            #[allow(clippy::cast_ptr_alignment)]
            let other: &Self = unsafe { &*(other as *const dyn AnyExtension as *const Self) };
            self.value.eq(&other.value)
        }
        fn field_number(&self) -> FieldNumber { self.num }

        fn try_merge_from(&mut self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<()>> {
            let tag = input.last_tag().unwrap();
            let num = tag.field();
            let wt = tag.wire_type();
            if self.num == num && (wt == WireType::LengthDelimited || wt == V::WIRE_TYPE) {
                input.add_entries_to::<_, Packed<V>>(&mut self.value).map(TryReadValue::Consumed)
            } else {
                Ok(TryReadValue::Yielded)
            }
        }
        fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
            builder.add_values::<_, Packed<V>>(&self.value, self.num)
        }
        fn write_to(&self, output: &mut CodedWriter<write::Any>) -> write::Result {
            output.write_values::<_, Packed<V>>(&self.value, self.num)
        }
        fn is_initialized(&self) -> bool {
            RepeatedValue::<Packed<V>>::is_initialized(&self.value)
        }
    }

    impl<V: ValueType> Debug for RepeatedExtensionValue<V>
        where V::Inner: Debug
    {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            self.value.fmt(f)
        }
    }
}

use internal::{ExtensionIdentifier, ExtensionType, AnyExtension, TryReadValue};

/// A message type that can be extended by third-party extension fields.
/// 
/// This trait exposes an `ExtensionSet` which can be used to get or set fields based on
/// an "extension identifier".
pub trait ExtendableMessage: Sized {
    /// Returns an immutable shared reference to the extension set in this message
    fn extensions(&self) -> &ExtensionSet<Self>;
    /// Returns a mutable unique reference to the extension set in this message
    fn extensions_mut(&mut self) -> &mut ExtensionSet<Self>;
}

/// An extension identifier for accessing an extension value from an ExtensionSet
pub struct Extension<T, V, D = <V as ValueType>::Inner>
    where
        V: ValueType,
        V::Inner: Borrow<D>,
        D: ?Sized + ToOwned<Owned = V::Inner> + 'static {
    t: PhantomData<fn(T) -> V>,
    num: FieldNumber,
    default: Option<Cow<'static, D>>
}

impl<T, V, D> ExtensionIdentifier for Extension<T, V, D>
    where
        T: ExtendableMessage + 'static,
        V: Value + 'static,
        V::Inner: Mergable + Borrow<D> + PartialEq + Clone + Debug + Send + Sync,
        D: ?Sized + ToOwned<Owned = V::Inner> + Sync + 'static
{
    fn field_number(&self) -> FieldNumber {
        self.num
    }
    fn message_type(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn try_read_value(&self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<Box<dyn AnyExtension>>> {
        if Some(Tag::new(self.num, V::WIRE_TYPE)) == input.last_tag() {
            input.read_value::<V>().map::<TryReadValue<Box<dyn AnyExtension>>, _>(|v| TryReadValue::Consumed(Box::new(self.new_entry(v))))
        } else {
            Ok(TryReadValue::Yielded)
        }
    }
}

impl<T, V, D> ExtensionType for Extension<T, V, D>
    where
        T: ExtendableMessage + 'static,
        V: Value + 'static,
        V::Inner: Mergable + Borrow<D> + PartialEq + Clone + Debug + Send + Sync,
        D: ?Sized + ToOwned<Owned = V::Inner> + Sync + 'static
{
    type Entry = internal::ExtensionValue<V>;
    type Extended = T;
    type Value = V::Inner;

    fn new_entry(&self, value: Self::Value) -> Self::Entry {
        internal::ExtensionValue {
            num: self.num,
            value
        }
    }
    fn entry_value(entry: Self::Entry) -> Self::Value {
        entry.value
    }
}

#[doc(hidden)]
impl<T, V, D> Extension<T, V, D>
    where
        V: ValueType,
        V::Inner: Borrow<D>,
        D: ?Sized + ToOwned<Owned = V::Inner> + 'static {
    pub const fn with_static_default(num: FieldNumber, default: &'static D) -> Self {
        Self {
            t: PhantomData,
            num,
            default: Some(Cow::Borrowed(default))
        }
    }
    pub const fn with_no_default(num: FieldNumber) -> Self {
        Self {
            t: PhantomData,
            num,
            default: None
        }
    }
}

#[doc(hidden)]
impl<T, V, D> Extension<T, V, D>
    where
        V: ValueType,
        V::Inner: Borrow<D>,
        D: Sized + ToOwned<Owned = V::Inner> + 'static {
    pub const fn with_owned_default(num: FieldNumber, default: V::Inner) -> Self {
        Self {
            t: PhantomData,
            num,
            default: Some(Cow::Owned(default))
        }
    }
}

/// An extension identifier for accessing a repeated extension value from an `ExtensionSet`
pub struct RepeatedExtension<T, V: ValueType> {
    t: PhantomData<fn(T) -> RepeatedField<V::Inner>>,
    num: FieldNumber
}

impl<T, V> ExtensionIdentifier for RepeatedExtension<T, V>
    where
        T: ExtendableMessage + 'static,
        V: Value + 'static,
        V::Inner: Clone + PartialEq + Debug + Send + Sync
{
    fn field_number(&self) -> FieldNumber {
        self.num
    }
    fn message_type(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn try_read_value(&self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<Box<dyn AnyExtension>>> {
        let tag = input.last_tag().unwrap();
        if Tag::new(self.num, V::WIRE_TYPE) == tag || (V::WIRE_TYPE.is_packable() && Tag::new(self.num, WireType::LengthDelimited) == tag) {
            let mut v = RepeatedField::new();
            input.add_entries_to::<_, V>(&mut v)?;

            Ok(TryReadValue::Consumed(Box::new(self.new_entry(v))))
        } else {
            Ok(TryReadValue::Yielded)
        }
    }
}

impl<T, V> ExtensionIdentifier for RepeatedExtension<T, Packed<V>>
    where
        T: ExtendableMessage + 'static,
        V: Value + Packable + 'static,
        V::Inner: Clone + PartialEq + Debug + Send + Sync
{
    fn field_number(&self) -> FieldNumber {
        self.num
    }
    fn message_type(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn try_read_value(&self, input: &mut CodedReader<read::Any>) -> read::Result<TryReadValue<Box<dyn AnyExtension>>> {
        let tag = input.last_tag().unwrap();
        if Tag::new(self.num, WireType::LengthDelimited) == tag || Tag::new(self.num, V::WIRE_TYPE) == tag {
            let mut v = RepeatedField::new();
            input.add_entries_to::<_, V>(&mut v)?;

            Ok(TryReadValue::Consumed(Box::new(self.new_entry(v))))
        } else {
            Ok(TryReadValue::Yielded)
        }
    }
}

impl<T, V> ExtensionType for RepeatedExtension<T, V>
    where
        T: ExtendableMessage + 'static,
        V: Value + 'static,
        V::Inner: Clone + PartialEq + Debug + Send + Sync
{
    type Entry = internal::RepeatedExtensionValue<V>;
    type Extended = T;
    type Value = RepeatedField<V::Inner>;

    fn new_entry(&self, value: Self::Value) -> Self::Entry {
        internal::RepeatedExtensionValue {
            num: self.num,
            value
        }
    }
    fn entry_value(entry: Self::Entry) -> Self::Value {
        entry.value
    }
}


impl<T, V> ExtensionType for RepeatedExtension<T, Packed<V>>
    where
        T: ExtendableMessage + 'static,
        V: Value + Packable + 'static,
        V::Inner: Clone + PartialEq + Debug + Send + Sync
{
    type Entry = internal::RepeatedExtensionValue<Packed<V>>;
    type Extended = T;
    type Value = RepeatedField<V::Inner>;

    fn new_entry(&self, value: Self::Value) -> Self::Entry {
        internal::RepeatedExtensionValue {
            num: self.num,
            value
        }
    }
    fn entry_value(entry: Self::Entry) -> Self::Value {
        entry.value
    }
}


/// A registry used to contain all the extensions from a generated code module
pub struct ExtensionRegistry {
    by_num: HashMap<FieldNumber, &'static dyn ExtensionIdentifier>
}

impl ExtensionRegistry {
    /// Returns whether an extension registry
    pub fn contains<T: ?Sized + ExtensionIdentifier>(&self, id: &T) -> bool {
        self.by_num
            .get(&id.field_number())
            .map(|b| *b as *const dyn ExtensionIdentifier as *const u8 == id as *const T as *const u8)
            .unwrap_or(false)
    }
}

impl Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.by_num.keys().fmt(f)
    }
}

/// A builder used to construct extension registries in generated code
#[derive(Default)]
pub struct RegistryBuilder {
    by_num: HashMap<FieldNumber, &'static dyn ExtensionIdentifier>,
}

impl RegistryBuilder {
    /// Creates a new registry builder for building an extension registry
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    /// Adds the extensions in the specified registry to this registry
    #[inline]
    pub fn add_registry(mut self, registry: &'static ExtensionRegistry) -> Result<Self, ExtensionConflict> {
        for (&num, &id) in &registry.by_num {
            if self.by_num.insert(num, id).is_some() {
                return Err(ExtensionConflict(num));
            }
        }

        Ok(self)
    }
    /// Adds an extension identifier to this registry
    #[inline]
    pub fn add_identifier(mut self, id: &'static dyn ExtensionIdentifier) -> Result<Self, ExtensionConflict> {
        let num = id.field_number();
        match self.by_num.insert(num, id) {
            Some(_) => Err(ExtensionConflict(num)),
            None => Ok(self)
        }
    }
    /// Returns the extension registry
    #[inline]
    pub fn build(self) -> ExtensionRegistry {
        ExtensionRegistry { by_num: self.by_num }
    }
}

/// An error returned when two extensions are added to a registry builder that use the same field number
pub struct ExtensionConflict(FieldNumber);

/// A set of extension values that can be accessed by using generated extension identifiers
pub struct ExtensionSet<T: ExtendableMessage> {
    t: PhantomData<fn(T)>,
    registry: Option<&'static ExtensionRegistry>,
    by_num: HashMap<FieldNumber, Box<dyn AnyExtension>>,
}

impl<T: ExtendableMessage + 'static> ExtensionSet<T> {
    fn registry_contains<I: ?Sized + ExtensionIdentifier>(&self, extension: &I) -> bool {
        self.registry.map_or(false, |r| r.contains(extension))
    }

    /// Returns a new set for this specified message
    pub fn new() -> Self {
        Default::default()
    }
    /// Returns the registry in use by the set
    pub fn registry(&self) -> Option<&'static ExtensionRegistry> {
        self.registry
    }
    /// Returns if the registry contained in this set is equal to the provided registry
    pub fn has_registry(&self, registry: Option<&'static ExtensionRegistry>) -> bool {
        match (self.registry(), registry) {
            (Some(r), Some(o)) => std::ptr::eq(r, o),
            (None, None) => true,
            _ => false
        }
    }
    /// Replaces the extension registry used by this set with another registry or None to not use extensions in this set.
    /// This returns the last registry used.
    /// 
    /// This clears all set extension values in this set even if you're replacing the registry with the same one.
    pub fn replace_registry(&mut self, new: Option<&'static ExtensionRegistry>) -> Option<&'static ExtensionRegistry> {
        self.by_num.clear();
        mem::replace(&mut self.registry, new)
    }

    /// Returns whether the specified extension is contained in the registry used by this set
    /// and if the field has a set value.
    pub fn has_extension<U: ?Sized + ExtensionIdentifier>(&self, extension: &U) -> bool {
        self.registry_contains(extension) && self.has_extension_unchecked(extension)
    }
    /// Returns whether a field in this set has the field number of the specified extension
    pub fn has_extension_unchecked<U: ?Sized + ExtensionIdentifier>(&self, extension: &U) -> bool {
        self.by_num.contains_key(&extension.field_number())
    }

    /// Gets the value of the specified extension if it's set. If the extension is not set, this returns None.
    pub fn value<U: ExtensionType<Extended = T>>(&self, extension: &U) -> Option<&U::Value> {
        if self.registry_contains(extension) {
            self.by_num.get(&extension.field_number()).map(|v| unsafe {
                (*(v.as_ref() as *const dyn AnyExtension as *const U::Entry)).as_ref()
            })
        } else {
            None
        }
    }

    /// Gets the value of the specified extension from the set or the default value for the extension if it exists
    pub fn value_or_default<'a, 'e: 'a, V, D>(&'a self, extension: &'e Extension<T, V, D>) -> Option<&'a D>
        where
            V: Value + 'static,
            V::Inner: Borrow<D> + Mergable + Clone + PartialEq + Debug + Send + Sync,
            D: ?Sized + ToOwned<Owned = V::Inner> + Sync + 'static
    {
        self.value(extension).map(|v| v.borrow()).or_else(|| extension.default.as_ref().map(|v| v.borrow()))
    }

    /// Returns a Field which can be used to modify an extension value
    pub fn field<'a, 'e, U: 'e + ExtensionType<Extended = T>>(&'a mut self, extension: &'e U) -> Option<Field<'a, 'e, U>> {
        if self.registry_contains(extension) {
            match self.by_num.entry(extension.field_number()) {
                hash_map::Entry::Occupied(entry) => Some(Field::Occupied(OccupiedField { extension, entry })),
                hash_map::Entry::Vacant(entry) => Some(Field::Vacant(VacantField { extension, entry })),
            }
        } else {
            None
        }
    }
}

impl<T: ExtendableMessage + 'static> Sealed for ExtensionSet<T> { }
impl<T: ExtendableMessage + 'static> FieldSet for ExtensionSet<T> {
    fn try_add_field_from<'a, U: Input>(&mut self, input: &'a mut CodedReader<U>) -> read::Result<TryRead<'a, U>> {
        if let Some(tag) = input.last_tag() {
            let field = tag.field();
            match self.by_num.entry(field) {
                hash_map::Entry::Occupied(entry) => {
                    let entry = entry.into_mut();
                    let mut any = input.as_any();
                    match entry.try_merge_from(&mut any)? {
                        TryReadValue::Consumed(()) => Ok(TryRead::Consumed),
                        TryReadValue::Yielded => {
                            drop(any);
                            Ok(TryRead::Yielded(input))
                        }
                    }
                },
                // if the value doesn't already exist, try to find it in our registry
                hash_map::Entry::Vacant(entry) => {
                    if let Some(registry) = self.registry {
                        if let Some(ext) = registry.by_num.get(&field) {
                            let mut any = input.as_any();
                            return match ext.try_read_value(&mut any)? {
                                TryReadValue::Consumed(b) => {
                                    entry.insert(b);
                                    Ok(TryRead::Consumed)
                                },
                                TryReadValue::Yielded => {
                                    drop(any);
                                    Ok(TryRead::Yielded(input))
                                }
                            };
                        }
                    }

                    Ok(TryRead::Yielded(input))
                },
            }
        } else {
            Ok(TryRead::Consumed)
        }
    }
    fn calculate_size(&self, builder: LengthBuilder) -> Option<LengthBuilder> {
        self.by_num
            .values()
            .try_fold(builder, |builder, field| field.calculate_size(builder))
    }
    fn write_to<U: Output>(&self, output: &mut CodedWriter<U>) -> write::Result {
        if !self.by_num.is_empty() {
            let mut output = output.as_any();
            for field in self.by_num.values() {
                field.write_to(&mut output)?;
            }
        }
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        for field in self.by_num.values() {
            if !field.is_initialized() {
                return false;
            }
        }

        true
    }
}

/// An extension field entry in an extension set.
pub enum Field<'a, 'e, T: 'e> {
    /// An occupied field
    Occupied(OccupiedField<'a, 'e, T>),
    /// A vacant field
    Vacant(VacantField<'a, 'e, T>),
}

impl<'a, 'e, T: 'e + ExtensionType> Field<'a, 'e, T> {
    /// Inserts a default value into the field, returning a mutable reference to the value
    pub fn or_insert(self, default: T::Value) -> &'a mut T::Value {
        match self {
            Field::Occupied(entry) => entry.into_mut(),
            Field::Vacant(entry) => entry.insert(default),
        }
    }

    /// Returns a mutable reference to the value, calling the specified function to 
    /// create the value and insert it if it does not exist.
    pub fn or_insert_with<F: FnOnce() -> T::Value>(self, default: F) -> &'a mut T::Value {
        match self {
            Field::Occupied(entry) => entry.into_mut(),
            Field::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Modifies an existing value in the field, doing nothing if the field is empty
    /// and returns the Field.
    pub fn and_modify<F: FnOnce(&mut T::Value)>(mut self, f: F) -> Self {
        match self {
            Field::Occupied(ref mut entry) => f(entry.get_mut()),
            Field::Vacant(_) => { }
        }

        self
    }
}

/// Represents an occupied field in an extension set
pub struct OccupiedField<'a, 'e, T: 'e> {
    extension: &'e T,
    entry: hash_map::OccupiedEntry<'a, FieldNumber, Box<dyn AnyExtension>>,
}

impl<'a, 'e, T: 'e + ExtensionType> OccupiedField<'a, 'e, T> {
    /// Gets the extension identifier for this field
    pub fn extension(&self) -> &'e T {
        self.extension
    }

    /// Takes ownership of the value, removing it from the set
    pub fn remove(self) -> T::Value {
        let raw = Box::into_raw(self.entry.remove());
        let casted = unsafe { Box::from_raw(raw as *mut T::Entry) };
        T::entry_value(*casted)
    }

    /// Gets a reference to the value in the field.
    pub fn get(&self) -> &T::Value {
        let ptr = self.entry.get().as_ref() as *const dyn AnyExtension as *const T::Entry;
        unsafe { (*ptr).as_ref() }
    }

    /// Gets a mutable reference to the value in the field.
    pub fn get_mut(&mut self) -> &mut T::Value {
        let ptr = self.entry.get_mut().as_mut() as *mut dyn AnyExtension as *mut T::Entry;
        unsafe { (*ptr).as_mut() }
    }

    /// Converts the field into a mutable reference to the value in the entry with a lifetime bound to the set.
    pub fn into_mut(self) -> &'a mut T::Value {
        let ptr = self.entry.into_mut().as_mut() as *mut dyn AnyExtension as *mut T::Entry;
        unsafe { (*ptr).as_mut() }
    }

    /// Sets the value of the field and returns the field's old value
    pub fn insert(&mut self, value: T::Value) -> T::Value {
        std::mem::replace(self.get_mut(), value)
    }
}

/// Represents a field without a value in the set
pub struct VacantField<'a, 'e, T: 'e> {
    extension: &'e T,
    entry: hash_map::VacantEntry<'a, FieldNumber, Box<dyn AnyExtension>>,
}

impl<'a, 'e, T: 'e + ExtensionType> VacantField<'a, 'e, T> {
    /// Gets the extension identifier for this field
    pub fn extension(&self) -> &'e T {
        self.extension
    }
    /// Inserts a value for the field, returning a mutable reference to the value
    pub fn insert(self, value: T::Value) -> &'a mut T::Value {
        let borrow = self.entry.insert(Box::new(self.extension.new_entry(value)));
        let ptr = borrow.as_mut() as *mut dyn AnyExtension as *mut T::Entry;
        unsafe { (*ptr).as_mut() }
    }
}

impl<T: ExtendableMessage> Default for ExtensionSet<T> {
    fn default() -> Self {
        Self {
            t: PhantomData,
            registry: None,
            by_num: Default::default()
        }
    }
}