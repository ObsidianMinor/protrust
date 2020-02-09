//! Defines the `CodedWriter`, a writer for writing protobuf encoded values to streams.

use crate::collections::{RepeatedValue, FieldSet};
use crate::io::{FieldNumber, WireType, Tag, Length, DEFAULT_BUF_SIZE};
use crate::raw::Value;
use std::convert::TryFrom;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use std::io::{self, Write, ErrorKind};
use std::mem::ManuallyDrop;
use std::ops::Range;
use std::ptr::{self, NonNull};
use std::slice;
use super::{raw_varint32_size, raw_varint64_size};

mod internal {
    use crate::internal::Sealed;
    use crate::io::{raw_varint32_size, raw_varint64_size};
    use std::convert::TryFrom;
    use std::io::{self, Write, ErrorKind};
    use std::ptr::{self, NonNull};
    use std::slice;
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
            usize::wrapping_sub(self.end as _, *self.current as _)
        }
        #[inline]
        fn capacity(&self) -> usize {
            usize::wrapping_sub(self.end as _, self.start as _)
        }
        #[inline]
        fn buffered(&self) -> usize {
            usize::wrapping_sub(*self.current as _, self.start as _)
        }
        #[inline]
        fn clear(&mut self) {
            *self.current = self.start;
        }
        fn flush(&mut self) -> Result {
            let slice = unsafe { slice::from_raw_parts(self.start, self.buffered()) };
            self.output.write_all(slice)?;
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
                self.output.write_all(&buf[..len])?;
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
                self.output.write_all(&buf[..len])?;
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
                self.output.write_all(&value)?;
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
                self.output.write_all(&value)?;
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
                self.output.write_all(value)?;
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
                Some(end) => usize::wrapping_sub(end.as_ptr() as _, *self.current as _) >= len,
                None => true
            }
        }
        fn as_borrowed_stream(&mut self) -> Option<BorrowedStream> {
            match &mut self.stream {
                Some(w) => {
                    Some(BorrowedStream {
                        output: *w,
                        start: self.start.map(NonNull::as_ptr).unwrap_or(ptr::null_mut()), // unchecked unwrap
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
            } else if let Some(mut buffer) = self.as_borrowed_stream() {
                buffer.write_varint32(value, len)
            } else {
                Err(io::Error::from(ErrorKind::WriteZero).into())
            }
        }
        fn write_varint64(&mut self, value: u64) -> Result {
            let len = raw_varint64_size(value).get() as usize;
            if self.can_write(len) {
                unsafe { write_varint64_unchecked(value, self.current); }
                Ok(())
            } else if let Some(mut buffer) = self.as_borrowed_stream() {
                buffer.write_varint64(value, len)
            } else {
                Err(io::Error::from(ErrorKind::WriteZero).into())
            }
        }
        fn write_bit32(&mut self, value: u32) -> Result {
            if self.can_write(4) {
                let value = u32::to_le_bytes(value);
                unsafe { write_bytes_unchecked(&value, self.current); }
                Ok(())
            } else if let Some(mut buffer) = self.as_borrowed_stream() {
                buffer.write_bit32(value)
            } else {
                Err(io::Error::from(ErrorKind::WriteZero).into())
            }
        }
        fn write_bit64(&mut self, value: u64) -> Result {
            if self.can_write(8) {
                let value = u64::to_le_bytes(value);
                unsafe { write_bytes_unchecked(&value, self.current); }
                Ok(())
            } else if let Some(mut buffer) = self.as_borrowed_stream() {
                buffer.write_bit64(value)
            } else {
                Err(io::Error::from(ErrorKind::WriteZero).into())
            }
        }
        fn write_length_delimited(&mut self, value: &[u8]) -> Result {
            let len = value.len();
            let delimiter = i32::try_from(len).map_err(|_| Error::ValueTooLarge)? as u32;
            self.write_varint32(delimiter)?;
            if self.can_write(len) {
                unsafe { write_bytes_unchecked(value, self.current); }
                Ok(())
            } else if let Some(mut buffer) = self.as_borrowed_stream() {
                buffer.write_bytes(value)
            } else {
                Err(io::Error::from(ErrorKind::WriteZero).into())
            }
        }
        #[allow(clippy::map_clone)]
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
    IoError(io::Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::ValueTooLarge => write!(f, "the value was too large to write to the output"),
            Error::IoError(_) => write!(f, "an error occured while writing to the output")
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            _ => None
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

/// A result for a [`CodedWriter`](struct.CodedWriter.html) read operation
pub type Result = std::result::Result<(), Error>;

/// A trait representing types that can be used as outputs in CodedOutput
pub trait Output: Writer { }
impl<T: Writer> Output for T { }

#[inline]
unsafe fn write_varint32_unchecked(mut value: u32, ptr: &mut *mut u8) {
    for _ in 0..5 {
        **ptr = value as u8 & 0x7f;
        value >>= 7;

        if value == 0 {
            *ptr = ptr.add(1);
            break;
        } else {
            **ptr |= 0x80;
            *ptr = ptr.add(1);
        }
    }
}

#[inline]
unsafe fn write_varint64_unchecked(mut value: u64, ptr: &mut *mut u8) {
    for _ in 0..10 {
        **ptr = value as u8 & 0x7f;
        value >>= 7;

        if value == 0 {
            *ptr = ptr.add(1);
            break;
        } else {
            **ptr |= 0x80;
            *ptr = ptr.add(1);
        }
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
    end: *mut u8,
}
impl<'a> SliceUnchecked<'a> {
    fn new(s: &'a mut [u8]) -> Self {
        let Range { start, end } = s.as_mut_ptr_range();
        Self { a: PhantomData, ptr: start, end }
    }
    fn into_inner(self) -> &'a mut [u8] {
        let len = usize::wrapping_sub(self.end as _, self.ptr as _);
        unsafe { slice::from_raw_parts_mut(self.ptr, len) }
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
        usize::wrapping_sub(self.end as _, self.start as _)
    }
    fn into_inner(self) -> &'a mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.start, self.len()) }
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
            Err(io::Error::from(ErrorKind::WriteZero).into())
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
            Err(io::Error::from(ErrorKind::WriteZero).into())
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
            Err(io::Error::from(ErrorKind::WriteZero).into())
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
            Err(io::Error::from(ErrorKind::WriteZero).into())
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
            Err(io::Error::from(ErrorKind::WriteZero).into())
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

#[derive(PartialEq, Eq)]
enum DropFlag {
    Moved,
    Owned,
}

/// A buffered stream output
pub struct Stream<T: Write> {
    output: ManuallyDrop<T>,
    start: NonNull<u8>,
    current: *mut u8,
    end: NonNull<u8>,
}
impl<T: Write> Stream<T> {
    fn with_capacity(cap: usize, output: T) -> Self {
        let Range { start, end } = Box::leak(vec![0; cap].into_boxed_slice()).as_mut_ptr_range();
        Self {
            output: ManuallyDrop::new(output),
            start: unsafe { NonNull::new_unchecked(start) },
            current: start,
            end: unsafe { NonNull::new_unchecked(end) },
        }
    }
    #[inline]
    fn remaining(&self) -> usize {
        usize::wrapping_sub(self.end.as_ptr() as _, self.current as _)
    }
    #[inline]
    fn capacity(&self) -> usize {
        usize::wrapping_sub(self.end.as_ptr() as _, self.start.as_ptr() as _)
    }
    #[inline]
    fn buffered(&self) -> usize {
        usize::wrapping_sub(self.current as _, self.start.as_ptr() as _)
    }
    #[inline]
    fn clear(&mut self) {
        self.current = self.start.as_ptr();
    }
    fn flush(&mut self) -> Result {
        let slice = unsafe { slice::from_raw_parts(self.start.as_ptr() as _, self.buffered()) };
        self.output.write_all(slice)?;
        self.clear();
        Ok(())
    }
    fn into_inner(mut self) -> T {
        let output = unsafe { ManuallyDrop::take(&mut self.output) };
        unsafe { self.drop_inner(DropFlag::Moved) };
        std::mem::forget(self);
        output
    }
    #[inline]
    unsafe fn drop_inner(&mut self, flag: DropFlag) {
        let raw_slice = slice::from_raw_parts_mut(self.start.as_ptr(), self.capacity());
        drop(Box::from_raw(raw_slice));
        if flag == DropFlag::Owned {
            ManuallyDrop::drop(&mut self.output);
        }
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
            self.output.write_all(&buf[..len])?;
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
            self.output.write_all(&buf[..len])?;
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
            self.output.write_all(&value)?;
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
            self.output.write_all(&value)?;
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
            self.output.write_all(value)?;
        } else {
            unsafe { write_bytes_unchecked(value, &mut self.current); }
        }
        Ok(())
    }

    fn as_any(&mut self) -> Any {
        Any {
            stream: Some(&mut *self.output),
            start: Some(self.start),
            current: &mut self.current,
            end: Some(self.end),
        }
    }
}
impl<T: Write> Drop for Stream<T> {
    fn drop(&mut self) {
        unsafe { self.drop_inner(DropFlag::Owned) }
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
    /// Returns ownership of the buffer at the current point in the slice
    pub fn into_inner(self) -> &'a mut [u8] {
        self.inner.into_inner()
    }
}

impl<'a> CodedWriter<SliceUnchecked<'a>> {
    /// Creates a coded writer that writes to the specified slice without performing any length checks
    /// 
    /// # Safety
    /// 
    /// Caution must be used when using the resulting writer as any writes outside of the slice are
    /// undefined behavior.
    pub unsafe fn with_slice_unchecked(s: &'a mut [u8]) -> Self {
        Self { inner: SliceUnchecked::new(s) }
    }
    /// Returns ownership of the buffer at the current point in the slice. This result of this is
    /// undefined if the writer has written past the end of the slice.
    pub fn into_inner(self) -> &'a mut [u8] {
        self.inner.into_inner()
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
    use crate::io::write::{self, Any, Output, CodedWriter};

    pub trait WriterOutput<'a> {
        type Writer: Output + 'a;

        fn new(b: &'a mut [u8]) -> CodedWriter<Self::Writer>;
        fn into_inner(w: CodedWriter<Self::Writer>) -> Result<&'a mut [u8], write::Error>;

        fn run<F: FnOnce(&mut CodedWriter<Self::Writer>) -> write::Result>(b: &'a mut [u8], f: F) -> Result<&'a mut [u8], write::Error> {
            let mut writer = Self::new(b);
            f(&mut writer)?;
            Self::into_inner(writer)
        }

        fn run_any<F: FnOnce(&mut CodedWriter<Any>) -> write::Result>(b: &'a mut [u8], f: F) -> Result<&'a mut [u8], write::Error> {
            let mut writer = Self::new(b);
            let mut any = writer.as_any();
            f(&mut any)?;
            Self::into_inner(writer)
        }
    }

    macro_rules! test {
        ($(($ti:ident | $tia:ident | size: $s:expr) = |$f:ident| $t:block => $p:pat $(if $pe:expr)?),+) => {
            $(
                pub fn $ti<T>() where for<'a> T: WriterOutput<'a> {
                    let mut output = vec![0; $s].into_boxed_slice();

                    let result = 
                        match T::run(&mut output, |$f| $t) {
                            Ok(r) => {
                                let remaining_len = r.len();
                                Ok(output.split_at(output.len() - remaining_len))
                            },
                            Err(e) => Err(e),
                        };

                    assert!(matches!(result, $p $(if $pe)?), "expected {}, got {:?}", stringify!($p $(if $pe)?), result);
                }

                pub fn $tia<T>() where for<'a> T: WriterOutput<'a> {
                    let mut output = vec![0; $s].into_boxed_slice();

                    let result = 
                        match T::run_any(&mut output, |$f| $t) {
                            Ok(r) => {
                                let remaining_len = r.len();
                                Ok(output.split_at(output.len() - remaining_len))
                            },
                            Err(e) => Err(e),
                        };

                    assert!(matches!(result, $p $(if $pe)?), "expected {}, got {:?}", stringify!($p $(if $pe)?), result);
                }
            )+
        };
    }

    test! {
        (write_varint32_zero | write_varint32_zero_any | size: 1) = |w| {
            w.write_varint32(0)
        } => Ok(([0], [])),

        (write_varint32_2byte | write_varint32_2byte_any | size: 2) = |w| {
            w.write_varint32(128)
        } => Ok(([128, 1], [])),

        (write_varint32_5byte | write_varint32_5byte_any | size: 5) = |w| {
            w.write_varint32(268435456)
        } => Ok(([128, 128, 128, 128, 1], [])),

        (write_varint64_zero | write_varint64_zero_any | size: 1) = |w| {
            w.write_varint64(0)
        } => Ok(([0], [])),

        (write_varint64_2byte | write_varint64_2byte_any | size: 2) = |w| {
            w.write_varint64(128)
        } => Ok(([128, 1], [])),

        (write_varint64_5byte | write_varint64_5byte_any | size: 5) = |w| {
            w.write_varint64(268435456)
        } => Ok(([128, 128, 128, 128, 1], [])),
        
        (write_varint64_10byte | write_varint64_10byte_any | size: 10) = |w| {
            w.write_varint64(0x8000000000000000)
        } => Ok(([128, 128, 128, 128, 128, 128, 128, 128, 128, 1], [])),

        (write_bit32 | write_bit32_any | size: 4) = |w| {
            w.write_bit32(0)
        } => Ok(([0, 0, 0, 0], [])),

        (write_bit64 | write_bit64_any | size: 8) = |w| {
            w.write_bit64(0)
        } => Ok(([0, 0, 0, 0, 0, 0, 0, 0], [])),

        (write_length_delimited | write_length_delimited_any | size: 4) = |w| {
            w.write_length_delimited(&[1, 2, 3])
        } => Ok(([3, 1, 2, 3], [])),

        (write_as_any | write_as_any_any | size: 6) = |w| {
            w.write_varint32(8)?;

            let mut any = w.as_any();
            any.write_length_delimited(&[1, 2, 3])?;

            w.write_varint32(1)
        } => Ok(([8, 3, 1, 2, 3, 1], []))
    }

    macro_rules! run {
        (
            $f:ty => {
                $($t:ident),*
            }
        ) => {
            $(
                #[test]
                fn $t() {
                    crate::io::write::test::$t::<$f>();
                }
            )*
        };
    }

    macro_rules! run_suite {
        ($f:ty) => {
            run! {
                $f => {
                    write_varint32_zero, write_varint32_zero_any,
                    write_varint32_2byte, write_varint32_2byte_any,
                    write_varint32_5byte, write_varint32_5byte_any,
                    write_varint64_zero, write_varint64_zero_any,
                    write_varint64_2byte, write_varint64_2byte_any,
                    write_varint64_5byte, write_varint64_5byte_any,
                    write_varint64_10byte, write_varint64_10byte_any,
                    write_bit32, write_bit32_any,
                    write_bit64, write_bit64_any,
                    write_length_delimited, write_length_delimited_any,
                    write_as_any, write_as_any_any
                }
            }
        };
    }

    mod suites {
        mod slice {
            use crate::io::write::{self, Slice, CodedWriter, test::WriterOutput};

            pub struct SliceOutput;
            impl<'a> WriterOutput<'a> for SliceOutput {
                type Writer = Slice<'a>;

                fn new(b: &'a mut [u8]) -> CodedWriter<Self::Writer> {
                    CodedWriter::with_slice(b)
                }
                fn into_inner(w: CodedWriter<Self::Writer>) -> Result<&'a mut [u8], write::Error> {
                    Ok(w.into_inner())
                }
            }

            run_suite!(SliceOutput);
        }
        mod slice_unchecked {
            use crate::io::write::{self, SliceUnchecked, CodedWriter, test::WriterOutput};

            pub struct SliceUncheckedOutput;
            impl<'a> WriterOutput<'a> for SliceUncheckedOutput {
                type Writer = SliceUnchecked<'a>;

                fn new(b: &'a mut [u8]) -> CodedWriter<Self::Writer> {
                    unsafe { CodedWriter::with_slice_unchecked(b) }
                }
                fn into_inner(w: CodedWriter<Self::Writer>) -> Result<&'a mut [u8], write::Error> {
                    Ok(w.into_inner())
                }
            }

            run_suite!(SliceUncheckedOutput);
        }
        mod stream {
            macro_rules! stream_case {
                ($i:ident($s:expr)) => {
                    use crate::io::write::{self, CodedWriter, Stream, test::WriterOutput};

                    pub struct $i;

                    impl<'a> WriterOutput<'a> for $i {
                        type Writer = Stream<&'a mut [u8]>;

                        fn new(b: &'a mut [u8]) -> CodedWriter<Self::Writer> {
                            CodedWriter::with_capacity($s, b)
                        }
                        fn into_inner(mut w: CodedWriter<Self::Writer>) -> Result<&'a mut [u8], write::Error> {
                            w.flush()?;
                            Ok(w.into_inner())
                        }
                    }
                };
            }

            mod default {
                stream_case!(StreamDefaultBuffer(crate::io::DEFAULT_BUF_SIZE));
                run_suite!(StreamDefaultBuffer);
            }

            mod no_buffer {
                stream_case!(StreamNoBuffer(0));
                run_suite!(StreamNoBuffer);
            }

            mod byte1_buffer {
                stream_case!(StreamTinyBuffer(1));
                run_suite!(StreamTinyBuffer);
            }

            mod byte5_buffer {
                stream_case!(StreamTinyBuffer(5));
                run_suite!(StreamTinyBuffer);
            }

            mod byte10_buffer {
                stream_case!(StreamTinyBuffer(10));
                run_suite!(StreamTinyBuffer);
            }
        }
    }
}