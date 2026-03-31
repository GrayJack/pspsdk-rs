/// The 64-bit system clock type.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias = "SceKernelSysClock")]
pub struct SystemClock {
    pub low: u32,
    pub hi: u32,
}

/// PSP Time structure.
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
