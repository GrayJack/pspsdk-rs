use core::{
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use crate::sys::{
    thread::{
        sceKernelCreateEventFlag, sceKernelSetEventFlag, sceKernelWaitEventFlag,
        EventFlagAttributes, EventFlagId, EventFlagWaitKinds,
    },
    SceError,
};

const UNINIT: u32 = u32::MAX;
const INITIALIZING: u32 = u32::MAX - 1;

const PARK_BIT: u32 = 0x01;

pub struct Parker {
    ev_flag: AtomicU32,
}

impl Parker {
    pub fn new() -> Self {
        let id = create_ev_flag().map(|ev| ev.to_inner()).unwrap_or(UNINIT);
        Self {
            ev_flag: AtomicU32::new(id),
        }
    }

    /// Creates a new thread parker. UNIX requires this to happen in-place.
    pub unsafe fn new_in_place(parker: *mut Parker) {
        unsafe { parker.write(Parker::new()) };
    }

    pub unsafe fn park(self: Pin<&Self>) {
        let Some(id) = self.get_id() else {
            return;
        };

        let mut out_bits = 0;
        let _res = sceKernelWaitEventFlag(
            id,
            PARK_BIT,
            EventFlagWaitKinds::Or | EventFlagWaitKinds::ClearPat,
            &mut out_bits,
            None,
        );
    }

    pub unsafe fn park_timeout(self: Pin<&Self>, dur: Duration) {
        let Some(id) = self.get_id() else {
            return;
        };

        let micros = dur.as_micros();
        let mut out_bits = 0;
        let mut timeout = u32::try_from(micros).unwrap_or(u32::MAX);
        let timeout2 = Some(&mut timeout);
        let _res = sceKernelWaitEventFlag(
            id,
            PARK_BIT,
            EventFlagWaitKinds::Or | EventFlagWaitKinds::ClearPat,
            &mut out_bits,
            timeout2,
        );
    }

    pub fn unpark(self: Pin<&Self>) {
        let Some(id) = self.get_id() else {
            return;
        };

        // Set the park bit. If the thread is waiting, it wakes up.
        // If not yet parked, the bit persists as a permit for the next park.
        let _res = sceKernelSetEventFlag(id, PARK_BIT);
    }
}

impl Parker {
    fn get_id(&self) -> Option<EventFlagId> {
        let mut i = 0;
        while i < 0x10 {
            i += 1;
            match self.ev_flag.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .ev_flag
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        match self.create_ev_flag() {
                            Ok(id) => return Some(id),
                            Err(_) => continue,
                        }
                    }
                },
                INITIALIZING => core::hint::spin_loop(),
                raw => return Some(unsafe { EventFlagId::from_raw_unchecked(raw) }),
            }
        }

        None
    }

    #[cold]
    fn create_ev_flag(&self) -> Result<EventFlagId, SceError> {
        let created = unsafe {
            sceKernelCreateEventFlag(
                c"SDK_ONCE".as_ptr().cast(),
                EventFlagAttributes::WaitMultiple,
                0,
                None,
            )
        };

        match created.into_result() {
            Ok(id) => {
                self.ev_flag.store(id.to_inner(), Ordering::Release);
                Ok(id)
            },
            Err(err) => {
                self.ev_flag.store(UNINIT, Ordering::Release);
                Err(err)
            },
        }
    }
}

unsafe impl Send for Parker {}
unsafe impl Sync for Parker {}

#[cold]
fn create_ev_flag() -> Result<EventFlagId, SceError> {
    unsafe {
        sceKernelCreateEventFlag(
            c"SDK_PARKER".as_ptr().cast(),
            EventFlagAttributes::WaitMultiple,
            0,
            None,
        )
        .into_result()
    }
}
