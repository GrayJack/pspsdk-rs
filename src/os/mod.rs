//! OS-specific functionality.

pub use io_core::sys::{AsInner, AsInnerMut, FromInner, IntoInner};

pub mod fd;

pub(crate) mod stdio;
