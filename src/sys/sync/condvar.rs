use core::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use crate::{
    sync::RawMutex,
    sys::{
        sync::SemaMutex,
        thread::{
            sceKernelCreateSema, sceKernelDeleteSema, sceKernelSignalSema, sceKernelWaitSema,
            SemaId, SemaphoreAttributes,
        },
        SceError,
    },
};


const UNINIT: u32 = u32::MAX;
const INITIALIZING: u32 = u32::MAX - 1;


pub struct CondVar {
    lock: SemaMutex,
    queue: AtomicU32,
    waiters: AtomicU32,
}

impl CondVar {
    pub const fn new() -> Self {
        Self {
            lock: SemaMutex::new(),
            queue: AtomicU32::new(UNINIT),
            waiters: AtomicU32::new(0),
        }
    }

    pub fn notify_one(&self) {
        self.lock.lock();
        let waiters = self.waiters.load(Ordering::Relaxed);

        if waiters > 0 {
            self.waiters.fetch_sub(1, Ordering::Relaxed);
            if let Some(id) = self.get_queue() {
                let res = sceKernelSignalSema(id, 1);
                debug_assert!(
                    res.is_ok(),
                    "failed to signal the queue semaphore: {:#X}",
                    res.as_inner()
                )
            }
        }

        unsafe { self.lock.unlock() };
    }

    pub fn notify_all(&self) {
        self.lock.lock();
        let waiters = self.waiters.swap(0, Ordering::Relaxed);

        if waiters > 0 {
            if let Some(queue) = self.get_queue() {
                let mut remaining = waiters;

                while remaining != 0 {
                    let to_signal = if remaining > i32::MAX as u32 {
                        i32::MAX
                    } else {
                        remaining as i32
                    };

                    let res = sceKernelSignalSema(id, to_signal);
                    debug_assert!(
                        res.is_ok(),
                        "failed to signal the queue semaphore: {:#X}",
                        res.as_inner()
                    );

                    remaining = remaining.saturating_sub(to_signal as u32);
                }
            }
        }

        unsafe { self.lock.unlock() };
    }

    pub unsafe fn wait<M: RawMutex>(&self, mutex: &M) {
        let _ = unsafe { self.wait_optional_timeout(mutex, None) };
    }

    pub unsafe fn wait_timeout<M: RawMutex>(&self, mutex: &M, timeout: Duration) -> bool {
        unsafe { self.wait_optional_timeout(mutex, Some(timeout)) }
    }

    unsafe fn wait_optional_timeout<M: RawMutex>(
        &self, mutex: &M, timeout: Option<Duration>,
    ) -> bool {
        self.lock.lock();
        self.waiters.fetch_add(1, Ordering::Relaxed);
        unsafe { self.lock.unlock() };

        unsafe { mutex.unlock() };

        let queue = match self.get_queue() {
            Some(q) => q,
            None => {
                // failed to create queue: try to reacquire mutex and undo waiter count
                mutex.lock();
                self.lock.lock();
                self.waiters.fetch_sub(1, Ordering::Relaxed);
                unsafe { self.lock.unlock() };
                return true;
            },
        };

        let mut timeout = timeout.map(|d| d.as_micros().min(u128::from(u32::MAX)) as u32);
        let res = sceKernelWaitSema(queue, 1, timeout.as_mut());

        match res.into_result() {
            Ok(()) => {
                // Woken by notifier. Notifier already decremented waiter count.
                mutex.lock();
                true
            },
            Err(err) => {
                match err {
                    SceError::KERNEL_WAIT_TIMEOUT => {
                        // Decrement waiter count under gate to avoid races
                        self.lock.lock();
                        self.waiters.fetch_sub(1, Ordering::Relaxed);
                        unsafe { self.lock.unlock() };

                        mutex.lock();
                        false
                    },
                    _ => {
                        // Best-effort recovery: decrement waiter and reacquire mutex
                        self.lock.lock();
                        self.waiters.fetch_sub(1, Ordering::Relaxed);
                        unsafe { self.lock.unlock() };
                        mutex.lock();
                        true
                    },
                }
            },
        }
    }
}

impl CondVar {
    fn get_queue(&self) -> Option<SemaId> {
        let mut i = 0;

        while i < 0x10 {
            i += 1;
            match self.queue.load(Ordering::Acquire) {
                UNINIT => {
                    if self
                        .queue
                        .compare_exchange(UNINIT, INITIALIZING, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        match self.create_queue() {
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
    fn create_queue(&self) -> Result<SemaId, SceError> {
        let created = unsafe {
            sceKernelCreateSema(
                c"SDK_CONDVAR_QUEUE".as_ptr().cast(),
                SemaphoreAttributes::default(),
                0,
                i32::MAX,
                None,
            )
        };

        match created.into_result() {
            Ok(id) => {
                self.queue.store(id.as_inner(), Ordering::Release);
                Ok(id)
            },
            Err(err) => {
                self.queue.store(UNINIT, Ordering::Release);
                Err(err)
            },
        }
    }
}

impl Drop for CondVar {
    fn drop(&mut self) {
        let raw = self.queue.load(Ordering::Relaxed);
        if raw != UNINIT && raw != INITIALIZING {
            let id = unsafe { SemaId::from_raw_unchecked(raw) };
            let res = unsafe { sceKernelDeleteSema(id) };

            // Keep Drop non-panicking in release, but catch issues in debug.
            debug_assert!(res.is_ok(), "failed to delete condvar semaphore: {:#X}", res.as_inner());
        }
    }
}
