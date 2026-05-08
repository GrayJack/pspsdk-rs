use core::fmt;

use alloc::{string::String, vec::Vec};

use crate::{
    io::{self, Read, Write},
    os,
    sys::SceError,
};

/// A handle to a raw instance of the standard input stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `pspsdk::io::stdio::stdin_raw` function.
pub struct StdinRaw(os::stdio::Stdin);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `pspsdk::io::stdio::stdout_raw` function.
pub struct StdoutRaw(os::stdio::Stdout);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `pspsdk::io::stdio::stderr_raw` function.
pub struct StderrRaw(os::stdio::Stderr);

/// Constructs a new raw handle to the standard input of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdin`. Data buffered by the `std::io::stdin`
/// handles is **not** available to raw handles returned from this function.
///
/// The returned handle has no external synchronization or buffering.
pub const fn stdin_raw() -> StdinRaw {
    StdinRaw(os::stdio::Stdin::new())
}

/// Constructs a new raw handle to the standard output stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdout`. Note that data is buffered by the
/// `std::io::stdout` handles so writes which happen via this raw handle may
/// appear before previous writes.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
pub const fn stdout_raw() -> StdoutRaw {
    StdoutRaw(os::stdio::Stdout::new())
}

/// Constructs a new raw handle to the standard error stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stderr`.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
pub const fn stderr_raw() -> StderrRaw {
    StderrRaw(os::stdio::Stderr::new())
}

impl Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        handle_ebadf(self.0.read(buf), || Ok(0))
    }

    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        handle_ebadf(self.0.read_buf(buf), || Ok(()))
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        handle_ebadf(self.0.read_vectored(bufs), || Ok(0))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        handle_ebadf(self.0.read_exact(buf), || {
            Err(io_core::io_const_error!(
                io::ErrorKind::UnexpectedEof,
                "failed to fill whole buffer"
            ))
        })
    }

    fn read_buf_exact(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        if buf.capacity() == 0 {
            return Ok(());
        }
        handle_ebadf(self.0.read_buf_exact(buf), || {
            Err(io_core::io_const_error!(
                io::ErrorKind::UnexpectedEof,
                "failed to fill whole buffer"
            ))
        })
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_end(buf), || Ok(0))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_string(buf), || Ok(0))
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

impl Write for StderrRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

fn handle_ebadf<T>(r: io::Result<T>, default: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    match r {
        Err(ref e) if is_ebadf(e) => default(),
        r => r,
    }
}

pub fn is_ebadf(err: &io::Error) -> bool {
    let raw = err.raw_os_error();
    raw == Some(SceError::BAD_FILE.as_inner() as i32)
        || raw == Some(SceError::KERNEL_BADF.as_inner() as i32)
}
