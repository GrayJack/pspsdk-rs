use core::{
    cell::RefCell,
    fmt,
    panic::{RefUnwindSafe, UnwindSafe},
};

use alloc::{string::String, vec::Vec};

use crate::{
    io::{self, BorrowedCursor, BufRead, BufReader, LineWriter, Lines, Read, Write},
    os,
    sync::{
        nonpoison::{OnceLock, ReentrantLock, ReentrantLockGuard},
        poison::{Mutex, MutexGuard},
    },
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
/// handles returned by `pspsdk::io::stdin`. Data buffered by the `pspsdk::io::stdin`
/// handles is **not** available to raw handles returned from this function.
///
/// The returned handle has no external synchronization or buffering.
pub const fn stdin_raw() -> StdinRaw {
    StdinRaw(os::stdio::Stdin::new())
}

/// Constructs a new raw handle to the standard output stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `pspsdk::io::stdout`. Note that data is buffered by the
/// `pspsdk::io::stdout` handles so writes which happen via this raw handle may
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
/// handles returned by `pspsdk::io::stderr`.
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

/// A handle to the standard input stream of a process.
///
/// Each handle is a shared reference to a global buffer of input data to this
/// process. A handle can be `lock`'d to gain full access to [`BufRead`] methods
/// (e.g., `.lines()`). Reads to this handle are otherwise locked with respect
/// to other reads.
///
/// This handle implements the `Read` trait, but beware that concurrent reads
/// of `Stdin` must be executed with care.
///
/// Created by the [`io::stdin`] method.
///
/// [`io::stdin`]: stdin
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use pspsdk::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     stdin.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

/// A locked reference to the [`Stdin`] handle.
///
/// This handle implements both the [`Read`] and [`BufRead`] traits, and
/// is constructed via the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use pspsdk::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     {
///         let mut handle = stdin.lock(); // We get `StdinLock` here.
///         handle.read_line(&mut buffer)?;
///     } // `StdinLock` is dropped here.
///     Ok(())
/// }
/// ```
#[must_use = "if unused stdin will immediately unlock"]
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<StdinRaw>>,
}

/// Constructs a new handle to the standard input of the current process.
///
/// Each handle returned is a reference to a shared global buffer whose access
/// is synchronized via a mutex. If you need more explicit control over
/// locking, see the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// Using implicit synchronization:
///
/// ```no_run
/// use pspsdk::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     io::stdin().read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
///
/// Using explicit synchronization:
///
/// ```no_run
/// use pspsdk::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin();
///     let mut handle = stdin.lock();
///
///     handle.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[must_use]
pub fn stdin() -> Stdin {
    static INSTANCE: OnceLock<Mutex<BufReader<StdinRaw>>> = OnceLock::new();
    Stdin {
        inner: INSTANCE.get_or_init(|| {
            Mutex::new(BufReader::with_capacity(os::stdio::STDIN_BUF_SIZE, stdin_raw()))
        }),
    }
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope.was The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pspsdk::io::{self, BufRead};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut buffer = String::new();
    ///     let stdin = io::stdin();
    ///     let mut handle = stdin.lock();
    ///
    ///     handle.read_line(&mut buffer)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock {
            inner: self.inner.lock_unchecked(),
        }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::read_line`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pspsdk::io;
    ///
    /// let mut input = String::new();
    /// match io::stdin().read_line(&mut input) {
    ///     Ok(n) => {
    ///         println!("{n} bytes read");
    ///         println!("{input}");
    ///     },
    ///     Err(error) => println!("error: {error}"),
    /// }
    /// ```
    ///
    /// You can run the example one of two ways:
    ///
    /// - Pipe some text to it, e.g., `printf foo | path/to/executable`
    /// - Give it text interactively by running the executable directly, in which case it will wait
    ///   for the Enter key to be pressed before continuing
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }

    /// Consumes this handle and returns an iterator over input lines.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::lines`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pspsdk::io;
    ///
    /// let lines = io::stdin().lines();
    /// for line in lines {
    ///     println!("got a line: {}", line.unwrap());
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

impl fmt::Debug for Stdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdin").finish_non_exhaustive()
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
}

impl Read for &Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
}

// only used by platform-dependent io::copy specializations, i.e. unused on some platforms
impl StdinLock<'_> {
    #[allow(unused)]
    pub(crate) fn as_mut_buf(&mut self) -> &mut BufReader<impl Read> {
        &mut self.inner
    }
}

impl Read for StdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.inner.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }
}

// impl SpecReadByte for StdinLock<'_> {
//     #[inline]
//     fn spec_read_byte(&mut self) -> Option<Result<u8, io::Error>> {
//         BufReader::spec_read_byte(&mut *self.inner)
//     }
// }

impl BufRead for StdinLock<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, n: usize) {
        self.inner.consume(n)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }
}

impl fmt::Debug for StdinLock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdinLock").finish_non_exhaustive()
    }
}

/// A handle to the global standard output stream of the current process.
///
/// Each handle shares a global buffer of data to be written to the standard
/// output stream. Access is also synchronized via a lock and explicit control
/// over locking is available via the [`lock`] method.
///
/// Created by the [`io::stdout`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// [`lock`]: Stdout::lock
/// [`io::stdout`]: stdout
pub struct Stdout {
    // FIXME: this should be LineWriter or BufWriter depending on the state of
    //        stdout (tty or not). Note that if this is not line buffered it
    //        should also flush-on-panic or some form of flush-on-abort.
    inner: &'static ReentrantLock<RefCell<LineWriter<StdoutRaw>>>,
}

/// A locked reference to the [`Stdout`] handle.
///
/// This handle implements the [`Write`] trait, and is constructed via
/// the [`Stdout::lock`] method. See its documentation for more.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
#[must_use = "if unused stdout will immediately unlock"]
pub struct StdoutLock<'a> {
    inner: ReentrantLockGuard<'a, RefCell<LineWriter<StdoutRaw>>>,
}

static STDOUT: OnceLock<ReentrantLock<RefCell<LineWriter<StdoutRaw>>>> = OnceLock::new();

/// Constructs a new handle to the standard output of the current process.
///
/// Each handle returned is a reference to a shared global buffer whose access
/// is synchronized via a mutex. If you need more explicit control over
/// locking, see the [`Stdout::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// Using implicit synchronization:
///
/// ```no_run
/// use pspsdk::io::{self, Write};
///
/// fn main() -> io::Result<()> {
///     io::stdout().write_all(b"hello world")?;
///
///     Ok(())
/// }
/// ```
///
/// Using explicit synchronization:
///
/// ```no_run
/// use pspsdk::io::{self, Write};
///
/// fn main() -> io::Result<()> {
///     let stdout = io::stdout();
///     let mut handle = stdout.lock();
///
///     handle.write_all(b"hello world")?;
///
///     Ok(())
/// }
/// ```
#[must_use]
pub fn stdout() -> Stdout {
    Stdout {
        inner: STDOUT
            .get_or_init(|| ReentrantLock::new(RefCell::new(LineWriter::new(stdout_raw())))),
    }
}

// Flush the data and disable buffering during shutdown
// by replacing the line writer by one with zero
// buffering capacity.
#[allow(dead_code)] // we use that later
pub fn cleanup() {
    let mut initialized = false;
    let stdout = STDOUT.get_or_init(|| {
        initialized = true;
        ReentrantLock::new(RefCell::new(LineWriter::with_capacity(0, stdout_raw())))
    });

    if !initialized {
        // The buffer was previously initialized, overwrite it here.
        // We use try_lock() instead of lock(), because someone
        // might have leaked a StdoutLock, which would
        // otherwise cause a deadlock here.
        if let Some(lock) = stdout.try_lock() {
            *lock.borrow_mut() = LineWriter::with_capacity(0, stdout_raw());
        }
    }
}

impl Stdout {
    /// Locks this handle to the standard output stream, returning a writable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the `Write` trait for writing data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pspsdk::io::{self, Write};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stdout = io::stdout().lock();
    ///
    ///     stdout.write_all(b"hello world")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn lock(&self) -> StdoutLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `ReentrantMutex` is
        // static.
        StdoutLock {
            inner: self.inner.lock(),
        }
    }
}

impl UnwindSafe for Stdout {}

impl RefUnwindSafe for Stdout {}

impl fmt::Debug for Stdout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdout").finish_non_exhaustive()
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        (&*self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        io::Write::is_write_vectored(&self)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (&*self).write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        (&*self).write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        (&*self).write_fmt(args)
    }
}

impl Write for &Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock().write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.lock().write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.lock().is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.lock().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.lock().write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        self.lock().write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        self.lock().write_fmt(args)
    }
}

impl UnwindSafe for StdoutLock<'_> {}

impl RefUnwindSafe for StdoutLock<'_> {}

impl Write for StdoutLock<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.inner.borrow_mut().write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.inner.borrow_mut().is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.borrow_mut().write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        self.inner.borrow_mut().write_all_vectored(bufs)
    }
}

impl fmt::Debug for StdoutLock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdoutLock").finish_non_exhaustive()
    }
}

/// A handle to the standard error stream of a process.
///
/// For more information, see the [`io::stderr`] method.
///
/// [`io::stderr`]: stderr
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
pub struct Stderr {
    inner: &'static ReentrantLock<RefCell<StderrRaw>>,
}

/// A locked reference to the [`Stderr`] handle.
///
/// This handle implements the [`Write`] trait and is constructed via
/// the [`Stderr::lock`] method. See its documentation for more.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
#[must_use = "if unused stderr will immediately unlock"]
pub struct StderrLock<'a> {
    inner: ReentrantLockGuard<'a, RefCell<StderrRaw>>,
}

/// Constructs a new handle to the standard error of the current process.
///
/// This handle is not buffered.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// Using implicit synchronization:
///
/// ```no_run
/// use pspsdk::io::{self, Write};
///
/// fn main() -> io::Result<()> {
///     io::stderr().write_all(b"hello world")?;
///
///     Ok(())
/// }
/// ```
///
/// Using explicit synchronization:
///
/// ```no_run
/// use pspsdk::io::{self, Write};
///
/// fn main() -> io::Result<()> {
///     let stderr = io::stderr();
///     let mut handle = stderr.lock();
///
///     handle.write_all(b"hello world")?;
///
///     Ok(())
/// }
/// ```
#[must_use]
pub fn stderr() -> Stderr {
    // Note that unlike `stdout()` we don't use `at_exit` here to register a
    // destructor. Stderr is not buffered, so there's no need to run a
    // destructor for flushing the buffer
    static INSTANCE: ReentrantLock<RefCell<StderrRaw>> =
        ReentrantLock::new(RefCell::new(stderr_raw()));

    Stderr { inner: &INSTANCE }
}

impl Stderr {
    /// Locks this handle to the standard error stream, returning a writable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Write`] trait for writing data.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::io::{self, Write};
    ///
    /// fn foo() -> io::Result<()> {
    ///     let stderr = io::stderr();
    ///     let mut handle = stderr.lock();
    ///
    ///     handle.write_all(b"hello world")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn lock(&self) -> StderrLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `ReentrantMutex` is
        // static.
        StderrLock {
            inner: self.inner.lock(),
        }
    }
}

impl UnwindSafe for Stderr {}

impl RefUnwindSafe for Stderr {}

impl fmt::Debug for Stderr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stderr").finish_non_exhaustive()
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        (&*self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        io::Write::is_write_vectored(&self)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (&*self).write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        (&*self).write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        (&*self).write_fmt(args)
    }
}

impl Write for &Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock().write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.lock().write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.lock().is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.lock().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.lock().write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        self.lock().write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        self.lock().write_fmt(args)
    }
}

impl UnwindSafe for StderrLock<'_> {}

impl RefUnwindSafe for StderrLock<'_> {}

impl Write for StderrLock<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.inner.borrow_mut().write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.inner.borrow_mut().is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.borrow_mut().write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        self.inner.borrow_mut().write_all_vectored(bufs)
    }
}

impl fmt::Debug for StderrLock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StderrLock").finish_non_exhaustive()
    }
}
