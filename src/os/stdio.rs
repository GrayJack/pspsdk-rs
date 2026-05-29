use core::mem::ManuallyDrop;

use crate::{
    io,
    os::fd::{FileDesc, FromRawFd},
    sys::io::{sceKernelStderr, sceKernelStdin, sceKernelStdout},
};

pub const STDIN_BUF_SIZE: usize = 256;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe { ManuallyDrop::new(FileDesc::from_raw_fd(sceKernelStdin().to_inner())).read(buf) }
    }

    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        unsafe {
            ManuallyDrop::new(FileDesc::from_raw_fd(sceKernelStdin().to_inner())).read_buf(buf)
        }
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe { ManuallyDrop::new(FileDesc::from_raw_fd(sceKernelStdout().to_inner())).write(buf) }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe { ManuallyDrop::new(FileDesc::from_raw_fd(sceKernelStderr().to_inner())).write(buf) }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
