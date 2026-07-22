use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{thread::CallbackId, SceError, SceIntoOkValue, SceResult, SceResultOk, SceSize};

/// Bitflags that are passed to the `arg` parameter of [`CallbackFunction`]
/// when registered with [`scePowerRegisterCallback`].
///
/// [`CallbackFunction`]: crate::sys::thread::CallbackFunction
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum PowerCallbackArg {
    /// Indicates the power switch is pushed, putting the unit into suspend mode.
    #[doc(alias("PSP_POWER_CB_POWER_SWITCH"))]
    PowerSwitch = 0x80000000,
    /// Indicates the hold switch is on.
    #[doc(alias("PSP_POWER_CB_HOLD_SWITCH"))]
    HoldSwitch = 0x40000000,
    /// Indicates the system is entering standby mode.
    #[doc(alias("PSP_POWER_CB_STANDBY"))]
    Standby = 0x00080000,
    /// Indicates the resume process has been completed.
    #[doc(alias("PSP_POWER_CB_RESUME_COMPLETE"))]
    ResumeComplete = 0x00040000,
    /// Indicates the system is resuming from suspend/standby mode.
    #[doc(alias("PSP_POWER_CB_RESUMING"))]
    Resuming = 0x00020000,
    /// Indicates the system is suspending, seems to occur due to inactivity.
    #[doc(alias("PSP_POWER_CB_SUSPENDING"))]
    Suspending = 0x00010000,
    /// Indicates the system is plugged into an AC outlet.
    #[doc(alias("PSP_POWER_CB_AC_POWER"))]
    AcPower = 0x00001000,
    /// Indicates the battery charge level is low.
    #[doc(alias("PSP_POWER_CB_BATTERY_LOW"))]
    BatteryLow = 0x00000100,
    /// Indicates there is a battery present in the unit.
    #[doc(alias("PSP_POWER_CB_BATTERY_EXIST"))]
    BatteryExist = 0x00000080,
    /// Indicates that the battery charge percentage status changed.
    #[doc(alias("PSP_POWER_CB_BATTPOWER"))]
    BatteryCharge = 0x0000007F,
}

/// Valid power callback slot.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PowerCallbackSlot(u32);

/// Represents the device support of the maximum PLL clock frequency while WLAN is in use.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum WlanCoexistenceClock {
    /// Device supports maximum of 222 MHz of the PLL clock frequency while WLAN is in use.
    ///
    /// Only 01g models have this limitation.
    MaxClock222MHz = 0,
    /// Device supports maximum of 333 MHz of the PLL clock frequency while WLAN is in use.
    ///
    /// Devices that returns this also support 266 MHz.
    ///
    /// All models after 01g model have support for this.
    MaxClock333MHz = 1,
}

/// Possible power lock values.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum PowerLockKind {
    /// A power lock that locks all timers.
    #[default]
    All = 0,
    /// A power lock that locks only auto-suspend related timers.
    ///
    /// I.e. prevents the system from suspending.
    SuspendOnly = 1,
    /// A power lock that locks only display related timers.
    ///
    /// I.e. prevents the system display from turning off.
    DisplayOnly = 6,
}

/// Possible power tick configurations.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum PowerTick {
    /// A power tick that cancels all timers.
    #[default]
    All = 0,
    /// A power tick that cancels only auto-suspend related timers.
    ///
    /// I.e. prevents the system from suspending.
    SuspendOnly = 1,
    /// A power tick that cancels only display related timers.
    ///
    /// I.e. prevents the system display from turning off.
    DisplayOnly = 6,
}

/// Represents the possible exclusive WLAN modes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum ExclusiveWlanMode {
    /// Unrestricted PLL clock to activate WLAN.
    #[default]
    UnrestrictedMode = 0,
    /// The PLL clock to activate WLAN must be below 222 MHz.
    LowPowerMode = 1,
    /// The PLL clock to activate WLAN must be below 266 MHz.
    MediumPowerMode = 2,
}


// FIXME: Add missing known functions (most requires reversing)
//
// - `scePowerGetCallbackMode`
// - `scePowerGetPowerSwMode`
// - `scePowerGetResumeCount`
// - `scePowerGetWlanActivity`
// - `scePowerGetInnerTemp`
// - `scePowerRequestSuspendTouchAndGo`
// - `scePowerSetCallbackMode`
// - `scePowerSetPowerSwMode`
#[psp_stub(libname = "scePower", flags = 0x4001, use_crate)]
extern "C" {
    /// Checks if power is supplied from external power source.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the console is plugged in external source, or `false` if not,
    /// error value otherwise.
    #[nid(0x87440F5E)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIsPowerOnline() -> SceResult<bool>;

    /// Checks if the battery charge level is low.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the battery charge level is low, or `false` if not,
    /// error value otherwise.
    #[nid(0xD3075926)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIsLowBattery() -> SceResult<bool>;

    /// Checks if a battery is present.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the battery is present, or `false` if not,
    /// error value otherwise.
    #[nid(0x0AFD0D8B)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIsBatteryExist() -> SceResult<bool>;

    /// Checks if the battery is charging.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the battery is charging, or `false` if not,
    /// error value otherwise.
    #[nid(0x1E490401)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIsBatteryCharging() -> SceResult<bool>;

    /// Gets the remaining battery life in percent.
    ///
    /// # Return Value
    ///
    /// Returns the battery life percentage (`[0,100]`), error value otherwise.
    #[nid(0x2085D15D)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryLifePercent() -> SceResult<u8>;

    /// Gets the remaining battery life in minutes.
    ///
    /// # Return Value
    ///
    /// Returns the remaining battery life time in minutes, error value otherwise.
    ///
    /// It always return zero if power is being supplied by a external power source.
    #[nid(0x8EFB3FA2)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryLifeTime() -> SceResult<u32>;

    /// Request the system to go into suspend mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xAC32C9CC)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerRequestSuspend() -> SceResult<()>;

    /// Request the system to go into standby mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x2B7C7CF4)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerRequestStandby() -> SceResult<()>;

    /// Register Power callback function.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot to register the callback function. Use [`PowerCallbackSlot::AVAILABLE`]
    ///   to register to a available slot and receive the slot value back.
    /// - `callback_id`: The callback ID from calling
    ///   [`sceKernelCreateCallback`](crate::sys::thread::sceKernelCreateCallback).
    ///
    /// # Return Value
    ///
    /// If given `slot` is [`PowerCallbackSlot::AVAILABLE`], returns the slot the callback was
    /// registered on success, an error value otherwise.
    ///
    /// If given `slot` is a slot number, returns [`PowerCallbackSlot::ZERO`] on success, an error
    /// value otherwise.
    #[nid(0x04B7766E)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerRegisterCallback(
        slot: PowerCallbackSlot, callback_id: CallbackId,
    ) -> SceResult<PowerCallbackSlot>;

    /// Unregisters Power callback function.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot to unregister the callback function.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDFA8BAF8)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerUnregisterCallback(slot: PowerCallbackSlot) -> SceResult<()>;

    /// Gets what is the maximum frequency the PLL while WLAN is in use.
    ///
    /// # Return Value
    ///
    /// Returns, on success, [`WlanCoexistenceClock::MaxClock222MHz`] if the max is 222 MHz (01g
    /// model), or [`WlanCoexistenceClock::MaxClock333MHz`] if max is 333 MHz, error value
    /// otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 6.30.
    #[nid(0xA85880D0)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerCheckWlanCoexistenceClock() -> SceResult<WlanCoexistenceClock>;

    /// Sets the clock frequencies.
    ///
    /// # Parameter
    ///
    /// - `pll_clock`: The PLL frequency to set, in MHz. Valid values: `[190,333]`.
    /// - `cpu_clock`: The CPU frequency to set, in MHz. Valid values: `[1,333]`.
    /// - `bus_clock`: The BUS frequency to set, in MHz. Valid values: `[1,166]`.
    ///
    /// Also, `cpu_clock <= pll_clock` and `bus_clock * 2 <= pll_clock`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x737486F2)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerSetClockFrequency(
        pll_clock: u32, cpu_clock: u32, bus_clock: u32,
    ) -> SceResult<()>;

    // Gets the current PLL clock in MHz.
    /// # Return Value
    ///
    /// Returns the current PLL clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(0x34F9C463)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetPllClockFrequencyInt() -> u32;

    /// Gets the current PLL clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current PLL clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(0xEA382A27)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetPllClockFrequencyFloat() -> u32;

    /// Sets the CPU clock.
    ///
    /// # Parameters
    ///
    /// - `clock`: The CPU frequency to set, in MHz. Valid values: `[1,333]`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x843FBF43)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerSetCpuClockFrequency(clock: u32) -> SceResult<()>;

    /// Gets the current CPU clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current CPU clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xFDB5BFE9)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetCpuClockFrequencyInt() -> u32;

    /// Gets the current CPU clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current CPU clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xB1A52C83)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetCpuClockFrequencyFloat() -> f32;

    /// Sets the BUS clock.
    ///
    /// # Parameters
    ///
    /// - `clock`: The BUS frequency to set, in MHz. Valid values: `[1,166]`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB8D7B3FB)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerSetBusClockFrequency(clock: u32) -> SceResult<()>;

    /// Gets the current BUS clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current BUS clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xBD681969)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBusClockFrequencyInt() -> u32;

    /// Gets the current BUS clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current BUS clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0x9BADB3EB)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBusClockFrequencyFloat() -> f32;

    /// Waits for the completion of the previous request.
    ///
    /// Between the request for a operation and its execution there is a "natural" delay time. With
    /// this function you can stop the thread execution and wait for the request execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3951AF53)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerWaitRequestCompletion() -> SceResult<()>;

    /// Checks if a power request is uncompleted.
    ///
    /// # Return Value
    ///
    /// Returns `true` if there is a uncompleted power request, 'false' otherwise.
    #[nid(0x7FA406DD)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIsRequest() -> bool;

    /// Gets the current maximum backlight level.
    ///
    /// # Return Value
    ///
    /// Returns the current maximum backlight level.
    ///
    /// Usually `3` when on battery and `4` on AC power, but may be different between released
    /// models.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0x442BFBAC)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBacklightMaximum() -> u8;

    /// Checks if a suspend is required.
    ///
    /// # Return Value
    ///
    /// Returns `true` if suspend is required, `false` otherwise.
    #[nid(0x78A1A796)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIsSuspendRequired() -> bool;

    /// Gets the status of the battery charging
    ///
    /// # Return Value
    ///
    /// Returns the status of the battery charging on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xB4432BC8)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryChargingStatus() -> SceResult<u32>;

    /// Gets the battery remaining capacity in milliampere hour (mAh).
    ///
    /// # Return Value
    ///
    /// Returns the battery remaining capacity on success, error value otherwise.
    #[nid(0x94F5A53F)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryRemainCapacity() -> SceResult<u32>;

    /// Gets the battery full capacity in milliampere hour (mAh).
    ///
    /// # Return Value
    ///
    /// Returns the battery remaining capacity on success, error value otherwise.
    #[nid(0xFD18A0FF)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryFullCapacity() -> SceResult<u32>;

    /// Gets a unknown battery information.
    ///
    /// # Return Value
    ///
    /// Returns a unknown battery info on success, error value otherwise.
    #[nid(0x862AE1A6)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryElec() -> SceResult<u32>;

    /// Gets the battery voltage level.
    ///
    /// # Return Value
    ///
    /// Returns the battery voltage level on success, error value otherwise.
    #[nid(0x483CE86B)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetBatteryVolt() -> SceResult<u32>;

    /// Gets the idle timer.
    ///
    /// # Return Value
    ///
    /// Returns the idle timer on success, error value otherwise.
    #[nid(0xEDC13FE5)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerGetIdleTimer() -> SceResult<u32>;

    /// Enables the idle timer.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Pass `0`.
    ///
    /// # Return Value
    ///
    /// Returns a unknown value on success, error value otherwise.
    #[nid(0x7F30B3B1)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIdleTimerEnable(unk: u32) -> SceResult<u32>;

    /// Disables the idle timer.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Pass `0`.
    ///
    /// # Return Value
    ///
    /// Returns a unknown value on success, error value otherwise.
    #[nid(0x972CE941)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerIdleTimerDisable(unk: u32) -> SceResult<u32>;

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
    /// The power lock should always be unlocked ([`scePowerUnlock`]) after processing
    /// whatever important thing you needed, because it blocks power off even with physical buttons.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(0xD6D016EF)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerLock(lock_kind: PowerLockKind) -> SceResult<()>;

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
    /// Calling this function without calling [`scePowerLock`] can mess up cases of power lock
    /// nesting.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(0xCA3D34C1)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerUnlock(lock_kind: PowerLockKind) -> SceResult<()>;

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
    /// software. For that reason, it is best to unlock (with [`scePowerVolatileMemUnlock`]) as
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
    #[nid(0x23C31FFE)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn scePowerVolatileMemLock(
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
    /// software. For that reason, it is best to unlock (with [`scePowerVolatileMemUnlock`]) as
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
    #[nid(0xFA97A599)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn scePowerVolatileMemTryLock(
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
    /// [`scePowerVolatileMemLock`] or [`scePowerVolatileMemTryLock`].
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(0xB3EDD801)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn scePowerVolatileMemUnlock(unk: u32) -> SceResult<()>;

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
    #[nid(0xEFD3C963)]
    #[cfg(not(feature = "kernel"))]
    pub fn scePowerTick(tick_kind: PowerTick) -> u32;
}

// FIXME: Add missing known functions (most requires reversing)
//
// - `scePowerBatteryDisableUsbCharging`
// - `scePowerBatteryEnableUsbCharging`
// - `scePowerBatteryForbidCharging`
// - `scePowerBatteryPermitCharging`
// - `scePowerEnd`
// - `scePowerGetBatteryType`
// - `scePowerGetCurrentDdrStrength`
// - `scePowerGetCurrentDdrVoltage`
// - `scePowerGetCurrentTachyonVoltage`
// - `scePowerGetDdrStrength`
// - `scePowerGetDdrVoltage`
// - `scePowerGetGeEdramRefreshMode`
// - `scePowerGetLedOffTiming`
// - `scePowerGetTachyonVoltage`
// - `scePowerGetUsbChargingCapability`
// - `scePowerGetWatchDog`
// - `scePowerInit`
// - `scePowerLimitPllClock`
// - `scePowerLimitScBusClock`
// - `scePowerLimitScCpuClock`
// - `scePowerRebootStart`
// - `scePowerSetDdrStrength`
// - `scePowerSetDdrVoltage`
// - `scePowerSetGeEdramRefreshMode`
// - `scePowerSetIdleCallback`
// - `scePowerSetPllUseMask`
// - `scePowerSetTachyonVoltage`
// - `scePowerSetWakeupCondition`
//
// SAME AS USER:
//
// - `scePowerGetCallbackMode`
// - `scePowerGetPowerSwMode`
// - `scePowerGetResumeCount`
// - `scePowerGetWlanActivity`
// - `scePowerGetInnerTemp`
// - `scePowerRequestSuspendTouchAndGo`
// - `scePowerSetCallbackMode`
// - `scePowerSetPowerSwMode`
#[cfg(feature = "kernel")]
#[psp_stub(libname = "scePower_driver", flags = 0x0001, use_crate)]
extern "C" {
    /// Checks if power is supplied from external power source.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the console is plugged in external source, or `false` if not,
    /// error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x872F4ECE }
        else if cfg!(feature = "psp_630") { 0xF8C9FAF5 }
        else if cfg!(feature = "psp_600") { 0x7F480684 }
        else if cfg!(feature = "psp_570") { 0xC6508E38 }
        else if cfg!(feature = "psp_500") { 0xD59CD29D }
        else if cfg!(feature = "psp_420") { 0xF06C32FC }
        else if cfg!(feature = "psp_395") { 0xC6D21BB6 }
        else if cfg!(feature = "psp_380") { 0xAC3E6CCF }
        else if cfg!(feature = "psp_370") { 0x86795186 }
        else { 0x87440F5E }
    )]
    pub fn scePowerIsPowerOnline() -> SceResult<bool>;

    /// Checks if the battery charge level is low.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the battery charge level is low, or `false` if not,
    /// error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xFA651CE1 }
        else if cfg!(feature = "psp_630") { 0xDE18E7C0 }
        else if cfg!(feature = "psp_600") { 0x279492B3 }
        else if cfg!(feature = "psp_570") { 0xF5E65968 }
        else if cfg!(feature = "psp_500") { 0x0D4E4569 }
        else if cfg!(feature = "psp_420") { 0x2400D1FE }
        else if cfg!(feature = "psp_395") { 0x45BB59FE }
        else if cfg!(feature = "psp_380") { 0x30D2EE6D }
        else if cfg!(feature = "psp_370") { 0xE7A7ACE1 }
        else { 0xD3075926 }
    )]
    pub fn scePowerIsLowBattery() -> SceResult<bool>;

    /// Checks if a battery is present.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the battery is present, or `false` if not,
    /// error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x8C873AA7 }
        else if cfg!(feature = "psp_630") { 0x81DCA5A5 }
        else if cfg!(feature = "psp_600") { 0xBC8823E8 }
        else if cfg!(feature = "psp_570") { 0xAAD174A8 }
        else if cfg!(feature = "psp_500") { 0x1A4EC14D }
        else if cfg!(feature = "psp_420") { 0xF0C2427D }
        else if cfg!(feature = "psp_395") { 0xFBFD57EB }
        else if cfg!(feature = "psp_380") { 0x9FC87DE9 }
        else if cfg!(feature = "psp_370") { 0x54A35829 }
        else { 0x0AFD0D8B }
    )]
    pub fn scePowerIsBatteryExist() -> SceResult<bool>;

    /// Checks if the battery is charging.
    ///
    /// # Return Value
    ///
    /// Returns on success `true` if the battery is charging, or `false` if not,
    /// error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x7A9EA6DE }
        else if cfg!(feature = "psp_630") { 0xAB842949 }
        else if cfg!(feature = "psp_600") { 0x5202A826 }
        else if cfg!(feature = "psp_570") { 0x27B07A6B }
        else if cfg!(feature = "psp_500") { 0x0AFD8E3A }
        else if cfg!(feature = "psp_420") { 0x9E3BE55C }
        else if cfg!(feature = "psp_395") { 0xD61C63BD }
        else if cfg!(feature = "psp_380") { 0x0C4922CF }
        else if cfg!(feature = "psp_370") { 0x3C5E45D8 }
        else { 0x1E490401 }
    )]
    pub fn scePowerIsBatteryCharging() -> SceResult<bool>;

    /// Gets the remaining battery life in percent.
    ///
    /// # Return Value
    ///
    /// Returns the battery life percentage (`[0,100]`), error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x31AEA94C }
        else if cfg!(feature = "psp_630") { 0x2CBFA597 }
        else if cfg!(feature = "psp_600") { 0xE5F8596E }
        else if cfg!(feature = "psp_570") { 0x9D6E86BC }
        else if cfg!(feature = "psp_500") { 0xE2552927 }
        else if cfg!(feature = "psp_420") { 0x501255B6 }
        else if cfg!(feature = "psp_395") { 0xAC664491 }
        else if cfg!(feature = "psp_380") { 0xE8F681DB }
        else if cfg!(feature = "psp_370") { 0x9C98446E }
        else { 0x2085D15D }
    )]
    pub fn scePowerGetBatteryLifePercent() -> SceResult<u8>;

    /// Gets the remaining battery life in minutes.
    ///
    /// # Return Value
    ///
    /// Returns the remaining battery life time in minutes, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xC79F9157 }
        else if cfg!(feature = "psp_630") { 0xC8749D1A }
        else if cfg!(feature = "psp_600") { 0xB3C30947 }
        else if cfg!(feature = "psp_570") { 0xBAE3FAFE }
        else if cfg!(feature = "psp_500") { 0xEAB997D5 }
        else if cfg!(feature = "psp_420") { 0x9E58B343 }
        else if cfg!(feature = "psp_395") { 0x9470D652 }
        else if cfg!(feature = "psp_380") { 0xDBCCC13D }
        else if cfg!(feature = "psp_370") { 0x39C5677E }
        else { 0x8EFB3FA2 }
    )]
    pub fn scePowerGetBatteryLifeTime() -> SceResult<u32>;

    /// Request the system to go into suspend mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x5C1333B7 }
        else if cfg!(feature = "psp_630") { 0x062CFDDC }
        else if cfg!(feature = "psp_600") { 0xC9885394 }
        else if cfg!(feature = "psp_570") { 0xE252DAD5 }
        else if cfg!(feature = "psp_500") { 0xED6605C6 }
        else if cfg!(feature = "psp_420") { 0x0832C709 }
        else if cfg!(feature = "psp_395") { 0xCF0F53E5 }
        else if cfg!(feature = "psp_380") { 0xE1837FC2 }
        else if cfg!(feature = "psp_370") { 0xA8D09A9A }
        else { 0xAC32C9CC }
    )]
    pub fn scePowerRequestSuspend() -> SceResult<()>;

    /// Request the system to go into standby mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x9B44CFD9 }
        else if cfg!(feature = "psp_630") { 0x334539BA }
        else if cfg!(feature = "psp_600") { 0x47F4E1A8 }
        else if cfg!(feature = "psp_570") { 0x053D6197 }
        else if cfg!(feature = "psp_500") { 0x47CDDC1A }
        else if cfg!(feature = "psp_420") { 0x82CD5D7B }
        else if cfg!(feature = "psp_395") { 0xD55A517C }
        else if cfg!(feature = "psp_380") { 0xD90994B9 }
        else if cfg!(feature = "psp_370") { 0x9B1A9C5F }
        else { 0x2B7C7CF4 }
    )]
    pub fn scePowerRequestStandby() -> SceResult<()>;

    /// Register Power callback function.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot to register the callback function. Use [`PowerCallbackSlot::AVAILABLE`]
    ///   to register to a available slot and receive the slot value back.
    /// - `callback_id`: The callback ID from calling
    ///   [`sceKernelCreateCallback`](crate::sys::thread::sceKernelCreateCallback).
    ///
    /// # Return Value
    ///
    /// If given `slot` is [`PowerCallbackSlot::AVAILABLE`], returns the slot the callback was
    /// registered on success, an error value otherwise.
    ///
    /// If given `slot` is a slot number, returns [`PowerCallbackSlot::ZERO`] on success, an error
    /// value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x766CD857 }
        else if cfg!(feature = "psp_630") { 0x8BA12BAF }
        else if cfg!(feature = "psp_600") { 0x1A41E0ED }
        else if cfg!(feature = "psp_570") { 0xCA24FA05 }
        else if cfg!(feature = "psp_500") { 0xAF23EFA6 }
        else if cfg!(feature = "psp_420") { 0x355FABF7 }
        else if cfg!(feature = "psp_395") { 0x19387EF4 }
        else if cfg!(feature = "psp_380") { 0x94BC4918 }
        else if cfg!(feature = "psp_370") { 0xD6E50D7B }
        else { 0x04B7766E }
    )]
    pub fn scePowerRegisterCallback(
        slot: PowerCallbackSlot, callback_id: CallbackId,
    ) -> SceResult<PowerCallbackSlot>;

    /// Unregisters Power callback function.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot to unregister the callback function.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x315B8CB6 }
        else if cfg!(feature = "psp_630") { 0xE51B6170 }
        else if cfg!(feature = "psp_600") { 0xCE5D389B }
        else if cfg!(feature = "psp_570") { 0x8C896E9C }
        else if cfg!(feature = "psp_500") { 0x5EE66C30 }
        else if cfg!(feature = "psp_420") { 0x4289FA47 }
        else if cfg!(feature = "psp_395") { 0x809F7127 }
        else if cfg!(feature = "psp_380") { 0x3D635CAC }
        else if cfg!(feature = "psp_370") { 0xDBBC8820 }
        else { 0xDFA8BAF8 }
    )]
    pub fn scePowerUnregisterCallback(slot: PowerCallbackSlot) -> SceResult<()>;

    /// Gets what is the maximum frequency the PLL while WLAN is in use.
    ///
    /// # Return Value
    ///
    /// Returns, on success, [`WlanCoexistenceClock::MaxClock222MHz`] if the max is 222 MHz (01g
    /// model), or [`WlanCoexistenceClock::MaxClock333MHz`] if max is 333 MHz, error value
    /// otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 6.30.
    #[nid(if cfg!(feature = "psp_660") { 0x693F6CF0 } else { 0x330AC84F })]
    pub fn scePowerCheckWlanCoexistenceClock() -> SceResult<WlanCoexistenceClock>;

    /// Sets the clock frequencies.
    ///
    /// # Parameter
    ///
    /// - `pll_clock`: The PLL frequency to set, in MHz. Valid values: `[19,333]`.
    /// - `cpu_clock`: The CPU frequency to set, in MHz. Valid values: `[1,333]`.
    /// - `bus_clock`: The BUS frequency to set, in MHz. Valid values: `[1,166]`.
    ///
    /// Also, `cpu_clock <= pll_clock` and `bus_clock * 2 <= pll_clock`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x469989AD }
        else if cfg!(feature = "psp_630") { 0x469989AD }
        else if cfg!(feature = "psp_600") { 0xEBD177D6 }
        else if cfg!(feature = "psp_570") { 0xEBD177D6 }
        else if cfg!(feature = "psp_500") { 0xEBD177D6 }
        else if cfg!(feature = "psp_420") { 0xEBD177D6 }
        else if cfg!(feature = "psp_395") { 0xEBD177D6 }
        else if cfg!(feature = "psp_380") { 0xEBD177D6 }
        else if cfg!(feature = "psp_370") { 0xEBD177D6 }
        else { 0x737486F2 }
    )]
    pub fn scePowerSetClockFrequency(
        pll_clock: u32, cpu_clock: u32, bus_clock: u32,
    ) -> SceResult<()>;

    /// Gets the current PLL clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current PLL clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(if cfg!(feature = "psp_660") { 0x67BD889B }
        else if cfg!(feature = "psp_630") { 0xDCC6E49B }
        else if cfg!(feature = "psp_600") { 0xBA93F79B }
        else if cfg!(feature = "psp_570") { 0xEF866AF8 }
        else if cfg!(feature = "psp_500") { 0x709CF70B }
        else if cfg!(feature = "psp_420") { 0xD54FEC9E }
        else if cfg!(feature = "psp_395") { 0x2A12371B }
        else if cfg!(feature = "psp_380") { 0xC3C33306 }
        else if cfg!(feature = "psp_370") { 0x93444D33 }
        else { 0x34F9C463 }
    )]
    pub fn scePowerGetPllClockFrequencyInt() -> u32;

    /// Gets the current PLL clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current PLL clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(if cfg!(feature = "psp_660") { 0xBA8CBCBF }
        else if cfg!(feature = "psp_630") { 0xCC998F67 }
        else if cfg!(feature = "psp_600") { 0x279FD567 }
        else if cfg!(feature = "psp_570") { 0xF70847A4 }
        else if cfg!(feature = "psp_500") { 0x3A821D92 }
        else if cfg!(feature = "psp_420") { 0xA6E22F84 }
        else if cfg!(feature = "psp_395") { 0xC874B21D }
        else if cfg!(feature = "psp_380") { 0x5FC2494F }
        else if cfg!(feature = "psp_370") { 0xBF1DA143 }
        else { 0xEA382A27 }
    )]
    pub fn scePowerGetPllClockFrequencyFloat() -> f32;

    /// Sets the CPU clock.
    ///
    /// # Parameters
    ///
    /// - `clock`: The CPU frequency to set, in MHz. Valid values: `[1,333]`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x53808CBB }
        else if cfg!(feature = "psp_630") { 0x473DE4F3 }
        else if cfg!(feature = "psp_600") { 0x37DCC2DA }
        else if cfg!(feature = "psp_570") { 0x8BDCB4CB }
        else if cfg!(feature = "psp_500") { 0xD8F765CE }
        else if cfg!(feature = "psp_420") { 0xACE5B0F3 }
        else if cfg!(feature = "psp_395") { 0xBD02C252 }
        else if cfg!(feature = "psp_380") { 0x4219F46E }
        else if cfg!(feature = "psp_370") { 0x24085F5C }
        else { 0x843FBF43 }
    )]
    pub fn scePowerSetCpuClockFrequency(clock: u32) -> SceResult<()>;

    /// Gets the current CPU clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current CPU clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xFDB5BFE9)]
    pub fn scePowerGetCpuClockFrequencyInt() -> u32;

    /// Gets the current CPU clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current CPU clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0xDC4395E2 }
        else if cfg!(feature = "psp_630") { 0xCC998F67 }
        else if cfg!(feature = "psp_600") { 0x4CAE06EF }
        else if cfg!(feature = "psp_570") { 0xAA3E3459 }
        else if cfg!(feature = "psp_500") { 0x7BC03040 }
        else if cfg!(feature = "psp_420") { 0x026C506C }
        else if cfg!(feature = "psp_395") { 0xAE51C2CE }
        else if cfg!(feature = "psp_380") { 0x717662CE }
        else if cfg!(feature = "psp_370") { 0x17CB450B }
        else { 0xB1A52C83 }
    )]
    pub fn scePowerGetCpuClockFrequencyFloat() -> f32;

    /// Sets the BUS clock.
    ///
    /// # Parameters
    ///
    /// - `clock`: The BUS frequency to set, in MHz. Valid values: `[1,166]`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xB71A8B2F }
        else if cfg!(feature = "psp_630") { 0xBEA1E507 }
        else if cfg!(feature = "psp_600") { 0xC5371DAD }
        else if cfg!(feature = "psp_570") { 0xFE5950D2 }
        else if cfg!(feature = "psp_500") { 0x0EE576B9 }
        else if cfg!(feature = "psp_420") { 0x9FE11E62 }
        else if cfg!(feature = "psp_395") { 0xFFFD6435 }
        else if cfg!(feature = "psp_380") { 0x00646020 }
        else if cfg!(feature = "psp_370") { 0xAF70529A }
        else { 0xB8D7B3FB }
    )]
    pub fn scePowerSetBusClockFrequency(clock: u32) -> SceResult<()>;

    /// Gets the current BUS clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current BUS clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x04711DFB }
        else if cfg!(feature = "psp_630") { 0x9F53A71F }
        else if cfg!(feature = "psp_600") { 0xBF5BA7FC }
        else if cfg!(feature = "psp_570") { 0x5DCA9C95 }
        else if cfg!(feature = "psp_500") { 0x78834264 }
        else if cfg!(feature = "psp_420") { 0xD54FEC9E }
        else if cfg!(feature = "psp_395") { 0x2A12371B }
        else if cfg!(feature = "psp_380") { 0x94C3991E }
        else if cfg!(feature = "psp_370") { 0x1688935C }
        else { 0xBD681969 }
    )]
    pub fn scePowerGetBusClockFrequencyInt() -> u32;

    /// Gets the current BUS clock in MHz.
    ///
    /// # Return Value
    ///
    /// Returns the current BUS clock in MHz.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x1FF8DA3B }
        else if cfg!(feature = "psp_630") { 0xCC998F67 }
        else if cfg!(feature = "psp_600") { 0x279FD567 }
        else if cfg!(feature = "psp_570") { 0xF70847A4 }
        else if cfg!(feature = "psp_500") { 0x50EAAA0C }
        else if cfg!(feature = "psp_420") { 0xACE3DA65 }
        else if cfg!(feature = "psp_395") { 0x80EC83AD }
        else if cfg!(feature = "psp_380") { 0x2AA395A0 }
        else if cfg!(feature = "psp_370") { 0x073462C1 }
        else { 0x9BADB3EB }
    )]
    pub fn scePowerGetBusClockFrequencyFloat() -> f32;

    /// Waits for the completion of the previous request.
    ///
    /// Between the request for a operation and its execution there is a "natural" delay time. With
    /// this function you can stop the thread execution and wait for the request execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x3300D85A }
        else if cfg!(feature = "psp_630") { 0x20EC212D }
        else if cfg!(feature = "psp_600") { 0x39274F61 }
        else if cfg!(feature = "psp_570") { 0x261DFCB3 }
        else if cfg!(feature = "psp_500") { 0x171B798E }
        else if cfg!(feature = "psp_420") { 0x2150BDC8 }
        else if cfg!(feature = "psp_395") { 0xAE17B5E4 }
        else if cfg!(feature = "psp_380") { 0x35ADACC7 }
        else if cfg!(feature = "psp_370") { 0x7F64910E }
        else { 0x3951AF53 }
    )]
    pub fn scePowerWaitRequestCompletion() -> SceResult<()>;

    /// Checks if a power request is uncompleted.
    ///
    /// # Return Value
    ///
    /// Returns `true` if there is a uncompleted power request, 'false' otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x566B8353 }
        else if cfg!(feature = "psp_630") { 0x5B459B92 }
        else if cfg!(feature = "psp_600") { 0xE5C40F85 }
        else if cfg!(feature = "psp_570") { 0xF4B0257D }
        else if cfg!(feature = "psp_500") { 0x4D3FA315 }
        else if cfg!(feature = "psp_420") { 0xCB25ECDE }
        else if cfg!(feature = "psp_395") { 0x9C0C0375 }
        else if cfg!(feature = "psp_380") { 0xF55083D8 }
        else if cfg!(feature = "psp_370") { 0x6B5FD1CE }
        else { 0x7FA406DD }
    )]
    pub fn scePowerIsRequest() -> bool;

    /// Gets the current maximum backlight level.
    ///
    /// # Return Value
    ///
    /// Returns the current maximum backlight level.
    ///
    /// Usually `3` when on battery and `4` on AC power, but may be different between released
    /// models.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x2509FF3B }
        else if cfg!(feature = "psp_630") { 0x57F6311D }
        else if cfg!(feature = "psp_600") { 0x4084E678 }
        else if cfg!(feature = "psp_570") { 0x37DB24B5 }
        else if cfg!(feature = "psp_500") { 0xF39CEE67 }
        else if cfg!(feature = "psp_420") { 0x6C714980 }
        else if cfg!(feature = "psp_395") { 0xC35907C2 }
        else if cfg!(feature = "psp_380") { 0xED21B8EF }
        else if cfg!(feature = "psp_370") { 0xC87DEC73 }
        else { 0x442BFBAC }
    )]
    pub fn scePowerGetBacklightMaximum() -> u8;

    /// Checks if a suspend is required.
    ///
    /// # Return Value
    ///
    /// Returns `true` if suspend is required, `false` otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x88C79735 }
        else if cfg!(feature = "psp_630") { 0xF36E1F37 }
        else if cfg!(feature = "psp_600") { 0x9E09D19B }
        else if cfg!(feature = "psp_570") { 0x48CF237C }
        else if cfg!(feature = "psp_500") { 0x0146A026 }
        else if cfg!(feature = "psp_420") { 0xA468C844 }
        else if cfg!(feature = "psp_395") { 0x56083981 }
        else if cfg!(feature = "psp_380") { 0x28764591 }
        else if cfg!(feature = "psp_370") { 0x78EC8DC9 }
        else { 0x78A1A796 }
    )]
    pub fn scePowerIsSuspendRequired() -> bool;

    /// Gets the status of the battery charging
    ///
    /// # Return Value
    ///
    /// Returns the status of the battery charging on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x67492C52 }
        else if cfg!(feature = "psp_630") { 0x481F5556 }
        else if cfg!(feature = "psp_600") { 0x5D9E954F }
        else if cfg!(feature = "psp_570") { 0xD2B3ACAD }
        else if cfg!(feature = "psp_500") { 0x56040BF5 }
        else if cfg!(feature = "psp_420") { 0x93B17094 }
        else if cfg!(feature = "psp_395") { 0x8045DB6F }
        else if cfg!(feature = "psp_380") { 0x2DA84F14 }
        else if cfg!(feature = "psp_370") { 0x6ECBC5FF }
        else { 0xB4432BC8 }
    )]
    pub fn scePowerGetBatteryChargingStatus() -> SceResult<u32>;

    /// Gets the battery remaining capacity in milliampere hour (mAh).
    ///
    /// # Return Value
    ///
    /// Returns the battery remaining capacity on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x41ADFF48 }
        else if cfg!(feature = "psp_630") { 0x08EC482C }
        else if cfg!(feature = "psp_600") { 0xB6782EAE }
        else if cfg!(feature = "psp_570") { 0xEE26AD30 }
        else if cfg!(feature = "psp_500") { 0xD13E3897 }
        else if cfg!(feature = "psp_420") { 0x82CA2D3D }
        else if cfg!(feature = "psp_395") { 0x98123F07 }
        else if cfg!(feature = "psp_380") { 0xA7BAD8D5 }
        else if cfg!(feature = "psp_370") { 0xF3B7966A }
        else { 0x94F5A53F }
    )]
    pub fn scePowerGetBatteryRemainCapacity() -> SceResult<u32>;

    /// Gets the battery full capacity in milliampere hour (mAh).
    ///
    /// # Return Value
    ///
    /// Returns the battery remaining capacity on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x003B1E03 }
        else if cfg!(feature = "psp_630") { 0x266A2BFC }
        else if cfg!(feature = "psp_600") { 0x6E7E0056 }
        else if cfg!(feature = "psp_570") { 0x9E29D658 }
        else if cfg!(feature = "psp_500") { 0x1FBCF6E7 }
        else if cfg!(feature = "psp_420") { 0x7B765CB1 }
        else if cfg!(feature = "psp_395") { 0xE51256C7 }
        else if cfg!(feature = "psp_380") { 0x0384FCBC }
        else if cfg!(feature = "psp_370") { 0xFF6450C3 }
        else { 0xFD18A0FF }
    )]
    pub fn scePowerGetBatteryFullCapacity() -> SceResult<u32>;

    /// Gets the current temperature of the battery in degree celsius (°C).
    ///
    /// # Return Value
    ///
    /// Returns the current temperature of the battery on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x40870DAC }
        else if cfg!(feature = "psp_630") { 0xC8324682 }
        else if cfg!(feature = "psp_600") { 0xE66978D7 }
        else if cfg!(feature = "psp_570") { 0x29D27698 }
        else if cfg!(feature = "psp_500") { 0x6EF40205 }
        else if cfg!(feature = "psp_420") { 0x6D72E83E }
        else if cfg!(feature = "psp_395") { 0x2E91F188 }
        else if cfg!(feature = "psp_380") { 0x122EDE17 }
        else if cfg!(feature = "psp_370") { 0xFD607FCA }
        else { 0x28E12023 }
    )]
    pub fn scePowerGetBatteryTemp() -> SceResult<u32>;

    /// Gets a unknown battery information.
    ///
    /// # Return Value
    ///
    /// Returns a unknown battery info on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x993B8C4A }
        else if cfg!(feature = "psp_630") { 0xC3EC1F7D }
        else if cfg!(feature = "psp_600") { 0xDC73B503 }
        else if cfg!(feature = "psp_570") { 0x7658EE77 }
        else if cfg!(feature = "psp_500") { 0xE69051A1 }
        else if cfg!(feature = "psp_420") { 0x6F4A4F32 }
        else if cfg!(feature = "psp_395") { 0x85030AF1 }
        else if cfg!(feature = "psp_380") { 0x313034E2 }
        else if cfg!(feature = "psp_370") { 0x55D9ACC8 }
        else { 0x862AE1A6 }
    )]
    pub fn scePowerGetBatteryElec() -> SceResult<u32>;

    /// Gets the battery voltage level.
    ///
    /// # Return Value
    ///
    /// Returns the battery voltage level on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xF7DE0E81 }
        else if cfg!(feature = "psp_630") { 0x4A1118E1 }
        else if cfg!(feature = "psp_600") { 0x3C7B0B84 }
        else if cfg!(feature = "psp_570") { 0x8BF3F16C }
        else if cfg!(feature = "psp_500") { 0x4685386B }
        else if cfg!(feature = "psp_420") { 0x29D73076 }
        else if cfg!(feature = "psp_395") { 0xA6A56861 }
        else if cfg!(feature = "psp_380") { 0x6BD1323A }
        else if cfg!(feature = "psp_370") { 0xF393BD85 }
        else { 0x483CE86B }
    )]
    pub fn scePowerGetBatteryVolt() -> SceResult<u32>;

    /// Gets the idle timer.
    ///
    /// # Return Value
    ///
    /// Returns the idle timer on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xDF336CDE }
        else if cfg!(feature = "psp_630") { 0x3C0B1828 }
        else if cfg!(feature = "psp_600") { 0xB361215A }
        else if cfg!(feature = "psp_570") { 0x775EBCF8 }
        else if cfg!(feature = "psp_500") { 0xCA3F6145 }
        else if cfg!(feature = "psp_420") { 0xDD702664 }
        else if cfg!(feature = "psp_395") { 0xE2E766C2 }
        else if cfg!(feature = "psp_380") { 0xA587FEC1 }
        else if cfg!(feature = "psp_370") { 0x152C1C97 }
        else { 0xEDC13FE5 }
    )]
    pub fn scePowerGetIdleTimer() -> SceResult<u32>;

    /// Enables the idle timer.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Pass `0`.
    ///
    /// # Return Value
    ///
    /// Returns a unknown value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1E3B1FAE }
        else if cfg!(feature = "psp_630") { 0x25C15BDD }
        else if cfg!(feature = "psp_600") { 0x69513373 }
        else if cfg!(feature = "psp_570") { 0x8D6399AB }
        else if cfg!(feature = "psp_500") { 0x5E5CA9CA }
        else if cfg!(feature = "psp_420") { 0xA100053D }
        else if cfg!(feature = "psp_395") { 0xE660E488 }
        else if cfg!(feature = "psp_380") { 0xCF7E405A }
        else if cfg!(feature = "psp_370") { 0xD58A119A }
        else { 0x7F30B3B1 }
    )]
    pub fn scePowerIdleTimerEnable(unk: u32) -> SceResult<u32>;

    /// Disables the idle timer.
    ///
    /// # Parameters
    ///
    /// - `unk`: Unknown. Pass `0`.
    ///
    /// # Return Value
    ///
    /// Returns a unknown value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x961A06A5 }
        else if cfg!(feature = "psp_630") { 0x6751719C }
        else if cfg!(feature = "psp_600") { 0x21361196 }
        else if cfg!(feature = "psp_570") { 0x3646A163 }
        else if cfg!(feature = "psp_500") { 0x9E5115BA }
        else if cfg!(feature = "psp_420") { 0xB9DABCC1 }
        else if cfg!(feature = "psp_395") { 0xA7DBAEB6 }
        else if cfg!(feature = "psp_380") { 0x0DFC24A9 }
        else if cfg!(feature = "psp_370") { 0x30A2475B }
        else { 0x972CE941 }
    )]
    pub fn scePowerIdleTimerDisable(unk: u32) -> SceResult<u32>;

    /// Sets the exclusive WLAN mode before activating WLAN.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.50.
    #[nid(if cfg!(feature = "psp_660") { 0x114B75AB }
        else if cfg!(feature = "psp_630") { 0xA7308D7A }
        else if cfg!(feature = "psp_600") { 0x207119B3 }
        else if cfg!(feature = "psp_570") { 0x8CEB3BE2 }
        else if cfg!(feature = "psp_500") { 0x1920C0A0 }
        else if cfg!(feature = "psp_420") { 0x37F62422 }
        else if cfg!(feature = "psp_395") { 0xB97AAA61 }
        else if cfg!(feature = "psp_380") { 0x8E85B7D9 }
        else if cfg!(feature = "psp_370") { 0x208DBFFA }
        else { 0xC71EE866 }
    )]
    pub fn scePowerSetExclusiveWlan(mode: ExclusiveWlanMode) -> SceResult<()>;

    /// Activates the WLAN device.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x2638EF48 }
        else if cfg!(feature = "psp_630") { 0x6121DF53 }
        else if cfg!(feature = "psp_600") { 0xA8909AE1 }
        else if cfg!(feature = "psp_570") { 0x9D3E1FCF }
        else if cfg!(feature = "psp_500") { 0xCEDCFA91 }
        else if cfg!(feature = "psp_420") { 0x23DA6344 }
        else if cfg!(feature = "psp_395") { 0x56C02F81 }
        else if cfg!(feature = "psp_380") { 0xFC51835B }
        else if cfg!(feature = "psp_370") { 0x54624251 }
        else { 0x6D2CA84B }
    )]
    pub fn scePowerWlanActivate() -> SceResult<()>;

    /// Deactivates the WLAN device.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x8C6BEFD9 }
        else if cfg!(feature = "psp_630") { 0xBB9E6960 }
        else if cfg!(feature = "psp_600") { 0x5D311801 }
        else if cfg!(feature = "psp_570") { 0x9FBAEE3D }
        else if cfg!(feature = "psp_500") { 0xAB61C640 }
        else if cfg!(feature = "psp_420") { 0x4C5F1AC9 }
        else if cfg!(feature = "psp_395") { 0x459F9500 }
        else if cfg!(feature = "psp_380") { 0x90A7A370 }
        else if cfg!(feature = "psp_370") { 0x722D5D34 }
        else { 0x23BB0A60 }
    )]
    pub fn scePowerWlanDeactivate() -> SceResult<()>;

    /// Checks if the WLAN can be activated.
    ///
    /// # Parameters
    ///
    /// - `pll_clock`: The PLL clock to check WLAN activation validity.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE52B4362 }
        else if cfg!(feature = "psp_630") { 0xA2E9E73F }
        else if cfg!(feature = "psp_600") { 0x9CDB22B5 }
        else if cfg!(feature = "psp_570") { 0xF4E8428A }
        else if cfg!(feature = "psp_500") { 0x775DA498 }
        else if cfg!(feature = "psp_420") { 0xB63376C8 }
        else if cfg!(feature = "psp_395") { 0x9AF4290C }
        else if cfg!(feature = "psp_380") { 0x114D3560 }
        else if cfg!(feature = "psp_370") { 0x4431FF21 }
        else { 0xD66EF08D }
    )]
    pub fn scePowerCheckWlanCondition(pll_clock: u32) -> SceResult<()>;

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
    /// The power lock should always be unlocked ([`scePowerUnlock`]) after processing
    /// whatever important thing you needed, because it blocks power off even with physical buttons.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(if cfg!(feature = "psp_660") { 0x6CF50928 }
        else if cfg!(feature = "psp_630") { 0xBFC88E63 }
        else if cfg!(feature = "psp_600") { 0x534D471F }
        else if cfg!(feature = "psp_570") { 0xB6CBB82B }
        else if cfg!(feature = "psp_500") { 0xBF483F05 }
        else if cfg!(feature = "psp_420") { 0xA4FA407B }
        else if cfg!(feature = "psp_395") { 0x339FE4CF }
        else if cfg!(feature = "psp_380") { 0x20A56D51 }
        else if cfg!(feature = "psp_370") { 0x715A56FB }
        else { 0xD6D016EF }
    )]
    pub fn scePowerLock(lock_kind: PowerLockKind) -> SceResult<()>;

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
    /// Calling this function without calling [`scePowerLock`] can mess up cases of power lock
    /// nesting.
    ///
    /// This functions is not marked `unsafe`, because it is not memory unsafe operation.
    #[nid(if cfg!(feature = "psp_660") { 0xC3024FE6 }
        else if cfg!(feature = "psp_630") { 0x4274C154 }
        else if cfg!(feature = "psp_600") { 0xD80E403B }
        else if cfg!(feature = "psp_570") { 0x89435864 }
        else if cfg!(feature = "psp_500") { 0x9816FD74 }
        else if cfg!(feature = "psp_420") { 0x0AF80B18 }
        else if cfg!(feature = "psp_395") { 0x3F976B98 }
        else if cfg!(feature = "psp_380") { 0xF3765475 }
        else if cfg!(feature = "psp_370") { 0x6A985D34 }
        else { 0xCA3D34C1 }
    )]
    pub fn scePowerUnlock(lock_kind: PowerLockKind) -> SceResult<()>;

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
    /// software. For that reason, it is best to unlock (with [`scePowerVolatileMemUnlock`]) as
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
    #[nid(if cfg!(feature = "psp_660") { 0x70F42744 }
        else if cfg!(feature = "psp_630") { 0x503F08C9 }
        else if cfg!(feature = "psp_600") { 0x12FFFD34 }
        else if cfg!(feature = "psp_570") { 0x6A80F769 }
        else if cfg!(feature = "psp_500") { 0xE2AFADF3 }
        else if cfg!(feature = "psp_420") { 0x3051F343 }
        else if cfg!(feature = "psp_395") { 0xCE239543 }
        else if cfg!(feature = "psp_380") { 0x486982EF }
        else if cfg!(feature = "psp_370") { 0x5EBAD646 }
        else { 0x23C31FFE }
    )]
    pub unsafe fn scePowerVolatileMemLock(
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
    /// software. For that reason, it is best to unlock (with [`scePowerVolatileMemUnlock`]) as
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
    #[nid(if cfg!(feature = "psp_660") { 0xA882AEB7 }
        else if cfg!(feature = "psp_630") { 0x37DB9C37 }
        else if cfg!(feature = "psp_600") { 0x74035D33 }
        else if cfg!(feature = "psp_570") { 0x62FC6634 }
        else if cfg!(feature = "psp_500") { 0x66335AEB }
        else if cfg!(feature = "psp_420") { 0x015DEF2B }
        else if cfg!(feature = "psp_395") { 0x4272CD0C }
        else if cfg!(feature = "psp_380") { 0x41594CBA }
        else if cfg!(feature = "psp_370") { 0x1E4418AB }
        else { 0xFA97A599 }
    )]
    pub unsafe fn scePowerVolatileMemTryLock(
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
    /// [`scePowerVolatileMemLock`] or [`scePowerVolatileMemTryLock`].
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(if cfg!(feature = "psp_660") { 0x5978B1C2 }
        else if cfg!(feature = "psp_630") { 0x88D4244D }
        else if cfg!(feature = "psp_600") { 0x8E71E273 }
        else if cfg!(feature = "psp_570") { 0x189269CE }
        else if cfg!(feature = "psp_500") { 0x3B969585 }
        else if cfg!(feature = "psp_420") { 0x521FF26A }
        else if cfg!(feature = "psp_395") { 0x7CAB0E2E }
        else if cfg!(feature = "psp_380") { 0x3208DD65 }
        else if cfg!(feature = "psp_370") { 0x268731BE }
        else { 0xB3EDD801 }
    )]
    pub unsafe fn scePowerVolatileMemUnlock(unk: u32) -> SceResult<()>;

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
    #[nid(if cfg!(feature = "psp_660") { 0x0EFEE60E }
        else if cfg!(feature = "psp_630") { 0x6C4F9920 }
        else if cfg!(feature = "psp_600") { 0xE59EF335 }
        else if cfg!(feature = "psp_570") { 0x938995C8 }
        else if cfg!(feature = "psp_500") { 0xA40EC0F7 }
        else if cfg!(feature = "psp_420") { 0xE3D0F738 }
        else if cfg!(feature = "psp_395") { 0xFC61028C }
        else if cfg!(feature = "psp_380") { 0xB9F4D185 }
        else if cfg!(feature = "psp_370") { 0x9C40E184 }
        else { 0xEFD3C963 }
    )]
    pub fn scePowerTick(tick_kind: PowerTick) -> u32;
}

impl PowerCallbackSlot {
    /// Callback slot to register the callback on available slot and receive the slot value back on
    /// [`scePowerRegisterCallback`].
    pub const AVAILABLE: Self = unsafe { Self::new_unchecked(0xFFFFFFFF) };
    /// Callback slot zero that also can be returned if passed [`PowerCallbackSlot::AVAILABLE`] to
    /// [`scePowerRegisterCallback`] as success value.
    pub const ZERO: Self = unsafe { Self::new_unchecked(0) };

    /// Create a new channel number from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceChannel`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        match raw {
            0u32..=15 => Some(unsafe { Self::new_unchecked(raw) }),
            0xFFFFFFFF => Some(Self::AVAILABLE),
            _ => None,
        }
    }

    /// Create a new channel number structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for PowerCallbackSlot {}
unsafe impl SceResultOk for PowerCallbackSlot {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        // On result Channel is never 0xFFFFFFFF
        match ok_value {
            0..=15 => Ok(unsafe { Self::new_unchecked(ok_value) }),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for PowerCallbackSlot {
    fn into_ok_value(self) -> u32 {
        self.0
    }
}

impl crate::private::Sealed for WlanCoexistenceClock {}
unsafe impl SceResultOk for WlanCoexistenceClock {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0 => Ok(Self::MaxClock222MHz),
            1 => Ok(Self::MaxClock333MHz),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for WlanCoexistenceClock {
    fn into_ok_value(self) -> u32 {
        match self {
            WlanCoexistenceClock::MaxClock222MHz => 0,
            WlanCoexistenceClock::MaxClock333MHz => 1,
        }
    }
}
