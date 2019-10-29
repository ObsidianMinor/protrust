//! An abstraction around input and output types that allows the lib to work in `no-std` scenarios.

use core::cmp;
use core::fmt::{self, Display, Formatter};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::error;

/// An error type returned when an error occurs while reading from or writing to a stream trait.
/// 
/// Encountering this error likely means the stream is invalidated and shouldn't continue to be used.
/// It also does not communicate the underlying source of the error and implementors of Read should use
/// some external way of communicating the underlying error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "stream error")
    }
}

#[cfg(feature = "std")]
impl error::Error for Error { }

/// The result of reading or writing to a Read or Write instance
pub type Result<T> = core::result::Result<T, Error>;

/// A trait for reading bytes from a source.
/// 
/// Like the std::io::Read trait, implementors of this trait are called 'readers'.
pub trait Read {
    /// Reads from the input into the specified buffer, returning the number of bytes read. 
    /// The input buffer is not guaranteed to be zeroed.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Skips a certain number of bytes from the input. If instance cannot skip the specified length,
    /// this should return an [`Error`](struct.Error.html)
    fn skip(&mut self, mut len: usize) -> Result<()> {
        const BUF_SIZE: usize = 2 * 1024;

        let mut buf = [0; BUF_SIZE];
        while len != 0 {
            let amnt = cmp::min(len, buf.len());
            if let Ok(read_len) = self.read(&mut buf[..amnt]) {
                len -= read_len;
            } else {
                return Err(Error);
            }
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized + std::io::Read> Read for T {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe { self.initializer().initialize(buf); }
        loop {
            match self.read(buf) {
                Ok(value) => return Ok(value),
                Err(ref err) if err.kind() == std::io::ErrorKind::Interrupted => { },
                Err(_) => return Err(Error)
            }
        }
    }
}

#[cfg(not(feature = "std"))]
impl<'a> Read for &'a [u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // like std, this is here to avoid the copy_from_slice memcpy overhead
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }
    fn skip(&mut self, len: usize) -> Result<()> {
        if len > self.len() {
            return Err(Error);
        }

        *self = &self[len..];
        Ok(())
    }
}

/// A trait for writing bytes to a destination.
/// 
/// Like the std::io::Write trait, implementors of this trait are called 'writers'.
pub trait Write {
    /// Writes all data in the buffer to the output
    fn write(&mut self, buf: &[u8]) -> Result<()>;
}

#[cfg(feature = "std")]
impl<T: std::io::Write> Write for T {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.write_all(buf).map_err(|_| Error)
    }
}

#[cfg(not(feature = "std"))]
impl<'a> Write for &'a mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        if buf.len() <= self.len() {
            let (a, b) = core::mem::replace(self, &mut []).split_at_mut(buf.len());
            a.copy_from_slice(buf);
            *self = b;
            Ok(())
        } else {
            Err(Error)
        }
    }
}

#[cfg(not(feature = "std"))]
impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.reserve(buf.len());
        let old_len = self.len();
        unsafe {
            self.set_len(old_len + buf.len());
        }
        self[old_len..].copy_from_slice(buf);
        Ok(())
    }
}