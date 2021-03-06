//! Defines the `CodedReader`, a reader for reading values from a protobuf encoded byte stream.

use crate::Message;
use crate::collections::{RepeatedValue, FieldSet, TryRead};
use crate::extend::ExtensionRegistry;
use crate::io::{Tag, WireType, FieldNumber, Length, ByteString, DEFAULT_BUF_SIZE};
use crate::raw::{self, Value};
use std::boxed::Box;
use std::cmp::{self, Ordering};
use std::convert::TryFrom;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read, ErrorKind};
use std::marker::PhantomData;
use std::result;
use std::string::FromUtf8Error;

/// A trait used by `CodedReader`s to efficiently skip bytes in an input.
pub trait Skip: Read {
    /// Skips a series of bytes from the input
    fn skip_exact(&mut self, amnt: Length) -> io::Result<()>;
}

impl<T: ?Sized + Read> Skip for T {
    default fn skip_exact(&mut self, amnt: Length) -> io::Result<()> {
        let mut take = self.take(amnt.get() as u64);
        let mut sink = io::sink();
        io::copy(&mut take, &mut sink)?;
        if take.limit() != 0 {
            Err(io::Error::from(ErrorKind::UnexpectedEof))
        } else {
            Ok(())
        }
    }
}

impl<T: ?Sized + Read + io::Seek> Skip for T {
    default fn skip_exact(&mut self, amnt: Length) -> io::Result<()> {
        self.seek(io::SeekFrom::Current(amnt.get() as i64)).map(|_| ())
    }
}

mod internal {
    use crate::io::{ByteString, Tag, Length, internal::Array, read::{Result, Error}};
    use std::cmp::{self, Ordering};
    use std::convert::TryFrom;
    use std::io::{self, Read as _, ErrorKind};
    use std::ops::Range;
    use std::ptr::{self, NonNull};
    use super::Skip as Read;

    /// State shared between all readers. This is borrowed by Any to manage state of a specialized reader
    #[derive(Default)]
    pub struct SharedState {
        pub recursion_depth: usize,
        pub last_tag: Option<Tag>,
        pub next_end_group: Option<Tag>,
    }

    /// A container for shared buffer manipulation logic.
    /// This does not track the lifetime of the buffer,
    /// so caution must by used when using or moving it.
    #[derive(Copy, Clone)]
    pub struct Buffer {
        start: NonNull<u8>,
        limit: NonNull<u8>,
        /// With no limit, this is None and the limit is the end
        end: Option<NonNull<u8>>,
    }

    impl Buffer {
        #[inline]
        pub fn from_slice(s: &[u8]) -> Self {
            let Range { start, end } = s.as_ptr_range();
            unsafe {
                Self {
                    start: NonNull::new_unchecked(start as _),
                    limit: NonNull::new_unchecked(end as _),
                    end: None
                }
            }
        }
        #[inline]
        pub fn to_limit_len(&self) -> usize {
            usize::wrapping_sub(self.limit.as_ptr() as _, self.start.as_ptr() as _)
        }
        #[inline]
        pub fn to_end_len(&self) -> usize {
            match self.end {
                Some(p) => usize::wrapping_sub(p.as_ptr() as _, self.start.as_ptr() as _),
                None => self.to_limit_len()
            }
        }
        #[inline]
        pub unsafe fn to_limit_as_slice<'a>(&self) -> &'a [u8] {
            std::slice::from_raw_parts(self.start.as_ptr() as _, self.to_limit_len())
        }
        #[inline]
        pub unsafe fn to_end_as_slice<'a>(&self) -> &'a [u8] {
            std::slice::from_raw_parts(self.start.as_ptr() as _, self.to_end_len())
        }
        #[inline]
        pub unsafe fn advance(&mut self, amnt: usize) {
            let new_pos = self.start.as_ptr().add(amnt);

            debug_assert!(new_pos <= self.limit.as_ptr());

            self.start = NonNull::new_unchecked(new_pos);
        }
        #[inline]
        pub fn reached_limit(&self) -> bool {
            self.start == self.limit
        }
        #[inline]
        pub fn has_limit(&self) -> bool {
            self.end.is_some()
        }
        #[inline]
        pub fn remaining_limit(&self) -> Option<i32> {
            self.has_limit().then(|| self.to_limit_len() as i32)
        }
        #[inline]
        pub fn reached_end(&self) -> bool {
            self.reached_limit() && !self.has_limit()
        }
        #[inline]
        pub unsafe fn peek_byte(&self) -> u8 {
            *self.start.as_ref()
        }
        #[inline]
        pub unsafe fn next_byte(&mut self) -> u8 {
            let b = self.peek_byte();
            self.advance(1);
            b
        }
        #[inline]
        pub unsafe fn copy_nonoverlapping(&mut self, slice: &mut [u8]) {
            ptr::copy_nonoverlapping(self.start.as_ref(), slice.as_mut_ptr(), slice.len());
            self.advance(slice.len());
        }
        /// Applies a limit to the buffer, returning a remainder for 
        /// any bytes that couldn't be limited. Assumes that:
        /// 
        ///  1. Limit does not extend beyond an existing limit
        /// 
        ///  2. Limit is not negative
        #[inline]
        pub unsafe fn apply_partial_limit(&mut self, limit: i32) -> i32 {
            let (limit, remainder) = 
                match i32::try_from(self.to_end_len()) {
                    Ok(len) if len < limit => (len, limit - len),
                    _ => (limit, 0)
                };
            self.apply_limit(limit);
            remainder
        }
        /// Applies a limit to the buffer. Assumes that:
        /// 
        ///  1. Limit does not extend beyond the end of the buffer
        /// 
        ///  2. Limit does not extend beyond an existing limit
        /// 
        ///  3. Limit is not negative
        #[inline]
        pub unsafe fn apply_limit(&mut self, limit: i32) {
            if self.end.is_none() {
                self.end = Some(self.limit);
            }
            self.limit = NonNull::new_unchecked(self.start.as_ptr().add(limit as usize));
        }
        #[inline]
        pub unsafe fn remove_limit(&mut self) {
            self.limit = self.end.take().unwrap();
        }
        #[inline]
        pub fn try_limited_as_array<A: Array>(&self) -> Option<&A> {
            if self.to_limit_len() >= A::LENGTH {
                Some(unsafe { &*(self.start.cast().as_ptr()) })
            } else {
                None
            }
        }
    }

    pub trait Reader {
        fn state(&self) -> &SharedState;
        fn state_mut(&mut self) -> &mut SharedState;

        fn push_limit(&mut self, limit: i32) -> io::Result<Option<i32>>;
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

        fn as_any(&mut self) -> Any;

        fn reached_end(&self) -> bool;
    }

    pub struct BorrowedStream<'a> {
        pub input: &'a mut dyn Read,
        pub buf: &'a mut [u8],
        pub remaining_limit: &'a mut i32,
        pub reached_eof: &'a mut bool,
    }

    /// Represents any input type for a CodedReader. This is slower than a
    /// generic stream input or slice, but is more flexible and can be used 
    /// in cases where the input or message type is unknown.
    pub struct Any<'a> {
        pub(super) stream: Option<BorrowedStream<'a>>,
        pub(super) buffer: &'a mut Buffer,
        pub(super) shared_state: &'a mut SharedState,
    }

    impl Any<'_> {
        fn read_buffer_partial<'a>(&mut self, slice: &'a mut [u8]) -> Result<&'a mut [u8]> {
            // check if we reached the end of the buffer
            // does not check if we've reached end of limit
            if self.buffer.reached_end() {
                return Ok(slice);
            }

            let limit_len = self.buffer.to_limit_len();
            match self.stream.as_ref() {
                Some(BorrowedStream { remaining_limit: &mut 0, .. }) | None => {
                    if limit_len < slice.len() {
                        unsafe { self.buffer.advance(limit_len); }
                        return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
                    }
    
                    unsafe {
                        self.buffer.copy_nonoverlapping(slice);
                    }
    
                    Ok(&mut [])
                },
                Some(_) => {
                    let (f, s) = slice.split_at_mut(cmp::min(limit_len, slice.len()));

                    if !f.is_empty() {
                        unsafe {
                            self.buffer.copy_nonoverlapping(f);
                        }
                    }

                    Ok(s)
                },
            }
        }
        fn read_direct(BorrowedStream { input: stream, remaining_limit: limit, .. }: &mut BorrowedStream, buf: &mut [u8]) -> Result<()> {
            if **limit < 0 {
                stream.read_exact(buf).map_err(Into::into)
            } else {
                let remaining_limit = **limit as usize;
                if remaining_limit == 0 {
                    Err(io::Error::from(ErrorKind::UnexpectedEof).into())
                } else if remaining_limit >= buf.len() {
                    **limit = i32::wrapping_sub(**limit, buf.len() as i32);
                    stream.read_exact(buf).map_err(Into::into)
                } else {
                    **limit = 0;
                    stream.read_exact(&mut buf[..remaining_limit])?;
                    Err(io::Error::from(ErrorKind::UnexpectedEof).into())
                }
            }
        }
        /// Attempts to refresh the buffer, returning a bool indicating if the data buffer was filled
        fn try_refresh(&mut self) -> Result<bool> {
            let BorrowedStream { input, buf, remaining_limit, reached_eof } = match &mut self.stream {
                Some(s) => s,
                None => return Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
            };
            let amnt = input.read(buf)?;

            *self.buffer = Buffer::from_slice(&buf[..amnt]);
            if **remaining_limit >= 0 {
                **remaining_limit = unsafe { self.buffer.apply_partial_limit(**remaining_limit) };
            }

            let refreshed = amnt != 0;
            **reached_eof = !refreshed;
            Ok(refreshed)
        }
        fn refresh(&mut self) -> Result<()> {
            self.try_refresh().and_then(|b| if b { Ok(()) } else { Err(io::Error::from(ErrorKind::UnexpectedEof).into()) })
        }
        fn read_byte(&mut self) -> Result<u8> {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)?;
            Ok(buf[0])
        }
        fn try_read_byte(&mut self) -> Result<Option<u8>> {
            if self.reached_end() {
                return Ok(None);
            }

            if self.buffer.to_limit_len() != 0 {
                unsafe { Ok(Some(self.buffer.next_byte())) }
            } else {
                match &mut self.stream {
                    Some(BorrowedStream { remaining_limit: &mut 0, .. }) | None => Ok(None),
                    Some(BorrowedStream { input, buf: [], remaining_limit, reached_eof }) => {
                        let mut buf = [0u8; 1];
                        let result = input.read(&mut buf)?;
                        if result != 0 {
                            if **remaining_limit > 0 {
                                **remaining_limit -= 1;
                            }
                            Ok(Some(buf[0]))
                        } else {
                            **reached_eof = true;
                            Ok(None)
                        }
                    },
                    Some(_) => {
                        self.try_refresh()
                            .map(|b| b.then(|| unsafe { self.buffer.next_byte() }))
                            .map_err(Into::into)
                    },
                }
            }
        }
        fn read_exact(&mut self, slice: &mut [u8]) -> Result<()> {
            if self.reached_end() {
                return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
            }

            let mut remaining_slice = self.read_buffer_partial(slice)?;
            if !remaining_slice.is_empty() {
                match &mut self.stream {
                    // if the remaining amnt to read is more than or equal to
                    // the size of the buffer then we read direct from the stream
                    // and adjust our remaining limit accordingly
                    Some(stream) if remaining_slice.len() >= stream.buf.len() => {
                        Self::read_direct(stream, remaining_slice)
                    },
                    Some(_) => {
                        loop {
                            self.refresh()?;
                            remaining_slice = self.read_buffer_partial(remaining_slice)?;

                            if remaining_slice.is_empty() {
                                break Ok(());
                            }
                        }
                    },
                    None => Err(io::Error::from(ErrorKind::UnexpectedEof).into())
                }
            } else {
                Ok(())
            }
        }
        fn skip(&mut self, amnt: i32) -> Result<()> {
            if self.reached_end() {
                return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
            }

            let amnt_usize = amnt as usize;
            let limit_buf_len = self.buffer.to_limit_len();
            if limit_buf_len >= amnt_usize {
                unsafe { self.buffer.advance(amnt_usize); }
                Ok(())
            } else {
                unsafe { self.buffer.advance(limit_buf_len); }
                let remaining_amnt = amnt - limit_buf_len as i32;
                match &mut self.stream {
                    Some(BorrowedStream { input, remaining_limit, .. }) => {
                        match (**remaining_limit).cmp(&0) {
                            Ordering::Less => {
                                input.skip_exact(unsafe { Length::new_unchecked(remaining_amnt) }).map_err(Into::into)
                            },
                            Ordering::Equal => Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
                            Ordering::Greater => {
                                let remaining = **remaining_limit;
                                let remaining_length = unsafe { Length::new_unchecked(remaining) };
                                if remaining > remaining_amnt {
                                    **remaining_limit = 0;
                                    input.skip_exact(remaining_length).map_err(Into::into)
                                } else {
                                    **remaining_limit = i32::wrapping_sub(**remaining_limit, remaining_amnt as i32);
                                    input.skip_exact(remaining_length).map_err(Into::into)
                                }
                            }
                        }
                    },
                    None => Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
                }
            }
        }
    }

    impl<'a> Reader for Any<'a> {
        fn state(&self) -> &SharedState {
            &self.shared_state
        }
        fn state_mut(&mut self) -> &mut SharedState {
            self.shared_state
        }

        fn push_limit(&mut self, limit: i32) -> io::Result<Option<i32>> {
            match &mut self.stream {
                Some(BorrowedStream { remaining_limit, .. }) => {
                    if **remaining_limit < 0 {
                        **remaining_limit = unsafe { self.buffer.apply_partial_limit(limit) };
                        Ok(None)
                    } else {
                        let remaining = i32::wrapping_add(self.buffer.to_limit_len() as i32, **remaining_limit);
                        if remaining < limit {
                            Err(ErrorKind::UnexpectedEof.into())
                        } else {
                            **remaining_limit = unsafe { self.buffer.apply_partial_limit(limit) };
                            Ok(Some(i32::wrapping_sub(remaining, limit)))
                        }
                    }
                },
                None => {
                    if let Some(existing_limit) = self.buffer.remaining_limit() {
                        if existing_limit < limit {
                            Err(ErrorKind::UnexpectedEof.into())
                        } else {
                            let old = i32::wrapping_sub(existing_limit, limit);
                            unsafe { self.buffer.apply_limit(limit) };
                            Ok(Some(old))
                        }
                    } else {
                        let limit_len = self.buffer.to_limit_len();
                        unsafe {
                            match i32::try_from(limit_len) {
                                Ok(end) if limit > end => {
                                    Err(ErrorKind::UnexpectedEof.into())
                                },
                                _ => {
                                    self.buffer.apply_limit(limit);
                                    Ok(None)
                                }
                            }
                        }
                    }
                }
            }
        }
        fn pop_limit(&mut self, old: Option<i32>) {
            unsafe {
                match &mut self.stream {
                    Some(BorrowedStream { remaining_limit, .. }) => {
                        match old {
                            Some(old) => {
                                **remaining_limit = self.buffer.apply_partial_limit(old);
                            },
                            None => {
                                self.buffer.remove_limit();
                                **remaining_limit = -1;
                            }
                        }
                    },
                    None => {
                        match old {
                            Some(old) => self.buffer.apply_limit(old),
                            None => self.buffer.remove_limit(),
                        }
                    }
                }
            }
        }
        fn reached_limit(&self) -> bool {
            match &self.stream {
                Some(BorrowedStream { remaining_limit, ..}) => self.buffer.reached_limit() && **remaining_limit == 0,
                None => self.buffer.reached_limit()
            }
        }

        #[inline]
        fn read_tag(&mut self) -> Result<Option<u32>> {
            let b = match self.try_read_byte()? {
                Some(b) if b < 0x80 => return Ok(Some(b as u32)),
                Some(b) => b,
                None => return Ok(None),
            };

            let mut result = (b & 0x7f) as u32;
            for i in 1..5 {
                let b = self.read_byte()?;
                result |= (b as u32 & 0x7f) << (7 * i);
                if b < 0x80 {
                    return Ok(Some(result));
                }
            }
            for _ in 5..10 {
                let b = self.read_byte()?;
                if b < 0x80 {
                    return Ok(Some(result));
                }
            }
            Err(Error::MalformedVarint)
        }
        fn read_varint32(&mut self) -> Result<u32> {
            let mut result = 0;
            for i in 0..5 {
                let b = self.read_byte()?;
                result |= (b as u32 & 0x7f) << (7 * i);
                if b < 0x80 {
                    return Ok(result);
                }
            }
            for _ in 5..10 {
                let b = self.read_byte()?;
                if b < 0x80 {
                    return Ok(result);
                }
            }
            Err(Error::MalformedVarint)
        }
        fn read_varint64(&mut self) -> Result<u64> {
            let mut result = 0;
            for i in 0..10 {
                let b = self.read_byte()?;
                result |= (b as u64 & 0x7f) << (7 * i);
                if b < 0x80 {
                    return Ok(result);
                }
            }
            Err(Error::MalformedVarint)
        }
        fn read_bit32(&mut self) -> Result<u32> {
            let mut result = [0u8; 4];
            self.read_exact(&mut result)?;
            Ok(u32::from_le_bytes(result))
        }
        fn read_bit64(&mut self) -> Result<u64> {
            let mut result = [0u8; 8];
            self.read_exact(&mut result)?;
            Ok(u64::from_le_bytes(result))
        }
        fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
            let len = 
                self.read_varint32()
                    .and_then(|v| Length::new(v as i32).ok_or(Error::NegativeSize))?
                    .get() as usize;
            let mut string = B::new(len);
            if len != 0 {
                self.read_exact(string.as_mut())?;
            }
            Ok(string)
        }

        fn skip_varint(&mut self) -> Result<()> {
            for _ in 0..10 {
                let b = self.read_byte()?;
                if b < 0x80 {
                    return Ok(());
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
            let len = 
                self.read_varint32()
                    .and_then(|v| Length::new(v as i32).ok_or(Error::NegativeSize))?
                    .get();
            if len != 0 {
                self.skip(len)
            } else {
                Ok(())
            }
        }

        fn as_any(&mut self) -> Any {
            Any {
                stream: 
                    self.stream
                        .as_mut()
                        .map(|BorrowedStream { input, buf, remaining_limit, reached_eof }|
                              BorrowedStream { input: *input, buf, remaining_limit, reached_eof }),
                buffer: &mut self.buffer,
                shared_state: &mut self.shared_state
            }
        }

        fn reached_end(&self) -> bool {
            match &self.stream {
                Some(BorrowedStream { reached_eof, .. }) => **reached_eof && self.buffer.reached_end(),
                None => self.buffer.reached_end()
            }
        }
    }

    unsafe impl Send for Any<'_> { }
    unsafe impl Sync for Any<'_> { }
}

use internal::{Reader, Buffer, SharedState};

pub use internal::Any;

/// The error type for [`CodedReader`](struct.CodedReader.html)
#[derive(Debug)]
pub enum Error {
    /// The input contained a malformed variable length integer
    MalformedVarint,
    /// The input contained a length delimited value which reported it had a negative size
    NegativeSize,
    /// The input attempted to recurse too deep into a nested structure
    RecursionLimitExceeded,
    /// The input contained an invalid tag (zero or the tag had an invalid wire format) or
    /// the tag was invalid in it's position
    InvalidTag(u32),
    /// An error occured while reading from the underlying `Read` object
    IoError(io::Error),
    /// The input contained an invalid UTF8 string
    InvalidString(FromUtf8Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Error {
        Error::IoError(value)
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
            Error::RecursionLimitExceeded => write!(fmt, "the input contained a nested data structure that exceeded the recursion limit"),
            Error::InvalidTag(val) => write!(fmt, "the input contained an tag that was either invalid or was unexpected at this point in the input: {}", val),
            Error::IoError(err) => write!(fmt, "an error occured in the underlying input: {}", err),
            Error::InvalidString(_) => write!(fmt, "the input contained an invalid UTF8 string")
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IoError(ref e) => Some(e),
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
impl<T: internal::Reader> Input for T { }

/// A type used for a [`CodedReader`] reading from a `slice` input.
/// 
/// [`CodedReader`]: struct.CodedReader.html
pub struct Slice<'a> {
    a: PhantomData<&'a [u8]>,
    buffer: Buffer,
    state: internal::SharedState,
}

impl<'a> Slice<'a> {
    fn new(value: &'a [u8]) -> Self {
        Self {
            a: PhantomData,
            buffer: Buffer::from_slice(value),
            state: Default::default(),
        }
    }
}

impl Reader for Slice<'_> {
    fn state(&self) -> &SharedState {
        &self.state
    }
    fn state_mut(&mut self) -> &mut SharedState {
        &mut self.state
    }

    fn push_limit(&mut self, limit: i32) -> io::Result<Option<i32>> {
        let old = match self.buffer.remaining_limit() {
            Some(remaining) => {
                if remaining < limit { // err out if the new limit goes beyond our current limit
                    return Err(ErrorKind::UnexpectedEof.into());
                }

                Some(remaining - limit)
            },
            None => {
                if self.buffer.to_end_len() < limit as usize {
                    return Err(ErrorKind::UnexpectedEof.into());
                }

                None
            },
        };

        unsafe { self.buffer.apply_limit(limit) };
        Ok(old)
    }
    fn pop_limit(&mut self, old: Option<i32>) {
        unsafe {
            match old {
                Some(old) => self.buffer.apply_limit(old),
                None => self.buffer.remove_limit(),
            }
        }
    }
    fn reached_limit(&self) -> bool {
        self.buffer.reached_limit()
    }

    fn read_tag(&mut self) -> Result<Option<u32>> {
        if !self.reached_limit() {
            let next = unsafe { self.buffer.peek_byte() }; // SAFETY: we haven't reached the end so we're fine
            if next < 0x80 {
                unsafe { self.buffer.advance(1); } // SAFETY: same as above
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
        if let Some::<&[u8; 10]>(arr) = self.buffer.try_limited_as_array() {
            let mut iter = arr.as_ref().iter().enumerate();
            for (i, &b) in iter.by_ref().take(5) {
                result |= ((b & 0x7f) as u32) << (7 * i);
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            for (i, &b) in iter {
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(Error::MalformedVarint)
        } else if let Some::<&[u8; 5]>(arr) = self.buffer.try_limited_as_array() {
            for (i, &b) in arr.iter().enumerate() {
                result |= ((b & 0x7f) as u32) << (7 * i);
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            unsafe { self.buffer.advance(5) };
            for (i, &b) in unsafe { self.buffer.to_limit_as_slice() }.iter().enumerate() {
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(io::Error::from(ErrorKind::UnexpectedEof).into())
        } else {
            let slice = unsafe { self.buffer.to_limit_as_slice() };
            for (i, &b) in slice.iter().enumerate() {
                result |= ((b & 0x7f) as u32) << (7 * i);
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(io::Error::from(ErrorKind::UnexpectedEof).into())
        }
    }
    fn read_varint64(&mut self) -> Result<u64> {
        let mut result = 0u64;
        let slice = unsafe { self.buffer.to_limit_as_slice() };
        if slice.len() < 10 {
            for (i, &b) in slice.iter().enumerate() {
                result |= ((b & 0x7f) as u64) << (7 * i);
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(io::Error::from(ErrorKind::UnexpectedEof).into())
        } else {
            for (i, &b) in slice.iter().enumerate().take(10) {
                result |= ((b & 0x7f) as u64) << (7 * i);
                if b < 0x80 {
                    unsafe { self.buffer.advance(i + 1); }
                    return Ok(result);
                }
            }
            Err(Error::MalformedVarint)
        }
    }
    fn read_bit32(&mut self) -> Result<u32> {
        self.buffer.try_limited_as_array()
            .ok_or(io::Error::from(ErrorKind::UnexpectedEof).into())
            .copied()
            .map(|arr| {
                unsafe { self.buffer.advance(4); } // since we already got the array, we know we have at least 4 bytes
                u32::from_le_bytes(arr)
            })
    }
    fn read_bit64(&mut self) -> Result<u64> {
        self.buffer.try_limited_as_array()
            .ok_or(io::Error::from(ErrorKind::UnexpectedEof).into())
            .copied()
            .map(|arr| {
                unsafe { self.buffer.advance(8); } // since we already got the array, we know we have at least 8 bytes
                u64::from_le_bytes(arr)
            })
    }
    fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        let len = self.read_varint32()? as i32;
        match len {
            len if len < 0 => Err(Error::NegativeSize),
            0 => Ok(ByteString::new(0)),
            len if len as usize > self.buffer.to_limit_len() => Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
            len => {
                let len = len as usize;
                let mut bytes = B::new(len);
                let slice = bytes.as_mut();
                unsafe { // we've checked that we have enough data to copy in the branch above
                    self.buffer.copy_nonoverlapping(slice);
                }
                Ok(bytes)
            }
        }
    }

    fn skip_varint(&mut self) -> Result<()> {
        if let Some::<&[u8; 10]>(arr) = self.buffer.try_limited_as_array() {
            for (&b, i) in arr.iter().zip(1..) {
                if b < 0x80 {
                    unsafe { self.buffer.advance(i); }
                    return Ok(());
                }
            }
            Err(Error::MalformedVarint)
        } else {
            for (&b, i) in unsafe { self.buffer.to_limit_as_slice() }.iter().zip(1..) {
                if b < 0x80 {
                    unsafe { self.buffer.advance(i); }
                    return Ok(());
                }
            }
            Err(io::Error::from(ErrorKind::UnexpectedEof).into())
        }
    }
    fn skip_bit32(&mut self) -> Result<()> {
        if self.buffer.to_limit_len() >= 4 {
            unsafe { self.buffer.advance(4); }
            Ok(())
        } else {
            Err(io::Error::from(ErrorKind::UnexpectedEof).into())
        }
    }
    fn skip_bit64(&mut self) -> Result<()> {
        if self.buffer.to_limit_len() >= 8 {
            unsafe { self.buffer.advance(8); }
            Ok(())
        } else {
            Err(io::Error::from(ErrorKind::UnexpectedEof).into())
        }
    }
    fn skip_length_delimited(&mut self) -> Result<()> {
        let len = self.read_varint32()? as i32;
        if len < 0 {
            Err(Error::NegativeSize)
        } else {
            let len = len as usize;
            if self.buffer.to_limit_len() >= len {
                unsafe { self.buffer.advance(len); }
                Ok(())
            } else {
                Err(io::Error::from(ErrorKind::UnexpectedEof).into())
            }
        }
    }

    fn as_any(&mut self) -> Any {
        Any {
            stream: None,
            buffer: &mut self.buffer,
            shared_state: &mut self.state
        }
    }

    fn reached_end(&self) -> bool {
        self.buffer.reached_end()
    }
}

unsafe impl Send for Slice<'_> { }
unsafe impl Sync for Slice<'_> { }

/// A type used for a [`CodedReader`] reading from a `Read` input. This input type buffers the stream's data.
/// 
/// [`CodedReader`]: struct.CodedReader.html
pub struct Stream<T> {
    input: T,
    buf: Box<[u8]>,
    buffer: Buffer,
    remaining_limit: i32,
    reached_eof: bool,
    state: SharedState,
}

impl<T: Read + Skip> Stream<T> {
    fn new(input: T, cap: usize) -> Self {
        let buf = vec![0; cap].into_boxed_slice();
        let buffer = Buffer::from_slice(&buf[0..0]);

        Stream {
            input,
            buf,
            buffer,
            remaining_limit: -1,
            reached_eof: false,
            state: Default::default(),
        }
    }
    fn into_inner(self) -> T {
        self.input
    }
    fn remaining_limit(&self) -> Option<i32> {
        self.buffer.remaining_limit().map(|i| i + self.remaining_limit)
    }
    fn try_refresh(&mut self) -> Result<bool> {
        let amnt = self.input.read(&mut self.buf)?;

        self.buffer = Buffer::from_slice(&self.buf[..amnt]);
        if self.remaining_limit >= 0 {
            self.remaining_limit = unsafe { self.buffer.apply_partial_limit(self.remaining_limit) };
        }

        let refreshed = amnt != 0;
        self.reached_eof = !refreshed;
        Ok(refreshed)
    }
    fn refresh(&mut self) -> Result<()> {
        self.try_refresh().and_then(|b| b.then_some(()).ok_or(io::Error::from(ErrorKind::UnexpectedEof).into()))
    }
    fn read_buffer_partial<'a>(&mut self, slice: &'a mut [u8]) -> Result<&'a mut [u8]> {
        // check if we reached the end of the buffer
        // does not check if we've reached end of limit
        if self.buffer.reached_end() {
            return Ok(slice);
        }

        let limit_len = self.buffer.to_limit_len();
        if self.remaining_limit == 0 {
            if limit_len < slice.len() {
                unsafe { self.buffer.advance(limit_len); }
                return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
            }

            unsafe {
                self.buffer.copy_nonoverlapping(slice);
            }

            Ok(&mut [])
        } else {
            let (f, s) = slice.split_at_mut(cmp::min(limit_len, slice.len()));

            if !f.is_empty() {
                unsafe {
                    self.buffer.copy_nonoverlapping(f);
                }
            }

            Ok(s)
        }
    }
    fn read_direct(&mut self, buf: &mut [u8]) -> Result<()> {
        if self.remaining_limit < 0 {
            self.input.read_exact(buf).map_err(Into::into)
        } else {
            let remaining_limit = self.remaining_limit as usize;
            if remaining_limit == 0 {
                Err(io::Error::from(ErrorKind::UnexpectedEof).into())
            } else if remaining_limit >= buf.len() {
                self.remaining_limit = i32::wrapping_sub(self.remaining_limit, buf.len() as i32);
                self.input.read_exact(buf).map_err(Into::into)
            } else {
                self.remaining_limit = 0;
                self.input.read_exact(&mut buf[..remaining_limit])?;
                Err(io::Error::from(ErrorKind::UnexpectedEof).into())
            }
        }
    }
    fn read_exact(&mut self, slice: &mut [u8]) -> Result<()> {
        if self.reached_end() {
            return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
        }

        let mut remaining_slice = self.read_buffer_partial(slice)?;
        if !remaining_slice.is_empty() {
            // if the remaining amnt to read is more than or equal to
            // the size of the buffer then we read direct from the stream
            // and adjust our remaining limit accordingly
            if remaining_slice.len() >= self.buf.len() {
                self.read_direct(remaining_slice)?;
            } else {
                loop {
                    self.refresh()?;
                    remaining_slice = self.read_buffer_partial(remaining_slice)?;

                    if remaining_slice.is_empty() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
    fn skip(&mut self, amnt: i32) -> Result<()> {
        let amnt_usize = amnt as usize;
        if self.reached_end() {
            return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
        }

        let limit_buf_len = self.buffer.to_limit_len();
        if limit_buf_len >= amnt_usize {
            unsafe { self.buffer.advance(amnt_usize); }
            Ok(())
        } else {
            unsafe { self.buffer.advance(limit_buf_len); }
            let remaining_amnt = amnt - limit_buf_len as i32;
            match self.remaining_limit.cmp(&0) {
                Ordering::Less => {
                    self.input.skip_exact(unsafe { Length::new_unchecked(remaining_amnt) }).map_err(Into::into)
                },
                Ordering::Equal => Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
                Ordering::Greater => {
                    let remaining_limit = self.remaining_limit;
                    let remaining_length = unsafe { Length::new_unchecked(remaining_limit) };
                    if remaining_limit > remaining_amnt {
                        self.remaining_limit = 0;
                        self.input.skip_exact(remaining_length).map_err(Into::into)
                    } else {
                        self.remaining_limit = i32::wrapping_sub(self.remaining_limit, remaining_amnt as i32);
                        self.input.skip_exact(remaining_length).map_err(Into::into)
                    }
                }
            }
        }
    }
    /// Attempts to refresh the buffer and return the next byte.
    /// If no buffer exists this tries to read the next byte.
    /// 
    /// This assumes that the limit hasn't been reached yet and
    /// is being used in conjunction with try_peek_byte which checks this and advance(1).
    fn try_read_byte(&mut self) -> Result<Option<u8>> {
        if self.reached_end() {
            return Ok(None);
        }

        if self.buffer.to_limit_len() != 0 {
            unsafe { Ok(Some(self.buffer.next_byte())) }
        } else if self.remaining_limit == 0 {
            Ok(None)
        } else if self.buf.len() != 0 {
            self.try_refresh()
                .map(|b| b.then(|| unsafe { self.buffer.next_byte() }))
                .map_err(Into::into)
        } else {
            let mut buf = [0u8; 1];
            let result = self.input.read(&mut buf)?;
            if result != 0 {
                if self.remaining_limit > 0 {
                    self.remaining_limit -= 1;
                }
                Ok(Some(buf[0]))
            } else {
                Ok(None)
            }
        }
    }
    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl<T: Read> Reader for Stream<T> {
    fn state(&self) -> &SharedState {
        &self.state
    }
    fn state_mut(&mut self) -> &mut SharedState {
        &mut self.state
    }

    fn push_limit(&mut self, limit: i32) -> io::Result<Option<i32>> {
        let old = match self.remaining_limit() {
            Some(remaining) => {
                // if we have some existing limit, check ahead of time to
                // make sure we don't extend behind the existing limit
                if remaining < limit {
                    return Err(ErrorKind::UnexpectedEof.into())
                }

                Some(remaining - limit)
            },
            None => None
        };
        self.remaining_limit = unsafe { self.buffer.apply_partial_limit(limit) };
        Ok(old)
    }
    fn pop_limit(&mut self, old: Option<i32>) {
        match old {
            Some(old) => {
                self.remaining_limit = unsafe { self.buffer.apply_partial_limit(old) };
            },
            None => {
                unsafe { self.buffer.remove_limit(); }
                self.remaining_limit = -1;
            },
        }
    }
    fn reached_limit(&self) -> bool {
        self.buffer.reached_limit() && self.remaining_limit == 0
    }

    #[inline]
    fn read_tag(&mut self) -> Result<Option<u32>> {
        let b = match self.try_read_byte()? {
            Some(b) if b < 0x80 => return Ok(Some(b as u32)),
            Some(b) => b,
            None => return Ok(None),
        };

        let mut result = (b & 0x7f) as u32;
        for i in 1..5 {
            let b = self.read_byte()?;
            result |= (b as u32 & 0x7f) << (7 * i);
            if b < 0x80 {
                return Ok(Some(result));
            }
        }
        for _ in 5..10 {
            let b = self.read_byte()?;
            if b < 0x80 {
                return Ok(Some(result));
            }
        }
        Err(Error::MalformedVarint)
    }
    fn read_varint32(&mut self) -> Result<u32> {
        let mut result = 0;
        for i in 0..5 {
            let b = self.read_byte()?;
            result |= (b as u32 & 0x7f) << (7 * i);
            if b < 0x80 {
                return Ok(result);
            }
        }
        for _ in 5..10 {
            let b = self.read_byte()?;
            if b < 0x80 {
                return Ok(result);
            }
        }
        Err(Error::MalformedVarint)
    }
    fn read_varint64(&mut self) -> Result<u64> {
        let mut result = 0;
        for i in 0..10 {
            let b = self.read_byte()?;
            result |= (b as u64 & 0x7f) << (7 * i);
            if b < 0x80 {
                return Ok(result);
            }
        }
        Err(Error::MalformedVarint)
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
        for _ in 0..10 {
            let b = self.read_byte()?;
            if b < 0x80 {
                return Ok(());
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
        match len.cmp(&0) {
            Ordering::Less => Err(Error::NegativeSize),
            Ordering::Equal => Ok(()),
            Ordering::Greater => self.skip(len)
        }
    }

    fn as_any(&mut self) -> Any {
        Any {
            stream: Some(internal::BorrowedStream {
                input: &mut self.input,
                buf: &mut self.buf,
                remaining_limit: &mut self.remaining_limit,
                reached_eof: &mut self.reached_eof,
            }),
            buffer: &mut self.buffer,
            shared_state: &mut self.state
        }
    }

    fn reached_end(&self) -> bool {
        self.buffer.reached_end() && self.reached_eof
    }
}

unsafe impl<T: Send> Send for Stream<T> { }
unsafe impl<T: Sync> Sync for Stream<T> { }

/// Handling options for unknown fields
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnknownFieldHandling {
    /// Stores unknown fields in a message's `UnknownFieldSet`
    Store,
    /// Skips unknown fields when they're encounted
    Skip,
}

impl Default for UnknownFieldHandling {
    fn default() -> Self {
        UnknownFieldHandling::Store
    }
}

impl UnknownFieldHandling {
    /// Returns whether the handling is set to skip unknown fields
    #[inline]
    pub fn skip(self) -> bool {
        self == UnknownFieldHandling::Skip
    }
}

#[derive(Clone, Debug)]
struct ReaderOptions {
    unknown_fields: UnknownFieldHandling,
    registry: Option<&'static ExtensionRegistry>,
    recursion_limit: usize,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        ReaderOptions {
            unknown_fields: UnknownFieldHandling::Store,
            registry: None,
            recursion_limit: 100,
        }
    }
}

/// A builder used to construct [`CodedReader`](struct.CodedReader.html) instances
#[derive(Clone, Debug, Default)]
pub struct Builder {
    options: ReaderOptions
}

impl Builder {
    /// Creates a new builder with the default configuration.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets unknown field handling for the reader. The default handling stores unknown fields.
    #[inline]
    pub fn unknown_fields(mut self, value: UnknownFieldHandling) -> Self {
        self.options.unknown_fields = value;
        self
    }
    /// Sets the registry extendable messages should use when being created. No registry is used by default.
    #[inline]
    pub fn registry(mut self, registry: Option<&'static ExtensionRegistry>) -> Self {
        self.options.registry = registry;
        self
    }
    /// Sets the recursion limit for a reader. The default limit is 100.
    #[inline]
    pub fn recursion_limit(mut self, limit: usize) -> Self {
        self.options.recursion_limit = limit;
        self
    }
    /// Constructs a [`CodedReader`](struct.CodedReader.html) using this builder and 
    /// the specified slice of bytes
    #[inline]
    pub fn with_slice<'a>(&self, inner: &'a [u8]) -> CodedReader<Slice<'a>> {
        CodedReader {
            inner: Slice::new(inner),
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
            options: self.options.clone()
        }
    }
}

/// A reader used by generated code to quickly parse field values without tag
/// wire type and field number checking.
/// 
/// This structure defers tag checking, making it faster to read fields when matching
/// on an existing raw field tag value.
#[must_use]
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
    pub fn and_then<R, F: FnOnce(&mut CodedReader<T>) -> Result<R>>(self, tag: Tag, f: F) -> Result<R> {
        debug_assert_eq!(self.tag, tag.get(), "Provided tag does not match read tag value");
        self.inner.set_last_tag(Some(tag));

        f(self.inner)
    }
    /// Reads a value from the input.
    ///
    /// This sets the last tag to be a tag made from the specified field number and the value's wire type.
    #[inline]
    pub fn read_value<V: Value>(self, field: FieldNumber) -> Result<V::Inner> {
        self.and_then(Tag::new(field, V::WIRE_TYPE), V::read_new)
    }
    /// Merges a value from the input with an existing value.
    /// 
    /// This sets the last tag to be a tag made from the specified field number and the value's wire type.
    #[inline]
    pub fn merge_value<V: Value>(self, field: FieldNumber, inner: &mut V::Inner) -> Result<()> {
        self.and_then(Tag::new(field, V::WIRE_TYPE), |input| input.merge_value::<V>(inner))
    }
    /// Adds entries to the specified collection.
    /// 
    /// This sets the last tag to be a tag made from the specified field number and the value's wire type.
    #[inline]
    pub fn add_entries_to<U: RepeatedValue<V>, V>(self, field: FieldNumber, value: &mut U) -> Result<()> {
        self.and_then(Tag::new(field, U::WIRE_TYPE), |input| input.add_entries_to::<U, V>(value))
    }

    /// Reads the field value using the specified function, checking if the tag is valid before running the function.
    #[inline]
    pub fn check_and_then<R, F: FnOnce(&'a mut CodedReader<T>) -> Result<R>>(self, f: F) -> Result<R> {
        let tag = Tag::try_from(self.tag).map_err(|_| Error::InvalidTag(self.tag))?;
        self.inner.set_last_tag(Some(tag));

        f(self.inner)
    }
    /// Checks if the tag is valid before attempting to add the field to the set
    #[inline]
    pub fn check_and_try_add_field_to<F: FieldSet>(self, set: &mut F) -> Result<TryRead<'a, T>> {
        self.check_and_then(|input| input.try_add_field_to::<F>(set))
    }
}

/// Represents a length delimited value that can be read in a specified format.
#[must_use]
pub struct Limit<'a, T: Input + 'a> {
    inner: &'a mut CodedReader<T>,
    old: Option<i32>,
}

impl<'a, T: Input + 'a> Limit<'a, T> {
    /// Reads a length delimited value using the specified function.
    pub fn then<R, F: FnOnce(&mut CodedReader<T>) -> Result<R>>(self, f: F) -> Result<R> {
        let result = f(self.inner)?;
        debug_assert!(self.inner.reached_limit());
        Ok(result)
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

/// A protobuf coded input reader that reads from a specified input.
pub struct CodedReader<T: Input> {
    inner: T,
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

    /// Returns the underlying stream value. This will discard any data that
    /// exists in the buffer.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
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

    /// Consumes the reader, returning the remaining slice
    pub fn into_inner(self) -> &'a [u8] {
        unsafe { self.inner.buffer.to_end_as_slice() }
    }
}

impl<T: Input> CodedReader<T> {
    fn increment_recursion_count(&mut self) -> Result<()> {
        let state = self.inner.state_mut();
        if state.recursion_depth == self.options.recursion_limit {
            Err(Error::RecursionLimitExceeded)
        } else {
            state.recursion_depth += 1;
            Ok(())
        }
    }
    fn decrement_recursion_count(&mut self) {
        self.inner.state_mut().recursion_depth -= 1;
    }
    #[inline]
    fn push_group(&mut self, field: FieldNumber) -> Option<Tag> {
        self.inner.state_mut().next_end_group.replace(Tag::new(field, WireType::EndGroup))
    }
    #[inline]
    fn pop_group(&mut self, old: Option<Tag>) {
        self.inner.state_mut().next_end_group = old;
    }
    fn set_last_tag(&mut self, tag: Option<Tag>) {
        self.inner.state_mut().last_tag = tag;
    }

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
        self.inner.state().last_tag
    }
    /// Returns a new CodedReader that can be used to temporarily 
    /// convert the reader into a non-generic reader over [`Any`] input.
    pub fn as_any(&mut self) -> CodedReader<Any> {
        CodedReader {
            inner: self.inner.as_any(),
            options: self.options.clone()
        }
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

    #[inline]
    fn read_raw_tag(&mut self) -> Result<Option<u32>> {
        let tag = self.inner.read_tag()?;
        let end_group = self.inner.state().next_end_group.map(Tag::get);
        if tag == end_group {
            Ok(None)
        } else {
            Ok(tag)
        }
    }

    /// Reads a field tag from the input
    pub fn read_tag(&mut self) -> Result<Option<Tag>> {
        let tag = 
            self.read_raw_tag()?
                .map(|v| Tag::try_from(v).map_err(|_| Error::InvalidTag(v)))
                .transpose()?;
        self.set_last_tag(tag);

        Ok(tag)
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
    /// Reads a 4-byte little endian value.
    pub fn read_bit32(&mut self) -> Result<u32> {
        self.inner.read_bit32()
    }
    /// Reads a 8-byte little endian value.
    pub fn read_bit64(&mut self) -> Result<u64> {
        self.inner.read_bit64()
    }
    /// Reads a length delimited string of bytes.
    pub fn read_length_delimited<B: ByteString>(&mut self) -> Result<B> {
        self.inner.read_length_delimited()
    }
    /// Reads a group, merging it's fields into the provided message instance.
    pub fn read_group<M: Message>(&mut self, value: &mut M) -> Result<()> {
        struct Guard<'a, T: Input + 'a> {
            inner: &'a mut CodedReader<T>,
            last_group: Option<Tag>,
        }
        impl<'a, T: Input + 'a> Drop for Guard<'a, T> {
            fn drop(&mut self) {
                self.inner.pop_group(self.last_group);
            }
        }

        if let Some(last_tag) = self.last_tag() {
            debug_assert!(last_tag.wire_type() == WireType::StartGroup, "attempted to read group from tag that wasn't a start group tag");
            let last_group = self.push_group(last_tag.field());
    
            let guard = Guard { inner: self, last_group };
            let result = value.merge_from(guard.inner);
            drop(guard);
    
            result
        } else {
            Ok(())
        }
    }
    /// Skips the last field read from the input
    pub fn skip(&mut self) -> Result<()> {
        if let Some(last_tag) = self.last_tag() {
            match last_tag.wire_type() {
                WireType::Varint => self.inner.skip_varint()?,
                WireType::Bit64 => self.inner.skip_bit64()?,
                WireType::LengthDelimited => self.inner.skip_length_delimited()?,
                WireType::StartGroup => {
                    self.recurse(|s| {
                        let end = Tag::new(last_tag.field(), WireType::EndGroup);
                        loop {
                            match s.read_tag()? {
                                Some(tag) if tag == end => break Ok(()),
                                Some(tag) if tag.wire_type() == WireType::EndGroup => return Err(Error::InvalidTag(tag.get())),
                                Some(_) => s.skip()?,
                                None => return Err(io::Error::from(ErrorKind::UnexpectedEof).into())
                            }
                        }
                    })?
                },
                WireType::EndGroup => { },
                WireType::Bit32 => self.inner.skip_bit32()?,
            }
        }

        Ok(())
    }

    /// Performs an operation, incrementing the recursion count beforehand.
    #[inline]
    pub fn recurse<R, F: FnOnce(&mut Self) -> Result<R>>(&mut self, f: F) -> Result<R> {
        struct Guard<'a, T: Input + 'a> {
            inner: &'a mut CodedReader<T>,
        }
        impl<'a, T: Input + 'a> Drop for Guard<'a, T> {
            fn drop(&mut self) {
                self.inner.decrement_recursion_count();
            }
        }

        self.increment_recursion_count()?;

        let guard = Guard { inner: self };
        let result = f(guard.inner);
        drop(guard);

        result
    }

    /// Reads a field value. This offloads checking of the tag's value, making it faster when reading
    /// many fields when the tag's underlying value already exists as a constant.
    #[inline]
    pub fn read_field<'a>(&'a mut self) -> Result<Option<FieldReader<'a, T>>> {
        self.inner.read_tag().map(move |t| t.map(move |t| FieldReader { inner: self, tag: t }))
    }
    /// Reads a new instance of the value from the reader.
    /// This is the inverse of `Value::read_new`.
    #[inline]
    pub fn read_value<V: Value>(&mut self) -> Result<V::Inner> {
        V::read_new(self)
    }
    /// Merges the reader's value with the value from the reader.
    /// This is the inverse of `Value::merge_from`.
    #[inline]
    pub fn merge_value<V: Value>(&mut self, value: &mut V::Inner) -> Result<()> {
        V::merge_from(value, self)
    }
    /// Adds field entries from the reader to the specified value.
    /// This is the inverse of `RepeatedValue::add_entries_from`.
    #[inline]
    pub fn add_entries_to<U: RepeatedValue<V>, V>(&mut self, value: &mut U) -> Result<()> {
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
    use crate::io::read::{Any, Input, Builder, CodedReader};
    use std::borrow::BorrowMut;

    pub trait ReaderInput<'a> {
        type Reader: Input + 'a;

        fn new(b: &'a [u8], builder: Builder) -> CodedReader<Self::Reader>;

        fn run<F: FnOnce(&mut CodedReader<Self::Reader>)>(b: &'a [u8], builder: Builder, f: F) {
            let mut reader = Self::new(b, builder);
            f(&mut reader)
        }
        fn run_any<F: FnOnce(&mut CodedReader<Any>)>(b: &'a [u8], builder: Builder, f: F) {
            let mut reader = Self::new(b, builder);
            let mut convert = reader.as_any();
            f(&mut convert)
        }
    }

    trait RunHelper<T: Input> {
        fn then<A: a::Action<T>>(self, a: A) -> Self;
    }

    impl<T: BorrowMut<CodedReader<U>>, U: Input> RunHelper<U> for T {
        fn then<A: a::Action<U>>(mut self, a: A) -> Self {
            a.run(self.borrow_mut());
            self
        }
    }

    mod actions {
        use std::fmt::Debug;
        use std::marker::PhantomData;
        use crate::io::{Tag, ByteString, read::{self, Input, CodedReader, Error}};

        pub trait Action<T: Input> {
            fn run(self, reader: &mut CodedReader<T>);
        }

        impl<T: Input, F: FnOnce(&mut CodedReader<T>)> Action<T> for F {
            fn run(self, reader: &mut CodedReader<T>) {
                self(reader)
            }
        }

        pub mod read_tag {
            use std::convert::TryFrom;
            use crate::io::{Tag, read::{Input, CodedReader}};
            use super::Action;

            pub struct ReadTag(Option<Tag>);

            impl<T: Input> Action<T> for ReadTag {
                fn run(self, reader: &mut CodedReader<T>) {
                    let r = reader.read_tag();
    
                    assert!(matches!(r, Ok(v) if v == self.0), "expected `{:?}`, got `{:?}`", self.0, r);
                    assert_eq!(self.0, reader.last_tag());
                }
            }

            pub fn none() -> ReadTag {
                ReadTag(None)
            }
            pub fn value(v: u32) -> ReadTag {
                ReadTag(Some(Tag::try_from(v).unwrap()))
            }
        }

        pub struct Pipe<T: Input, R: ReadAction<T>, A: AssertAction<R::Output>> {
            t: PhantomData<fn(T)>,
            r: R,
            a: A
        }
        impl<T: Input, R: ReadAction<T>, A: AssertAction<R::Output>> Action<T> for Pipe<T, R, A> {
            fn run(self, r: &mut CodedReader<T>) {
                self.a.assert(self.r.read(r))
            }
        }

        pub trait ReadAction<T: Input> {
            type Output;

            fn read(self, reader: &mut CodedReader<T>) -> Self::Output;

            fn with<A: AssertAction<Self::Output>>(self, a: A) -> Pipe<T, Self, A> where Self: Sized {
                Pipe { t: PhantomData, r: self, a }
            }
        }
        impl<T: Input, O, F: FnOnce(&mut CodedReader<T>) -> O> ReadAction<T> for F {
            type Output = O;

            fn read(self, reader: &mut CodedReader<T>) -> Self::Output {
                self(reader)
            }
        }

        pub fn try_read_tag<T: Input>(r: &mut CodedReader<T>) -> read::Result<Option<Tag>> { r.read_tag() }
        pub fn read_varint32<T: Input>(r: &mut CodedReader<T>) -> read::Result<u32> { r.read_varint32() }
        pub fn read_varint64<T: Input>(r: &mut CodedReader<T>) -> read::Result<u64> { r.read_varint64() }
        pub fn read_bit32<T: Input>(r: &mut CodedReader<T>) -> read::Result<u32> { r.read_bit32() }
        pub fn read_bit64<T: Input>(r: &mut CodedReader<T>) -> read::Result<u64> { r.read_bit64() }
        pub fn read_length_delimited<B: ByteString, T: Input>(r: &mut CodedReader<T>) -> read::Result<B> { r.read_length_delimited() }
        pub fn skip<T: Input>(r: &mut CodedReader<T>) -> read::Result<()> { r.skip() }
        pub fn read_limited<T: Input, R, F: FnOnce(&mut CodedReader<T>) -> read::Result<R>>(f: F) -> impl FnOnce(&mut CodedReader<T>) -> read::Result<R> {
            move |r| r.read_limit()?.then(f)
        }

        /// An assertion action that asserts some thing about a provided value
        pub trait AssertAction<V>: Sized {
            fn assert(self, value: V);
        }

        impl<V, F: FnOnce(V)> AssertAction<V> for F {
            fn assert(self, value: V) {
                self(value)
            }
        }

        pub fn value<V: PartialEq + Debug, E: Debug>(value: V) -> impl FnOnce(Result<V, E>) {
            move |v| assert!(matches!(&v, Ok(v) if v == &value), "expected `{:?}`, got `{:?}`", value, v)
        }
        pub fn io_error<T: Debug>(r: Result<T, Error>) {
            assert!(matches!(r, Err(Error::IoError(_))), "expected `{:?}`, got `{:?}`", "Error::IoError(_)", r)
        }
        pub fn malformed_varint<T: Debug>(r: Result<T, Error>) {
            assert!(matches!(r, Err(Error::MalformedVarint)), "expected `{:?}`, got `{:?}`", Err::<T, _>(Error::MalformedVarint), r)
        }
        pub fn invalid_tag<T: Debug>(tag: u32) -> impl FnOnce(Result<T, Error>) {
            move |r| assert!(matches!(r, Err(Error::InvalidTag(t)) if t == tag))
        }
        pub fn negative_size<T: Debug>(r: Result<T, Error>) {
            assert!(matches!(r, Err(Error::NegativeSize)), "expected `{:?}`, got `{:?}`", Err::<T, _>(Error::NegativeSize), r)
        }
    }

    use actions as a;

    use a::ReadAction;

    macro_rules! test {
        ($(($ti:ident | $tia:ident $(| init: || $init:expr)?) = $x:expr => |$f:ident| $t:block),+) => {
            $(
                pub fn $ti<T: for<'a> ReaderInput<'a>>() {
                    static INPUT: &'static [u8] = &$x;

                    let builder =
                        None
                        $(.or_else(|| Some($init)))?
                        .unwrap_or_else(Builder::new);

                    T::run(INPUT, builder, |$f| $t);
                }

                pub fn $tia<T: for<'a> ReaderInput<'a>>() {
                    static INPUT: &'static [u8] = &$x;

                    let builder =
                        None
                        $(.or_else(|| Some($init)))?
                        .unwrap_or_else(Builder::new);

                    T::run_any(INPUT, builder, |$f| $t);
                }
            )+
        };
    }

    test! {
        (read_no_tag_from_empty | read_no_tag_from_empty_any) = [] => |r| {
            r.then(a::read_tag::none());
        },
        (read_tag | read_tag_any) = [8] => |r| {
            r.then(a::read_tag::value(8))
             .then(a::read_tag::none());
        },
        (read_2byte_tag | read_2byte_tag_any) = [128, 1] => |r| {
            r.then(a::read_tag::value(0x80))
             .then(a::read_tag::none());
        },
        (read_5byte_tag | read_5byte_tag_any) = [128, 128, 128, 128, 1] => |r| {
            r.then(a::read_tag::value(0x10000000))
             .then(a::read_tag::none());
        },
        (read_10byte_tag | read_10byte_tag_any) = [128, 128, 128, 128, 129, 128, 128, 128, 128, 0] => |r| {
            r.then(a::read_tag::value(0x10000000))
             .then(a::read_tag::none());
        },
        (read_truncated_tag | read_truncated_tag_any) = [128] => |r| {
            r.then(a::try_read_tag.with(a::io_error));
        },
        (read_truncated_9byte_tag | read_truncated_9byte_tag_any) = [128u8; 9] => |r| {
            r.then(a::try_read_tag.with(a::io_error));
        },
        (read_malformed_tag | read_malformed_tag_any) = [128u8; 10] => |r| {
            r.then(a::try_read_tag.with(a::malformed_varint));
        },
        (read_truncated_varint32_empty | read_truncated_varint32_empty_any) = [] => |r| {
            r.then(a::read_varint32.with(a::io_error));
        },
        (read_truncated_varint32_5byte | read_truncated_varint32_5byte_any) = [128u8; 5] => |r| {
            r.then(a::read_varint32.with(a::io_error));
        },
        (read_truncated_varint32_9byte | read_truncated_varint32_9byte_any) = [128u8; 9] => |r| {
            r.then(a::read_varint32.with(a::io_error));
        },
        (read_malformed_varint32 | read_malformed_varint32_any) = [128u8; 10] => |r| {
            r.then(a::read_varint32.with(a::malformed_varint));
        },
        (read_varint32 | read_varint32_any) = [1] => |r| {
            r.then(a::read_varint32.with(a::value(1)))
             .then(a::read_tag::none());
        },
        (read_varint32_5byte | read_varint32_5byte_any) = [128, 128, 128, 128, 1] => |r| {
            r.then(a::read_varint32.with(a::value(0x10000000)))
             .then(a::read_tag::none());
        },
        (read_varint32_10byte | read_varint32_10byte_any) = [128, 128, 128, 128, 128, 128, 128, 128, 128, 1] => |r| {
            r.then(a::read_varint32.with(a::value(0))) // discard all top bits
             .then(a::read_tag::none());
        },
        (read_truncated_varint64_empty | read_truncated_varint64_empty_any) = [] => |r| {
            r.then(a::read_varint64.with(a::io_error));
        },
        (read_truncated_varint64_9byte | read_truncated_varint64_9byte_any) = [128u8; 9] => |r| {
            r.then(a::read_varint64.with(a::io_error));
        },
        (read_malformed_varint64 | read_malformed_varint64_any) = [128u8; 10] => |r| {
            r.then(a::read_varint64.with(a::malformed_varint));
        },
        (read_varint64 | read_varint64_any) = [1] => |r| {
            r.then(a::read_varint64.with(a::value(1)))
             .then(a::read_tag::none());
        },
        (read_varint64_10byte | read_varint64_10byte_any) = [128, 128, 128, 128, 128, 128, 128, 128, 128, 1] => |r| {
            r.then(a::read_varint64.with(a::value(0x8000000000000000)))
             .then(a::read_tag::none());
        },
        (read_truncated_bit32 | read_truncated_bit32_any) = [] => |r| {
            r.then(a::read_bit32.with(a::io_error));
        },
        (read_truncated_bit32_3byte | read_truncated_bit32_3byte_any) = [0u8; 3] => |r| {
            r.then(a::read_bit32.with(a::io_error));
        },
        (read_bit32 | read_bit32_any) = [0x78, 0x56, 0x34, 0x12] => |r| {
            r.then(a::read_bit32.with(a::value(0x12345678u32)))
             .then(a::read_tag::none());
        },
        (read_truncated_bit64 | read_truncated_bit64_any) = [] => |r| {
            r.then(a::read_bit64.with(a::io_error));
        },
        (read_truncated_bit64_7byte | read_truncated_bit64_7byte_any) = [0u8; 7] => |r| {
            r.then(a::read_bit64.with(a::io_error));
        },
        (read_bit64 | read_bit64_any) = [0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12] => |r| {
            r.then(a::read_bit64.with(a::value(0x1234567890ABCDEFu64)))
             .then(a::read_tag::none());
        },
        (read_length_delimited | read_length_delimited_any) = 
            [12, b'H', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', b'!'] 
                => |r| {
            r.then(a::read_length_delimited::<Vec<u8>, _>
                .with(a::value(b"Hello world!".as_ref().to_owned())))
             .then(a::read_tag::none());
        },
        (read_length_delimited_truncated | read_length_delimited_truncated_any) = [12] => |r| {
            r.then(a::read_length_delimited::<Vec<u8>, _>.with(a::io_error));
        },
        (read_length_delimited_byte_truncated | read_length_delimited_byte_truncated_any) =
            [12, b'H', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd']
                => |r| {
            r.then(a::read_length_delimited::<Vec<u8>, _>.with(a::io_error));
        },
        (skip_varint | skip_varint_any) = [8, 128, 128, 128, 128, 128, 128, 128, 128, 128, 0] => |r| {
            r.then(a::read_tag::value(8))
             .then(a::skip.with(a::value(())))
             .then(a::read_tag::none());
        },
        (skip_varint_truncated | skip_varint_truncated_any) = [8] => |r| {
            r.then(a::read_tag::value(8))
             .then(a::skip.with(a::io_error));
        },
        (skip_varint_9byte_truncated | skip_varint_9byte_truncated_any) = [8, 128, 128, 128, 128, 128, 128, 128, 128, 128] => |r| {
            r.then(a::read_tag::value(8))
             .then(a::skip.with(a::io_error));
        },
        (skip_varint_malformed | skip_varint_malformed_any) = [8, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128] => |r| {
            r.then(a::read_tag::value(8))
             .then(a::skip.with(a::malformed_varint));
        },
        (skip_bit32 | skip_bit32_any) = [13, 0, 0, 0, 0] => |r| {
            r.then(a::read_tag::value(13))
             .then(a::skip.with(a::value(())))
             .then(a::read_tag::none());
        },
        (skip_bit32_truncated | skip_bit32_truncated_any) = [13] => |r| {
            r.then(a::read_tag::value(13))
             .then(a::skip.with(a::io_error));
        },
        (skip_bit32_3byte_truncated | skip_bit32_3byte_truncated_any) = [13, 0, 0, 0] => |r| {
            r.then(a::read_tag::value(13))
             .then(a::skip.with(a::io_error));
        },
        (skip_bit64 | skip_bit64_any) = [9, 0, 0, 0, 0, 0, 0, 0, 0] => |r| {
            r.then(a::read_tag::value(9))
             .then(a::skip.with(a::value(())))
             .then(a::read_tag::none());
        },
        (skip_bit64_truncated | skip_bit64_truncated_any) = [9] => |r| {
            r.then(a::read_tag::value(9))
             .then(a::skip.with(a::io_error));
        },
        (skip_bit64_7byte_truncated | skip_bit64_7byte_truncated_any) = [9, 0, 0, 0, 0, 0, 0, 0] => |r| {
            r.then(a::read_tag::value(9))
             .then(a::skip.with(a::io_error));
        },
        (skip_length_delimited | skip_length_delimited_any) = [10, 2, 0, 0] => |r| {
            r.then(a::read_tag::value(10))
             .then(a::skip.with(a::value(())));
        },
        (skip_length_delimited_truncated | skip_length_delimited_truncated_any) = [10, 2] => |r| {
            r.then(a::read_tag::value(10))
             .then(a::skip.with(a::io_error));
        },
        (skip_length_delimited_truncated_byte | skip_length_delimited_truncated_byte_any) = [10, 2, 0] => |r| {
            r.then(a::read_tag::value(10))
             .then(a::skip.with(a::io_error));
        },
        (skip_group | skip_group_any) = [11, 16, 0, 12] => |r| {
            r.then(a::read_tag::value(11))
             .then(a::skip.with(a::value(())))
             .then(a::read_tag::none());
        },
        // throw an end tag for field 2 in the middle of the field 1 group
        (skip_group_other_field_end | skip_group_other_field_end_any) = [11, 20, 12] => |r| {
            r.then(a::read_tag::value(11))
             .then(a::skip.with(a::invalid_tag(20)));
        },
        (read_delimited_varint_field | read_delimited_varint_field_any) = [10, 2, 10, 1] => |r| {
            r.then(a::read_tag::value(10))
            .then(a::read_limited(|r| {
                r.then(a::read_tag::value(10))
                 .then(a::read_varint32.with(a::value(1)))
                 .then(a::read_tag::none());
                Ok(())
              }).with(a::value(())))
             .then(a::read_tag::none());
        },
        // this should throw an error at *some point*, streams can't check ahead of time but flat inputs can
        (read_truncated_delimited_field | read_truncated_delimited_field_any) = [10, 2, 10] => |r| {
            r.then(a::read_tag::value(10))
             .then(a::read_limited(|r| {
                r.then(a::read_tag::value(10));
                r.read_varint32()
              }).with(a::io_error));
        },
        (read_negative_delimited_field | read_negative_delimited_field_any) = [10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1] => |r| {
            r.then(a::read_tag::value(10))
             .then(a::read_limited(|_| Ok(())).with(a::negative_size));
        },
        (read_nested_delimited_field | read_nested_delimited_field_any) = [10, 8, 8, 1, 18, 2, 8, 1, 16, 2] => |r| {
            r.then(a::read_tag::value(10))
             .then(a::read_limited(|r| {
                r.then(a::read_tag::value(8))
                 .then(a::read_varint32.with(a::value(1)))
                 .then(a::read_tag::value(18))
                 .then(a::read_limited(|r| {
                    r.then(a::read_tag::value(8))
                     .then(a::read_varint32.with(a::value(1)));
                    Ok(())
                 }).with(a::value(())))
                 .then(a::read_tag::value(16))
                 .then(a::read_varint32.with(a::value(2)));
                 Ok(())
             }).with(a::value(())));
        }
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
                    crate::io::read::test::$t::<$f>();
                }
            )*
        };
    }

    macro_rules! run_suite {
        ($f:ty) => {
            run! {
                $f => {
                    read_no_tag_from_empty, read_no_tag_from_empty_any,
                    read_tag, read_tag_any,
                    read_2byte_tag, read_2byte_tag_any,
                    read_5byte_tag, read_5byte_tag_any,
                    read_10byte_tag, read_10byte_tag_any,
                    read_truncated_tag, read_truncated_tag_any,
                    read_truncated_9byte_tag, read_truncated_9byte_tag_any,
                    read_malformed_tag, read_malformed_tag_any,
                    read_truncated_varint32_empty, read_truncated_varint32_empty_any,
                    read_truncated_varint32_5byte, read_truncated_varint32_5byte_any,
                    read_truncated_varint32_9byte, read_truncated_varint32_9byte_any,
                    read_malformed_varint32, read_malformed_varint32_any,
                    read_varint32, read_varint32_any,
                    read_varint32_5byte, read_varint32_5byte_any,
                    read_varint32_10byte, read_varint32_10byte_any,
                    read_truncated_varint64_empty, read_truncated_varint64_empty_any,
                    read_truncated_varint64_9byte, read_truncated_varint64_9byte_any,
                    read_malformed_varint64, read_malformed_varint64_any,
                    read_varint64, read_varint64_any,
                    read_varint64_10byte, read_varint64_10byte_any,
                    read_truncated_bit32, read_truncated_bit32_any,
                    read_truncated_bit32_3byte, read_truncated_bit32_3byte_any,
                    read_bit32, read_bit32_any,
                    read_truncated_bit64, read_truncated_bit64_any,
                    read_truncated_bit64_7byte, read_truncated_bit64_7byte_any,
                    read_bit64, read_bit64_any,
                    read_length_delimited, read_length_delimited_any,
                    read_length_delimited_truncated, read_length_delimited_truncated_any,
                    read_length_delimited_byte_truncated, read_length_delimited_byte_truncated_any,
                    skip_varint, skip_varint_any,
                    skip_varint_truncated, skip_varint_truncated_any,
                    skip_varint_9byte_truncated, skip_varint_9byte_truncated_any,
                    skip_varint_malformed, skip_varint_malformed_any,
                    skip_bit32, skip_bit32_any,
                    skip_bit32_truncated, skip_bit32_truncated_any,
                    skip_bit32_3byte_truncated, skip_bit32_3byte_truncated_any,
                    skip_bit64, skip_bit64_any,
                    skip_bit64_truncated, skip_bit64_truncated_any,
                    skip_bit64_7byte_truncated, skip_bit64_7byte_truncated_any,
                    skip_length_delimited, skip_length_delimited_any,
                    skip_length_delimited_truncated, skip_length_delimited_truncated_any,
                    skip_length_delimited_truncated_byte, skip_length_delimited_truncated_byte_any,
                    skip_group, skip_group_any,
                    skip_group_other_field_end, skip_group_other_field_end_any,
                    read_delimited_varint_field, read_delimited_varint_field_any,
                    read_truncated_delimited_field, read_truncated_delimited_field_any,
                    read_negative_delimited_field, read_negative_delimited_field_any,
                    read_nested_delimited_field, read_nested_delimited_field_any
                }
            }
        };
    }

    mod suites {
        mod slice {
            use crate::io::read::{Slice, Builder, CodedReader, test::ReaderInput};

            pub struct SliceInput;
            impl<'a> ReaderInput<'a> for SliceInput {
                type Reader = Slice<'a>;

                fn new(b: &'a [u8], build: Builder) -> CodedReader<Self::Reader> {
                    build.with_slice(b)
                }
            }

            run_suite!(SliceInput);
        }

        mod stream {
            macro_rules! stream_case {
                ($i:ident($s:expr)) => {
                    use crate::io::read::{CodedReader, Builder, Stream, test::ReaderInput};

                    pub struct $i;

                    impl<'a> ReaderInput<'a> for $i {
                        type Reader = Stream<&'a [u8]>;

                        fn new(b: &'a [u8], build: Builder) -> CodedReader<Self::Reader> {
                            build.with_capacity($s, b)
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