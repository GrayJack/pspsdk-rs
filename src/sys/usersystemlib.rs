//! The `usersustemlib.prx` exported functions.

use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{
    thread::{LwMutexInfo, LwMutexWorkArea, ThreadId, TlsPoolId},
    SceResult, SceSize,
};


// FIXME: Add missing functions
//
// - `sceKernelCpuResumeIntr`
// - `sceKernelCpuResumeIntrWithSync`
// - `sceKernelCpuSuspendIntr`
// - `sceKernelIsCpuIntrEnable`
// - `sceKernelIsCpuIntrSuspended`
#[psp_stub(libname = "Kernel_Library", flags = 0x0009, use_crate)]
unsafe extern "C" {
    /// Gets the remaining free size of the calling thread stack (?)
    ///
    /// # Return Value
    ///
    /// The remaining free size of the calling thread stack (probably in bytes).
    #[nid(0xD13BDE95)]
    pub safe fn sceKernelCheckThreadStack() -> SceSize;

    /// Gets the address of a user TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The user TLS memory pool UID.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the user TLS memory pool on success, null value on error.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0xFA835CDE)]
    pub safe fn sceKernelGetTlsAddr(id: TlsPoolId) -> *mut c_void;

    /// Locks a lightweight mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0xBEA46419)]
    pub safe fn sceKernelLockLwMutex(
        work_area: &mut LwMutexWorkArea, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Locks a lightweight mutex a number of times, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x1FC64E09)]
    pub safe fn sceKernelLockLwMutexCB(
        work_area: &mut LwMutexWorkArea, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to lock a lightweight mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0xDC692EE3)]
    pub safe fn sceKernelTryLockLwMutex(
        work_area: &mut LwMutexWorkArea, lock_count: u32,
    ) -> SceResult<()>;

    /// Unlocks a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `unlock_count`: The number of times to unlock a mutex after resource acquisition. It must
    ///   be `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x15B6446B)]
    pub safe fn sceKernelUnlockLwMutex(
        work_area: &mut LwMutexWorkArea, unlock_count: u32,
    ) -> SceResult<()>;

    /// Gets the current state of a lightweight mutex.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `info` **[[InOut parameter]]**: A reference to [`LwMutexInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0xC1734599)]
    pub safe fn sceKernelReferLwMutexStatus(
        work_area: &mut LwMutexWorkArea, info: &mut LwMutexInfo,
    ) -> SceResult<()>;

    /// Gets the thread UID of the calling thread.
    ///
    /// # Returns Value
    ///
    /// Returns the thread UID of the calling thread on success, error value otherwise.
    #[nid(0x293B45B8)]
    pub safe fn sceKernelGetThreadId() -> SceResult<ThreadId>;

    /// What can I say? It is `memcpy` with a weird prefix.
    #[nid(0x1839852A)]
    pub unsafe fn sceKernelMemcpy(dst: *mut u8, src: *const u8, num: SceSize) -> *mut u8;

    /// What can I say? It is `memset` with a weird prefix.
    #[nid(0xA089ECA4)]
    pub unsafe fn sceKernelMemset(ptr: *mut u8, value: i32, num: SceSize) -> *mut u8;
}
