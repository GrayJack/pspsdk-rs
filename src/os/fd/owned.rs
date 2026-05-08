//! Owned and borrowed Unix-like file descriptors.

use core::{fmt, marker::PhantomData, mem::ManuallyDrop};


use crate::{
    io,
    os::{
        fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd},
        AsInner, FromInner, IntoInner,
    },
    sys::io::{sceIoClose, sceIoLseek, sceIoRead, sceIoWrite, FileId, Whence},
};

/// A borrowed file descriptor.
///
/// This has a lifetime parameter to tie it to the lifetime of something that owns the file
/// descriptor. For the duration of that lifetime, it is guaranteed that nobody will close the file
/// descriptor.
///
/// This uses `repr(transparent)` and has the representation of a host file
/// descriptor, so it can be used in FFI in places where a file descriptor is
/// passed as an argument, it is not captured or consumed.
///
/// This type does not have a [`ToOwned`][crate::borrow::ToOwned]
/// implementation. Calling `.to_owned()` on a variable of this type will call
/// it on `&BorrowedFd` and use `Clone::clone()` like `ToOwned` does for all
/// types implementing `Clone`. The result will be descriptor borrowed under
/// the same lifetime.
///
/// To obtain an [`OwnedFd`], you can use [`BorrowedFd::try_clone_to_owned`]
/// instead, but this is not supported on all platforms.
#[derive(Copy, Clone)]
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
pub struct BorrowedFd<'fd> {
    fd: FileId,
    _phantom: PhantomData<&'fd OwnedFd>,
}

/// An owned file descriptor.
///
/// This closes the file descriptor on drop. It is guaranteed that nobody else will close the file
/// descriptor.
///
/// This uses `repr(transparent)` and has the representation of a host file
/// descriptor, so it can be used in FFI in places where a file descriptor is
/// passed as a consumed argument or returned as an owned value.
///
/// You can use [`AsFd::as_fd`] to obtain a [`BorrowedFd`].
#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
pub struct OwnedFd {
    fd: FileId,
}

impl BorrowedFd<'_> {
    /// Returns a `BorrowedFd` holding the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `fd` must remain open for the duration of
    /// the returned `BorrowedFd`.
    ///
    /// # Panics
    ///
    /// Panics if the raw file descriptor has the value `-1`.
    #[inline]
    #[track_caller]
    pub const unsafe fn borrow_raw(fd: RawFd) -> Self {
        Self {
            fd: FileId::from_raw(fd).expect("fd not in valid range"),
            _phantom: PhantomData,
        }
    }

    /// Creates a new `OwnedFd` instance that shares the same underlying file
    /// description as the existing `BorrowedFd` instance.
    pub fn try_clone_to_owned(&self) -> io::Result<OwnedFd> {
        Err(io_core::io_const_error!(
            io::ErrorKind::Unsupported,
            "operation not supported on this platform"
        ))
    }
}

impl BorrowedFd<'_> {
    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let res = unsafe { sceIoRead(self.fd, buf.as_mut_ptr().cast(), buf.len()) };
        res.into_result().map_err(Into::into)
    }

    #[inline]
    pub fn read_buf(&self, mut cursor: io::BorrowedCursor<'_>) -> io::Result<()> {
        let res =
            unsafe { sceIoRead(self.fd, cursor.as_mut().as_mut_ptr().cast(), cursor.capacity()) };

        let ret = res.into_result().map_err(Into::<io::Error>::into)?;

        // SAFETY: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            cursor.advance(ret);
        }

        Ok(())
    }

    #[inline]
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            sceIoWrite(self.fd, buf.as_ptr().cast(), buf.len())
                .into_result()
                .map_err(Into::into)
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
        let (whence, pos) = match pos {
            // Casting to `i64` is fine, too large values will end up as
            // negative which will cause an error in `lseek`.
            io::SeekFrom::Start(off) => (Whence::Start, off as i64),
            io::SeekFrom::End(off) => (Whence::End, off),
            io::SeekFrom::Current(off) => (Whence::Current, off),
        };
        sceIoLseek(self.fd, pos, whence).into_result().map_err(From::from)
    }

    #[inline]
    #[doc(alias = "tell")]
    pub fn stream_position(&self) -> io::Result<u64> {
        self.seek(io::SeekFrom::Current(0))
    }
}

impl OwnedFd {
    pub(crate) fn as_file_id(&self) -> FileId {
        self.fd
    }

    /// Creates a new `OwnedFd` instance that shares the same underlying file
    /// description as the existing `OwnedFd` instance.
    pub fn try_clone(&self) -> io::Result<Self> {
        self.as_fd().try_clone_to_owned()
    }
}

impl AsRawFd for BorrowedFd<'_> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_inner()
    }
}

impl AsRawFd for OwnedFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_inner()
    }
}

impl IntoRawFd for OwnedFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        ManuallyDrop::new(self).fd.as_inner()
    }
}

impl AsInner<FileId> for OwnedFd {
    #[inline]
    fn as_inner(&self) -> &FileId {
        &self.fd
    }
}

impl AsInner<FileId> for BorrowedFd<'_> {
    #[inline]
    fn as_inner(&self) -> &FileId {
        &self.fd
    }
}

impl IntoInner<FileId> for OwnedFd {
    fn into_inner(self) -> FileId {
        self.fd
    }
}

impl FromInner<FileId> for OwnedFd {
    fn from_inner(inner: FileId) -> Self {
        Self { fd: inner }
    }
}

impl FromRawFd for OwnedFd {
    /// Constructs a new instance of `Self` from the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `fd` must be open and suitable for assuming
    /// [ownership][io-safety]. The resource must not require any cleanup other than `close`.
    ///
    /// [io-safety]: io#io-safety
    ///
    /// # Panics
    ///
    /// Panics if the raw file descriptor is not in the valid range (same as [`SceUid`] range).
    #[inline]
    #[track_caller]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            fd: FileId::from_raw(fd).expect("fd not in valid range"),
        }
    }
}

impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let mut res = sceIoClose(self.fd);

            if res.is_ok() {
                return;
            }

            // Retry a few times if unable to close
            let mut atempts = 0;
            while atempts < 0x10 && res.is_err() {
                res = sceIoClose(self.fd);
                atempts += 1;
            }
        }
    }
}

impl fmt::Debug for BorrowedFd<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedFd").field("fd", &self.fd).finish()
    }
}

impl fmt::Debug for OwnedFd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedFd").field("fd", &self.fd).finish()
    }
}

impl io::Read for &BorrowedFd<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(buf)
    }
}

impl fmt::Write for &BorrowedFd<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl io::Write for &BorrowedFd<'_> {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        (**self).write(data)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for &BorrowedFd<'_> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }
}

/// A trait to borrow the file descriptor from an underlying object.
pub trait AsFd {
    /// Borrows the file descriptor.
    fn as_fd(&self) -> BorrowedFd<'_>;
}

impl<T: AsFd + ?Sized> AsFd for &T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

impl<T: AsFd + ?Sized> AsFd for &mut T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

impl AsFd for BorrowedFd<'_> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        *self
    }
}

impl AsFd for OwnedFd {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        // Safety: `OwnedFd` and `BorrowedFd` have the same validity
        // invariants, and the `BorrowedFd` is bounded by the lifetime
        // of `&self`.
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl<T: AsFd + ?Sized> AsFd for alloc::sync::Arc<T> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd + ?Sized> AsFd for alloc::rc::Rc<T> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}

impl<T: AsFd + ?Sized> AsFd for alloc::boxed::Box<T> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        (**self).as_fd()
    }
}
