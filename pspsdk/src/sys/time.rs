//! Time operation and management.

#[cfg(feature = "non-stub-code")]
mod non_stub;
#[cfg(feature = "non-stub-code")]
pub(crate) use non_stub::{Instant, SystemTime, UNIX_EPOCH};
use pspsdk_macros::{psp_fw_cfg, psp_stub};

use crate::sys::{thread::CallbackId, SceIntoOkValue, SceResult, SceResultOk};

/// The 64-bit system clock type.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias = "SceKernelSysClock")]
pub struct SystemClock {
    pub low: u32,
    pub hi: u32,
}

/// PSP Date Time structure.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias = "ScePspDateTime")]
pub struct DateTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minutes: u16,
    pub seconds: u16,
    pub microseconds: u32,
}

/// The possible month values
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Month {
    #[default]
    January = 1,
    February = 2,
    March  = 3,
    April  = 4,
    May    = 5,
    June   = 6,
    July   = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

/// The possible values for day of the week.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum DayOfWeek {
    #[default]
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

#[psp_stub(libname = "sceRtc", flags = 0x4009, use_crate)]
unsafe extern "C" {
    /// Gets the resolution of the real-time clock tick.
    ///
    /// # Return Value
    ///
    /// Returns the number of ticks per second.
    #[nid(0xC41C2853)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetTickResolution() -> u32;

    /// Gets the current tick count.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3F7AD767)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetCurrentTick(tick: &mut u64) -> SceResult<()>;

    /// Gets the current clock, adjusted for specified timezone.
    ///
    /// # Parameters
    ///
    /// - `time` **[[Out parameter]]**: A pointer to [`DateTime`] to receive the date-time
    ///   information.
    /// - `timezone`: The timezone to adjust the `time` to. Unit is minutes from UTC.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4CFA57B0)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcGetCurrentClock(time: *mut DateTime, timezone: i32) -> SceResult<()>;


    /// Gets the current clock in the currently set local time.
    ///
    /// # Parameters
    ///
    /// - `time` **[[Out parameter]]**: A pointer to [`DateTime`] to receive the date-time
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE7C27D1B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcGetCurrentClockLocalTime(time: *mut DateTime) -> SceResult<()>;

    /// Gets the current network tick count. (It seems like it needs to be logged in PSN to not
    /// return a error).
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.00.
    #[psp_fw_cfg(500..)]
    #[nid(0xF5FCC995)]
    pub safe fn sceRtcGetCurrentNetworkTick(tick: &mut u64) -> SceResult<()>;

    /// Gets the current alarm tick count.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.00 on kernel-level, and 3.95 on
    /// userland-level.
    #[psp_fw_cfg(395..)]
    #[nid(0xC2DDBEB5)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetAlarmTick(tick: &mut u64) -> SceResult<()>;

    /// Sets the RTC alarm tick count.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[In parameter]]**: A reference to the tick count to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.00 on kernel-level, and 3.95 on
    /// userland-level.
    #[psp_fw_cfg(395..)]
    #[nid(0x7D1FBED3)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcSetAlarmTick(tick: &u64) -> SceResult<()>;

    /// Checks if the RTC alarm is alarmed.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the RTC is alarmed, `false` if not alarmed, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(0x81FCDA34)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcIsAlarmed() -> SceResult<bool>;

    /// Registers a callback to be executed when RTC alarm changes state.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID to register.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(0xFB3B18CD)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcRegisterCallback(id: CallbackId) -> SceResult<()>;

    /// Unregisters a RTC alarm callback previously registered.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID to unregister.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(0x6A676D2D)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcUnregisterCallback(id: CallbackId) -> SceResult<()>;

    /// Gets the ticks from the last time reset has occurred by drained battery.
    ///
    /// # Return Value
    ///
    /// Returns the time in ticks from the last time reset has occurred by drained battery.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0x011F03C1)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetAccumulativeTime() -> u64;

    /// Gets the ticks from the last time a battery drained system reset has occurred based on the
    /// system time.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(0x203CEB0D)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetLastReincarnatedTime(tick: &mut u64) -> SceResult<()>;

    /// Gets the tick of the last time the system time was adjusted.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(0x62685E98)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetLastAdjustedTime(tick: &mut u64) -> SceResult<()>;

    /// Formats the tick time in the RFC2822 format, adjusted for specified timezone.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC2822
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    /// - `timezone`: The timezone to adapt the tick time, in minutes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xC663B3B9)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcFormatRFC2822(buf: *mut u8, tick: &u64, timezone: i32) -> SceResult<()>;

    /// Formats the tick time in the RFC2822 format in the currently set local time.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC2822
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7DE6711B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcFormatRFC2822LocalTime(buf: *mut u8, tick: &u64) -> SceResult<()>;

    /// Formats the tick time in the RFC3339 format, adjusted for specified timezone.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC3339
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    /// - `timezone`: The timezone to adapt the tick time, in minutes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0498FB3C)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcFormatRFC3339(buf: *mut u8, tick: &u64, timezone: i32) -> SceResult<()>;

    /// Formats the tick time in the RFC3339 format in the currently set local time.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC3339
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x27F98543)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcFormatRFC3339LocalTime(buf: *mut u8, tick: &u64) -> SceResult<()>;

    /// Parses a formatted buffer text date-time in ticks.
    ///
    /// The supported formats are: RFC2822, RFC3339.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the time in ticks.
    /// - `date_time_buf` **[[In parameter]]**: A pointer to the buffer with a text representation
    ///   of a date-time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDFBC5F16)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcParseDateTime(tick: &mut u64, date_time_buf: *const u8) -> SceResult<()>;

    /// Parses a RFC3339-formatted buffer text date-time in ticks.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the time in ticks.
    /// - `date_time_buf` **[[In parameter]]**: A pointer to the buffer with a text representation
    ///   of a date-time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x28E1E988)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcParseRFC3339(tick: &mut u64, date_time_buf: *const u8) -> SceResult<()>;

    /// Converts the a [`DateTime`] to ticks.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to a date-time structure.
    /// - `tick` **[[Out parameter]]**: A reference to receive the converted time in ticks.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6FF40ACC)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetTick(date_time: &DateTime, tick: &mut u64) -> SceResult<()>;

    /// Converts a time in ticks to [`DateTime`].
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[Out parameter]]**: A pointer to receive a converted date-time.
    /// - `tick` **[[In parameter]]**: A reference of the time in ticks.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7ED29E40)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcSetTick(date_time: *mut DateTime, tick: &u64) -> SceResult<()>;

    /// Add two ticks.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `num_ticks`: The number of ticks to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x44F45E05)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddTicks(
        dest_tick: *mut u64, src_tick: *const u64, num_ticks: u64,
    ) -> SceResult<()>;

    /// Add microseconds to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `ms`: The number of microseconds to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x26D25A5D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddMicroseconds(
        dest_tick: *mut u64, src_tick: *const u64, ms: u64,
    ) -> SceResult<()>;


    /// Add seconds to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `seconds`: The number of seconds to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF2A4AFE5)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddSeconds(
        dest_tick: *mut u64, src_tick: *const u64, seconds: u64,
    ) -> SceResult<()>;

    /// Add minutes to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `minutes`: The number of minutes to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE6605BCA)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddMinutes(
        dest_tick: *mut u64, src_tick: *const u64, minutes: u64,
    ) -> SceResult<()>;

    /// Add hours to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `hours`: The number of hours to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x26D7A24A)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddHours(
        dest_tick: *mut u64, src_tick: *const u64, hours: u32,
    ) -> SceResult<()>;

    /// Add days to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `days`: The number of days to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE51B4B7A)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddDays(
        dest_tick: *mut u64, src_tick: *const u64, days: u32,
    ) -> SceResult<()>;

    /// Add weeks to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `weeks`: The number of weeks to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCF3A2CA8)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddWeeks(
        dest_tick: *mut u64, src_tick: *const u64, weeks: u32,
    ) -> SceResult<()>;

    /// Add months to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `months`: The number of months to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDBF74F1B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddMonths(
        dest_tick: *mut u64, src_tick: *const u64, months: u32,
    ) -> SceResult<()>;

    /// Add years to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `years`: The number of years to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x42842C77)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcTickAddYears(
        dest_tick: *mut u64, src_tick: *const u64, years: u32,
    ) -> SceResult<()>;

    /// Converts a UTC time tick to local-time time tick.
    ///
    /// # Parameters
    ///
    /// - `utc_tick` **[[In parameter]]**: A reference to the tick in UTC.
    /// - `local_time` **[[Out parameter]]**: A reference to receive the tick in local-time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x34885E0D)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcConvertUtcToLocalTime(utc_tick: &u64, local_time: &mut u64) -> SceResult<()>;

    /// Converts a UTC time tick to local-time time tick.
    ///
    /// # Parameters
    ///
    /// - `local_time` **[[In parameter]]**: A reference to the tick in local-time.
    /// - `utc_tick` **[[Out parameter]]**: A reference to receive the tick in UTC.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x779242A2)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcConvertLocalTimeToUtc(local_time: &u64, utc_tick: &mut u64) -> SceResult<()>;

    /// Converts a date-time to DOS time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `dos_time` **[[Out parameter]]**: A reference to receive the DOS time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x36075567)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetDosTime(date_time: &DateTime, dos_time: &mut u32) -> SceResult<()>;

    /// Converts a MS DOS time to date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `dos_time`: The DOS time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF006F264)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcSetDosTime(date_time: *mut DateTime, dos_time: u32) -> SceResult<()>;

    /// Gets the POSIX `time_t` from a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `posix_time` **[[Out parameter]]**: A reference to receive the POSIX time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x27C4594C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetTime_t(date_time: &DateTime, posix_time: &mut u32) -> SceResult<()>;

    /// Converts a POSIX `time_t` time to a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `posix_time`: The POSIX time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF006F264)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcSetTime_t(date_time: *mut DateTime, posix_time: u32) -> SceResult<()>;

    /// Gets the POSIX `time64_t` from a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `posix_time` **[[Out parameter]]**: A reference to receive the POSIX time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(0xE1C93E47)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetTime64_t(date_time: &DateTime, posix_time: &mut u64) -> SceResult<()>;

    /// Converts a POSIX `time64_t` time to a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `posix_time`: The POSIX time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(0x1909C99B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcSetTime64_t(date_time: *mut DateTime, posix_time: u64) -> SceResult<()>;

    /// Gets the WIN32 `FILETIME` from a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `win32_time` **[[Out parameter]]**: A reference to receive the WIN32 `FILETIME` time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCF561893)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetWin32FileTime(date_time: &DateTime, win32_time: &mut u64)
        -> SceResult<()>;

    /// Converts a WIN32 `FILETIME` time to a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `win32_time`: The WIN32 `FILETIME` time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7ACE4C04)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceRtcSetWin32FileTime(
        date_time: *mut DateTime, win32_time: u64,
    ) -> SceResult<()>;

    /// Checks if the given year is a leap year.
    ///
    /// # Parameters
    ///
    /// - `year`: The year value to check.
    ///
    /// # Return Value
    ///
    /// Returns if it is a leap year, error value if the value is not valid.
    #[nid(0x42307A17)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcIsLeapYear(year: u32) -> SceResult<bool>;

    /// Gets the number of days of a month in a year.
    ///
    /// # Parameters
    ///
    /// - `year`: The year value requested.
    /// - `month`: The month value requested
    ///
    /// # Return Value
    ///
    /// Returns the number of days of a month in a year on success, error value otherwise.
    #[nid(0x05EF322C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetDaysInMonth(year: u32, month: Month) -> SceResult<u32>;

    /// Gets the day of the week of a date.
    ///
    /// # Parameters
    ///
    /// - `year`: The year value requested.
    /// - `month`: The month value requested
    /// - `day`: The day value requested
    ///
    /// # Return Value
    ///
    /// Returns the day of the week on success, error value otherwise.
    #[nid(0x57726BC1)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcGetDayOfWeek(year: u32, month: Month, day: u32) -> SceResult<DayOfWeek>;

    /// Checks if a PSP date-time is valid.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: The date-time to check.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4B1B5E82)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceRtcCheckValid(date_time: &DateTime) -> SceResult<()>;
}

// FIXME: Add missing
//
// - `sceRtcReset`
// - `sceRtcInit`
// - `sceRtcResume`
// - `sceRtcSuspend`
// - `sceRtcEnd`
#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceRtc_driver", flags = 0x0009, use_crate)]
unsafe extern "C" {
    /// Gets the resolution of the real-time clock tick.
    ///
    /// # Return Value
    ///
    /// Returns the number of ticks per second.
    #[nid(if cfg!(feature = "psp_660") { 0xC66D9686 }
        // else if cfg!(feature = "psp_630") { 0xC41C2853 }
        else if cfg!(feature = "psp_600") { 0xBB63A22D }
        else if cfg!(feature = "psp_570") { 0x27340707 }
        else if cfg!(feature = "psp_500") { 0xA97E04FE }
        else if cfg!(feature = "psp_420") { 0x5230FFE0 }
        else if cfg!(feature = "psp_395") { 0x1487DDCC }
        else if cfg!(feature = "psp_380") { 0x54DBB453 }
        else if cfg!(feature = "psp_370") { 0x52AF26AA }
        else { 0xC41C2853 }
    )]
    pub safe fn sceRtcGetTickResolution() -> u32;

    /// Gets the current tick count.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3F7AD767)]
    pub safe fn sceRtcGetCurrentTick(tick: &mut u64) -> SceResult<()>;

    /// Gets the current clock, adjusted for specified timezone.
    ///
    /// # Parameters
    ///
    /// - `time` **[[Out parameter]]**: A pointer to [`DateTime`] to receive the date-time
    ///   information.
    /// - `timezone`: The timezone to adjust the `time` to. Unit is minutes from UTC.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4CFA57B0)]
    pub unsafe fn sceRtcGetCurrentClock(time: *mut DateTime, timezone: i32) -> SceResult<()>;


    /// Gets the current clock in the currently set local time.
    ///
    /// # Parameters
    ///
    /// - `time` **[[Out parameter]]**: A pointer to [`DateTime`] to receive the date-time
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x9012B140 }
        else if cfg!(feature = "psp_630") { 0x0287B1C2 }
        else if cfg!(feature = "psp_600") { 0xA0B3BF0F }
        else if cfg!(feature = "psp_570") { 0x1CAC7E9C }
        else if cfg!(feature = "psp_500") { 0x8ED5F3B4 }
        else if cfg!(feature = "psp_420") { 0x64D8AD31 }
        else if cfg!(feature = "psp_395") { 0x1D90BCF2 }
        else if cfg!(feature = "psp_380") { 0xA19F9CEB }
        else if cfg!(feature = "psp_370") { 0x6A06446A }
        else { 0xE7C27D1B }
    )]
    pub unsafe fn sceRtcGetCurrentClockLocalTime(time: *mut DateTime) -> SceResult<()>;

    /// Gets the current RTC alarm tick count.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.00 on kernel-level, and 3.95 on
    /// userland-level.
    #[nid(if cfg!(feature = "psp_660") { 0x7D8E37E1 }
        else if cfg!(feature = "psp_630") { 0xAB8F477C }
        else if cfg!(feature = "psp_600") { 0x49FF9A51 }
        else if cfg!(feature = "psp_570") { 0x838A041E }
        else if cfg!(feature = "psp_500") { 0xE2A7FEA9 }
        else if cfg!(feature = "psp_420") { 0xCD32689F }
        else if cfg!(feature = "psp_395") { 0x4527DCC9 }
        else if cfg!(feature = "psp_380") { 0x81931136 }
        else if cfg!(feature = "psp_370") { 0x63B7E163 }
        else { 0xC2DDBEB5 }
    )]
    pub safe fn sceRtcGetAlarmTick(tick: &mut u64) -> SceResult<()>;

    /// Sets the RTC alarm tick count.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[In parameter]]**: A reference to the tick count to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.00 on kernel-level, and 3.95 on
    /// userland-level.
    #[nid(if cfg!(feature = "psp_660") { 0xE09880CF }
        else if cfg!(feature = "psp_630") { 0x54B9C589 }
        else if cfg!(feature = "psp_600") { 0x68AED59A }
        else if cfg!(feature = "psp_570") { 0x2E422A2B }
        else if cfg!(feature = "psp_500") { 0xADAF231F }
        else if cfg!(feature = "psp_420") { 0xF15D5D7B }
        else if cfg!(feature = "psp_395") { 0x55AC1C23 }
        else if cfg!(feature = "psp_380") { 0x827BCB3F }
        else if cfg!(feature = "psp_370") { 0x329E8E3A }
        else { 0x7D1FBED3 }
    )]
    pub safe fn sceRtcSetAlarmTick(tick: &u64) -> SceResult<()>;

    /// Checks if the RTC alarm is alarmed.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the RTC is alarmed, `false` if not alarmed, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(if cfg!(feature = "psp_660") { 0xCF76CFE5 }
        else if cfg!(feature = "psp_630") { 0xBEBE31B8 }
        else if cfg!(feature = "psp_600") { 0x46C695AA }
        else if cfg!(feature = "psp_570") { 0xD411E636 }
        else if cfg!(feature = "psp_500") { 0x8D5859FD }
        else if cfg!(feature = "psp_420") { 0xEB699EC0 }
        else if cfg!(feature = "psp_395") { 0xC0B071BF }
        else if cfg!(feature = "psp_380") { 0xAF771B2B }
        else if cfg!(feature = "psp_370") { 0xED6F0B57 }
        else { 0x81FCDA34 }
    )]
    pub safe fn sceRtcIsAlarmed() -> SceResult<bool>;

    /// Registers a callback to be executed when RTC alarm changes state.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID to register.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(if cfg!(feature = "psp_660") { 0x530A903E }
        else if cfg!(feature = "psp_630") { 0x12DDA3D7 }
        else if cfg!(feature = "psp_600") { 0xE1F7B409 }
        else if cfg!(feature = "psp_570") { 0x1E23C856 }
        else if cfg!(feature = "psp_500") { 0x437A247E }
        else if cfg!(feature = "psp_420") { 0x03EEB0E5 }
        else if cfg!(feature = "psp_395") { 0x5A6232E2 }
        else if cfg!(feature = "psp_380") { 0x75E26856 }
        else if cfg!(feature = "psp_370") { 0x5EF2152F }
        else { 0xFB3B18CD }
    )]
    pub safe fn sceRtcRegisterCallback(id: CallbackId) -> SceResult<()>;

    /// Unregisters a RTC alarm callback previously registered.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID to unregister.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # What is RTC Alarm?
    ///
    /// The RTC alarm feature allows the system to wake-up and resume functionality when the alarm
    /// time is reached and the system is in suspended state.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(if cfg!(feature = "psp_660") { 0x366669D6 }
        else if cfg!(feature = "psp_630") { 0x642544CD }
        else if cfg!(feature = "psp_600") { 0x19C3C426 }
        else if cfg!(feature = "psp_570") { 0xF6A4E9F4 }
        else if cfg!(feature = "psp_500") { 0xE225354D }
        else if cfg!(feature = "psp_420") { 0xBDC6FCC7 }
        else if cfg!(feature = "psp_395") { 0x72C7578F }
        else if cfg!(feature = "psp_380") { 0x83B37B1B }
        else if cfg!(feature = "psp_370") { 0x6ED49D14 }
        else { 0x6A676D2D }
    )]
    pub safe fn sceRtcUnregisterCallback(id: CallbackId) -> SceResult<()>;

    /// Gets the ticks from the last time a battery drained system reset has occurred.
    ///
    /// This tick count is not depended on the system set time.
    ///
    /// # Return Value
    ///
    /// Returns the time in ticks from the last time a battery drained system reset has occurred.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0x011F03C1)]
    pub safe fn sceRtcGetAccumulativeTime() -> u64;

    /// Gets the ticks from the last time a battery drained system reset has occurred based on the
    /// system time.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(if cfg!(feature = "psp_660") { 0x7C6E9610 }
        else if cfg!(feature = "psp_630") { 0xA01B48AC }
        else if cfg!(feature = "psp_600") { 0xC55A5A0C }
        else if cfg!(feature = "psp_570") { 0x792C0B61 }
        else if cfg!(feature = "psp_500") { 0x1E758B56 }
        else if cfg!(feature = "psp_420") { 0xDFF50450 }
        else if cfg!(feature = "psp_395") { 0x9E97321F }
        else if cfg!(feature = "psp_380") { 0x5E78FA4D }
        else if cfg!(feature = "psp_370") { 0x5A67B6CD }
        else { 0x203CEB0D }
    )]
    pub safe fn sceRtcGetLastReincarnatedTime(tick: &mut u64) -> SceResult<()>;

    /// Gets the tick of the last time the system time was adjusted.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the tick count.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(if cfg!(feature = "psp_660") { 0xE98FEC46 }
        else if cfg!(feature = "psp_630") { 0x65725896 }
        else if cfg!(feature = "psp_600") { 0x7D70D63F }
        else if cfg!(feature = "psp_570") { 0x5F1CB6F8 }
        else if cfg!(feature = "psp_500") { 0xC5CAB31B }
        else if cfg!(feature = "psp_420") { 0x48958A19 }
        else if cfg!(feature = "psp_395") { 0x3201752F }
        else if cfg!(feature = "psp_380") { 0xF4599400 }
        else if cfg!(feature = "psp_370") { 0x9F913B6C }
        else { 0x62685E98 }
    )]
    pub safe fn sceRtcGetLastAdjustedTime(tick: &mut u64) -> SceResult<()>;

    /// Formats the tick time in the RFC2822 format, adjusted for specified timezone.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC2822
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    /// - `timezone`: The timezone to adapt the tick time, in minutes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1A86F5FD }
        else if cfg!(feature = "psp_630") { 0x2CF257D1 }
        else if cfg!(feature = "psp_600") { 0x108E9C68 }
        else if cfg!(feature = "psp_570") { 0xA92452C6 }
        else if cfg!(feature = "psp_500") { 0x572B8CBF }
        else if cfg!(feature = "psp_420") { 0x8DB18FAD }
        else if cfg!(feature = "psp_395") { 0xDD31FDAA }
        else if cfg!(feature = "psp_380") { 0xBCB94EAA }
        else if cfg!(feature = "psp_370") { 0x1D060B71 }
        else { 0xC663B3B9 }
    )]
    pub unsafe fn sceRtcFormatRFC2822(buf: *mut u8, tick: &u64, timezone: i32) -> SceResult<()>;

    /// Formats the tick time in the RFC2822 format in the currently set local time.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC2822
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x27FAEC90 }
        else if cfg!(feature = "psp_630") { 0x23A51249 }
        else if cfg!(feature = "psp_600") { 0xB1E3C114 }
        else if cfg!(feature = "psp_570") { 0x159A037C }
        else if cfg!(feature = "psp_500") { 0xF96624B7 }
        else if cfg!(feature = "psp_420") { 0xEF224AB2 }
        else if cfg!(feature = "psp_395") { 0x0D89B645 }
        else if cfg!(feature = "psp_380") { 0xEEA04107 }
        else if cfg!(feature = "psp_370") { 0xF3DCF13E }
        else { 0x7DE6711B }
    )]
    pub unsafe fn sceRtcFormatRFC2822LocalTime(buf: *mut u8, tick: &u64) -> SceResult<()>;

    /// Formats the tick time in the RFC3339 format, adjusted for specified timezone.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC3339
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    /// - `timezone`: The timezone to adapt the tick time, in minutes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1FCE9E23 }
        else if cfg!(feature = "psp_630") { 0x8FFE941C }
        else if cfg!(feature = "psp_600") { 0x05487F6A }
        else if cfg!(feature = "psp_570") { 0x0F820408 }
        else if cfg!(feature = "psp_500") { 0x4EAAC4C4 }
        else if cfg!(feature = "psp_420") { 0xCE045F01 }
        else if cfg!(feature = "psp_395") { 0x2E770A29 }
        else if cfg!(feature = "psp_380") { 0x3C820CE0 }
        else if cfg!(feature = "psp_370") { 0x9C4D880E }
        else { 0x0498FB3C }
    )]
    pub unsafe fn sceRtcFormatRFC3339(buf: *mut u8, tick: &u64, timezone: i32) -> SceResult<()>;

    /// Formats the tick time in the RFC3339 format in the currently set local time.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the text in RFC3339
    ///   format.
    /// - `tick` **[[In parameter]]**: The tick time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x8DED141A }
        else if cfg!(feature = "psp_630") { 0x1F0BED41 }
        else if cfg!(feature = "psp_600") { 0x5D6735B0 }
        else if cfg!(feature = "psp_570") { 0xC1065387 }
        else if cfg!(feature = "psp_500") { 0x5F93BBF1 }
        else if cfg!(feature = "psp_420") { 0xC653F46F }
        else if cfg!(feature = "psp_395") { 0xFDF94ECE }
        else if cfg!(feature = "psp_380") { 0xF53A4626 }
        else if cfg!(feature = "psp_370") { 0x760AB616 }
        else { 0x27F98543 }
    )]
    pub unsafe fn sceRtcFormatRFC3339LocalTime(buf: *mut u8, tick: &u64) -> SceResult<()>;

    /// Parses a formatted buffer text date-time in ticks.
    ///
    /// The supported formats are: RFC2822, RFC3339.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the time in ticks.
    /// - `date_time_buf` **[[In parameter]]**: A pointer to the buffer with a text representation
    ///   of a date-time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xC3A806EE }
        else if cfg!(feature = "psp_630") { 0x2A7A5D67 }
        else if cfg!(feature = "psp_600") { 0xFC555073 }
        else if cfg!(feature = "psp_570") { 0x63736127 }
        else if cfg!(feature = "psp_500") { 0x8A4D4358 }
        else if cfg!(feature = "psp_420") { 0xC1645DA8 }
        else if cfg!(feature = "psp_395") { 0xF25CC10E }
        else if cfg!(feature = "psp_380") { 0xEE42E500 }
        else if cfg!(feature = "psp_370") { 0xFAF3E79B }
        else { 0xDFBC5F16 }
    )]
    pub unsafe fn sceRtcParseDateTime(tick: &mut u64, date_time_buf: *const u8) -> SceResult<()>;

    /// Parses a RFC3339-formatted buffer text date-time in ticks.
    ///
    /// # Parameters
    ///
    /// - `tick` **[[Out parameter]]**: A reference to receive the time in ticks.
    /// - `date_time_buf` **[[In parameter]]**: A pointer to the buffer with a text representation
    ///   of a date-time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xBDA60897 }
        else if cfg!(feature = "psp_630") { 0x87EA3D9F }
        else if cfg!(feature = "psp_600") { 0xDDC10EEE }
        else if cfg!(feature = "psp_570") { 0xCD834C32 }
        else if cfg!(feature = "psp_500") { 0xAE7F75D9 }
        else if cfg!(feature = "psp_420") { 0x8806828D }
        else if cfg!(feature = "psp_395") { 0x9BAD7278 }
        else if cfg!(feature = "psp_380") { 0x0B1F9034 }
        else if cfg!(feature = "psp_370") { 0xD5FBF00D }
        else { 0x28E1E988 }
    )]
    pub unsafe fn sceRtcParseRFC3339(tick: &mut u64, date_time_buf: *const u8) -> SceResult<()>;

    /// Converts the a [`DateTime`] to ticks.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to a date-time structure.
    /// - `tick` **[[Out parameter]]**: A reference to receive the converted time in ticks.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6FF40ACC)]
    pub safe fn sceRtcGetTick(date_time: &DateTime, tick: &mut u64) -> SceResult<()>;

    /// Converts a time in ticks to [`DateTime`].
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[Out parameter]]**: A pointer to receive a converted date-time.
    /// - `tick` **[[In parameter]]**: A reference of the time in ticks.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE7B3ABF4 }
        // else if cfg!(feature = "psp_630") { 0x87EA3D9F }
        else if cfg!(feature = "psp_600") { 0x43F38ED8 }
        else if cfg!(feature = "psp_570") { 0x71A4E9EA }
        else if cfg!(feature = "psp_500") { 0x071C387D }
        // else if cfg!(feature = "psp_420") { 0x8806828D }
        else if cfg!(feature = "psp_395") { 0x7B57C228 }
        else if cfg!(feature = "psp_380") { 0x1A67A7B7 }
        // else if cfg!(feature = "psp_370") { 0x1A67A7B7 }
        else { 0x7ED29E40 }
    )]
    pub unsafe fn sceRtcSetTick(date_time: *mut DateTime, tick: &u64) -> SceResult<()>;

    /// Add two ticks.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `num_ticks`: The number of ticks to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x44F45E05)]
    pub unsafe fn sceRtcTickAddTicks(
        dest_tick: *mut u64, src_tick: *const u64, num_ticks: u64,
    ) -> SceResult<()>;

    /// Add microseconds to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `ms`: The number of microseconds to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xB84AC7D7 }
        else if cfg!(feature = "psp_630") { 0x5EF1D882 }
        else if cfg!(feature = "psp_600") { 0x3BFB5E74 }
        else if cfg!(feature = "psp_570") { 0x8CC6D696 }
        else if cfg!(feature = "psp_500") { 0x7DDFB70B }
        else if cfg!(feature = "psp_420") { 0xD84DFE1E }
        else if cfg!(feature = "psp_395") { 0xCF363677 }
        else if cfg!(feature = "psp_380") { 0x12265180 }
        else if cfg!(feature = "psp_370") { 0x8807D616 }
        else { 0x26D25A5D }
    )]
    pub unsafe fn sceRtcTickAddMicroseconds(
        dest_tick: *mut u64, src_tick: *const u64, ms: u64,
    ) -> SceResult<()>;


    /// Add seconds to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `seconds`: The number of seconds to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x89FA4262 }
        else if cfg!(feature = "psp_630") { 0xC2C55D5C }
        // else if cfg!(feature = "psp_600") { 0xF2A4AFE5 }
        else if cfg!(feature = "psp_570") { 0x3E4042C6 }
        else if cfg!(feature = "psp_500") { 0x2E5C3F77 }
        else if cfg!(feature = "psp_420") { 0x4CFFE40F }
        else if cfg!(feature = "psp_395") { 0xEB92DFE5 }
        else if cfg!(feature = "psp_380") { 0xD0C3FB54 }
        else if cfg!(feature = "psp_370") { 0xB873ACBB }
        else { 0xF2A4AFE5 }
    )]
    pub unsafe fn sceRtcTickAddSeconds(
        dest_tick: *mut u64, src_tick: *const u64, seconds: u64,
    ) -> SceResult<()>;

    /// Add minutes to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `minutes`: The number of minutes to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x77138347 }
        else if cfg!(feature = "psp_630") { 0x4A2C0756 }
        else if cfg!(feature = "psp_600") { 0xC59704E1 }
        else if cfg!(feature = "psp_570") { 0xE1673802 }
        else if cfg!(feature = "psp_500") { 0x8606E5FF }
        else if cfg!(feature = "psp_420") { 0xEDB9EDB9 }
        else if cfg!(feature = "psp_395") { 0xF9A8725B }
        else if cfg!(feature = "psp_380") { 0x7E8BDD07 }
        else if cfg!(feature = "psp_370") { 0x13C610F3 }
        else { 0xE6605BCA }
    )]
    pub unsafe fn sceRtcTickAddMinutes(
        dest_tick: *mut u64, src_tick: *const u64, minutes: u64,
    ) -> SceResult<()>;

    /// Add hours to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `hours`: The number of hours to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x8413CADC }
        else if cfg!(feature = "psp_630") { 0x994BA17E }
        else if cfg!(feature = "psp_600") { 0x63EF93F0 }
        else if cfg!(feature = "psp_570") { 0xE709472D }
        else if cfg!(feature = "psp_500") { 0xCC5C2591 }
        else if cfg!(feature = "psp_420") { 0xBCB7EFBC }
        else if cfg!(feature = "psp_395") { 0xCB653139 }
        else if cfg!(feature = "psp_380") { 0xC70202E7 }
        else if cfg!(feature = "psp_370") { 0x3D9A259A }
        else { 0x26D7A24A }
    )]
    pub unsafe fn sceRtcTickAddHours(
        dest_tick: *mut u64, src_tick: *const u64, hours: u32,
    ) -> SceResult<()>;

    /// Add days to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `days`: The number of days to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xCB0538FD }
        else if cfg!(feature = "psp_630") { 0x405A9884 }
        else if cfg!(feature = "psp_600") { 0x258213AF }
        else if cfg!(feature = "psp_570") { 0x8AD3AF35 }
        else if cfg!(feature = "psp_500") { 0x12096792 }
        else if cfg!(feature = "psp_420") { 0xC5189980 }
        else if cfg!(feature = "psp_395") { 0x8A4BBB76 }
        else if cfg!(feature = "psp_380") { 0xC39EC1C2 }
        else if cfg!(feature = "psp_370") { 0x9A837A89 }
        else { 0xE51B4B7A }
    )]
    pub unsafe fn sceRtcTickAddDays(
        dest_tick: *mut u64, src_tick: *const u64, days: u32,
    ) -> SceResult<()>;

    /// Add weeks to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `weeks`: The number of weeks to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x80F21937 }
        else if cfg!(feature = "psp_630") { 0x0C47B23C }
        else if cfg!(feature = "psp_600") { 0xC4A12BFC }
        else if cfg!(feature = "psp_570") { 0x93B110F2 }
        else if cfg!(feature = "psp_500") { 0x0614158C }
        else if cfg!(feature = "psp_420") { 0x0E4E3341 }
        else if cfg!(feature = "psp_395") { 0xE3DE227A }
        else if cfg!(feature = "psp_380") { 0x48B6C879 }
        else if cfg!(feature = "psp_370") { 0x9DEEFED2 }
        else { 0xCF3A2CA8 }
    )]
    pub unsafe fn sceRtcTickAddWeeks(
        dest_tick: *mut u64, src_tick: *const u64, weeks: u32,
    ) -> SceResult<()>;

    /// Add months to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `months`: The number of months to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE45726F6 }
        else if cfg!(feature = "psp_630") { 0x383685EA }
        else if cfg!(feature = "psp_600") { 0xEB013962 }
        else if cfg!(feature = "psp_570") { 0xCE348915 }
        else if cfg!(feature = "psp_500") { 0xDC2DCB22 }
        else if cfg!(feature = "psp_420") { 0xA472944C }
        else if cfg!(feature = "psp_395") { 0x823BB89D }
        else if cfg!(feature = "psp_380") { 0x8C581F45 }
        else if cfg!(feature = "psp_370") { 0x2D18D771 }
        else { 0xDBF74F1B }
    )]
    pub unsafe fn sceRtcTickAddMonths(
        dest_tick: *mut u64, src_tick: *const u64, months: u32,
    ) -> SceResult<()>;

    /// Add years to a tick.
    ///
    /// # Parameters
    ///
    /// - `dest_tick` **[[Out parameter]]**: A pointer to tick to receive the result of the
    ///   addition.
    /// - `src_tick` **[[In parameter]]**: A pointer to the source tick value.
    /// - `years`: The number of years to add.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xAAAE90FF }
        else if cfg!(feature = "psp_630") { 0xE25D31B3 }
        else if cfg!(feature = "psp_600") { 0x76C60BEB }
        else if cfg!(feature = "psp_570") { 0x35421509 }
        else if cfg!(feature = "psp_500") { 0x9FE36EB7 }
        else if cfg!(feature = "psp_420") { 0x3A98F8B0 }
        else if cfg!(feature = "psp_395") { 0x65A9E87A }
        else if cfg!(feature = "psp_380") { 0x5BB54061 }
        else if cfg!(feature = "psp_370") { 0xC094DCE7 }
        else { 0x42842C77 }
    )]
    pub unsafe fn sceRtcTickAddYears(
        dest_tick: *mut u64, src_tick: *const u64, years: u32,
    ) -> SceResult<()>;

    /// Converts a UTC time tick to local-time time tick.
    ///
    /// # Parameters
    ///
    /// - `utc_tick` **[[In parameter]]**: A reference to the tick in UTC.
    /// - `local_time` **[[Out parameter]]**: A reference to receive the tick in local-time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x4E267E02 }
        else if cfg!(feature = "psp_630") { 0xFF530D9F }
        else if cfg!(feature = "psp_600") { 0x2F20DAF0 }
        else if cfg!(feature = "psp_570") { 0x88FB3AB0 }
        else if cfg!(feature = "psp_500") { 0x7A7FC8DD }
        else if cfg!(feature = "psp_420") { 0xA56EF4FD }
        else if cfg!(feature = "psp_395") { 0x490DE552 }
        else if cfg!(feature = "psp_380") { 0xFA9EA291 }
        else if cfg!(feature = "psp_370") { 0xB55B2E56 }
        else { 0x34885E0D }
    )]
    pub safe fn sceRtcConvertUtcToLocalTime(utc_tick: &u64, local_time: &mut u64) -> SceResult<()>;

    /// Converts a UTC time tick to local-time time tick.
    ///
    /// # Parameters
    ///
    /// - `local_time` **[[In parameter]]**: A reference to the tick in local-time.
    /// - `utc_tick` **[[Out parameter]]**: A reference to receive the tick in UTC.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x3E66CB7E }
        else if cfg!(feature = "psp_630") { 0x9FAADB5A }
        else if cfg!(feature = "psp_600") { 0x6A41FE5E }
        else if cfg!(feature = "psp_570") { 0x0AE53A81 }
        else if cfg!(feature = "psp_500") { 0x0174FDC7 }
        else if cfg!(feature = "psp_420") { 0x8A66E608 }
        else if cfg!(feature = "psp_395") { 0x8A33F7F7 }
        else if cfg!(feature = "psp_380") { 0xF7F90654 }
        else if cfg!(feature = "psp_370") { 0x4BCD44C6 }
        else { 0x779242A2 }
    )]
    pub safe fn sceRtcConvertLocalTimeToUtc(local_time: &u64, utc_tick: &mut u64) -> SceResult<()>;

    /// Converts a date-time to DOS time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `dos_time` **[[Out parameter]]**: A reference to receive the DOS time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xA4A5BF1B }
        else if cfg!(feature = "psp_630") { 0xDA941BC2 }
        else if cfg!(feature = "psp_600") { 0xE9B6C08A }
        else if cfg!(feature = "psp_570") { 0xB87DA667 }
        else if cfg!(feature = "psp_500") { 0x0AC64851 }
        else if cfg!(feature = "psp_420") { 0x3C2F31B4 }
        else if cfg!(feature = "psp_395") { 0xBAB904EB }
        else if cfg!(feature = "psp_380") { 0xD42E555F }
        else if cfg!(feature = "psp_370") { 0xF66CAAF1 }
        else { 0x36075567 }
    )]
    pub safe fn sceRtcGetDosTime(date_time: &DateTime, dos_time: &mut u32) -> SceResult<()>;

    /// Converts a MS DOS time to date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `dos_time`: The DOS time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x74772CCC }
        else if cfg!(feature = "psp_630") { 0xC34F73EB }
        else if cfg!(feature = "psp_600") { 0xB9B1CD57 }
        else if cfg!(feature = "psp_570") { 0x3A78AD91 }
        else if cfg!(feature = "psp_500") { 0x91476E3B }
        else if cfg!(feature = "psp_420") { 0x7F493A10 }
        else if cfg!(feature = "psp_395") { 0x3B3FE75F }
        else if cfg!(feature = "psp_380") { 0x6ADD70FA }
        else if cfg!(feature = "psp_370") { 0xFB0E4338 }
        else { 0xF006F264 }
    )]
    pub unsafe fn sceRtcSetDosTime(date_time: *mut DateTime, dos_time: u32) -> SceResult<()>;

    /// Gets the POSIX `time_t` from a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `posix_time` **[[Out parameter]]**: A reference to receive the POSIX time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE86D8FC0 }
        else if cfg!(feature = "psp_630") { 0xD2CCB1A6 }
        else if cfg!(feature = "psp_600") { 0x7B463901 }
        else if cfg!(feature = "psp_570") { 0x9A57351B }
        else if cfg!(feature = "psp_500") { 0x812DDBDB }
        else if cfg!(feature = "psp_420") { 0x5DDA96C2 }
        else if cfg!(feature = "psp_395") { 0x6EF9F2EE }
        else if cfg!(feature = "psp_380") { 0x75256EC9 }
        else if cfg!(feature = "psp_370") { 0x196F58A5 }
        else { 0x27C4594C }
    )]
    pub safe fn sceRtcGetTime_t(date_time: &DateTime, posix_time: &mut u32) -> SceResult<()>;

    /// Converts a POSIX `time_t` time to a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `posix_time`: The POSIX time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x74772CCC }
        else if cfg!(feature = "psp_630") { 0xC34F73EB }
        else if cfg!(feature = "psp_600") { 0xB9B1CD57 }
        else if cfg!(feature = "psp_570") { 0x3A78AD91 }
        else if cfg!(feature = "psp_500") { 0x91476E3B }
        else if cfg!(feature = "psp_420") { 0x7F493A10 }
        else if cfg!(feature = "psp_395") { 0x3B3FE75F }
        else if cfg!(feature = "psp_380") { 0x6ADD70FA }
        else if cfg!(feature = "psp_370") { 0xFB0E4338 }
        else { 0xF006F264 }
    )]
    pub unsafe fn sceRtcSetTime_t(date_time: *mut DateTime, posix_time: u32) -> SceResult<()>;

    /// Gets the POSIX `time64_t` from a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `posix_time` **[[Out parameter]]**: A reference to receive the POSIX time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(if cfg!(feature = "psp_660") { 0x94225550 }
        else if cfg!(feature = "psp_630") { 0x0C8E0D5C }
        else if cfg!(feature = "psp_600") { 0x4F9F0E2C }
        else if cfg!(feature = "psp_570") { 0x37E3B6F0 }
        else if cfg!(feature = "psp_500") { 0xFF99CCA9 }
        else if cfg!(feature = "psp_420") { 0xD5D76B55 }
        else if cfg!(feature = "psp_395") { 0x53B45DE9 }
        else if cfg!(feature = "psp_380") { 0xCB65A3F2 }
        else if cfg!(feature = "psp_370") { 0xCC4BBE7E }
        else { 0xE1C93E47 }
    )]
    pub safe fn sceRtcGetTime64_t(date_time: &DateTime, posix_time: &mut u64) -> SceResult<()>;

    /// Converts a POSIX `time64_t` time to a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `posix_time`: The POSIX time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[psp_fw_cfg(200..)]
    #[nid(if cfg!(feature = "psp_660") { 0xCF4E0EE0 }
        else if cfg!(feature = "psp_630") { 0xDACE3710 }
        else if cfg!(feature = "psp_600") { 0xCD653A7E }
        else if cfg!(feature = "psp_570") { 0x21A0B885 }
        else if cfg!(feature = "psp_500") { 0x6B429FFE }
        else if cfg!(feature = "psp_420") { 0x7E24808A }
        else if cfg!(feature = "psp_395") { 0x5D7B2149 }
        else if cfg!(feature = "psp_380") { 0xEFD97708 }
        else if cfg!(feature = "psp_370") { 0x47F08611 }
        else { 0x1909C99B }
    )]
    pub unsafe fn sceRtcSetTime64_t(date_time: *mut DateTime, posix_time: u64) -> SceResult<()>;

    /// Gets the WIN32 `FILETIME` from a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A reference to the `DateTime` to convert.
    /// - `win32_time` **[[Out parameter]]**: A reference to receive the WIN32 `FILETIME` time.
    ///
    ///  # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xB2B6578C }
        else if cfg!(feature = "psp_630") { 0x9C7EC49F }
        else if cfg!(feature = "psp_600") { 0x120ED794 }
        else if cfg!(feature = "psp_570") { 0x84DF2A56 }
        else if cfg!(feature = "psp_500") { 0xD614AAA9 }
        else if cfg!(feature = "psp_420") { 0x7A669963 }
        else if cfg!(feature = "psp_395") { 0xA6638C1C }
        else if cfg!(feature = "psp_380") { 0x1E47B825 }
        else if cfg!(feature = "psp_370") { 0x9D2687CA }
        else { 0xCF561893 }
    )]
    pub safe fn sceRtcGetWin32FileTime(date_time: &DateTime, win32_time: &mut u64)
        -> SceResult<()>;

    /// Converts a WIN32 `FILETIME` time to a PSP date-time.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: A pointer to the `DateTime` to receive the time.
    /// - `win32_time`: The WIN32 `FILETIME` time to convert.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xCEF8FE8E }
        else if cfg!(feature = "psp_630") { 0xE8E3D9CB }
        else if cfg!(feature = "psp_600") { 0x36CB9E04 }
        else if cfg!(feature = "psp_570") { 0x260B364E }
        else if cfg!(feature = "psp_500") { 0x76C3D1EB }
        else if cfg!(feature = "psp_420") { 0x4CB97800 }
        else if cfg!(feature = "psp_395") { 0xCF20212E }
        else if cfg!(feature = "psp_380") { 0xF188632D }
        else if cfg!(feature = "psp_370") { 0x768FFF12 }
        else { 0x7ACE4C04 }
    )]
    pub unsafe fn sceRtcSetWin32FileTime(
        date_time: *mut DateTime, win32_time: u64,
    ) -> SceResult<()>;

    /// Checks if the given year is a leap year.
    ///
    /// # Parameters
    ///
    /// - `year`: The year value to check.
    ///
    /// # Return Value
    ///
    /// Returns if it is a leap year on success, error value if the value is not valid.
    #[nid(if cfg!(feature = "psp_660") { 0x00F66D06 }
        else if cfg!(feature = "psp_630") { 0x2894B167 }
        else if cfg!(feature = "psp_600") { 0x2510903A }
        else if cfg!(feature = "psp_570") { 0x0C533352 }
        else if cfg!(feature = "psp_500") { 0xDD5C39CD }
        else if cfg!(feature = "psp_420") { 0x263967E8 }
        else if cfg!(feature = "psp_395") { 0xC42D8FD5 }
        else if cfg!(feature = "psp_380") { 0x62AEA063 }
        else if cfg!(feature = "psp_370") { 0xF9AA5B68 }
        else { 0x42307A17 }
    )]
    pub safe fn sceRtcIsLeapYear(year: u32) -> SceResult<bool>;

    /// Gets the number of days of a month in a year.
    ///
    /// # Parameters
    ///
    /// - `year`: The year value requested.
    /// - `month`: The month value requested
    ///
    /// # Return Value
    ///
    /// Returns the number of days of a month in a year on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1DAB3CF3 }
        else if cfg!(feature = "psp_630") { 0x47E66184 }
        else if cfg!(feature = "psp_600") { 0xA99453A4 }
        else if cfg!(feature = "psp_570") { 0x4F7AD54D }
        else if cfg!(feature = "psp_500") { 0x94CBD13A }
        else if cfg!(feature = "psp_420") { 0xD51B02B1 }
        else if cfg!(feature = "psp_395") { 0x5D6ECA22 }
        else if cfg!(feature = "psp_380") { 0x8859D3BF }
        else if cfg!(feature = "psp_370") { 0xBA49B6F7 }
        else { 0x05EF322C }
    )]
    pub safe fn sceRtcGetDaysInMonth(year: u32, month: Month) -> SceResult<u32>;

    /// Gets the day of the week of a date.
    ///
    /// # Parameters
    ///
    /// - `year`: The year value requested.
    /// - `month`: The month value requested
    /// - `day`: The day value requested
    ///
    /// # Return Value
    ///
    /// Returns the day of the week on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x321A839A }
        else if cfg!(feature = "psp_630") { 0x1D887B2E }
        else if cfg!(feature = "psp_600") { 0x6F2FC44B }
        else if cfg!(feature = "psp_570") { 0xA7716919 }
        else if cfg!(feature = "psp_500") { 0xDD5E471A }
        else if cfg!(feature = "psp_420") { 0x6D7CEA71 }
        else if cfg!(feature = "psp_395") { 0xE9BA8E5D }
        else if cfg!(feature = "psp_380") { 0x27C1B679 }
        else if cfg!(feature = "psp_370") { 0x5B318E25 }
        else { 0x57726BC1 }
    )]
    pub safe fn sceRtcGetDayOfWeek(year: u32, month: Month, day: u32) -> SceResult<DayOfWeek>;

    /// Checks if a PSP date-time is valid.
    ///
    /// # Parameters
    ///
    /// - `date_time` **[[In parameter]]**: The date-time to check.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4B1B5E82)]
    pub safe fn sceRtcCheckValid(date_time: &DateTime) -> SceResult<()>;
}

impl crate::private::Sealed for DayOfWeek {}
unsafe impl SceResultOk for DayOfWeek {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        match ok_value {
            0 => Some(Self::Sunday),
            1 => Some(Self::Monday),
            2 => Some(Self::Tuesday),
            3 => Some(Self::Wednesday),
            4 => Some(Self::Sunday),
            5 => Some(Self::Sunday),
            6 => Some(Self::Sunday),
            _ => None,
        }
    }
}
unsafe impl SceIntoOkValue for DayOfWeek {
    fn into_ok_value(self) -> u32 {
        self as u8 as u32
    }
}
