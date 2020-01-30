//! Defines the `CodedWriter`, a writer for writing protobuf encoded values to streams.

use alloc::boxed::Box;
use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::marker::PhantomData;
use core::ops::Range;
use core::ptr::{self, NonNull};
use core::slice;
use crate::collections::{RepeatedValue, FieldSet};
use crate::io::{FieldNumber, WireType, Tag, Length, DEFAULT_BUF_SIZE, stream::{self, Write}};
use crate::raw::Value;
use super::{raw_varint32_size, raw_varint64_size};

#[cfg(feature = "std")]
use std::error;

mod internal {
    use core::convert::TryFrom;
    use core::ptr::{self, NonNull};
    use core::slice;
    use crate::internal::Sealed;
    use crate::io::{raw_varint32_size, raw_varint64_size, stream::{self, Write}};
    use super::{Result, Error, write_varint32_unchecked, write_varint64_unchecked, write_bytes_unchecked};

    pub trait Writer {
        fn write_varint32(&mut self, value: u32) -> Result;
        fn write_varint64(&mut self, value: u64) -> Result;
        fn write_bit32(&mut self, value: u32) -> Result;
        fn write_bit64(&mut self, value: u64) -> Result;
        fn write_length_delimited(&mut self, value: &[u8]) -> Result;

        fn as_any(&mut self) -> Any;
    }

    struct BorrowedStream<'a> {
        pub output: &'a mut dyn Write,
        pub start: *mut u8,
        pub current: &'a mut *mut u8,
        pub end: *mut u8,
    }

    impl BorrowedStream<'_> {
        #[inline]
        fn remaining(&self) -> usize {
            usize::wrapping_sub(*self.current as _, self.end as _)
        }
        #[inline]
        fn capacity(&self) -> usize {
            usize::wrapping_sub(self.start as _, self.end as _)
        }
        #[inline]
        fn buffered(&self) -> usize {
            usize::wrapping_sub(self.start as _, *self.current as _)
        }
        #[inline]
        fn clear(&mut self) {
            *self.current = self.start;
        }
        fn flush(&mut self) -> Result {
            let slice = unsafe { slice::from_raw_parts(self.start, self.buffered()) };
            self.output.write(slice)?;
            self.clear();
            Ok(())
        }

        fn write_varint32(&mut self, value: u32, len: usize) -> Result {
            if self.remaining() < len {
                self.flush()?;
            }
            if len >= self.capacity() {
                let mut buf = [0; 5];
                unsafe { write_varint32_unchecked(value, &mut buf.as_mut_ptr()); }
                self.output.write(&buf[..len])?;
            } else {
                unsafe { write_varint32_unchecked(value, &mut self.current); }
            }
            Ok(())
        }
        fn write_varint64(&mut self, value: u64, len: usize) -> Result {
            if self.remaining() < len {
                self.flush()?;
            }
            if len >= self.capacity() {
                let mut buf = [0; 10];
                unsafe { write_varint64_unchecked(value, &mut buf.as_mut_ptr()); }
                self.output.write(&buf[..len])?;
            } else {
                unsafe { write_varint64_unchecked(value, &mut self.current); }
            }
            Ok(())
        }
        fn write_bit32(&mut self, value: u32) -> Result {
            let value = u32::to_le_bytes(value);
            if self.remaining() < value.len() {
                self.flush()?;
            }
            if value.len() >= self.capacity() {
                self.output.write(&value)?;
            } else {
                unsafe { write_bytes_unchecked(&value, &mut self.current); }
            }
            Ok(())
        }
        fn write_bit64(&mut self, value: u64) -> Result {
            let value = u64::to_le_bytes(value);
            if self.remaining() < value.len() {
                self.flush()?;
            }
            if value.len() >= self.capacity() {
                self.output.write(&value)?;
            } else {
                unsafe { write_bytes_unchecked(&value, &mut self.current); }
            }
            Ok(())
        }
        fn write_bytes(&mut self, value: &[u8]) -> Result {
            let len = value.len();
            if self.remaining() < len {
                self.flush()?;
            }
            if len >= self.capacity() {
                self.output.write(value)?;
            } else {
                unsafe { write_bytes_unchecked(value, &mut self.current); }
            }
            Ok(())
        }
    }

    /// 
    pub struct Any<'a> {
        pub(super) stream: Option<&'a mut dyn Write>,
        pub(super) start: Option<NonNull<u8>>,
        /// The current position of the write pointer
        pub(super) current: &'a mut *mut u8,
        /// The end of the buffer, used in length checks
        pub(super) end: Option<NonNull<u8>>,
    }
    impl<'a> Any<'a> {
        /// Returns whether `len` bytes can be written safely to the buffer
        #[inline]
        fn can_write(&self, len: usize) -> bool {
            match self.end {
                Some(end) => usize::wrapping_sub(*self.current as _, end.as_ptr() as _) > len,
                None => true
            }
        }
        fn as_borrowed_stream(&mut self) -> Option<BorrowedStream> {
            match &mut self.stream {
                Some(w) => {
                    Some(BorrowedStream {
                        output: *w,
                        start: self.start.map(NonNull::as_ptr).unwrap_or(ptr::null_mut()),
                        current: self.current,
                        end: self.end.map(NonNull::as_ptr).unwrap_or(ptr::null_mut())
                    })
                },
                None => None
            }
        }
    }
    impl Sealed for Any<'_> { }
    impl Writer for Any<'_> {
        fn write_varint32(&mut self, value: u32) -> Result {
            let len = raw_varint32_size(value).get() as usize;
            if self.can_write(len) {
                unsafe { write_varint32_unchecked(value, self.current); }
                Ok(())
            } else {
                if let Some(mut buffer) = self.as_borrowed_stream() {
                    buffer.write_varint32(value, len)
                } else {
                    Err(stream::Error.into())
                }
            }
        }
        fn write_varint64(&mut self, value: u64) -> Result {
            let len = raw_varint64_size(value).get() as usize;
            if self.can_write(len) {
                unsafe { write_varint64_unchecked(value, self.current); }
                Ok(())
            } else {
                if let Some(mut buffer) = self.as_borrowed_stream() {
                    buffer.write_varint64(value, len)
                } else {
                    Err(stream::Error.into())
                }
            }
        }
        fn write_bit32(&mut self, value: u32) -> Result {
            if self.can_write(4) {
                let value = u32::to_le_bytes(value);
                unsafe { write_bytes_unchecked(&value, self.current); }
                Ok(())
            } else {
                if let Some(mut buffer) = self.as_borrowed_stream() {
                    buffer.write_bit32(value)
                } else {
                    Err(stream::Error.into())
                }
            }
        }
        fn write_bit64(&mut self, value: u64) -> Result {
            if self.can_write(8) {
                let value = u64::to_le_bytes(value);
                unsafe { write_bytes_unchecked(&value, self.current); }
                Ok(())
            } else {
                if let Some(mut buffer) = self.as_borrowed_stream() {
                    buffer.write_bit64(value)
                } else {
                    Err(stream::Error.into())
                }
            }
        }
        fn write_length_delimited(&mut self, value: &[u8]) -> Result {
            let len = value.len();
            let delimiter = i32::try_from(len).map_err(|_| Error::ValueTooLarge)? as u32;
            self.write_varint32(delimiter)?;
            if self.can_write(len) {
                unsafe { write_bytes_unchecked(value, self.current); }
                Ok(())
            } else {
                if let Some(mut buffer) = self.as_borrowed_stream() {
                    buffer.write_bytes(value)
                } else {
                    Err(stream::Error.into())
                }
            }
        }
        fn as_any<'a>(&'a mut self) -> Any<'a> {
            Any {
                stream: self.stream.as_mut().map::<&'a mut dyn Write, _>(|w| *w),
                start: self.start,
                current: &mut self.current,
                end: self.end
            }
        }
    }
}

use internal::Writer;
pub use internal::Any;

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

/// A trait representing types that can be used as outputs in CodedOutput
pub trait Output: Writer { }
impl<T: Writer> Output for T { }

#[inline]
unsafe fn write_varint32_unchecked(mut value: u32, ptr: &mut *mut u8) {
    for _ in 0..5 {
        **ptr = value as u8 & 0x7f;

        if value == 0 {
            *ptr = ptr.add(1);
            break;
        } else {
            **ptr |= 0x80;
            *ptr = ptr.add(1);
        }

        value >>= 7;
    }
}

#[inline]
unsafe fn write_varint64_unchecked(mut value: u64, ptr: &mut *mut u8) {
    for _ in 0..10 {
        **ptr = value as u8 & 0x7f;

        if value == 0 {
            *ptr = ptr.add(1);
            break;
        } else {
            **ptr |= 0x80;
            *ptr = ptr.add(1);
        }

        value >>= 7;
    }
}

#[inline]
unsafe fn write_bytes_unchecked(slice: &[u8], ptr: &mut *mut u8) {
    match slice.len() {
        0 => { },
        1 => {
            **ptr = *slice.get_unchecked(0);
            *ptr = ptr.add(1);
        },
        len => {
            ptr::copy_nonoverlapping(slice.as_ptr(), *ptr, len);
            *ptr = ptr.add(len);
        }
    }
}

/// A slice output. This removes all safety checks and writes directly to the slice without performing any length checks.
pub struct SliceUnchecked<'a> {
    a: PhantomData<&'a mut [u8]>,
    ptr: *mut u8,
}
impl<'a> SliceUnchecked<'a> {
    fn new(s: &'a mut [u8]) -> Self {
        Self { a: PhantomData, ptr: s.as_mut_ptr() }
    }
}
impl Writer for SliceUnchecked<'_> {
    fn write_varint32(&mut self, value: u32) -> Result {
        unsafe { write_varint32_unchecked(value, &mut self.ptr); }
        Ok(())
    }
    fn write_varint64(&mut self, value: u64) -> Result {
        unsafe { write_varint64_unchecked(value, &mut self.ptr); }
        Ok(())
    }
    fn write_bit32(&mut self, value: u32) -> Result {
        let value = u32::to_le_bytes(value);
        unsafe { write_bytes_unchecked(&value, &mut self.ptr); }
        Ok(())
    }
    fn write_bit64(&mut self, value: u64) -> Result {
        let value = u64::to_le_bytes(value);
        unsafe { write_bytes_unchecked(&value, &mut self.ptr); }
        Ok(())
    }
    fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        let len = value.len() as i32;
        unsafe {
            write_varint32_unchecked(len as u32, &mut self.ptr);
            write_bytes_unchecked(&value, &mut self.ptr);
        }
        Ok(())
    }

    fn as_any(&mut self) -> Any {
        Any {
            stream: None,
            start: None,
            current: &mut self.ptr,
            end: None
        }
    }
}

/// A slice output. This elides many checks associated with a standard stream output.
pub struct Slice<'a> {
    a: PhantomData<&'a mut [u8]>,
    start: *mut u8,
    end: *mut u8,
}
impl<'a> Slice<'a> {
    fn new(s: &'a mut [u8]) -> Self {
        let Range { start, end } = s.as_mut_ptr_range();
        Self {
            a: PhantomData,
            start,
            end
        }
    }
    fn len(&self) -> usize {
        usize::wrapping_sub(self.start as _, self.end as _)
    }
}
impl Writer for Slice<'_> {
    fn write_varint32(&mut self, value: u32) -> Result {
        let size = raw_varint32_size(value).get() as usize;
        if self.len() >= size {
            // This is safe because the size of the varint has been calculated ahead of time
            unsafe {
                write_varint32_unchecked(value, &mut self.start);
            }
            debug_assert!(self.start <= self.end);
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }
    fn write_varint64(&mut self, value: u64) -> Result {
        let size = raw_varint64_size(value).get() as usize;
        if self.len() >= size {
            unsafe { 
                write_varint64_unchecked(value, &mut self.start);
            }
            debug_assert!(self.start <= self.end);
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }
    fn write_bit32(&mut self, value: u32) -> Result {
        const LEN: usize = 4;
        if self.len() >= LEN {
            let value = value.to_le_bytes();
            unsafe {
                write_bytes_unchecked(&value, &mut self.start);
            }
            debug_assert!(self.start <= self.end);
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }
    fn write_bit64(&mut self, value: u64) -> Result {
        const LEN: usize = 8;
        if self.len() >= LEN {
            let value = value.to_le_bytes();
            unsafe {
                write_bytes_unchecked(&value, &mut self.start);
            }
            debug_assert!(self.start <= self.end);
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }
    fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        let len = i32::try_from(value.len()).map_err(|_| Error::ValueTooLarge)? as u32;
        let len_len = raw_varint32_size(len as u32).get() as usize;
        let total_len = (len as usize) + (len_len);
        if self.len() >= total_len {
            unsafe {
                write_varint32_unchecked(len, &mut self.start);
                write_bytes_unchecked(&value, &mut self.start);
            }
            debug_assert!(self.start <= self.end);
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }

    fn as_any(&mut self) -> Any {
        Any {
            stream: None,
            start: None,
            current: &mut self.start,
            end: Some(unsafe { NonNull::new_unchecked(self.end) }),
        }
    }
}

/// A buffered stream output
pub struct Stream<T> {
    output: T,
    start: NonNull<u8>,
    current: *mut u8,
    end: NonNull<u8>,
}
impl<T: Write> Stream<T> {
    fn with_capacity(cap: usize, output: T) -> Self {
        let Range { start, end } = Box::leak(alloc::vec![0; cap].into_boxed_slice()).as_mut_ptr_range();
        Self {
            output,
            start: unsafe { NonNull::new_unchecked(start) },
            current: start,
            end: unsafe { NonNull::new_unchecked(end) },
        }
    }
    #[inline]
    fn remaining(&self) -> usize {
        usize::wrapping_sub(self.current as _, self.end.as_ptr() as _)
    }
    #[inline]
    fn capacity(&self) -> usize {
        usize::wrapping_sub(self.start.as_ptr() as _, self.end.as_ptr() as _)
    }
    #[inline]
    fn buffered(&self) -> usize {
        usize::wrapping_sub(self.start.as_ptr() as _, self.current as _)
    }
    #[inline]
    fn clear(&mut self) {
        self.current = self.start.as_ptr();
    }
    fn flush(&mut self) -> Result {
        let slice = unsafe { slice::from_raw_parts(self.start.as_ptr() as _, self.buffered()) };
        self.output.write(slice)?;
        self.clear();
        Ok(())
    }
    fn into_inner(self) -> T {
        self.output
    }
}
impl<T: Write> Writer for Stream<T> {
    fn write_varint32(&mut self, value: u32) -> Result {
        let len = raw_varint32_size(value).get() as usize;
        if self.remaining() < len {
            self.flush()?;
        }
        if len >= self.capacity() {
            let mut buf = [0; 5];
            unsafe { write_varint32_unchecked(value, &mut buf.as_mut_ptr()); }
            self.output.write(&buf[..len])?;
        } else {
            unsafe { write_varint32_unchecked(value, &mut self.current); }
        }
        Ok(())
    }
    fn write_varint64(&mut self, value: u64) -> Result {
        let len = raw_varint64_size(value).get() as usize;
        if self.remaining() < len {
            self.flush()?;
        }
        if len >= self.capacity() {
            let mut buf = [0; 10];
            unsafe { write_varint64_unchecked(value, &mut buf.as_mut_ptr()); }
            self.output.write(&buf[..len])?;
        } else {
            unsafe { write_varint64_unchecked(value, &mut self.current); }
        }
        Ok(())
    }
    fn write_bit32(&mut self, value: u32) -> Result {
        let value = u32::to_le_bytes(value);
        if self.remaining() < value.len() {
            self.flush()?;
        }
        if value.len() >= self.capacity() {
            self.output.write(&value)?;
        } else {
            unsafe { write_bytes_unchecked(&value, &mut self.current); }
        }
        Ok(())
    }
    fn write_bit64(&mut self, value: u64) -> Result {
        let value = u64::to_le_bytes(value);
        if self.remaining() < value.len() {
            self.flush()?;
        }
        if value.len() >= self.capacity() {
            self.output.write(&value)?;
        } else {
            unsafe { write_bytes_unchecked(&value, &mut self.current); }
        }
        Ok(())
    }
    fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        let len = value.len();
        let delimiter = i32::try_from(len).map_err(|_| Error::ValueTooLarge)? as u32;
        self.write_varint32(delimiter)?;
        if self.remaining() < len {
            self.flush()?;
        }
        if len >= self.capacity() {
            self.output.write(value)?;
        } else {
            unsafe { write_bytes_unchecked(value, &mut self.current); }
        }
        Ok(())
    }

    fn as_any(&mut self) -> Any {
        Any {
            stream: Some(&mut self.output),
            start: Some(self.start),
            current: &mut self.current,
            end: Some(self.end),
        }
    }
}

/// A protobuf coded output writer that writes to the specified output
pub struct CodedWriter<T: Output> {
    inner: T,
}

impl<'a> CodedWriter<Slice<'a>> {
    /// Creates a coded writer that writes to the specified slice
    pub fn with_slice(s: &'a mut [u8]) -> Self {
        Self { inner: Slice::new(s), }
    }
}

impl<'a> CodedWriter<SliceUnchecked<'a>> {
    /// Creates a coded writer that writes to the specified slice without performing any length checks
    pub unsafe fn with_slice_unchecked(s: &'a mut [u8]) -> Self {
        Self { inner: SliceUnchecked::new(s) }
    }
}

impl<T: Write> CodedWriter<Stream<T>> {
    /// Creates a coded writer that writes to the specified stream with the default buffer capacity
    pub fn with_stream(inner: T) -> Self {
        Self::with_capacity(DEFAULT_BUF_SIZE, inner)
    }
    /// Creates a coded writer that writes to the specified stream with the specified buffer capacity
    pub fn with_capacity(cap: usize, inner: T) -> Self {
        Self { inner: Stream::with_capacity(cap, inner) }
    }

    /// Flushes the stream buffer
    pub fn flush(&mut self) -> Result {
        self.inner.flush()
    }
    /// Returns ownership of the inner stream, discarding any data in the buffer
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T: Output> CodedWriter<T> {
    /// Converts the generic writer into a writer over Any input
    pub fn as_any(&mut self) -> CodedWriter<Any> {
        CodedWriter {
            inner: self.inner.as_any()
        }
    }

    /// Writes a 32-bit varint value to the output
    #[inline]
    pub fn write_varint32(&mut self, value: u32) -> Result {
        self.inner.write_varint32(value)
    }
    /// Writes a 64-bit varint value to the output
    #[inline]
    pub fn write_varint64(&mut self, value: u64) -> Result {
        self.inner.write_varint64(value)
    }
    /// Writes a little-endian 4-byte integer to the output
    #[inline]
    pub fn write_bit32(&mut self, value: u32) -> Result {
        self.inner.write_bit32(value)
    }
    /// Writes an little-endian 8-byte integer to the output
    #[inline]
    pub fn write_bit64(&mut self, value: u64) -> Result {
        self.inner.write_bit64(value)
    }
    /// Writes a length delimited string of bytes to the output
    #[inline]
    pub fn write_length_delimited(&mut self, value: &[u8]) -> Result {
        self.inner.write_length_delimited(value)
    }

    /// Writes a length to the output
    #[inline]
    pub fn write_length(&mut self, length: Length) -> Result {
        self.write_varint32(length.get() as u32)
    }
    /// Writes a tag to the output
    #[inline]
    pub fn write_tag(&mut self, tag: Tag) -> Result {
        self.write_varint32(tag.get())
    }

    /// Writes the value to the output. This uses an alias to `Value::write_to`.
    #[inline]
    pub fn write_value<V: Value>(&mut self, value: &V::Inner) -> Result {
        V::write_to(value, self)
    }
    /// Writes the value to the output using the field number and the wire type of the value.
    #[inline]
    pub fn write_field<V: Value>(&mut self, num: FieldNumber, value: &V::Inner) -> Result {
        self.write_tag(Tag::new(num, V::WIRE_TYPE))?;
        self.write_value::<V>(value)?;
        if V::WIRE_TYPE != WireType::StartGroup {
            self.write_tag(Tag::new(num, WireType::EndGroup))?;
        }
        Ok(())
    }
    /// Writes the values in the repeated field to the output. This uses an alias to `RepeatedValue::write_to`.
    #[inline]
    pub fn write_values<U: RepeatedValue<V>, V>(&mut self, value: &U, num: FieldNumber) -> Result {
        value.write_to(self, num)
    }
    /// Writes the fields in the set to the output. This uses an alias to `FieldSet::write_to`.
    #[inline]
    pub fn write_fields<U: FieldSet>(&mut self, value: &U) -> Result {
        value.write_to(self)
    }
}

#[cfg(test)]
mod test {
    
}