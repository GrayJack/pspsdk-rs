use core::time::Duration;

use crate::sys::{
    thread::sceKernelGetSystemTimeWide,
    time::{sceRtcGetCurrentTick, sceRtcGetTickResolution},
};

// PSP epoch is 0001-01-01 00:00:00 UTC.
// Unix epoch is 1970-01-01 00:00:00 UTC.
// Offset in seconds: 62,135,596,800
// PSP tick resolution is 1,000,000 ticks/sec (microsecond precision).
const PSP_TO_UNIX_EPOCH_SECS: u64 = 62_135_596_800;

pub(crate) const UNIX_EPOCH: SystemTime = SystemTime(Duration::from_secs(PSP_TO_UNIX_EPOCH_SECS));

// Based on sceRtcGetCurrentTick, i.e. ticks since PSP epoch which is 0001-01-01.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(crate) struct SystemTime(Duration);

impl SystemTime {
    pub const MAX: Self = Self(Duration::MAX);
    pub const MIN: Self = Self(Duration::ZERO);

    #[allow(clippy::manual_checked_ops)]
    pub fn now() -> Self {
        let mut tick = 0;
        let res = sceRtcGetCurrentTick(&mut tick);
        debug_assert!(res.is_ok(), "failed to get current tick: {:#X}", res.as_inner());
        let resolution = sceRtcGetTickResolution() as u64;

        if resolution > 0 {
            let seconds = tick / resolution;
            let remaining = tick % resolution;
            let nanosecs = (remaining * 1_000_000_000) / resolution;
            Self(Duration::new(seconds, nanosecs as u32))
        } else {
            // Assume microseconds resolution as fallback
            Self(Duration::from_micros(tick))
        }
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.0.checked_sub(other.0).ok_or_else(|| other.0 - self.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_add(*other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_sub(*other)?))
    }
}

// Monotonic clock based on sceKernelGetSystemTimeWide() -- microseconds since boot.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(crate) struct Instant(Duration);

impl Instant {
    pub fn now() -> Self {
        let micros = sceKernelGetSystemTimeWide();
        Self(Duration::from_micros(micros))
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        self.0.checked_sub(other.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Self> {
        self.0.checked_add(*other).map(Self)
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Self> {
        self.0.checked_sub(*other).map(Self)
    }
}
