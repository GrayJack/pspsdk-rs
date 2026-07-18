//! Power management abstractions.

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::{
    io,
    sys::suspend::{
        sceKernelPowerLock, sceKernelPowerUnlock, sceKernelVolatileMemLock,
        sceKernelVolatileMemTryLock, sceKernelVolatileMemUnlock,
    },
};

pub use crate::sys::power::PowerLockKind;

/// A guard for power lock.
///
/// When it gets out of the scope, it will execute a power unlock.
#[derive(Debug)]
pub struct PowerLockGuard {
    lock_kind: PowerLockKind,
}

impl PowerLockGuard {
    #[inline]
    fn new(lock_kind: PowerLockKind) -> io::Result<Self> {
        // Using `sceKernelPowerLock` because it is loaded with sysmem.prx that is loaded before
        // power.prx. Also, `scePowerLock` is a thin wrapper on `sceKernelPowerLock`
        sceKernelPowerLock(lock_kind).into_result()?;
        Ok(Self { lock_kind })
    }
}

impl Drop for PowerLockGuard {
    #[inline]
    fn drop(&mut self) {
        // Using `sceKernelPowerUnlock` because it is loaded with sysmem.prx that is loaded before
        // power.prx. Also, `scePowerUnlock` is a thin wrapper on `sceKernelPowerUnlock`
        let res = sceKernelPowerUnlock(self.lock_kind);

        if res.is_err() {
            let _ = sceKernelPowerUnlock(self.lock_kind);
        }
    }
}

/// The guard of the acquired memory from successful calls to [`acquire_volatile_memory`] or
/// [`acquire_volatile_memory_blocking`].
///
/// It can be dereferenced into a slice of bytes of the PSP volatile memory, and it will
/// automatically unlocked when it reaches out of the scope.
#[derive(Debug)]
pub struct VolatileMemoryLockGuard<'a> {
    mem: &'a mut [u8],
}

impl<'a> VolatileMemoryLockGuard<'a> {
    #[inline]
    fn lock() -> io::Result<Self> {
        let mut ptr = MaybeUninit::uninit();
        let mut size = 0;

        unsafe {
            // Using `sceKernelVolatileMemLock` because it is loaded with sysmem.prx that is loaded
            // before power.prx. Also, `scePowerVolatileMemLock` is a thin wrapper on
            // `sceKernelVolatileMemLock`
            sceKernelVolatileMemLock(0, ptr.as_mut_ptr(), &mut size).into_result()?;

            let ptr = ptr.assume_init();
            debug_assert!(!ptr.is_null());
            Ok(Self {
                mem: core::slice::from_raw_parts_mut(ptr, size),
            })
        }
    }

    #[inline]
    fn try_lock() -> io::Result<Self> {
        let mut ptr = MaybeUninit::uninit();
        let mut size = 0;

        unsafe {
            // Using `sceKernelVolatileMemTryLock` because it is loaded with sysmem.prx that is
            // loaded before power.prx. Also, `scePowerVolatileMemTryLock` is a thin
            // wrapper on `sceKernelVolatileMemTryLock`
            sceKernelVolatileMemTryLock(0, ptr.as_mut_ptr(), &mut size).into_result()?;

            let ptr = ptr.assume_init();
            debug_assert!(!ptr.is_null());
            Ok(Self {
                mem: core::slice::from_raw_parts_mut(ptr, size),
            })
        }
    }
}

impl<'a> Drop for VolatileMemoryLockGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // Using `sceKernelVolatileMemUnlock` because it is loaded with sysmem.prx that is
            // loaded before power.prx. Also, `scePowerVolatileMemUnlock` is a thin
            // wrapper on `sceKernelVolatileMemUnlock`
            let res = sceKernelVolatileMemUnlock(0);

            if res.is_err() {
                let _ = sceKernelVolatileMemUnlock(0);
            }
        }
    }
}

impl<'a> Deref for VolatileMemoryLockGuard<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.mem
    }
}

impl<'a> DerefMut for VolatileMemoryLockGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mem
    }
}


/// Locks power state of the device with a specific lock kind.
///
/// That means that device power off and suspend is delayed until the power lock is unlocked.
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::power::{self, PowerLockKind};
/// use pspsdk::io;
///
/// fn psp_main() -> io::Result<()> {
///     pspsdk::enable_home_button();
///
///     /* stuff... */
///     if some_state {
///         let guard = power::lock(PowerLockKind::SuspendOnly)?
///         /* Do something while suspend if blocked */
///     } // the guard is dropped here and unlocks the power lock
/// }
/// ```
#[inline]
pub fn lock(lock_kind: PowerLockKind) -> io::Result<PowerLockGuard> {
    PowerLockGuard::new(lock_kind)
}

/// Locks and grants access to the device volatile memory (non-blocking).
///
/// If the volatile memory is currently being used by other processes, this function will return
/// a error.
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::{io, power};
///
/// fn psp_main() -> io::Result<()> {
///     pspsdk::enable_home_button();
///
///     {
///         let guard = power::acquire_volatile_memory()?;
///         // Do somethings with the memory
///     }
/// }
/// ```
#[inline]
pub fn acquire_volatile_memory<'a>() -> io::Result<VolatileMemoryLockGuard<'a>> {
    VolatileMemoryLockGuard::try_lock()
}

/// Locks and grants access to the device volatile memory (blocking).
///
/// If the volatile memory is currently being used by other processes, the process that called
/// this function will enter a wait state until the volatile stops being used.
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::{io, power};
///
/// fn psp_main() -> io::Result<()> {
///     pspsdk::enable_home_button();
///
///     {
///         let guard = power::acquire_volatile_memory_blocking()?;
///         // Do somethings with the memory
///     }
/// }
/// ```
#[inline]
pub fn acquire_volatile_memory_blocking<'a>() -> io::Result<VolatileMemoryLockGuard<'a>> {
    VolatileMemoryLockGuard::lock()
}
