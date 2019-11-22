//! Defines the `CodedWriter`, a writer for writing protobuf encoded values to streams.

use core::fmt::{self, Display, Formatter};
use core::mem::ManuallyDrop;
use core::ptr::NonNull;
use crate::collections::{RepeatedValue, FieldSet};
use crate::internal::Sealed;
use crate::io::{Length, stream::{self, Write}, Tag};
use crate::raw::Value;
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error;

mod internal {
    use core::marker::PhantomData;
    use super::Any;

    pub trait Writer {
        fn into_any<'a>(&'a mut self) -> Any<'a>;
        fn from_any<'a>(&'a mut self, any: Any<'a>);
    }

    pub struct FlatBuffer<'a> {
        a: PhantomData<&'a mut [u8]>
    }

    impl Writer for FlatBuffer<'_> {
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
    use crate::io::{CodedWriter, Length};
    use crate::raw::{Uint32, Uint64};

    #[test]
    fn varint32_encode() {
        fn try_encode(value: u32, bytes: &[u8]) {
            let mut output = [0u8; 5];
            let mut writer = CodedWriter::with_slice(&mut output);
            writer.write_varint32(value).unwrap();

            let len = Length::for_value::<Uint32>(&value).unwrap().get() as usize;
            let slice = &output[0..len];

            assert_eq!(slice, bytes);
        }

        try_encode(0, &[0x80]);
        try_encode(127, &[0xFF]);
        try_encode(16_383, &[0x7F, 0xFF]);
        try_encode(2_097_151, &[0x7F, 0x7F, 0xFF]);
        try_encode(268_435_455, &[0x7F, 0x7F, 0x7F, 0xFF]);
        try_encode(u32::max_value(), &[0x7F, 0x7F, 0x7F, 0x7F, 0x8F]);
    }
    #[test]
    fn varint64_encode() {
        fn try_encode(value: u64, bytes: &[u8]) {
            let mut output = [0u8; 10];
            let mut writer = CodedWriter::with_slice(&mut output);
            writer.write_varint64(value).unwrap();

            let len = Length::for_value::<Uint64>(&value).unwrap().get() as usize;
            let slice = &output[0..len];

            assert_eq!(slice, bytes);
        }

        try_encode(0, &[0x80]);
        try_encode(127, &[0xFF]);
        try_encode(16_383, &[0x7F, 0xFF]);
        try_encode(2_097_151, &[0x7F, 0x7F, 0xFF]);
        try_encode(268_435_455, &[0x7F, 0x7F, 0x7F, 0xFF]);
        try_encode(u32::max_value() as u64, &[0x7F, 0x7F, 0x7F, 0x7F, 0x8F]);
        try_encode(u64::max_value(), &[0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x81]);
    }
    #[test]
    fn bit32_encode() {
        let value = 25;

        let mut output = [0u8; 4];
        let mut writer = CodedWriter::with_slice(&mut output);
        writer.write_bit32(value).unwrap();

        assert_eq!(output, value.to_le_bytes());

        let mut output = [0u8; 4];
        let mut write = output.as_mut();
        let mut writer = CodedWriter::with_write(&mut write);
        writer.write_bit32(value).unwrap();

        assert_eq!(output, value.to_le_bytes());
    }
    #[test]
    fn bit64_encode() {
        let value = 25;

        let mut output = [0u8; 8];
        let mut writer = CodedWriter::with_slice(&mut output);
        writer.write_bit64(value).unwrap();

        assert_eq!(output, value.to_le_bytes());

        let mut output = [0u8; 8];
        let mut write = output.as_mut();
        let mut writer = CodedWriter::with_write(&mut write);
        writer.write_bit64(value).unwrap();

        assert_eq!(output, value.to_le_bytes());
    }
    #[test]
    fn raw_bytes_encode() {
        let mut output = [0u8];
        let mut writer = CodedWriter::with_slice(&mut output);
        writer.write_bytes(&[1]).unwrap();

        assert_eq!(output, [1]);

        let mut output = [0u8];
        let mut write = output.as_mut();
        let mut writer = CodedWriter::with_write(&mut write);
        writer.write_bytes(&[1]).unwrap();

        assert_eq!(output, [1]);
    }

    // test that writing a value to less than the value's required space remaining returns an error
    macro_rules! fail_write_int {
        ($n:ident, $f:ident) => {
            #[test]
            fn $f() {
                let mut empty = [].as_mut();

                let mut writer = CodedWriter::with_slice(empty);
                assert!(writer.$f(10).is_err());

                let mut writer = CodedWriter::with_write(&mut empty);
                assert!(writer.$f(10).is_err());
            }
        };
    }

    fail_write_int!(fail_write_varint32, write_varint32);
    fail_write_int!(fail_write_varint64, write_varint64);
    fail_write_int!(fail_write_bit32, write_bit32);
    fail_write_int!(fail_write_bit64, write_bit64);

    #[test]
    fn fail_write_bytes() {
        let mut empty = [].as_mut();

        let mut writer = CodedWriter::with_slice(empty);
        assert!(writer.write_bytes(&[1]).is_err());

        let mut writer = CodedWriter::with_write(&mut empty);
        assert!(writer.write_bytes(&[1]).is_err());
    }
}