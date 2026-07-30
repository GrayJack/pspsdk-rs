use core::{
    mem, ptr,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    time::Duration,
};

use alloc::boxed::Box;

use crate::{
    allocators::PartitionAlloc,
    sync::{RawMutex, RawMutexTimed},
    sys::{
        is_interrupt_enabled,
        mem::MemoryPartitionId,
        thread::{
            _sceKernelLockLwMutex, _sceKernelTryLockLwMutex, _sceKernelUnlockLwMutex,
            sceKernelCreateLwMutex, sceKernelCreateMutex, sceKernelCreateSema,
            sceKernelDeleteLwMutex, sceKernelDeleteMutex, sceKernelDeleteSema, sceKernelLockMutex,
            sceKernelPollSema, sceKernelSignalSema, sceKernelTryLockMutex, sceKernelUnlockMutex,
            sceKernelWaitSema, LwMutexWorkArea, MutexAttributes, MutexId, SemaId,
            SemaphoreAttributes,
        },
        SceError,
    },
};

const UNINIT: u32 = u32::MAX;
const INITIALIZING: u32 = u32::MAX - 1;

/// A raw mutex based on [`sys::thread`](crate::sys::thread) mutex API.
///
/// This is the more general mutex, handled by the kernel. It has a downside to be slower than
/// [`LwMutex`].
///
/// Another niche downside of this type is that the required API was only introduced on PSP firmware
/// version 2.70, so if the software was made to run on lower firmware can't use this kind of mutex.
/// An alternative is to use [`SemaMutex`] (always available, i.e. since 1.00).
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
        let Some(id) = self.get_id() else {
            return false;
        };

        let res = sceKernelTryLockMutex(id, 1);
        res.into_result().is_ok()
    }

    #[inline]
    #[track_caller]
    pub fn lock(&self) {
        let id = self.get_id().unwrap_or_else(|| panic!("failed to init mutex"));

        if is_interrupt_enabled() {
            let res = sceKernelLockMutex(id, 1, None);

            if res.is_err() {
                panic!("failed to lock mutex: {:#X}", res.as_inner());
            }
        }
    }

    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        let Some(id) = self.get_id() else {
            return false;
        };

        if is_interrupt_enabled() {
            let mut timeout = u32::try_from(timeout.as_micros()).unwrap_or(u32::MAX);

            let res = sceKernelLockMutex(id, 1, Some(&mut timeout));

            match res.into_result() {
                Ok(_) => true,
                Err(SceError::KERNEL_WAIT_TIMEOUT) => false,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let Some(id) = self.get_id() else {
            return;
        };

        let _res = sceKernelUnlockMutex(id, 1);
    }
}

impl Mutex {
    #[inline(never)]
    fn get_id(&self) -> Option<MutexId> {
        let mut i = 0;
        while i < 0x10 {
            i += 1;
            match self.id.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .id
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        match self.create_id() {
                            Ok(id) => return Some(id),
                            Err(_err) => {
                                continue;
                            },
                        }
                    }
                },
                INITIALIZING => {
                    crate::sys::spin_loop();
                },
                raw_id => {
                    let id = unsafe { mem::transmute::<u32, MutexId>(raw_id) };
                    return Some(id);
                },
            }
        }

        None
    }

    /// Creates the mutex and store its UID.
    #[cold]
    fn create_id(&self) -> Result<MutexId, SceError> {
        let created = unsafe {
            sceKernelCreateMutex(c"SDK_MUTEX".as_ptr().cast(), MutexAttributes::default(), 0, None)
        };

        match created.into_result() {
            Ok(id) => {
                self.id.store(id.to_inner(), Ordering::Release);
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

    #[inline]
    fn lock(&self) {
        self.lock();
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        unsafe { self.unlock() };
    }
}

impl RawMutexTimed for Mutex {
    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        self.try_lock_for(timeout)
    }
}

/// A raw reentrant mutex based on [`sys::thread`](crate::sys::thread) mutex API.
///
/// Implementation wise, it is the same as [`Mutex`], but enables the flag to be recursive/reentrant
/// on creation/initialization.
pub struct ReentrantMutex {
    id: AtomicU32,
}

impl ReentrantMutex {
    #[inline]
    pub const fn new() -> Self {
        Self {
            id: AtomicU32::new(UNINIT),
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        let Some(id) = self.get_id() else {
            return false;
        };

        let res = sceKernelTryLockMutex(id, 1);
        res.into_result().is_ok()
    }

    #[inline]
    #[track_caller]
    pub fn lock(&self) {
        let id = self.get_id().unwrap_or_else(|| panic!("failed to init mutex"));

        if is_interrupt_enabled() {
            let res = sceKernelLockMutex(id, 1, None);
            if res.is_err() {
                panic!("failed to lock mutex: {:#X}", res.as_inner());
            }
        }
    }

    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        let Some(id) = self.get_id() else {
            return false;
        };

        if is_interrupt_enabled() {
            let mut timeout = u32::try_from(timeout.as_micros()).unwrap_or(u32::MAX);

            let res = sceKernelLockMutex(id, 1, Some(&mut timeout));
            match res.into_result() {
                Ok(_) => true,
                Err(SceError::KERNEL_WAIT_TIMEOUT) => false,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let Some(id) = self.get_id() else {
            return;
        };

        let _res = sceKernelUnlockMutex(id, 1);
    }
}

impl ReentrantMutex {
    #[inline(never)]
    fn get_id(&self) -> Option<MutexId> {
        let mut i = 0;
        while i < 0x10 {
            i += 1;
            match self.id.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .id
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        match self.create_id() {
                            Ok(id) => return Some(id),
                            Err(_) => continue,
                        }
                    }
                },
                INITIALIZING => {
                    crate::sys::spin_loop();
                },
                raw_id => {
                    let id = unsafe { mem::transmute::<u32, MutexId>(raw_id) };
                    return Some(id);
                },
            }
        }

        None
    }

    /// Creates the mutex and store its UID.
    #[cold]
    fn create_id(&self) -> Result<MutexId, SceError> {
        let created = unsafe {
            sceKernelCreateMutex(
                c"SDK_REENT_MUTEX".as_ptr().cast(),
                MutexAttributes::ReentrantLock,
                0,
                None,
            )
        };

        match created.into_result() {
            Ok(id) => {
                self.id.store(id.to_inner(), Ordering::Release);
                Ok(id)
            },
            Err(err) => {
                self.id.store(UNINIT, Ordering::Release);
                Err(err)
            },
        }
    }
}

impl Drop for ReentrantMutex {
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

impl crate::private::Sealed for ReentrantMutex {}
impl RawMutex for ReentrantMutex {
    const NEW: Self = Self::new();

    #[inline]
    fn lock(&self) {
        self.lock();
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        unsafe { self.unlock() };
    }
}

impl RawMutexTimed for ReentrantMutex {
    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        self.try_lock_for(timeout)
    }
}

/// A raw lightweight mutex based on [`sys::thread`](crate::sys::thread) lightweight mutex API.
///
/// This implementation has the advantage to be faster and more lightweight process-wise, but it has
/// the downside of the entire mutex structure requiring to live in user RAM partition (this is
/// automatically handled by this type), i.e. it uses slightly more user RAM space.
///
/// Another niche downside of this type is that the required API was only introduced on PSP firmware
/// version 3.95, so if the software was made to run on lower firmware can't use this kind of mutex.
/// An alternative is to use either [`Mutex`] (available on firmware `>= 2.70`) or [`SemaMutex`]
/// (always available, i.e. since 1.00).
pub struct LwMutex {
    state: AtomicU32,
    work_area: AtomicPtr<LwMutexWorkArea>,
}

impl LwMutex {
    pub const fn new() -> Self {
        Self {
            state: AtomicU32::new(UNINIT),
            work_area: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[inline]
    #[track_caller]
    pub fn lock(&self) {
        let work_area = self.get_work_area().unwrap_or_else(|| panic!("failed to init lwmutex"));

        if is_interrupt_enabled() {
            let res = _sceKernelLockLwMutex(work_area, 1, None);
            if res.is_err() {
                panic!("failed to lock lwmutex: {:#X}", res.as_inner());
            }
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        let Some(work_area) = self.get_work_area() else {
            return false;
        };

        _sceKernelTryLockLwMutex(work_area, 1).into_result().is_ok()
    }

    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        let Some(work_area) = self.get_work_area() else {
            return false;
        };

        if is_interrupt_enabled() {
            let mut timeout = u32::try_from(timeout.as_micros()).unwrap_or(u32::MAX);

            let res = _sceKernelLockLwMutex(work_area, 1, Some(&mut timeout));

            match res.into_result() {
                Ok(_) => true,
                Err(SceError::KERNEL_WAIT_TIMEOUT) => false,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let Some(work_area) = self.get_work_area() else {
            return;
        };

        let _ = _sceKernelUnlockLwMutex(work_area, 1);
    }
}

impl LwMutex {
    fn get_work_area(&self) -> Option<&mut LwMutexWorkArea> {
        let mut i = 0;
        while i < 0x10 {
            i += 1;
            match self.state.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .state
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        match self.create_work_area() {
                            Ok(wa) => return Some(wa),
                            Err(_) => continue,
                        }
                    }
                },
                INITIALIZING => core::hint::spin_loop(),
                _ => {
                    let work_area = self.work_area.load(Ordering::Acquire);
                    debug_assert!(!work_area.is_null());
                    let work_area = unsafe { work_area.as_mut_unchecked() };
                    return Some(work_area);
                },
            }
        }
        None
    }

    #[cold]
    fn create_work_area(&self) -> Result<&mut LwMutexWorkArea, SceError> {
        let work_area = Box::new_in(
            LwMutexWorkArea::default_new(),
            PartitionAlloc::new(MemoryPartitionId::MainUser),
        );

        let (work_area, _) = Box::into_raw_with_allocator(work_area);

        self.work_area.store(work_area, Ordering::Release);

        let created = unsafe {
            sceKernelCreateLwMutex(
                work_area,
                c"SDK_LW_MUTEX".as_ptr().cast(),
                MutexAttributes::default(),
                0,
                None,
            )
        };

        match created.into_result() {
            Ok(()) => {
                self.state.store(0, Ordering::Release);
                Ok(unsafe { &mut *work_area })
            },
            Err(err) => {
                self.state.store(UNINIT, Ordering::Release);
                // drop box
                let _box = unsafe {
                    Box::from_raw_in(work_area, PartitionAlloc::new(MemoryPartitionId::MainUser))
                };

                drop(_box);
                self.work_area.store(ptr::null_mut(), Ordering::Release);
                Err(err)
            },
        }
    }
}

impl Drop for LwMutex {
    fn drop(&mut self) {
        let state = self.state.load(Ordering::Relaxed);
        let work_area = self.work_area.load(Ordering::Relaxed);
        if state != UNINIT && state != INITIALIZING && !work_area.is_null() {
            let mut work_area = unsafe {
                Box::from_raw_in(work_area, PartitionAlloc::new(MemoryPartitionId::MainUser))
            };
            let res = sceKernelDeleteLwMutex(work_area.as_mut());

            // Keep Drop non-panicking in release, but catch issues in debug.
            debug_assert!(res.is_ok(), "failed to delete mutex: {:#X}", res.as_inner());
        }
    }
}

impl crate::private::Sealed for LwMutex {}
impl RawMutex for LwMutex {
    const NEW: Self = Self::new();

    #[inline]
    fn lock(&self) {
        self.lock();
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        unsafe { self.unlock() };
    }
}

impl RawMutexTimed for LwMutex {
    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        self.try_lock_for(timeout)
    }
}

/// A raw mutex based on [`sys::thread`](crate::sys::thread) semaphore API.
///
/// This type exists because [`Mutex`] and [`LwMutex`] related System API are not available on all
/// PSP firmware versions; this on the other hand, is available since the first PSP firmware
/// version. Use this in the case you are constrained on the firmware version.
pub struct SemaMutex {
    sema: AtomicU32,
}

impl SemaMutex {
    pub const fn new() -> Self {
        Self {
            sema: AtomicU32::new(UNINIT),
        }
    }

    #[inline]
    #[track_caller]
    pub fn lock(&self) {
        let id = self.get_id().unwrap_or_else(|| panic!("failed to init mutex"));

        if is_interrupt_enabled() {
            let res = sceKernelWaitSema(id, 1, None);
            if res.is_err() {
                panic!("failed to lock mutex: {:#X}", res.as_inner());
            }
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        let Some(id) = self.get_id() else {
            return false;
        };

        sceKernelPollSema(id, 1).into_result().is_ok()
    }

    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        let Some(id) = self.get_id() else {
            return false;
        };

        if is_interrupt_enabled() {
            let mut timeout = u32::try_from(timeout.as_micros()).unwrap_or(u32::MAX);

            let res = sceKernelWaitSema(id, 1, Some(&mut timeout));

            match res.into_result() {
                Ok(_) => true,
                Err(SceError::KERNEL_WAIT_TIMEOUT) => false,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let Some(id) = self.get_id() else {
            return;
        };

        let _ = sceKernelSignalSema(id, 1);
    }
}

impl SemaMutex {
    fn get_id(&self) -> Option<SemaId> {
        let mut i = 0;
        while i < 0x10 {
            i += 1;
            match self.sema.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .sema
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        match self.create_id() {
                            Ok(id) => return Some(id),
                            Err(_) => continue,
                        }
                    }
                },
                INITIALIZING => core::hint::spin_loop(),
                raw => return Some(unsafe { SemaId::from_raw_unchecked(raw) }),
            }
        }
        None
    }

    #[cold]
    fn create_id(&self) -> Result<SemaId, SceError> {
        let created = unsafe {
            sceKernelCreateSema(
                c"SDK_SEMA_MUTEX".as_ptr().cast(),
                SemaphoreAttributes::default(),
                1,
                1,
                None,
            )
        };

        match created.into_result() {
            Ok(id) => {
                self.sema.store(id.to_inner(), Ordering::Release);
                Ok(id)
            },
            Err(err) => {
                self.sema.store(UNINIT, Ordering::Release);
                Err(err)
            },
        }
    }
}

impl Drop for SemaMutex {
    fn drop(&mut self) {
        let raw = self.sema.load(Ordering::Relaxed);
        if raw != UNINIT && raw != INITIALIZING {
            let id = unsafe { SemaId::from_raw_unchecked(raw) };
            let res = sceKernelDeleteSema(id);
            debug_assert!(res.is_ok(), "failed to delete semaphore mutex: {:#X}", res.as_inner());
        }
    }
}

impl crate::private::Sealed for SemaMutex {}
impl RawMutex for SemaMutex {
    const NEW: Self = Self::new();

    #[inline]
    fn lock(&self) {
        self.lock();
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        unsafe { self.unlock() };
    }
}

impl RawMutexTimed for SemaMutex {
    #[inline]
    fn try_lock_for(&self, timeout: Duration) -> bool {
        self.try_lock_for(timeout)
    }
}

pub struct SpinMutex {
    locked: AtomicBool,
}

impl SpinMutex {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    #[inline]
    #[track_caller]
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            crate::sys::spin_loop();
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

impl crate::private::Sealed for SpinMutex {}
impl RawMutex for SpinMutex {
    const NEW: Self = Self::new();

    #[inline]
    fn lock(&self) {
        self.lock();
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        unsafe { self.unlock() };
    }
}
