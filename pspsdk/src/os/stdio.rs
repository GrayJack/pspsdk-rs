use core::mem::ManuallyDrop;

use crate::{
    io,
    os::fd::{FileDesc, FromRawFd},
    sys::{
        self,
        io::{sceKernelStderr, sceKernelStdin, sceKernelStdout},
    },
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

    fn read_buf(&mut self, buf: io::BorrowedCursor<'_, u8>) -> io::Result<()> {
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
        let res = unsafe {
            ManuallyDrop::new(FileDesc::from_raw_fd(sceKernelStdout().to_inner())).write(buf)
        };
        if let Ok(0) = res {
            Ok(buf.len())
        } else {
            res
        }
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
        let res = unsafe {
            ManuallyDrop::new(FileDesc::from_raw_fd(sceKernelStderr().to_inner())).write(buf)
        };
        if let Ok(0) = res {
            Ok(buf.len())
        } else {
            res
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[allow(dead_code, reason = "behavior changes in compilation context")]
pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
