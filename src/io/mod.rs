use alloc::string::{String, ToString};


#[doc(hidden)]
pub mod printing;

mod stdio;
pub use stdio::{stderr_raw, stdin_raw, stdout_raw, StderrRaw, StdinRaw, StdoutRaw};

// Re-export io_core::io
pub use io_core::io::*;
use io_core::os::OsFunctions;

use crate::sys::SceError;

impl From<SceError> for Error {
    fn from(value: SceError) -> Self {
        Error::from_raw_os_error(value.as_inner() as i32)
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
    error as u32 == SceError::INTERRUPTED.as_inner()
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
