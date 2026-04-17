use core::{
    mem,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    sync::RawMutex,
    sys::{
        thread::{
            sceKernelCreateMutex, sceKernelDeleteMutex, sceKernelLockMutex, sceKernelTryLockMutex,
            sceKernelUnlockMutex, MutexAttributes, MutexId,
        },
        SceError,
    },
};

const UNINIT: u32 = u32::MAX;
const INITIALIZING: u32 = u32::MAX - 1;

pub struct Mutex {
    id: AtomicU32,
}

impl Mutex {
    #[inline]
    pub const fn new() -> Self {
        Self {
            id: AtomicU32::new(UNINIT),
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        let Ok(id) = self.get_id() else {
            return false;
        };

        let res = sceKernelTryLockMutex(id, 1);
        res.into_result().is_ok()
    }

    #[inline]
    pub fn lock(&self) {
        let id = self
            .get_id()
            .unwrap_or_else(|err| panic!("failed to init mutex: {:#X}", err.as_inner()));

        let res = sceKernelLockMutex(id, 1, None);
        if res.is_err() {
            panic!("failed to lock mutex: {:#X}", res.as_inner());
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let Ok(id) = self.get_id() else {
            return;
        };

        let _res = sceKernelUnlockMutex(id, 1);
    }
}

impl Mutex {
    #[inline(never)]
    fn get_id(&self) -> Result<MutexId, SceError> {
        loop {
            match self.id.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .id
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return self.create_id();
                    }
                },
                INITIALIZING => {
                    core::hint::spin_loop();
                },
                raw_id => {
                    let id = unsafe { mem::transmute::<u32, MutexId>(raw_id) };
                    return Ok(id);
                },
            }
        }
    }

    /// Creates the mutex and store its UID.
    #[cold]
    fn create_id(&self) -> Result<MutexId, SceError> {
        let created = unsafe {
            sceKernelCreateMutex(c"SDK_MUTEX".as_ptr().cast(), MutexAttributes::default(), 0, None)
        };

        match created.into_result() {
            Ok(id) => {
                self.id.store(id.as_inner(), Ordering::Release);
                Ok(id)
            },
            Err(err) => {
                self.id.store(UNINIT, Ordering::Release);
                Err(err)
            },
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        let raw_id = self.id.load(Ordering::Relaxed);

        // With &mut self, concurrent initialization should be impossible.
        debug_assert_ne!(raw_id, INITIALIZING, "attempt to drop mutex while initializing");

        if raw_id != UNINIT && raw_id != INITIALIZING {
            let id = unsafe { mem::transmute::<u32, MutexId>(raw_id) };
            let res = sceKernelDeleteMutex(id);

            // Keep Drop non-panicking in release, but catch issues in debug.
            debug_assert!(res.is_ok(), "failed to delete mutex: {:#X}", res.as_inner());
        }
    }
}

impl crate::private::Sealed for Mutex {}
impl RawMutex for Mutex {
    const NEW: Self = Self::new();

    fn lock(&self) {
        self.lock();
    }

    fn try_lock(&self) -> bool {
        self.try_lock()
    }

    unsafe fn unlock(&self) {
        unsafe { self.unlock() };
    }
}
