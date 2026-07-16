//! Owned and borrowed Unix-like file descriptors.

use core::fmt;

pub use io_core::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use io_core::sys::{AsInner, FromInner, IntoInner};

mod owned;
pub use owned::{AsFd, BorrowedFd, OwnedFd};

use crate::{
    io,
    sys::{
        self,
        io::{sceIoLseek, sceIoRead, sceIoWrite, Whence},
    },
};

/// A high-level abstraction on PSP file descriptor to create a `File` abstraction
#[derive(Debug)]
pub(crate) struct FileDesc(OwnedFd);

impl FileDesc {
    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        if sys::is_interrupt_enabled() {
            let res = unsafe { sceIoRead(self.0.as_file_id(), buf.as_mut_ptr().cast(), buf.len()) };
            res.into_result().map_err(Into::into)
        } else {
            Err(io::Error::from(sys::SceError::IO))
        }
    }

    #[inline]
    pub fn read_buf(&self, mut cursor: io::BorrowedCursor<'_>) -> io::Result<()> {
        if sys::is_interrupt_enabled() {
            let res = unsafe {
                sceIoRead(
                    self.0.as_file_id(),
                    cursor.as_mut().as_mut_ptr().cast(),
                    cursor.capacity(),
                )
            };

            let ret = res.into_result()?;

            // SAFETY: `ret` bytes were written to the initialized portion of the buffer
            unsafe {
                cursor.advance(ret);
            }

            Ok(())
        } else {
            Err(io::Error::from(sys::SceError::IO))
        }
    }

    #[inline]
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        if sys::is_interrupt_enabled() {
            unsafe {
                sceIoWrite(self.0.as_file_id(), buf.as_ptr().cast(), buf.len())
                    .into_result()
                    .map_err(Into::into)
            }
        } else {
            Err(io::Error::from(sys::SceError::IO))
        }
    }

    #[inline]
    pub fn write_all(&self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(io_core::io_const_error!(
                        io::ErrorKind::WriteZero,
                        "failed to write whole buffer"
                    ));
                },
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {},
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    #[inline]
    pub fn seek(&self, pos: io::SeekFrom) -> io::Result<u64> {
        if sys::is_interrupt_enabled() {
            let (whence, pos) = match pos {
                // Casting to `i64` is fine, too large values will end up as
                // negative which will cause an error in `lseek`.
                io::SeekFrom::Start(off) => (Whence::Start, off as i64),
                io::SeekFrom::End(off) => (Whence::End, off),
                io::SeekFrom::Current(off) => (Whence::Current, off),
            };
            sceIoLseek(self.0.as_file_id(), pos, whence)
                .into_result()
                .map_err(Into::into)
        } else {
            Err(io::Error::from(sys::SceError::IO))
        }
    }

    #[inline]
    #[doc(alias = "tell")]
    pub fn stream_position(&self) -> io::Result<u64> {
        self.seek(io::SeekFrom::Current(0))
    }
}

impl io::Read for &FileDesc {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: io::BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }
}

impl io::Write for &FileDesc {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        (**self).write(data)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }
}

impl fmt::Write for &FileDesc {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl io::Seek for &FileDesc {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn stream_position(&mut self) -> io_core::io::Result<u64> {
        (**self).stream_position()
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(unsafe { FromRawFd::from_raw_fd(raw_fd) })
    }
}
