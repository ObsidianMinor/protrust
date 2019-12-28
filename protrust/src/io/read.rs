//! Defines the `CodedReader`, a reader for reading values from a protobuf encoded byte stream.

use alloc::boxed::Box;
use alloc::string::FromUtf8Error;
use core::cmp;
use core::convert::{TryFrom, TryInto};
use core::fmt::{self, Display, Formatter};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZeroUsize;
use core::ops;
use core::ptr::NonNull;
use core::result;
use core::slice;
use crate::collections::{RepeatedValue, FieldSet, TryRead};
use crate::extend::ExtensionRegistry;
use crate::io::{Tag, WireType, ByteString, stream::{self, Read}};
use crate::raw::{self, Value};
use trapper::Wrapper;

#[cfg(feature = "std")]
use std::error;

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

mod internal {
    use core::result;
    use crate::io::{ByteString, stream, read::{Result, Any}};

    pub trait Reader {
        fn push_limit(&mut self, limit: i32) -> result::Result<Option<i32>, stream::Error>;
        fn pop_limit(&mut self, old: Option<i32>);
        fn reached_limit(&self) -> bool;

        fn read_tag(&mut self) -> Result<Option<u32>>;
        fn read_varint32(&mut self) -> Result<u32>;
        fn read_varint64(&mut self) -> Result<u64>;
        fn read_bit32(&mut self) -> Result<u32>;
        fn read_bit64(&mut self) -> Result<u64>;
        fn read_length_delimited<B: ByteString>(&mut self) -> Result<B>;

        fn skip_varint(&mut self) -> Result<()>;
        fn skip_bit32(&mut self) -> Result<()>;
        fn skip_bit64(&mut self) -> Result<()>;
        fn skip_length_delimited(&mut self) -> Result<()>;

        fn into_any<'a>(&'a mut self) -> Any<'a>;
        fn from_any<'a>(&'a mut self, any: Any<'a>);

        fn reached_end(&self) -> bool;
    }
}

use internal::Reader;
use super::internal::Array;

/// The error type for [`CodedReader`](struct.CodedReader.html)
#[derive(Debug)]
pub enum Error {
    /// The input contained a malformed variable length integer
    MalformedVarint,
    /// The input contained a length delimited value which reported it had a negative size
    NegativeSize,
    /// The input contained an invalid tag (zero or the tag had an invalid wire format)
    InvalidTag(u32),
    /// An error occured while reading from the underlying `Read` object
    StreamError(stream::Error),
    /// The input contained an invalid UTF8 string
    InvalidString(FromUtf8Error),
}

impl From<stream::Error> for Error {
    fn from(value: stream::Error) -> Error {
        Error::StreamError(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Error {
        Error::InvalidString(value)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Error::MalformedVarint => write!(fmt, "the input contained an invalid variable length integer"),
            Error::NegativeSize => write!(fmt, "the input contained a length delimited value which reported it had a negative size"),
            Error::InvalidTag(val) => write!(fmt, "the input contained an tag that was either invalid or was unexpected at this point in the input: {}", val),
            Error::StreamError(_) => write!(fmt, "an error occured in the underlying input"),
            Error::InvalidString(_) => write!(fmt, "the input contained an invalid UTF8 string")
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::StreamError(ref e) => Some(e),
            Error::InvalidString(ref e) => Some(e),
            _ => None,
        }
    }
}

/// A result for a [`CodedReader`](struct.CodedReader.html) read operation
pub type Result<T> = result::Result<T, Error>;

/// An input type that can be used to create a `Reader` for a [`CodedReader`] instance.
/// 
/// [`CodedReader`]: struct.CodedReader.html
pub trait Input: internal::Reader { }

/// A type used for a [`CodedReader`] reading from a `slice` input.
/// 
/// [`CodedReader`]: struct.CodedReader.html
pub struct Slice<'a> {
    a: PhantomData<&'a [u8]>,
    start: *const u8,
    limit: *const u8,
    end: *const u8,
}

impl<'a> Slice<'a> {
    fn new(value: &'a [u8]) -> Self {
        let start = value.as_ptr();
        let end = unsafe { start.add(value.len()) };
        Self {
            a: PhantomData,
            start,
            end,
            limit: end
        }
    }
    fn len_limited(&self) -> usize {
        usize::wrapping_sub(self.limit as _, self.start as _)
    }
    unsafe fn next_byte_unchecked(&self) -> u8 {
        *self.start
    }
    fn try_limited_as_array<A: Array>(&self) -> Option<&'a A> {
        if self.len_limited() >= A::LENGTH {
            unsafe { Some(&*(self.start as *const A)) }
        } else {
            None
        }
    }
    unsafe fn advance(&mut self, amnt: usize) {
        let new_pos = self.start.add(amnt);

        debug_assert!(new_pos <= self.limit, "advanced past end of limit: {:p} < {:p}", new_pos, self.limit);
        debug_assert!(new_pos <= self.end, "advanced past end of buffer: {:p} < {:p}", new_pos, self.end);

        self.start = new_pos;
    }
    fn as_limited_slice(&self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.start, self.len_limited()) }
    }
    fn read_varint64<A: Array>(&mut self, arr: &A) -> u64 {
        let mut result = 0u64;
        for (i, &b) in arr.as_ref().iter().enumerate() {
            result |= ((b & 0x7F) as u64) << (7 * i);
        }
        unsafe { self.advance(A::LENGTH); }
        result
    }
}

impl Reader for Slice<'_> {
    fn push_limit(&mut self, limit: i32) -> result::Result<Option<i32>, stream::Error> {
        if self.limit == self.end {
            Ok(None)
        } else {
            let len = self.len_limited() as i32;
            if limit <= len {
                Ok(Some(i32::wrapping_sub(len, limit)))
            } else {
                Err(stream::Error)
            }
        }
    }
    fn pop_limit(&mut self, old: Option<i32>) {
        match old {
            Some(old) => {
                // if a user forgets a limit, this shouldn't cause undefined behavior as it's still in range,
                // but it won't do what the user wants
                self.limit = unsafe { self.start.add(old as usize) };
                debug_assert!(self.limit <= self.end, "advanced past end of buffer when popping limit");
            },
            None => self.limit = self.end,
        }
    }
    fn reached_limit(&self) -> bool {
        self.start >= self.limit
    }

    fn read_tag(&mut self) -> Result<Option<u32>> {
        if !self.reached_limit() {
            let next = unsafe { self.next_byte_unchecked() }; // we haven't reached the end so we're fine
            if next < 0x80 {
                unsafe { self.advance(1); }
                Ok(Some(next as u32))
            } else {
                self.read_varint32().map(Some)
            }
        } else {
            Ok(None)
        }
    }
    fn read_varint32(&mut self) -> Result<u32> {
        let mut result = 0u32;
        if let Some::<&[u8; 10]>(arr) = self.try_limited_as_array() {
            for (i, &b) in arr[0..5].iter().enumerate() {
                result |= ((b & 0x7f) as u32) << (7 * i);
                if b < 0x80 {
                    unsafe { self.advance(i + 1); }
                    return Ok(result);
                }
            }
            for (i, &b) in arr[5..10].iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.advance(i + 1 + 5); } // add 5 for the first bytes we've read
                    return Ok(result);
                }
            }
            Err(Error::MalformedVarint)
        } else
        if let Some::<&[u8; 5]>(arr) = self.try_limited_as_array() {
            for (i, &b) in arr.iter().enumerate() {
                result |= ((b & 0x7f) as u32) << (7 * i);
                if b < 0x80 {
                    unsafe { self.advance(i + 1); }
                    return Ok(result);
                }
            }
            unsafe { self.advance(5); }
            for (i, &b) in self.as_limited_slice().iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(stream::Error.into())
        } else {
            for (i, &b) in self.as_limited_slice().iter().enumerate() {
                result |= ((b & 0x7f) as u32) << (7 * i);
                if b < 0x80 {
                    unsafe { self.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(stream::Error.into())
        }
    }
    fn read_varint64(&mut self) -> Result<u64> {
        if let Some::<&[u8; 10]>(arr) = self.try_limited_as_array() {
            match () {
                _ if arr[0] < 0x80 => Ok(self.read_varint64::<[u8; 1]> (&arr[0..1 ].try_into().unwrap())),
                _ if arr[1] < 0x80 => Ok(self.read_varint64::<[u8; 2]> (&arr[0..2 ].try_into().unwrap())),
                _ if arr[2] < 0x80 => Ok(self.read_varint64::<[u8; 3]> (&arr[0..3 ].try_into().unwrap())),
                _ if arr[3] < 0x80 => Ok(self.read_varint64::<[u8; 4]> (&arr[0..4 ].try_into().unwrap())),
                _ if arr[4] < 0x80 => Ok(self.read_varint64::<[u8; 5]> (&arr[0..5 ].try_into().unwrap())),
                _ if arr[5] < 0x80 => Ok(self.read_varint64::<[u8; 6]> (&arr[0..6 ].try_into().unwrap())),
                _ if arr[6] < 0x80 => Ok(self.read_varint64::<[u8; 7]> (&arr[0..7 ].try_into().unwrap())),
                _ if arr[7] < 0x80 => Ok(self.read_varint64::<[u8; 8]> (&arr[0..8 ].try_into().unwrap())),
                _ if arr[8] < 0x80 => Ok(self.read_varint64::<[u8; 9]> (&arr[0..9 ].try_into().unwrap())),
                _ if arr[9] < 0x80 => Ok(self.read_varint64::<[u8; 10]>(&arr[0..10].try_into().unwrap())),
                _ => Err(Error::MalformedVarint)
            }
        } else {
            let mut result = 0u64;
            for (i, &b) in self.as_limited_slice().iter().enumerate() {
                result |= ((b & 0x7f) as u64) << (7 * i);
                if b < 0x80 {
                    unsafe { self.advance(i); }
                    return Ok(result);
                }
            }
            Err(stream::Error.into())
        }
    }
    fn read_bit32(&mut self) -> Result<u32> {
        self.try_limited_as_array()
            .ok_or(stream::Error.into())
            .map(|&arr| {
                unsafe { self.advance(4); } // since we already got the array, we know we have at least 4 bytes
                u32::from_le_bytes(arr)
            })
    }
    fn read_bit64(&mut self) -> Result<u64> {
        self.try_limited_as_array()
            .ok_or(stream::Error.into())
            .map(|&arr| {
                unsafe { self.advance(8); } // since we already got the array, we know we have at least 8 bytes
                u64::from_le_bytes(arr)
            })
    }
    fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        let len = self.read_varint32()? as i32;
        match len {
            len if len < 0 => Err(Error::NegativeSize),
            0 => Ok(ByteString::new(0)),
            len if len as usize > self.len_limited() => Err(stream::Error.into()),
            len => {
                let len = len as usize;
                let mut bytes = B::new(len);
                let bp = bytes.as_mut().as_mut_ptr();
                unsafe { // we've checked that we have enough data to copy in the branch above
                    self.start.copy_to_nonoverlapping(bp, len);
                    self.advance(len);
                }
                Ok(bytes)
            }
        }
    }

    fn skip_varint(&mut self) -> Result<()> {
        if let Some::<&[u8; 10]>(arr) = self.try_limited_as_array() {
            for (i, &b) in arr.iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.advance(i); }
                    return Ok(());
                }
            }
            Err(Error::MalformedVarint)
        } else {
            for (i, &b) in self.as_limited_slice().iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.advance(i); }
                    return Ok(());
                }
            }
            Err(stream::Error.into())
        }
    }
    fn skip_bit32(&mut self) -> Result<()> {
        if self.len_limited() <= 4 {
            unsafe { self.advance(4); }
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }
    fn skip_bit64(&mut self) -> Result<()> {
        if self.len_limited() <= 8 {
            unsafe { self.advance(8); }
            Ok(())
        } else {
            Err(stream::Error.into())
        }
    }
    fn skip_length_delimited(&mut self) -> Result<()> {
        let len = self.read_varint32()? as i32;
        if len < 0 {
            Err(Error::NegativeSize)
        } else {
            let len = len as usize;
            if self.len_limited() <= len {
                unsafe { self.advance(len); }
                Ok(())
            } else {
                Err(stream::Error.into())
            }
        }
    }

    fn into_any<'a>(&'a mut self) -> Any<'a> {
        Any {
            stream_data: None,
            remaining_limit: 0,

            start: self.start,
            limit: self.limit,
            end: self.end,

            reached_end: self.reached_end()
        }
    }
    fn from_any<'a>(&'a mut self, any: Any<'a>) {
        self.start = any.start;
        self.limit = any.limit;
        // we don't need to move back end since it'll never change
    }

    fn reached_end(&self) -> bool {
        self.start > self.end
    }
}

impl Input for Slice<'_> { }

unsafe impl Send for Slice<'_> { }
unsafe impl Sync for Slice<'_> { }

/// A type used for a [`CodedReader`] reading from a `Read` input. This input type buffers the stream's data.
/// 
/// [`CodedReader`]: struct.CodedReader.html
pub struct Stream<T> {
    input: T,
    buf: Box<[u8]>,
    start: *const u8,
    limit: *const u8,
    end: *const u8,
    remaining_limit: i32,
    reached_end: bool
}

impl<T: Read> Stream<T> {
    fn new(input: T, cap: usize) -> Self {
        let buf = alloc::vec![0; cap].into_boxed_slice();
        let ptr = buf.as_ptr();

        Stream {
            input,
            buf,
            start: ptr,
            limit: ptr,
            end: ptr,
            remaining_limit: -1,
            reached_end: false
        }
    }
    fn buf_len(&self) -> usize {
        usize::wrapping_sub(self.start as _, self.limit as _)
    }
    fn buf(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.start, self.buf_len()) }
    }
    fn refresh(&mut self) -> Result<usize> {
        unimplemented!()
    }
    fn try_as_array<A: Array>(&self) -> Option<&A> {
        if self.buf_len() >= A::LENGTH {
            unsafe { Some(&*self.start.cast()) }
        } else {
            None
        }
    }
    fn reached_limit(&self) -> bool {
        self.start > self.limit && self.remaining_limit == 0
    }
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        if !self.reached_limit() && self.buf_len() == 0 && buf.len() > self.buf.len() {
            while buf.len() > 0 {
                let amnt = self.input.read(buf)?;
                buf = &mut buf[..amnt];
            }
        } else {
            while buf.len() > 0 {
                let current_buf = self.buf();
                if buf.len() <= current_buf.len() {
                    buf[..current_buf.len()].copy_from_slice(current_buf);
                    unsafe { self.advance(buf.len()); }
                    break;
                } else {
                    let (buffered, read) = buf.split_at_mut(self.buf_len());
                    buffered.copy_from_slice(current_buf);
                    unsafe { self.advance(buffered.len()); }
                    buf = read;

                    if self.refresh()? == 0 {
                        self.reached_end = true;
                        return Err(stream::Error.into());
                    }
                }
            }
        }

        Ok(())
    }
    fn skip(&mut self, amnt: usize) -> Result<()> {
        if self.buf.len() > 512 {
            if self.buf_len() >= amnt {
                unsafe { self.advance(amnt); }
                return Ok(());
            } else {
                while amnt > 0 {

                }
            }
        } else {
            let mut tmp = [0; 512];

        }

        Ok(())
    }
    unsafe fn advance(&mut self, amnt: usize) {
        self.start = self.start.add(amnt);
    }
    fn try_read_byte(&mut self) -> Result<Option<u8>> {
        unimplemented!()
    }
    fn read_byte(&mut self) -> Result<u8> {
        unimplemented!()
    }
    fn reached_end_of_buffer(&self) -> bool {
        self.start > self.end
    }
    fn read_tag_fallback(&mut self, b: u8) -> Result<u32> {
        unimplemented!()
    }
    fn read_varint32_slow(&mut self) -> Result<u32> {
        unimplemented!()
    }
    fn read_varint64_slow(&mut self) -> Result<u64> {
        unimplemented!()
    }
}

impl<T: Read> Reader for Stream<T> {
    fn push_limit(&mut self, limit: i32) -> result::Result<Option<i32>, stream::Error> {
        let limit_len_in_buf = cmp::min(self.buf_len(), limit as usize);
        let remaining_limit = limit - (limit_len_in_buf as i32);
        let old_remaining_limit = self.remaining_limit;

        self.limit = unsafe { self.start.add(limit_len_in_buf) };
        self.remaining_limit = remaining_limit;

        if old_remaining_limit < 0 {
            Ok(None)
        } else {
            Ok(Some(old_remaining_limit))
        }
    }
    fn pop_limit(&mut self, old: Option<i32>) {
        match old {
            Some(old) => {
                unimplemented!()
            },
            None => self.remaining_limit = -1,
        }
    }
    fn reached_limit(&self) -> bool {
        self.reached_limit()
    }

    fn read_tag(&mut self) -> Result<Option<u32>> {
        match self.try_read_byte()? {
            Some(b) => {
                if b < 0x80 {
                    Ok(Some(b as u32))
                } else {
                    self.read_tag_fallback(b).map(Some)
                }
            },
            None => Ok(None)
        }
    }
    fn read_varint32(&mut self) -> Result<u32> {
        if let Some::<&[u8; 10]>(buf) = self.try_as_array() {
            let mut result = 0;
            for (i, &b) in buf[0..5].iter().enumerate() {

            }
            for (i, &b) in buf[5..10].iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.advance(i); }
                    return Ok(result);
                }
            }
            Err(Error::MalformedVarint)
        } else {
            self.read_varint32_slow()
        }
    }
    fn read_varint64(&mut self) -> Result<u64> {
        if let Some::<&[u8; 10]>(buf) = self.try_as_array() {
            unimplemented!()
        } else {
            self.read_varint64_slow()
        }
    }
    fn read_bit32(&mut self) -> Result<u32> {
        let mut value = [0u8; 4];
        self.read_exact(&mut value)?;
        Ok(u32::from_le_bytes(value))
    }
    fn read_bit64(&mut self) -> Result<u64> {
        let mut value = [0u8; 8];
        self.read_exact(&mut value)?;
        Ok(u64::from_le_bytes(value))
    }
    fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        let len = self.read_varint32()? as i32;
        if len < 0 {
            Err(Error::NegativeSize)
        } else {
            let mut b = B::new(len as usize);
            if len != 0 {
                self.read_exact(b.as_mut())?;
            }
            Ok(b)
        }
    }

    fn skip_varint(&mut self) -> Result<()> {
        if let Some::<&[u8; 10]>(buf) = self.try_as_array() {
            for (i, &b) in buf.iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.advance(i); }
                    return Ok(());
                }
            }
        } else {
            for i in 0..10 {
                let b = self.read_byte()?;
                if b < 0x80 {
                    return Ok(());
                }
            }
        }

        Err(Error::MalformedVarint)
    }
    fn skip_bit32(&mut self) -> Result<()> {
        self.skip(4)
    }
    fn skip_bit64(&mut self) -> Result<()> {
        self.skip(8)
    }
    fn skip_length_delimited(&mut self) -> Result<()> {
        let len = self.read_varint32()? as i32;
        if len < 0 {
            Err(Error::NegativeSize)
        } else {
            self.skip(len as usize)
        }
    }

    fn into_any<'a>(&'a mut self) -> Any<'a> {
        Any {
            stream_data: Some((&mut self.input, &mut self.buf)),
            remaining_limit: self.remaining_limit,

            start: self.start,
            limit: self.limit,
            end: self.end,

            reached_end: self.reached_end
        }
    }
    fn from_any<'a>(&'a mut self, any: Any<'a>) {
        self.remaining_limit = any.remaining_limit;
        self.start = any.start;
        self.limit = any.limit;
        self.end = any.end;
        self.reached_end = any.reached_end;
    }

    fn reached_end(&self) -> bool {
        self.reached_end
    }
}

impl<T: Read> Input for Stream<T> { }

unsafe impl<T: Send> Send for Stream<T> { }
unsafe impl<T: Sync> Sync for Stream<T> { }

#[derive(Clone, Debug)]
struct ReaderOptions {
    unknown_fields: UnknownFieldHandling,
    registry: Option<&'static ExtensionRegistry>,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        ReaderOptions {
            unknown_fields: UnknownFieldHandling::Store,
            registry: None
        }
    }
}

/// A builder used to construct [`CodedReader`](struct.CodedReader.html) instances
#[derive(Clone, Debug, Default)]
pub struct Builder {
    options: ReaderOptions
}

/// Handling options for unknown fields
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnknownFieldHandling {
    /// Stores unknown fields in a message's `UnknownFieldSet`
    Store,
    /// Skips unknown fields when they're encounted
    Skip,
}

impl UnknownFieldHandling {
    /// Returns whether the handling is set to skip unknown fields
    pub fn skip(self) -> bool {
        self == UnknownFieldHandling::Skip
    }
}

impl Builder {
    /// Creates a new builder with the default configuration
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets unknown field handling for the reader
    #[inline]
    pub fn unknown_fields(mut self, value: UnknownFieldHandling) -> Self {
        self.options.unknown_fields = value;
        self
    }
    /// Sets the registry extendable messages should use when being created
    #[inline]
    pub fn registry(mut self, registry: Option<&'static ExtensionRegistry>) -> Self {
        self.options.registry = registry;
        self
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and 
    /// the specified slice of bytes
    #[inline]
    pub fn with_slice<'a>(&self, inner: &'a [u8]) -> CodedReader<Slice<'a>> {
        CodedReader {
            inner: Slice::new(inner),
            last_tag: None,
            options: self.options.clone()
        }
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and 
    /// the specified [`Read`](stream/trait.Read.html) object with the default buffer capacity
    #[inline]
    pub fn with_stream<T: Read>(&self, inner: T) -> CodedReader<Stream<T>> {
        self.with_capacity(DEFAULT_BUF_SIZE, inner)
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and
    /// the specified [`Read`](stream/trait.Read.html) object with the specified buffer capacity
    #[inline]
    pub fn with_capacity<T: Read>(&self, capacity: usize, inner: T) -> CodedReader<Stream<T>> {
        CodedReader {
            inner: Stream::new(inner, capacity),
            last_tag: None,
            options: self.options.clone()
        }
    }
}

/// Represents any input type for a CodedReader. This is slower than a
/// generic stream input or slice, but is more flexible and can be used 
/// in cases where the input or message type is unknown.
pub struct Any<'a> {
    stream_data: Option<(&'a mut dyn Read, &'a mut [u8])>,
    /// Values < 0 indicate no limit
    remaining_limit: i32,

    start: *const u8,
    /// With no limit, this is equal to end
    limit: *const u8,
    end: *const u8,

    reached_end: bool
}

impl Any<'_> {
    fn reached_limit(&self) -> bool {
        self.start > self.limit && self.remaining_limit >= 0
    }
    #[inline]
    fn next_byte(&self) -> Option<u8> {
        if !self.reached_limit() {
            unsafe { Some(*self.start) }
        } else {
            None
        }
    }
    #[inline]
    fn try_read_byte(&mut self) -> Option<u8> {
        if !self.reached_limit() {
            let result = unsafe { Some(*self.start) };
            unsafe { self.advance(1); }
            result
        } else {
            None
        }
    }
    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        if !self.reached_limit() {
            let result = unsafe { Ok(*self.start) };
            unsafe { self.advance(1); }
            result
        } else {
            unimplemented!()
        }
    }
    fn is_stream(&self) -> bool {
        self.stream_data.is_some()
    }
    fn len(&self) -> usize {
        usize::wrapping_sub(self.limit as _, self.start as _)
    }
    fn refresh(&mut self) -> Result<Option<NonZeroUsize>> {
        match &mut self.stream_data {
            Some((input, buf)) => {
                let new_buf_len = input.read(buf)?;

                self.start = buf.as_ptr();
                self.end = unsafe { self.start.add(new_buf_len) };
                if self.remaining_limit < 0 {
                    self.limit = self.end;
                } else {
                    let mut remaining_limit = self.remaining_limit as usize;
                    if remaining_limit < new_buf_len {
                        self.remaining_limit = 0;
                        self.limit = unsafe { self.end.sub(remaining_limit) };
                    } else {
                        remaining_limit -= new_buf_len;
                        self.remaining_limit = remaining_limit as i32;
                        self.limit = self.end;
                    }
                }

                Ok(NonZeroUsize::new(new_buf_len))
            },
            _ => Ok(None)
        }
    }

    unsafe fn advance(&mut self, amnt: usize) {
        let new_pos = self.start.add(amnt);

        debug_assert!(new_pos < self.limit, "advanced past end of limit");
        debug_assert!(new_pos < self.end, "advanced past end of buffer");

        self.start = new_pos;
    }

    fn skip(&mut self, mut amnt: usize) -> Result<()> {
        loop {
            let buf_len = self.len();
            if amnt <= buf_len {
                unsafe { self.advance(amnt); }
                return Ok(());
            } else {
                if let None = self.refresh()? {
                    return Err(stream::Error.into());
                } else {
                    amnt -= buf_len;
                }
            }
        }
    }
}

impl Input for Any<'_> { }
impl Reader for Any<'_> {
    fn push_limit(&mut self, limit: i32) -> result::Result<Option<i32>, stream::Error> {
        unimplemented!()
    }
    fn pop_limit(&mut self, old: Option<i32>) {
        unimplemented!()
    }
    fn reached_limit(&self) -> bool {
        unimplemented!()
    }

    #[inline]
    fn read_tag(&mut self) -> Result<Option<u32>> {
        if self.reached_limit() {
            Ok(None)
        } else
        if self.reached_end() {
            match self.refresh()? {
                None => Ok(None),
                _ => self.read_varint32().map(Some)
            }
        } else {
            self.read_varint32().map(Some)
        }
    }
    fn read_varint32(&mut self) -> Result<u32> {
        unimplemented!()
    }
    fn read_varint64(&mut self) -> Result<u64> {
        unimplemented!()
    }
    fn read_bit32(&mut self) -> Result<u32> {
        unimplemented!()
    }
    fn read_bit64(&mut self) -> Result<u64> {
        unimplemented!()
    }
    fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        unimplemented!()
    }

    fn skip_varint(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn skip_bit32(&mut self) -> Result<()> {
        self.skip(4)
    }
    fn skip_bit64(&mut self) -> Result<()> {
        self.skip(8)
    }
    fn skip_length_delimited(&mut self) -> Result<()> {
        let len = self.read_varint32()? as i32;
        if len < 0 {
            Err(Error::NegativeSize)
        } else {
            self.skip(len as usize)
        }
    }

    fn into_any<'a>(&'a mut self) -> Any<'a> {
        Any {
            stream_data: self.stream_data.as_mut().map::<(&'a mut dyn Read, &'a mut [u8]), _>(|(s, b)| (s, b)),
            remaining_limit: self.remaining_limit,

            start: self.start,
            limit: self.limit,
            end: self.end,

            reached_end: self.reached_end
        }
    }
    fn from_any<'a>(&'a mut self, any: Any<'a>) {
        self.remaining_limit = any.remaining_limit;
        self.start = any.start;
        self.limit = any.limit;
        self.end = any.end;
        self.reached_end = any.reached_end;
    }

    fn reached_end(&self) -> bool {
        unimplemented!()
    }
}

unsafe impl Send for Any<'_> { }
unsafe impl Sync for Any<'_> { }

/// Provides a bridge for a generic [`CodedReader`] to be converted
/// to a [`CodedReader`]`<`[`Any`]`>` and vice versa.
/// 
/// This allows certain code to bridge gaps where not all merge functions
/// can be generic over an input like extension or reflection contexts.
/// 
/// [`CodedReader`]: struct.CodedReader.html
/// [`Any`]: struct.Any.html
pub struct AnyConverter<'a, T: Input + 'a> {
    src: NonNull<CodedReader<T>>,
    brdg: ManuallyDrop<CodedReader<Any<'a>>>
}

impl<'a, T: Input> AnyConverter<'a, T> {
    fn new(src: &'a mut CodedReader<T>) -> Self {
        let src_ptr = unsafe { NonNull::new_unchecked(src) }; // don't use from since the borrow moves into the from call
        let brdg = 
            CodedReader {
                inner: src.inner.into_any(),
                last_tag: src.last_tag,
                options: src.options.clone()
            };
        Self {
            src: src_ptr,
            brdg: ManuallyDrop::new(brdg)
        }
    }
}

impl<'a, T: Input> ops::Deref for AnyConverter<'a, T> {
    type Target = CodedReader<Any<'a>>;

    fn deref(&self) -> &CodedReader<Any<'a>> {
        &self.brdg
    }
}

impl<'a, T: Input> ops::DerefMut for AnyConverter<'a, T> {
    fn deref_mut(&mut self) -> &mut CodedReader<Any<'a>> {
        &mut self.brdg
    }
}

impl<'a, T: Input> Drop for AnyConverter<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let src: &'a mut CodedReader<T> = &mut *self.src.as_ptr();

            src.last_tag = self.brdg.last_tag;
            src.inner.from_any(ManuallyDrop::take(&mut self.brdg).inner);
        }
    }
}

/// A reader used by generated code to quickly parse field values without tag
/// wire type and field number checking.
/// 
/// This structure defers tag checking, making it faster to read fields when matching
/// on an existing field tag value.
pub struct FieldReader<'a, T: Input + 'a> {
    inner: &'a mut CodedReader<T>,
    tag: u32,
}

impl<'a, T: Input + 'a> FieldReader<'a, T> {
    /// Gets the tag value read from the CodedReader
    #[inline]
    pub fn tag(&self) -> u32 {
        self.tag
    }
    /// Reads the field value using the specified function, passing the checked tag value to set as the last tag
    #[inline]
    pub fn read_value<F: FnOnce(&mut CodedReader<T>) -> Result<()>>(self, tag: Tag, f: F) -> Result<()> {
        debug_assert_eq!(self.tag, tag.get(), "Provided tag does not match read tag value");
        self.inner.last_tag = Some(tag);

        f(self.inner)
    }

    /// Reads the field value using the specified function, checking the tag before running the function.
    #[inline]
    pub fn check_and_read_value<F: FnOnce(&mut CodedReader<T>) -> Result<()>>(self, f: F) -> Result<()> {
        let tag = Tag::try_from(self.tag).map_err(|_| Error::InvalidTag(self.tag))?;
        self.inner.last_tag = Some(tag);

        f(self.inner)
    }
}

/// Represents a length delimited value that can be read in a specified format.
pub struct Limit<'a, T: Input + 'a> {
    inner: &'a mut CodedReader<T>,
    old: Option<i32>,
}

impl<'a, T: Input + 'a> Limit<'a, T> {
    /// Reads a length delimited value using the specified function.
    pub fn then<F: FnOnce(&mut CodedReader<T>) -> Result<()>>(self, f: F) -> Result<()> {
        f(self.inner)
    }

    /// Reads multiple values in a length delimited value using the specified function.
    pub fn for_all<F: FnMut(&mut CodedReader<T>) -> Result<()>>(self, mut f: F) -> Result<()> {
        while !self.inner.reached_limit() {
            f(self.inner)?;
        }
        Ok(())
    }
}

impl<'a, T: Input + 'a> Drop for Limit<'a, T> {
    fn drop(&mut self) {
        self.inner.pop_limit(self.old);
    }
}

/// A coded input reader that reads from a specified input.
pub struct CodedReader<T: Input> {
    inner: T,
    last_tag: Option<Tag>,
    options: ReaderOptions,
}

impl<T: Read> CodedReader<Stream<T>> {
    /// Creates a new [`CodedReader`] in the default configuration
    /// over the specified [`Read`] with the default buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
    pub fn with_stream(inner: T) -> Self {
        Builder::new().with_stream(inner)
    }
    /// Creates a new [`CodedReader`] in the default configuration
    /// over the specified [`Read`] with the specified buffer capacity.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`Read`]: streams/trait.Read.html
    pub fn with_capacity(capacity: usize, inner: T) -> Self {
        Builder::new().with_capacity(capacity, inner)
    }
}

impl<'a> CodedReader<Slice<'a>> {
    /// Creates a new [`CodedReader`] over the borrowed [`slice`]
    /// in the default configuration. This is optimized to read directly
    /// from the slice, making it faster than reading from a [`Read`] object.
    /// 
    /// [`CodedReader`]: struct.CodedReader.html
    /// [`slice`]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    /// [`Read`]: streams/trait.Read.html
    pub fn with_slice(inner: &'a [u8]) -> Self {
        Builder::new().with_slice(inner)
    }
}

impl<T: Input> CodedReader<T> {
    /// Gets handling options for unknown fields read with this reader.
    pub fn unknown_field_handling(&self) -> UnknownFieldHandling {
        self.options.unknown_fields
    }
    /// Gets the registry extendable messages should be created with when
    /// reading from this reader.
    pub fn registry(&self) -> Option<&'static ExtensionRegistry> {
        self.options.registry
    }
    /// Gets the last tag read by the reader.
    pub fn last_tag(&self) -> Option<Tag> {
        self.last_tag
    }
    /// Returns an [`AnyConverter`] that can be used to temporarily 
    /// convert the reader into a non-generic reader over [`Any`] input.
    pub fn as_any<'a>(&'a mut self) -> AnyConverter<'a, T> {
        AnyConverter::new(self)
    }

    /// Reads a length value from the input.
    /// 
    /// # Errors
    /// 
    /// If a negative length is read, this returns a `NegativeSize` error.
    pub fn read_limit<'a>(&'a mut self) -> Result<Limit<'a, T>> {
        let limit = self.read_value::<raw::Int32>()?;
        if limit < 0 {
            Err(Error::NegativeSize)
        } else {
            let old = self.inner.push_limit(limit)?;
            Ok(Limit { inner: self, old })
        }
    }
    fn pop_limit(&mut self, old: Option<i32>) {
        self.inner.pop_limit(old)
    }
    fn reached_limit(&self) -> bool {
        self.inner.reached_limit()
    }

    /// Reads a field tag from the input
    pub fn read_tag(&mut self) -> Result<Option<Tag>> {
        self.last_tag = 
            self.inner.read_tag()?
                .map(|v| Tag::try_from(v).map_err(|_| Error::InvalidTag(v)))
                .transpose()?;
        Ok(self.last_tag)
    }
    /// Reads a 32-bit varint field value. This is functionally similar to [`read_varint64`](#method.read_varint64),
    /// but is optimised for 32-bit values and will discard any top bits from 64-bit values.
    pub fn read_varint32(&mut self) -> Result<u32> {
        self.inner.read_varint32()
    }
    /// Reads a 64-bit varint field value.
    pub fn read_varint64(&mut self) -> Result<u64> {
        self.inner.read_varint64()
    }
    /// Reads a 4-byte little endian value
    pub fn read_bit32(&mut self) -> Result<u32> {
        self.inner.read_bit32()
    }
    /// Reads a 8-byte little endian value
    pub fn read_bit64(&mut self) -> Result<u64> {
        self.inner.read_bit64()
    }
    /// Reads a length delimited string of bytes.
    pub fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        self.inner.read_length_delimited()
    }
    /// Skips the last field read from the input
    pub fn skip(&mut self) -> Result<()> {
        if let Some(last_tag) = self.last_tag() {
            match last_tag.wire_type() {
                WireType::Varint => self.inner.skip_varint()?,
                WireType::Bit64 => self.inner.skip_bit64()?,
                WireType::LengthDelimited => self.inner.skip_length_delimited()?,
                WireType::StartGroup => {
                    let end = Tag::new(last_tag.number(), WireType::EndGroup);
                    loop {
                        match self.read_tag()? {
                            Some(tag) if tag == end => break,
                            Some(_) => self.skip()?,
                            None => return Err(Error::StreamError(stream::Error))
                        }
                    }
                },
                WireType::EndGroup => { },
                WireType::Bit32 => self.inner.skip_bit32()?,
            }
        }

        Ok(())
    }

    /// Reads a field value. This offloads checking of the tag's value, making it faster when reading
    /// many fields when the tag's underlying value already exists in code.
    #[inline]
    pub fn read_field<'a>(&'a mut self) -> Result<Option<FieldReader<'a, T>>> {
        self.inner.read_tag().map(move |t| t.map(move |t| FieldReader { inner: self, tag: t }))
    }
    /// Reads a new instance of the value from the reader.
    /// This is the inverse of `Value::read_new`.
    #[inline]
    pub fn read_value<V: Value + Wrapper>(&mut self) -> Result<V::Inner> {
        V::read_new(self).map(V::unwrap)
    }
    /// Merges the reader's value with the value from the reader.
    /// This is the inverse of `Value::merge_from`.
    #[inline]
    pub fn merge_value<V: Value + Wrapper>(&mut self, value: &mut V::Inner) -> Result<()> {
        V::wrap_mut(value).merge_from(self)
    }
    /// Adds field entries from the reader to the specified value.
    /// This is the inverse of `RepeatedValue::add_entries_from`.
    #[inline]
    pub fn add_entries_to<U, V: RepeatedValue<U>>(&mut self, value: &mut V) -> Result<()> {
        value.add_entries_from(self)
    }
    /// Tries to add the field value to the field set.
    /// This is the inverse of `FieldSet::try_add_field_from`.
    #[inline]
    pub fn try_add_field_to<'a, U: FieldSet>(&'a mut self, value: &mut U) -> Result<TryRead<'a, T>> {
        value.try_add_field_from(self)
    }
}

#[cfg(test)]
mod test {
    use crate::io::{stream, read::{Reader, Any, Error}};

    trait Checked {
        /// Asserts whether the input type is in a valid state
        fn assert_is_valid(&self);
    }

    trait ReaderInput<'a> {
        type Reader: Reader + Checked + 'a;

        fn new(b: &'a [u8]) -> Self::Reader;
    }

    trait Case {
        fn input() -> &'static [u8];
        fn run<R: Reader>(r: &mut R);

        fn run_with<R: ReaderInput<'static>>() {
            let mut r = R::new(Self::input());
            Self::run(&mut r);
            r.assert_is_valid();
        }
        fn run_with_any<R: ReaderInput<'static>>() {
            let mut r = R::new(Self::input());
            let mut any = r.into_any();
            Self::run(&mut any);
        }
    }

    macro_rules! test {
        ($($ti:ident = $x:expr => |$f:ident| $t:block),+) => {
            $(
                pub struct $ti;

                impl $ti {
                    const INPUT: &'static [u8] = &$x;
                }

                impl Case for $ti {
                    fn input() -> &'static [u8] { Self::INPUT }
                    fn run<R: Reader>($f: &mut R) {
                        $t
                    }
                }
            )+
        };
    }

    test! {
        ReadNoTagFromEmpty = [] => |r| {
            let result = r.read_tag();

            assert!(matches!(result, Ok(None)));
        },
        ReadTag = [8] => |r| {
            let result = r.read_tag();
            
            assert!(matches!(result, Ok(Some(8))));

            let result = r.read_tag();

            assert!(matches!(result, Ok(None)));
        },
        Read2ByteTag = [128, 1] => |r| {
            let result = r.read_tag();
    
            assert!(matches!(result, Ok(Some(0x80))));
    
            let result = r.read_tag();
    
            assert!(matches!(result, Ok(None)));
        },
        Read5ByteTag = [128, 128, 128, 128, 1] => |r| {
            let result = r.read_tag();
    
            assert!(matches!(result, Ok(Some(0x10000000))));
    
            let result = r.read_tag();
    
            assert!(matches!(result, Ok(None)));
        },
        Read10ByteTag = [128, 128, 128, 128, 129, 128, 128, 128, 128, 0] => |r| {
            let result = r.read_tag();
    
            assert!(matches!(result, Ok(Some(0x10000000))));
    
            let result = r.read_tag();
    
            assert!(matches!(result, Ok(None)));
        },
        ReadTruncatedTag = [128] => |r| {
            let result = r.read_tag();
    
            assert!(matches!(result, Err(Error::StreamError(stream::Error))));
        },
        ReadTruncated9ByteTag = [128u8; 9] => |r| {
            let result = r.read_tag();
    
            assert!(matches!(result, Err(Error::StreamError(stream::Error))));
        },
        ReadMalformedTag = [128u8; 10] => |r| {
            let result = r.read_tag();

            assert!(matches!(result, Err(Error::MalformedVarint)));
        }
    }

    macro_rules! run {
        (
            $f:ty => {
                $($t:ident => $tp:ident),*
            }
        ) => {
            $(
                #[test]
                fn $t() {
                    <crate::io::read::test::$tp as Case>::run_with::<$f>()
                }
            )*
        };
    }

    macro_rules! run_suite {
        ($f:ty) => {
            run! {
                $f => {
                    read_no_tag_from_empty => ReadNoTagFromEmpty,
                    read_tag => ReadTag,
                    read_2byte_tag => Read2ByteTag,
                    read_5byte_tag => Read5ByteTag,
                    read_10byte_tag => Read10ByteTag,
                    read_truncated_tag => ReadTruncatedTag,
                    read_truncated_9byte_tag => ReadTruncated9ByteTag,
                    read_malformed_tag => ReadMalformedTag
                }
            }
        };
    }

    mod slice {
        use crate::io::{stream, read::{Slice, Error, internal::Reader}};
        use super::{ReaderInput, Checked, Case};

        impl Checked for Slice<'_> {
            fn assert_is_valid(&self) {
                assert!(self.start <= self.limit);
                assert!(self.start <= self.end);
            }
        }

        impl<'a> ReaderInput<'a> for Slice<'a> {
            type Reader = Self;

            fn new(b: &'a [u8]) -> Self {
                Slice::new(b)
            }
        }

        run_suite!(Slice);
    }

    mod stream {
        use crate::io::{stream, read::{self, Stream, Error, internal::Reader}};
        use super::{ReaderInput, Checked, Case};

        impl Checked for Stream<&'_ [u8]> {
            fn assert_is_valid(&self) {
                unimplemented!()
            }
        }

        impl<'a> ReaderInput<'a> for Stream<&'a [u8]> {
            type Reader = Self;

            fn new(b: &'a [u8]) -> Self {
                Stream::new(b, read::DEFAULT_BUF_SIZE)
            }
        }

        run_suite!(Stream);

        mod no_buffer {
            use crate::io::{stream, read::{Stream, Error, internal::Reader}};
            use super::{ReaderInput, Checked, Case};

            pub struct StreamNoBuffer;

            impl<'a> ReaderInput<'a> for StreamNoBuffer {
                type Reader = Stream<&'a [u8]>;

                fn new(b: &'a [u8]) -> Self::Reader {
                    Stream::new(b, 0)
                }
            }

            run_suite!(StreamNoBuffer);
        }
    }
}