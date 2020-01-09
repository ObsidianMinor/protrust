//! An abstraction around input and output types that allows the lib to work in `no-std` scenarios.

#[cfg(not(feature = "std"))]
use core::cmp;
use core::fmt::{self, Display, Formatter};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::error;

/// An error type returned when an error occurs while reading from or writing to a stream trait.
/// 
/// Encountering this error likely means the stream is invalidated and shouldn't continue to be used.
/// It also does not communicate the underlying source of the error and implementors of Read or Write should use
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

    /// Reads from the input into the specified buffer until the buffer is filled.
    /// The input buffer is not guaranteed to be zeroed.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;

    /// Skips a certain number of bytes from the input. If instance cannot skip the specified length,
    /// this should return an [`Error`](struct.Error.html)
    fn skip(&mut self, len: usize) -> Result<()>;
}

#[cfg(not(feature = "std"))]
impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_exact(buf)
    }
    #[inline]
    fn skip(&mut self, len: usize) -> Result<()> {
        (**self).skip(len)
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
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        unsafe { self.initializer().initialize(buf); }
        loop {
            match self.read(buf) {
                Ok(0) if buf.is_empty() => break Err(Error),
                Ok(0) => break Ok(()),
                Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => { },
                Err(_) => break Err(Error),
            }
        }
    }
    fn skip(&mut self, len: usize) -> Result<()> {
        let mut by_ref = self;
        let mut take = <&mut T as std::io::Read>::take(&mut by_ref, len as u64);
        let mut sink = std::io::sink();
        std::io::copy(&mut take, &mut sink).map_err(|_| Error)?;
        Ok(())
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
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        match self.get(..buf.len()) {
            Some(input) => {
                buf.copy_from_slice(input);
                Ok(())
            },
            None => Err(Error)
        }
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

#[cfg(test)]
#[cfg(not(feature = "std"))]
mod test {
    use crate::io::stream::{Write, Read, Error};

    #[test]
    fn read_all() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut buf = [0u8; 10];

        let mut read: &[u8] = &data;
        let result = read.read(&mut buf);

        assert_eq!(result, Ok(10));
        assert_eq!(read, &[]);
        assert_eq!(data, buf);
    }
    #[test]
    fn read_less() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut buf = [0u8; 5];

        let mut read: &[u8] = &data;
        let result = read.read(&mut buf);

        assert_eq!(result, Ok(5));
        assert_eq!(read, &data[5..10]);
        assert_eq!(&data[0..5], &buf);
    }
    #[test]
    fn read_more() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut buf = [0u8; 11];

        let mut read: &[u8] = &data;
        let result = read.read(&mut buf);

        assert_eq!(result, Ok(10));
        assert_eq!(read, &[]);
        assert_eq!(data, &buf[0..10]);
    }
    #[test]
    fn read_none() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.read(&mut []);

        assert_eq!(result, Ok(0));
        assert_eq!(read, &data);
    }
    #[test]
    fn skip_all() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(10);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &[]);
    }
    #[test]
    fn skip_less() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(5);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &data[5..10]);

        let result = read.skip(5);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &[]);
    }
    #[test]
    fn skip_more() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(11);

        assert_eq!(result, Err(Error));
    }
    #[test]
    fn skip_none() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(0);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &data);
    }

    #[test]
    fn write_slice_all() {
        let mut buf = [0u8; 10];
        let data = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(write, &mut []);
        assert_eq!(&buf, data);
    }
    #[test]
    fn write_slice_less() {
        let mut buf = [0u8; 10];
        let data = &[1, 2, 3, 4, 5];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(write.len(), 5);
        assert_eq!(&buf[0..5], data);
    }
    #[test]
    fn write_slice_more() {
        let mut buf = [0u8; 10];
        let data = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Err(Error));
    }
    #[test]
    fn write_slice_none() {
        let mut buf = [0u8; 10];
        let data = &[];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(&*write, &[0u8; 10]);
    }

    #[test]
    fn write_vec() {
        let mut vec = alloc::vec::Vec::new();
        let data = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let result = vec.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(vec.len(), 10);
        assert_eq!(vec.as_slice(), data);

        let result = vec.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(vec.len(), 20);
        assert_eq!(&vec[0..10], data);
        assert_eq!(&vec[10..20], data);
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test {
    use crate::io::stream::{Write, Read, Error};

    #[test]
    fn read_all() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut buf = [0u8; 10];

        let mut read: &[u8] = &data;
        let result = read.read(&mut buf);

        assert_eq!(result, Ok(10));
        assert_eq!(read, &[]);
        assert_eq!(data, buf);
    }
    #[test]
    fn read_less() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut buf = [0u8; 5];

        let mut read: &[u8] = &data;
        let result = read.read(&mut buf);

        assert_eq!(result, Ok(5));
        assert_eq!(read, &data[5..10]);
        assert_eq!(&data[0..5], &buf);
    }
    #[test]
    fn read_more() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut buf = [0u8; 11];

        let mut read: &[u8] = &data;
        let result = read.read(&mut buf);

        assert_eq!(result, Ok(10));
        assert_eq!(read, &[]);
        assert_eq!(data, &buf[0..10]);
    }
    #[test]
    fn read_none() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.read(&mut []);

        assert_eq!(result, Ok(0));
        assert_eq!(read, &data);
    }
    #[test]
    fn skip_all() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(10);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &[]);
    }
    #[test]
    fn skip_less() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(5);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &data[5..10]);

        let result = read.skip(5);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &[]);
    }
    #[test]
    fn skip_more() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(11);

        assert_eq!(result, Err(Error));
    }
    #[test]
    fn skip_none() {
        let data: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut read: &[u8] = &data;
        let result = read.skip(0);

        assert_eq!(result, Ok(()));
        assert_eq!(read, &data);
    }

    #[test]
    fn write_all() {
        let mut buf = [0u8; 10];
        let data = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(write, &mut []);
        assert_eq!(&buf, data);
    }
    #[test]
    fn write_less() {
        let mut buf = [0u8; 10];
        let data = &[1, 2, 3, 4, 5];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(write.len(), 5);
        assert_eq!(&buf[0..5], data);
    }
    #[test]
    fn write_more() {
        let mut buf = [0u8; 10];
        let data = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Err(Error));
    }
    #[test]
    fn write_none() {
        let mut buf = [0u8; 10];
        let data = &[];

        let mut write: &mut [u8] = &mut buf;
        let result = write.write(data);

        assert_eq!(result, Ok(()));
        assert_eq!(&*write, &[0u8; 10]);
    }
}