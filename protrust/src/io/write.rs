//! Defines the `CodedWriter`, a writer for writing protobuf encoded values to streams.

use core::fmt::{self, Display, Formatter};
use core::mem::ManuallyDrop;
use core::ops;
use core::ptr::NonNull;
use crate::collections::{RepeatedValue, FieldSet};
use crate::internal::Sealed;
use crate::io::{Length, stream::{self, Write}, Tag};
use crate::raw::Value;
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error;

mod internal {
    use core::convert::TryFrom;
    use core::mem;
    use crate::io::{internal::Array, stream, raw_varint32_size, raw_varint64_size};
    use super::{Any, Result, Error};

    pub trait Writer {
        fn write_varint32(&mut self, value: u32) -> Result;
        fn write_varint64(&mut self, value: u64) -> Result;
        fn write_bit32(&mut self, value: u32) -> Result;
        fn write_bit64(&mut self, value: u64) -> Result;
        fn write_length_delimited(&mut self, value: &[u8]) -> Result;

        fn into_any<'a>(&'a mut self) -> Any<'a>;
        fn from_any<'a>(&'a mut self, any: Any<'a>);
    }

    pub struct FlatBuffer<'a> {
        inner: &'a mut [u8]
    }

    impl<'a> FlatBuffer<'a> {
        #[inline]
        pub fn write_array<A: Array>(&mut self, value: A) -> Result {
            if A::LENGTH > self.inner.len() {
                Err(stream::Error.into())
            } else {
                let (a, b) = mem::replace(&mut self.inner, &mut []).split_at_mut(A::LENGTH);
                a.copy_from_slice(value.as_ref());
                self.inner = b;
                Ok(())
            }
        }
    }

    impl Writer for FlatBuffer<'_> {
        fn write_varint32(&mut self, value: u32) -> Result {
            unimplemented!()
        }
        fn write_varint64(&mut self, value: u64) -> Result {
            unimplemented!()
        }
        fn write_bit32(&mut self, value: u32) -> Result {
            self.write_array(u32::to_le_bytes(value))
        }
        fn write_bit64(&mut self, value: u64) -> Result {
            self.write_array(u64::to_le_bytes(value))
        }
        fn write_length_delimited(&mut self, value: &[u8]) -> Result {
            let len = i32::try_from(value.len()).map_err(|_| Error::ValueTooLarge)?;
            self.write_varint32(len as u32)?;
            if self.inner.len() > value.len() {
                self.inner[..value.len()].copy_from_slice(value);
                Ok(())
            } else {
                Err(stream::Error.into())
            }
        }

        fn into_any<'a>(&'a mut self) -> Any<'a> {
            unimplemented!()
        }
        fn from_any<'a>(&'a mut self, any: Any<'a>) {
            unimplemented!()
        }
    }
}

use internal::Writer;

/// The error type for [`CodedWriter`](struct.CodedWriter.html)
#[derive(Debug)]
pub enum Error {
    /// An error used to indicate a value was provided that was 
    /// too large to write to an output.
    ValueTooLarge,
    /// An error occured while writing data to the output.
    /// For slice outputs, this is used to indicate if
    /// not all data could be written to the slice.
    IoError(stream::Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::ValueTooLarge => write!(f, "the value was too large to write to the output"),
            Error::IoError(_) => write!(f, "an error occured while writing to the output")
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            _ => None
        }
    }
}

impl From<stream::Error> for Error {
    fn from(e: stream::Error) -> Self {
        Self::IoError(e)
    }
}

/// A result for a [`CodedWriter`](struct.CodedWriter.html) read operation
pub type Result = core::result::Result<(), Error>;

pub trait Output: Sealed {
    type Writer: internal::Writer;
}

pub struct Slice<'a>(&'a mut [u8]);
impl Sealed for Slice<'_> { }
impl<'a> Output for Slice<'a> {
    type Writer = internal::FlatBuffer<'a>;
}

pub struct Stream<T>(T);

pub struct Any<'a> {
    inner: Option<&'a mut dyn Write>
}
impl Sealed for Any<'_> { }
impl<'a> Output for Any<'a> {
    type Writer = Self;
}
impl internal::Writer for Any<'_> {
    fn write_varint32(&mut self, mut value: u32) -> Result {
        unimplemented!()
    }
    fn write_varint64(&mut self, mut value: u64) -> Result {
        unimplemented!()
    }
    fn write_bit32(&mut self, value: u32) -> Result {
        unimplemented!()
    }
    fn write_bit64(&mut self, value: u64) -> Result {
        unimplemented!()
    }
    fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        unimplemented!()
    }
    fn into_any<'a>(&'a mut self) -> Any<'a> {
        Any { inner: self.inner.as_mut().map::<&'a mut dyn Write, _>(|v| &mut **v) }
    }
    fn from_any<'a>(&'a mut self, _any: Any<'a>) { }
}

pub struct AnyConverter<'a, T: Output + 'a> {
    src: NonNull<CodedWriter<T>>,
    brdg: ManuallyDrop<CodedWriter<Any<'a>>>
}

impl<'a, T: Output> AnyConverter<'a, T> {
    fn new(src: &'a mut CodedWriter<T>) -> Self {
        Self {
            src: unsafe { NonNull::new_unchecked(src) }, // don't use from since the borrow moves into the from call
            brdg: ManuallyDrop::new(CodedWriter { inner: src.inner.into_any() })
        }
    }
}

impl<'a, T: Output> ops::Deref for AnyConverter<'a, T> {
    type Target = CodedWriter<Any<'a>>;

    fn deref(&self) -> &CodedWriter<Any<'a>> {
        &self.brdg
    }
}

impl<'a, T: Output> ops::DerefMut for AnyConverter<'a, T> {
    fn deref_mut(&mut self) -> &mut CodedWriter<Any<'a>> {
        &mut self.brdg
    }
}

impl<'a, T: Output> Drop for AnyConverter<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let src: &'a mut CodedWriter<T> = &mut *self.src.as_ptr();
            src.inner.from_any(ManuallyDrop::take(&mut self.brdg).inner);
        }
    }
}

pub struct CodedWriter<T: Output> {
    inner: T::Writer,
}

impl<T: Output> CodedWriter<T> {
    pub fn as_any<'a>(&'a mut self) -> AnyConverter<'a, T> {
        AnyConverter::new(self)
    }

    pub fn write_varint32(&mut self, value: u32) -> Result {
        unimplemented!()
    }
    pub fn write_varint64(&mut self, value: u64) -> Result {
        unimplemented!()
    }
    pub fn write_bit32(&mut self, value: u32) -> Result {
        unimplemented!()
    }
    pub fn write_bit64(&mut self, value: u64) -> Result {
        unimplemented!()
    }
    pub fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        unimplemented!()
    }

    #[inline]
    pub fn write_length(&mut self, length: Length) -> Result {
        self.write_varint32(length.get() as u32)
    }
    #[inline]
    pub fn write_tag(&mut self, tag: Tag) -> Result {
        self.write_varint32(tag.get())
    }

    pub fn write_value<V: Value + Wrapper>(&mut self, value: &V::Inner) -> Result {
        V::wrap_ref(value).write_to(self)
    }
    pub fn write_values<U: RepeatedValue<V> + Wrapper, V>(&mut self, value: &U::Inner, tag: Tag) -> Result {
        U::wrap_ref(value).write_to(self, tag)
    }
    pub fn write_fields<U: FieldSet>(&mut self, value: &U) -> Result {
        value.write_to(self)
    }
}

#[cfg(test)]
mod test {
    
}