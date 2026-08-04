//! Suspend operations.
use pspsdk_macros::{psp_fw_cfg, psp_stub};

use crate::sys::{
    power::{PowerLockKind, PowerTick},
    SceResult, SceSize,
};

#[psp_stub(libname = "sceSuspendForUser", flags = 0x4008, use_crate)]
unsafe extern "C" {
    /// Locks power state of the device.
    ///
    /// That means that device power off is delayed until the power lock is unlocked.
    ///
    /// # Parameters
    ///
    /// - `lock_kind`: The power processing lock kind.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Precautions
    ///
    /// The power lock should always be unlocked ([`sceKernelPowerUnlock`]) after processing
    /// whatever important thing you needed, because it blocks power off even with physical buttons.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(0xEADB1BD7)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelPowerLock(lock_kind: PowerLockKind) -> SceResult<()>;

    /// Unlocks power state of the device.
    ///
    /// # Parameters
    ///
    /// - `lock_kind`: The power processing lock kind.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Precautions
    ///
    /// Calling this function without calling [`sceKernelPowerLock`] can mess up cases of power lock
    /// nesting.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(0x3AEE7261)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelPowerUnlock(lock_kind: PowerLockKind) -> SceResult<()>;

    /// Locks and grants access to the device volatile memory (blocking).
    ///
    /// If the volatile memory is currently being used by other processes, the process that called
    /// this function will enter a wait state until the volatile stops being used.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Always zero on reversed code.
    /// - `ptr`: A pointer to receive the head pointer of the volatile memory.
    /// - `size`: A reference to receive the size of the volatile memory.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Precautions
    ///
    /// The volatile memory is used by the PSP system for the utility library and to save eDRAM
    /// during suspend/resume time and those will be blocked while this memory is being used by your
    /// software. For that reason, it is best to unlock (with [`sceKernelVolatileMemUnlock`]) as
    /// soon as you done with using that piece of memory.
    ///
    /// If you need to use this memory for long periods and want to maintain such functionalities
    /// working, you, theoretically, can [create] and [register] a power callback that handles
    /// values of the `arg` parameter of [`CallbackFunction`] (namely [`PowerCallbackArg::Standby`]
    /// and [`PowerCallbackArg::Suspending`] to unlock and [`PowerCallbackArg::ResumeComplete`] ro
    /// lock again).
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    ///
    /// [create]: crate::sys::thread::sceKernelCreateCallback
    /// [register]: crate::sys::power::scePowerRegisterCallback
    /// [`CallbackFunction`]: crate::sys::thread::CallbackFunction
    /// [`PowerCallbackArg::Standby`]: crate::sys::power::PowerCallbackArg::Standby
    /// [`PowerCallbackArg::Suspending`]: crate::sys::power::PowerCallbackArg::Suspending
    /// [`PowerCallbackArg::ResumeComplete`]: crate::sys::power::PowerCallbackArg::ResumeComplete
    #[psp_fw_cfg(150..)]
    #[nid(0x3E0271D3)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelVolatileMemLock(
        unk: u32, ptr: *mut *mut u8, size: &mut SceSize,
    ) -> SceResult<()>;

    /// Locks and grants access to the device volatile memory (non-blocking).
    ///
    /// If the volatile memory is currently being used by other processes, this function will return
    /// a error.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Always zero on reversed code.
    /// - `ptr`: A pointer to receive the head pointer of the volatile memory.
    /// - `size`: A reference to receive the size of the volatile memory.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// Specifically [`SceError::POWER_CANNOT_LOCK_VMEM`] when volatile memory is in use by other
    /// processes.
    ///
    /// # Precautions
    ///
    /// The volatile memory is used by the PSP system for the utility library and to save eDRAM
    /// during suspend/resume time and those will be blocked while this memory is being used by your
    /// software. For that reason, it is best to unlock (with [`sceKernelVolatileMemUnlock`]) as
    /// soon as you done with using that piece of memory.
    ///
    /// If you need to use this memory for long periods and want to maintain such functionalities
    /// working, you, theoretically, can [create] and [register] a power callback that handles
    /// values of the `arg` parameter of [`CallbackFunction`] (namely [`PowerCallbackArg::Standby`]
    /// and [`PowerCallbackArg::Suspending`] to unlock and [`PowerCallbackArg::ResumeComplete`] to
    /// lock again).
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    ///
    /// [create]: crate::sys::thread::sceKernelCreateCallback
    /// [register]: crate::sys::power::scePowerRegisterCallback
    /// [`CallbackFunction`]: crate::sys::thread::CallbackFunction
    /// [`SceError::POWER_CANNOT_LOCK_VMEM`]: crate::sys::SceError::POWER_CANNOT_LOCK_VMEM
    /// [`PowerCallbackArg::Standby`]: crate::sys::power::PowerCallbackArg::Standby
    /// [`PowerCallbackArg::Suspending`]: crate::sys::power::PowerCallbackArg::Suspending
    /// [`PowerCallbackArg::ResumeComplete`]: crate::sys::power::PowerCallbackArg::ResumeComplete
    #[psp_fw_cfg(150..)]
    #[nid(0xA14F40B2)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelVolatileMemTryLock(
        unk: u32, ptr: *mut *mut u8, size: &mut SceSize,
    ) -> SceResult<()>;

    /// Unlock access to the device volatile memory.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Always zero on reversed code.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// Once unlocked, access to the volatile memory to given pointer is no longer valid.
    ///
    /// # Precautions
    ///
    /// This function should not be called without having a previous successful call to either
    /// [`sceKernelVolatileMemLock`] or [`sceKernelVolatileMemTryLock`].
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xA569E425)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelVolatileMemUnlock(unk: u32) -> SceResult<()>;

    /// Configures the system to cancel the count of idle timers to prevent the system to enter in
    /// power save state (entirely or partial).
    ///
    /// # Parameters
    ///
    /// - `tick_kind`: The configuration of what timer counts to cancel.
    ///
    /// # Return Value
    ///
    /// Always returns zero.
    #[nid(0x090CCB3F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelPowerTick(tick_kind: PowerTick) -> u32;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceSuspendForKernel", flags = 0x0009, use_crate)]
unsafe extern "C" {
    /// Locks power state of the device.
    ///
    /// That means that device power off is delayed until the power lock is unlocked.
    ///
    /// # Parameters
    ///
    /// - `lock_kind`: The power processing lock kind.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Precautions
    ///
    /// The power lock should always be unlocked ([`sceKernelPowerUnlock`]) after processing
    /// whatever important thing you needed, because it blocks power off even with physical buttons.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(0xEADB1BD7)]
    pub safe fn sceKernelPowerLock(lock_kind: PowerLockKind) -> SceResult<()>;

    /// Unlocks power state of the device.
    ///
    /// # Parameters
    ///
    /// - `lock_kind`: The power processing lock kind.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Precautions
    ///
    /// Calling this function without calling [`sceKernelPowerLock`] can mess up cases of power lock
    /// nesting.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(0x3AEE7261)]
    pub safe fn sceKernelPowerUnlock(lock_kind: PowerLockKind) -> SceResult<()>;

    /// Locks and grants access to the device volatile memory (blocking).
    ///
    /// If the volatile memory is currently being used by other processes, the process that called
    /// this function will enter a wait state until the volatile stops being used.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Always zero on reversed code.
    /// - `ptr`: A pointer to receive the head pointer of the volatile memory.
    /// - `size`: A reference to receive the size of the volatile memory.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Precautions
    ///
    /// The volatile memory is used by the PSP system for the utility library and to save eDRAM
    /// during suspend/resume time and those will be blocked while this memory is being used by your
    /// software. For that reason, it is best to unlock (with [`sceKernelVolatileMemUnlock`]) as
    /// soon as you done with using that piece of memory.
    ///
    /// If you need to use this memory for long periods and want to maintain such functionalities
    /// working, you, theoretically, can [create] and [register] a power callback that handles
    /// values of the `arg` parameter of [`CallbackFunction`] (namely [`PowerCallbackArg::Standby`]
    /// and [`PowerCallbackArg::Suspending`] to unlock and [`PowerCallbackArg::ResumeComplete`] ro
    /// lock again).
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    ///
    /// [create]: crate::sys::thread::sceKernelCreateCallback
    /// [register]: crate::sys::power::scePowerRegisterCallback
    /// [`CallbackFunction`]: crate::sys::thread::CallbackFunction
    /// [`PowerCallbackArg::Standby`]: crate::sys::power::PowerCallbackArg::Standby
    /// [`PowerCallbackArg::Suspending`]: crate::sys::power::PowerCallbackArg::Suspending
    /// [`PowerCallbackArg::ResumeComplete`]: crate::sys::power::PowerCallbackArg::ResumeComplete
    #[psp_fw_cfg(150..)]
    #[nid(0x3E0271D3)]
    pub unsafe fn sceKernelVolatileMemLock(
        unk: u32, ptr: *mut *mut u8, size: &mut SceSize,
    ) -> SceResult<()>;

    /// Locks and grants access to the device volatile memory (non-blocking).
    ///
    /// If the volatile memory is currently being used by other processes, this function will return
    /// a error.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Always zero on reversed code.
    /// - `ptr`: A pointer to receive the head pointer of the volatile memory.
    /// - `size`: A reference to receive the size of the volatile memory.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// Specifically [`SceError::POWER_CANNOT_LOCK_VMEM`] when volatile memory is in use by other
    /// processes.
    ///
    /// # Precautions
    ///
    /// The volatile memory is used by the PSP system for the utility library and to save eDRAM
    /// during suspend/resume time and those will be blocked while this memory is being used by your
    /// software. For that reason, it is best to unlock (with [`sceKernelVolatileMemUnlock`]) as
    /// soon as you done with using that piece of memory.
    ///
    /// If you need to use this memory for long periods and want to maintain such functionalities
    /// working, you, theoretically, can [create] and [register] a power callback that handles
    /// values of the `arg` parameter of [`CallbackFunction`] (namely [`PowerCallbackArg::Standby`]
    /// and [`PowerCallbackArg::Suspending`] to unlock and [`PowerCallbackArg::ResumeComplete`] to
    /// lock again).
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    ///
    /// [create]: crate::sys::thread::sceKernelCreateCallback
    /// [register]: crate::sys::power::scePowerRegisterCallback
    /// [`CallbackFunction`]: crate::sys::thread::CallbackFunction
    /// [`SceError::POWER_CANNOT_LOCK_VMEM`]: crate::sys::SceError::POWER_CANNOT_LOCK_VMEM
    /// [`PowerCallbackArg::Standby`]: crate::sys::power::PowerCallbackArg::Standby
    /// [`PowerCallbackArg::Suspending`]: crate::sys::power::PowerCallbackArg::Suspending
    /// [`PowerCallbackArg::ResumeComplete`]: crate::sys::power::PowerCallbackArg::ResumeComplete
    #[psp_fw_cfg(150..)]
    #[nid(0xA14F40B2)]
    pub unsafe fn sceKernelVolatileMemTryLock(
        unk: u32, ptr: *mut *mut u8, size: &mut SceSize,
    ) -> SceResult<()>;

    /// Unlock access to the device volatile memory.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Always zero on reversed code.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// Once unlocked, access to the volatile memory to given pointer is no longer valid.
    ///
    /// # Precautions
    ///
    /// This function should not be called without having a previous successful call to either
    /// [`sceKernelVolatileMemLock`] or [`sceKernelVolatileMemTryLock`].
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xA569E425)]
    pub unsafe fn sceKernelVolatileMemUnlock(unk: u32) -> SceResult<()>;

    /// Configures the system to cancel the count of idle timers to prevent the system to enter in
    /// power save state (entirely or partial).
    ///
    /// # Parameters
    ///
    /// - `tick_kind`: The configuration of what timer counts to cancel.
    ///
    /// # Return Value
    ///
    /// Always returns zero.
    #[nid(0x090CCB3F)]
    pub safe fn sceKernelPowerTick(tick_kind: PowerTick) -> u32;
}
