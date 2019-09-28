//! Defines the `CodedWriter`, a writer for writing protobuf encoded values to streams.

use core::convert::TryInto;
use core::fmt::{self, Display, Formatter};
use core::mem;
use core::ptr;
use core::slice;
use crate::collections;
use crate::io::{Length, stream::{self, Write}, Tag};
use crate::raw;
use either::{Either, Left, Right};
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error;

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

/// A coded input writer that writes to a borrowed [`Write`].
/// 
/// [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html
pub struct CodedWriter<'a> {
    inner: Either<&'a mut dyn Write, &'a mut [u8]>
}

impl<'a> CodedWriter<'a> {
    /// Creates a new [`CodedWriter`] over the borrowed [`Write`].
    /// 
    /// [`CodedWriter`]: struct.CodedWriter.html
    /// [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html
    #[inline]
    pub fn with_write(inner: &'a mut dyn Write) -> Self {
        Self { inner: Left(inner) }
    }
    /// Creates a new [`CodedWriter`] over the borrowed [`slice`].
    /// 
    /// [`CodedWriter`]: struct.CodedWriter.html
    /// [`slice`]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    #[inline]
    pub fn with_slice(inner: &'a mut [u8]) -> Self {
        Self { inner: Right(inner) }
    }

    /// Writes a tag to the output.
    #[inline]
    pub fn write_tag(&mut self, tag: Tag) -> Result {
        self.write_value::<raw::Uint32>(&tag.get())
    }

    /// Writes a length to the output.
    #[inline]
    pub fn write_length(&mut self, length: Length) -> Result {
        self.write_value::<raw::Uint32>(&(length.get() as u32))
    }

    /// Writes a 32-bit varint to the output. This is the same as upcasting 
    /// the value to a u64 and writing that, however this is more optimized 
    /// for writing 32-bit values.
    #[inline]
    pub fn write_varint32(&mut self, mut value: u32) -> Result {
        match self.inner {
            Left(ref mut write) => {
                let mut buf = [0u8; 5];
                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            let part = buf.get_unchecked(0..i);
                            write.write(part)?;
                            return Ok(())
                        }
                    }
                }
            },
            Right(ref mut buf) => {
                if raw::raw_varint32_size(value).get() as usize > buf.len() {
                    return Err(stream::Error.into());
                }

                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(i), buf.len() - i);
                            return Ok(())
                        }
                    }
                }
            }
        }
    }

    /// Writes a 64-bit varint to the output.
    #[inline]
    pub fn write_varint64(&mut self, mut value: u64) -> Result {
        match self.inner {
            Left(ref mut write) => {
                let mut buf = [0u8; 10];
                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            let part = buf.get_unchecked(0..i);
                            write.write(part)?;
                            return Ok(())
                        }
                    }
                }
            },
            Right(ref mut buf) => {
                if raw::raw_varint64_size(value).get() as usize > buf.len() {
                    return Err(stream::Error.into());
                }

                let mut i = 0;
                loop {
                    unsafe {
                        *buf.get_unchecked_mut(i) = (value & 0x7F) as u8;
                        value >>= 7;
                        i += 1;
                        if value == 0 {
                            *buf.get_unchecked_mut(i - 1) |= 0x80;
                            *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(i), buf.len() - i);
                            return Ok(())
                        }
                    }
                }
            }
        }
    }

    /// Writes a 32-bit little endian integer to the output.
    #[inline]
    pub fn write_bit32(&mut self, value: u32) -> Result {
        let value = value.to_le_bytes();
        const SIZE: usize = mem::size_of::<u32>();

        match self.inner {
            Left(ref mut i) => i.write(&value)?,
            Right(ref mut buf) => {
                if buf.len() < SIZE {
                    return Err(stream::Error.into());
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), SIZE);
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(SIZE), buf.len() - SIZE);
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes a 64-bit little endian integer to the output.
    #[inline]
    pub fn write_bit64(&mut self, value: u64) -> Result {
        let value = value.to_le_bytes();
        const SIZE: usize = mem::size_of::<u64>();
        match self.inner {
            Left(ref mut i) => i.write(&value)?,
            Right(ref mut buf) => {
                if buf.len() >= SIZE {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), SIZE);
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(SIZE), buf.len() - SIZE);
                    }
                } else {
                    return Err(stream::Error.into());
                }
            }
        }
        Ok(())
    }

    /// Writes raw bytes to the output. This should be used carefully as to not corrupt the coded output.
    #[inline]
    pub fn write_bytes(&mut self, value: &[u8]) -> Result {
        match &mut self.inner {
            Left(writer) => writer.write(value)?,
            Right(buf) => {
                if value.len() <= buf.len() {
                    unsafe {
                        ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), value.len());
                        *buf = slice::from_raw_parts_mut(buf.as_mut_ptr().add(value.len()), buf.len() - value.len());
                    }
                } else {
                    return Err(stream::Error.into());
                }
            }
        }
        Ok(())
    }

    /// Writes a length delimited set of bytes to the output.
    #[inline]
    pub fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        let len: i32 = value.len().try_into().map_err(|_| Error::ValueTooLarge)?;
        self.write_length(Length(len))?;
        self.write_bytes(value)
    }

    /// Writes a generic value to the output.
    #[inline]
    pub fn write_value<T: raw::Value + Wrapper>(&mut self, value: &T::Inner) -> Result {
        T::wrap_ref(value).write_to(self)
    }

    /// Writes a collection of values to the output.
    #[inline]
    pub fn write_values<T>(&mut self, value: &impl collections::RepeatedValue<T>, tag: Tag) -> Result {
        value.write_to(self, tag)
    }

    /// Writes a collection of fields to the output.
    #[inline]
    pub fn write_fields(&mut self, value: &impl crate::FieldSet) -> Result {
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