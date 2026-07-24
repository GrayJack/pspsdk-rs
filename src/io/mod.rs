//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! The `pspsdk::io` module contains a number of common things you'll need
//! when doing input and output. The most core part of this module is
//! the [`Read`] and [`Write`] traits, which provide the
//! most general interface for reading and writing input and output.
//!
//! The traits and most types are re-export of the [`io_core`] crate.
//!
//! ## Read and Write
//!
//! Because they are traits, [`Read`] and [`Write`] are implemented by a number
//! of other types, and you can implement them for your types too. As such,
//! you'll see a few different types of I/O throughout the documentation in
//! this module: [`File`]s, [`TcpStream`]s, and sometimes even [`Vec<T>`]s. For
//! example, [`Read`] adds a [`read`][`Read::read`] method, which we can use on
//! [`File`]s:
//!
//! ```no_run
//! use pspsdk::{
//!     fs::File,
//!     io::{self, Read},
//!     println,
//! };
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt")?;
//!     let mut buffer = [0; 10];
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer)?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`Read`] and [`Write`] are so important, implementors of the two traits have a
//! nickname: readers and writers. So you'll sometimes see 'a reader' instead
//! of 'a type that implements the [`Read`] trait'. Much easier!
//!
//! ## Seek and BufRead
//!
//! Beyond that, there are two important traits that are provided: [`Seek`]
//! and [`BufRead`]. Both of these build on top of a reader to control
//! how the reading happens. [`Seek`] lets you control where the next byte is
//! coming from:
//!
//! ```no_run
//! use pspsdk::{
//!     fs::File,
//!     io::{self, Read, SeekFrom},
//!     println,
//! };
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt")?;
//!     let mut buffer = [0; 10];
//!
//!     // skip to the last 10 bytes of the file
//!     f.seek(SeekFrom::End(-10))?;
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer)?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`BufRead`] uses an internal buffer to provide a number of other ways to read, but
//! to show it off, we'll need to talk about buffers in general. Keep reading!
//!
//! ## BufReader and BufWriter
//!
//! Byte-based interfaces are unwieldy and can be inefficient, as we'd need to be
//! making near-constant calls to the operating system. To help with this,
//! `pspsdk::io` comes with two structs, [`BufReader`] and [`BufWriter`], which wrap
//! readers and writers. The wrapper uses a buffer, reducing the number of
//! calls and providing nicer methods for accessing exactly what you want.
//!
//! For example, [`BufReader`] works with the [`BufRead`] trait to add extra
//! methods to any reader:
//!
//! ```no_run
//! use pspsdk::{
//!     fs::File,
//!     io,
//!     io::{BufRead, BufReader, Read},
//!     println,
//! };
//!
//! fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt")?;
//!     let mut reader = BufReader::new(f);
//!     let mut buffer = String::new();
//!
//!     // read a line into buffer
//!     reader.read_line(&mut buffer)?;
//!
//!     println!("{buffer}");
//!     Ok(())
//! }
//! ```
//!
//! [`BufWriter`] doesn't add any new ways of writing; it just buffers every call
//! to [`write`][`Write::write`]:
//!
//! ```no_run
//! use pspsdk::{
//!     fs::File,
//!     io,
//!     io::{BufWriter, Write},
//!     println,
//! };
//!
//! fn main() -> io::Result<()> {
//!     let f = File::create("foo.txt")?;
//!     {
//!         let mut writer = BufWriter::new(f);
//!
//!         // write a byte to the buffer
//!         writer.write(&[42])?;
//!     } // the buffer is flushed once writer goes out of scope
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Standard input and output
//!
//! A very common source of input is standard input:
//!
//! ```no_run
//! use pspsdk::io;
//!
//! fn main() -> io::Result<()> {
//!     let mut input = String::new();
//!
//!     io::stdin().read_line(&mut input)?;
//!
//!     println!("You typed: {}", input.trim());
//!     Ok(())
//! }
//! ```
//!
//! Note that you cannot use the [`?` operator] in functions that do not return
//! a [`Result<T, E>`][`Result`]. Instead, you can call [`.unwrap()`]
//! or `match` on the return value to catch any possible errors:
//!
//! ```no_run
//! use pspsdk::io;
//!
//! let mut input = String::new();
//!
//! io::stdin().read_line(&mut input).unwrap();
//! ```
//!
//! And a very common source of output is standard output:
//!
//! ```no_run
//! use pspsdk::{io, Write};
//!
//! fn main() -> io::Result<()> {
//!     io::stdout().write(&[42])?;
//!     Ok(())
//! }
//! ```
//!
//! Of course, using [`io::stdout`] directly is less common than something like
//! [`println!`](crate::println).
//!
//! ## Iterator types
//!
//! A large number of the structures provided by `pspsdk::io` are for various
//! ways of iterating over I/O. For example, [`Lines`] is used to split over
//! lines:
//!
//! ```no_run
//! use pspsdk::{
//!     fs::File,
//!     io,
//!     io::{BufRead, BufReader, Read},
//!     println,
//! };
//!
//! fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt")?;
//!     let reader = BufReader::new(f);
//!
//!     for line in reader.lines() {
//!         println!("{}", line?);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Functions
//!
//! There are a number of [functions][functions-list] that offer access to various
//! features. For example, we can use three of these functions to copy everything
//! from standard input to standard output:
//!
//! ```no_run
//! use pspsdk::io;
//!
//! fn main() -> io::Result<()> {
//!     io::copy(&mut io::stdin(), &mut io::stdout())?;
//!     Ok(())
//! }
//! ```
//!
//! [functions-list]: #functions-1
//!
//! ## io::Result
//!
//! Last, but certainly not least, is [`io::Result`]. This type is used
//! as the return type of many `pspsdk::io` functions that can cause an error, and
//! can be returned from your own functions as well. Many of the examples in this
//! module use the [`?` operator]:
//!
//! ```
//! use pspsdk::{io, println};
//!
//! fn read_input() -> io::Result<()> {
//!     let mut input = String::new();
//!
//!     io::stdin().read_line(&mut input)?;
//!
//!     println!("You typed: {}", input.trim());
//!
//!     Ok(())
//! }
//! ```
//!
//! The return type of `read_input()`, [`io::Result<()>`][`io::Result`], is a very
//! common type for functions which don't have a 'real' return value, but do want to
//! return errors if they happen. In this case, the only purpose of this function is
//! to read the line and print it, so we use `()`.
//!
//! ## I/O Safety
//!
//! Rust follows an I/O safety discipline that is comparable to its memory safety discipline. This
//! means that file descriptors can be *exclusively owned*. (Here, "file descriptor" is meant to
//! subsume similar concepts that exist across a wide range of operating systems even if they might
//! use a different name, such as "handle".) An exclusively owned file descriptor is one that no
//! other code is allowed to access in any way, but the owner is allowed to access and even close
//! it any time. A type that owns its file descriptor should usually close it in its `drop`
//! function. Types like [`File`] own their file descriptor. Similarly, file descriptors
//! can be *borrowed*, granting the temporary right to perform operations on this file descriptor.
//! This indicates that the file descriptor will not be closed for the lifetime of the borrow, but
//! it does *not* imply any right to close this file descriptor, since it will likely be owned by
//! someone else.
//!
//! As this try to be as close to the Rust standard library as possible, the PSP platform-specific
//! parts of the library lives at [`os`](crate::os) module.
//!
//! To uphold I/O safety, it is crucial that no code acts on file descriptors it does not own or
//! borrow, and no code closes file descriptors it does not own. In other words, a safe function
//! that takes a regular integer, treats it as a file descriptor, and acts on it, is *unsound*.
//!
//! Not upholding I/O safety and acting on a file descriptor without proof of ownership can lead to
//! misbehavior and even Undefined Behavior in code that relies on ownership of its file
//! descriptors: a closed file descriptor could be re-allocated, so the original owner of that file
//! descriptor is now working on the wrong file. Some code might even rely on fully encapsulating
//! its file descriptors with no operations being performed by any other part of the program.
//!
//! Note that exclusive ownership of a file descriptor does *not* imply exclusive ownership of the
//! underlying kernel object that the file descriptor references (also called "open file
//! description" on some operating systems). File descriptors basically work like [`Arc`]: when you
//! receive an owned file descriptor, you cannot know whether there are any other file descriptors
//! that reference the same kernel object. However, when you create a new kernel object, you know
//! that you are holding the only reference to it. Just be careful not to lend it to anyone, since
//! they can obtain a clone and then you can no longer know what the reference count is! In that
//! sense, [`OwnedFd`] is like `Arc` and [`BorrowedFd<'a>`] is like `&'a Arc` (and similar for the
//! Windows types). In particular, given a `BorrowedFd<'a>`, you are not allowed to close the file
//! descriptor -- just like how, given a `&'a Arc`, you are not allowed to decrement the reference
//! count and potentially free the underlying object. There is no equivalent to `Box` for file
//! descriptors in the standard library (that would be a type that guarantees that the reference
//! count is `1`), however, it would be possible for a crate to define a type with those semantics.
//!
//! [`File`]: crate::fs::File
//! [`TcpStream`]: crate::net::TcpStream
//! [`io::stdout`]: stdout
//! [`io::Result`]: self::Result
//! [`?` operator]: https://doc.rust-lang.org/book/appendix-02-operators.html
//! [`Result`]: core::result::Result
//! [`.unwrap()`]: core::result::Result::unwrap
//! [`OwnedFd`]: crate::os::fd::OwnedFd
//! [`BorrowedFd<'a>`]: crate::os::fd::BorrowedFd
//! [`Arc`]: alloc::sync::Arc
//! [`Vec<T>`]: alloc::vec::Vec
use alloc::string::{String, ToString};


#[doc(hidden)]
pub mod printing;

mod stdio;
pub use stdio::{
    stderr, stderr_raw, stdin, stdin_raw, stdout, stdout_raw, Stderr, StderrLock, StderrRaw, Stdin,
    StdinLock, StdinRaw, Stdout, StdoutLock, StdoutRaw,
};

pub(crate) use stdio::cleanup;

// Re-export io_core::io
pub use io_core::io::*;
use io_core::os::OsFunctions;

use crate::sys::{SceError, SceIntoOkValue, SceResult, SceResultOk};

impl From<SceError> for Error {
    fn from(value: SceError) -> Self {
        Error::from_raw_os_error(value.to_inner() as i32)
    }
}

impl From<Error> for SceError {
    fn from(value: Error) -> Self {
        let res = value.raw_os_error().and_then(|raw| Self::from_raw(raw as u32));
        match res {
            Some(raw) => raw,
            None => match value.kind() {
                ErrorKind::NotFound => Self::FILE_NOT_FOUND,
                ErrorKind::PermissionDenied => Self::NO_PERM,
                ErrorKind::ConnectionRefused => Self::CONNECTION_REFUSED,
                ErrorKind::ConnectionReset => Self::CONNECTION_RESET,
                ErrorKind::HostUnreachable => Self::HOST_UNREACHABLE,
                ErrorKind::NetworkUnreachable => Self::NETWORK_UNREACHABLE,
                ErrorKind::ConnectionAborted => Self::CONNECTION_ABORTED,
                ErrorKind::NotConnected => Self::NOT_CONNECTED,
                ErrorKind::AddrInUse => Self::ADDR_IN_USE,
                ErrorKind::AddrNotAvailable => Self::ADDR_NOT_AVAILABLE,
                ErrorKind::NetworkDown => Self::NETWORK_DOWN,
                ErrorKind::BrokenPipe => Self::BROKEN_PIPE,
                ErrorKind::AlreadyExists => Self::FILE_ALREADY_EXISTS,
                ErrorKind::WouldBlock => Self::STD_ALREADY,
                ErrorKind::NotADirectory => Self::NOT_A_DIRECTORY,
                ErrorKind::IsADirectory => Self::IS_DIRECTORY,
                ErrorKind::DirectoryNotEmpty => Self::DIRECTORY_IS_NOT_EMPTY,
                ErrorKind::ReadOnlyFilesystem => Self::READ_ONLY,
                ErrorKind::FilesystemLoop => Self::TOO_MANY_SYMBOLIC_LINKS,
                ErrorKind::StaleNetworkFileHandle => Self::STALE_NETWORK_HANDLE,
                ErrorKind::InvalidInput => Self::STD_INVALID_ARGUMENT,
                ErrorKind::InvalidData => Self::STD_INVALID_ARGUMENT,
                ErrorKind::TimedOut => Self::TIMEOUT,
                ErrorKind::WriteZero => Self::NO_FREE_BUF_SPACE,
                ErrorKind::StorageFull => Self::NO_SPACE,
                ErrorKind::NotSeekable => Self::NOT_SEEKABLE,
                ErrorKind::QuotaExceeded => Self::FILE_QUOTA_EXCEEDED,
                ErrorKind::FileTooLarge => Self::FILE_IS_TOO_BIG,
                ErrorKind::ResourceBusy => Self::DEVICE_BUSY,
                ErrorKind::ExecutableFileBusy => Self::BUSY,
                ErrorKind::Deadlock => Self::DEADLOCK,
                ErrorKind::CrossesDevices => Self::CROSS_DEV_LINK,
                ErrorKind::TooManyLinks => Self::TOO_MANY_SYMBOLIC_LINKS,
                ErrorKind::InvalidFilename => Self::INVALID_NAME,
                ErrorKind::ArgumentListTooLong => Self::ARG_LIST_TOO_LONG,
                ErrorKind::Interrupted => Self::INTERRUPTED,
                ErrorKind::Unsupported => Self::NOT_SUPPORTED,
                ErrorKind::UnexpectedEof => Self::IO,
                ErrorKind::OutOfMemory => Self::NO_MEMORY,
                ErrorKind::InProgress => Self::IN_PROGRESS,
                ErrorKind::Other => Self::IO,
                _ => Self::IO,
            },
        }
    }
}

#[rustfmt::skip]
#[allow(clippy::redundant_field_names)]
pub(crate) const PSP_OS_FUNCS: &OsFunctions = &OsFunctions {
    error_string: error_string,
    error_str: error_str,
    decode_error_kind: decode_error_kind,
    is_interrupted: is_interrupted,
    last_os_error: || 0,
};

pub(crate) fn decode_error_kind(error: i32) -> ErrorKind {
    use ErrorKind;

    match error as u32 {
        0x00..=0x80000000 => ErrorKind::Uncategorized,
        // Safety: It is in the valid range
        _ => match unsafe { SceError::from_raw_unchecked(error as u32) } {
            SceError::FILE_NOT_FOUND => ErrorKind::NotFound,
            SceError::NO_PERM => ErrorKind::PermissionDenied,
            SceError::CONNECTION_REFUSED => ErrorKind::ConnectionRefused,
            SceError::CONNECTION_RESET => ErrorKind::ConnectionReset,
            SceError::HOST_UNREACHABLE => ErrorKind::HostUnreachable,
            SceError::NETWORK_UNREACHABLE => ErrorKind::NetworkUnreachable,
            SceError::CONNECTION_ABORTED => ErrorKind::ConnectionAborted,
            SceError::NOT_CONNECTED => ErrorKind::NotConnected,
            SceError::ADDR_IN_USE => ErrorKind::AddrInUse,
            SceError::ADDR_NOT_AVAILABLE => ErrorKind::AddrNotAvailable,
            SceError::NETWORK_DOWN => ErrorKind::NetworkDown,
            SceError::BROKEN_PIPE => ErrorKind::BrokenPipe,
            SceError::FILE_ALREADY_EXISTS => ErrorKind::AlreadyExists,
            SceError::STD_ALREADY => ErrorKind::WouldBlock,
            SceError::NOT_A_DIRECTORY => ErrorKind::NotADirectory,
            SceError::IS_DIRECTORY => ErrorKind::IsADirectory,
            SceError::DIRECTORY_IS_NOT_EMPTY => ErrorKind::DirectoryNotEmpty,
            SceError::READ_ONLY => ErrorKind::ReadOnlyFilesystem,
            SceError::STALE_NETWORK_HANDLE => ErrorKind::StaleNetworkFileHandle,
            SceError::STD_INVALID_ARGUMENT => ErrorKind::InvalidInput,
            SceError::TIMEOUT => ErrorKind::TimedOut,
            SceError::NO_FREE_BUF_SPACE => ErrorKind::WriteZero,
            SceError::NO_SPACE => ErrorKind::StorageFull,
            SceError::NOT_SEEKABLE => ErrorKind::NotSeekable,
            SceError::FILE_QUOTA_EXCEEDED => ErrorKind::QuotaExceeded,
            SceError::FILE_IS_TOO_BIG => ErrorKind::FileTooLarge,
            SceError::DEVICE_BUSY => ErrorKind::ResourceBusy,
            SceError::BUSY => ErrorKind::ExecutableFileBusy,
            SceError::DEADLOCK => ErrorKind::Deadlock,
            SceError::CROSS_DEV_LINK => ErrorKind::CrossesDevices,
            SceError::TOO_MANY_SYMBOLIC_LINKS => ErrorKind::TooManyLinks,
            SceError::INVALID_NAME => ErrorKind::InvalidFilename,
            SceError::ARG_LIST_TOO_LONG => ErrorKind::ArgumentListTooLong,
            SceError::INTERRUPTED => ErrorKind::Interrupted,
            SceError::NOT_SUPPORTED => ErrorKind::Unsupported,
            SceError::NO_MEMORY => ErrorKind::OutOfMemory,
            SceError::IN_PROGRESS => ErrorKind::InProgress,
            x if x >= SceError::OPERATION_NOT_PERMITTED && x <= SceError::WRONG_MEDIUM => {
                ErrorKind::Other
            },
            _ => ErrorKind::Uncategorized,
        },
    }
}

pub(crate) fn is_interrupted(error: i32) -> bool {
    error as u32 == SceError::INTERRUPTED.to_inner()
}

/// Expand for other error facilities
pub(crate) fn error_str(error: i32) -> &'static str {
    match error as u32 {
        0x00..=0x80000000 => "",
        // Safety: It is in the valid range
        err => unsafe { SceError::from_raw_unchecked(err).error_msg() },
    }
}

pub(crate) fn error_string(error: i32) -> String {
    error_str(error).to_string()
}

impl<T: SceResultOk, E> From<SceResult<T>> for core::result::Result<T, E>
where
    SceError: Into<E>,
{
    fn from(value: SceResult<T>) -> Self {
        value.into_result().map_err(Into::into)
    }
}

impl<T: SceIntoOkValue, E: Into<SceError>> From<core::result::Result<T, E>> for SceResult<T> {
    fn from(value: core::result::Result<T, E>) -> Self {
        match value {
            Ok(v) => SceResult::new(v.into_ok_value()),
            Err(e) => SceResult::new(e.into().to_inner()),
        }
    }
}
