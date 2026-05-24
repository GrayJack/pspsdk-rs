use core::cell::UnsafeCell;

use crate::{
    sync::RawRwLock,
    sys::sync::{Condvar, LwMutex, Mutex, SemaMutex},
};

struct State {
    readers: u32,
    writer_active: bool,
    write_waiters: u32,
}

/// A raw rwlock with writer-preference based on [`sys::thread`](crate::sys::thread) mutex API.
///
/// This is the more general rwlock, handled by the kernel. It has a downside to be slower than
/// [`LwRwLock`].
///
/// Another niche downside of this type is that the required API was only introduced on PSP firmware
/// version 2.70, so if the software was made to run on lower firmware can't use this kind of mutex.
/// An alternative is to use [`SemaRwLock`] (always available, i.e. since 1.00).
pub struct RwLock {
    lock: Mutex,
    cond: Condvar,
    state: UnsafeCell<State>,
}


impl RwLock {
    #[inline]
    pub const fn new() -> Self {
        Self {
            lock: Mutex::new(),
            cond: Condvar::new(),
            state: UnsafeCell::new(State {
                readers: 0,
                writer_active: false,
                write_waiters: 0,
            }),
        }
    }

    #[inline]
    fn read(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        // prefer writers: if a writer is active or waiting, block readers
        while state.writer_active || state.write_waiters != 0 {
            unsafe { self.cond.wait(&self.lock) };
        }

        state.readers += 1;
        unsafe { self.lock.unlock() };
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        let ok = !state.writer_active && state.write_waiters == 0;

        if ok {
            state.readers += 1;
        }

        unsafe { self.lock.unlock() };
        ok
    }

    #[inline]
    fn write(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        // Wait until no readers and no active writer
        while state.writer_active || state.readers != 0 {
            unsafe { self.cond.wait(&self.lock) };
        }

        state.write_waiters -= 1;
        state.writer_active = true;

        unsafe { self.lock.unlock() };
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        let ok = !state.writer_active && state.readers == 0;

        if ok {
            state.writer_active = true;
        }

        unsafe { self.lock.unlock() };
        ok
    }

    #[inline]
    #[track_caller]
    unsafe fn read_unlock(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.readers > 0, "`read_unlock` without reader");
        let readers = {
            let r = state.readers - 1;
            state.readers = r;
            r
        };

        // If last reader and writers are waiting, wake one writer.
        if readers == 0 && state.write_waiters > 0 {
            self.cond.notify_one();
        }
        unsafe { self.lock.unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn write_unlock(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.writer_active, "`write_unlock` without writer");
        state.writer_active = false;

        // Prefer writers: wake one writer if any waiting, otherwise wake all readers
        if state.write_waiters > 0 {
            self.cond.notify_one();
        } else {
            self.cond.notify_all();
        }

        unsafe { self.lock.unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn downgrade(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.writer_active, "downgrade without writer");
        state.writer_active = false;
        state.readers = state.readers.saturating_add(1);

        // Allow readers to proceed. Writers remain blocked until readers drain.
        self.cond.notify_all();
        unsafe { self.lock.unlock() };
    }
}

impl RwLock {
    /// Accesses to this state MUST only happen while `self.lock` is held.
    unsafe fn state(&self) -> &mut State {
        unsafe { &mut *self.state.get() }
    }
}

unsafe impl Sync for RwLock {}

impl crate::private::Sealed for RwLock {}
impl RawRwLock for RwLock {
    const NEW: Self = Self::new();

    #[inline]
    fn read(&self) {
        self.read()
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.try_read()
    }

    #[inline]
    fn write(&self) {
        self.write();
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.try_write()
    }

    #[inline]
    unsafe fn read_unlock(&self) {
        unsafe { self.read_unlock() };
    }

    #[inline]
    unsafe fn write_unlock(&self) {
        unsafe { self.write_unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn downgrade(&self) {
        unsafe { self.downgrade() };
    }
}

/// A raw lightweight rwlock with writer-preference based on [`sys::thread`](crate::sys::thread)
/// lightweight mutex API.
///
/// This implementation has the advantage to be faster and more lightweight process-wise, but it has
/// the downside of the entire rwlock structure requiring to live in user RAM partition (this is
/// automatically handled by this type), i.e. it uses slightly more user RAM space.
///
/// Another niche downside of this type is that the required API was only introduced on PSP firmware
/// version 3.95, so if the software was made to run on lower firmware can't use this kind of mutex.
/// An alternative is to use either [`RwLock`] (available on firmware `>= 2.70`) or [`SemaRwLock`]
/// (always available, i.e. since 1.00).
pub struct LwRwLock {
    lock: LwMutex,
    cond: Condvar,
    state: UnsafeCell<State>,
}


impl LwRwLock {
    #[inline]
    pub const fn new() -> Self {
        Self {
            lock: LwMutex::new(),
            cond: Condvar::new(),
            state: UnsafeCell::new(State {
                readers: 0,
                writer_active: false,
                write_waiters: 0,
            }),
        }
    }

    #[inline]
    fn read(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        // prefer writers: if a writer is active or waiting, block readers
        while state.writer_active || state.write_waiters != 0 {
            unsafe { self.cond.wait(&self.lock) };
        }

        state.readers += 1;
        unsafe { self.lock.unlock() };
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        let ok = !state.writer_active && state.write_waiters == 0;

        if ok {
            state.readers += 1;
        }

        unsafe { self.lock.unlock() };
        ok
    }

    #[inline]
    fn write(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        // Wait until no readers and no active writer
        while state.writer_active || state.readers != 0 {
            unsafe { self.cond.wait(&self.lock) };
        }

        state.write_waiters -= 1;
        state.writer_active = true;

        unsafe { self.lock.unlock() };
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        let ok = !state.writer_active && state.readers == 0;

        if ok {
            state.writer_active = true;
        }

        unsafe { self.lock.unlock() };
        ok
    }

    #[inline]
    #[track_caller]
    unsafe fn read_unlock(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.readers > 0, "`read_unlock` without reader");
        let readers = {
            let r = state.readers - 1;
            state.readers = r;
            r
        };

        // If last reader and writers are waiting, wake one writer.
        if readers == 0 && state.write_waiters > 0 {
            self.cond.notify_one();
        }
        unsafe { self.lock.unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn write_unlock(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.writer_active, "`write_unlock` without writer");
        state.writer_active = false;

        // Prefer writers: wake one writer if any waiting, otherwise wake all readers
        if state.write_waiters > 0 {
            self.cond.notify_one();
        } else {
            self.cond.notify_all();
        }

        unsafe { self.lock.unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn downgrade(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.writer_active, "downgrade without writer");
        state.writer_active = false;
        state.readers = state.readers.saturating_add(1);

        // Allow readers to proceed. Writers remain blocked until readers drain.
        self.cond.notify_all();
        unsafe { self.lock.unlock() };
    }
}

impl LwRwLock {
    /// Accesses to this state MUST only happen while `self.lock` is held.
    unsafe fn state(&self) -> &mut State {
        unsafe { &mut *self.state.get() }
    }
}

unsafe impl Sync for LwRwLock {}

impl crate::private::Sealed for LwRwLock {}
impl RawRwLock for LwRwLock {
    const NEW: Self = Self::new();

    #[inline]
    fn read(&self) {
        self.read()
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.try_read()
    }

    #[inline]
    fn write(&self) {
        self.write();
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.try_write()
    }

    #[inline]
    unsafe fn read_unlock(&self) {
        unsafe { self.read_unlock() };
    }

    #[inline]
    unsafe fn write_unlock(&self) {
        unsafe { self.write_unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn downgrade(&self) {
        unsafe { self.downgrade() };
    }
}

/// A raw rwlock with writer-preference based on [`sys::thread`](crate::sys::thread) semaphore API.
///
/// This type exists because [`RwLock`] and [`LwRwLock`] related System API are not available on all
/// PSP firmware versions; this on the other hand, is available since the first PSP firmware
/// version. Use this in the case you are constrained on the firmware version.
pub struct SemaRwLock {
    lock: SemaMutex,
    cond: Condvar,
    state: UnsafeCell<State>,
}

impl SemaRwLock {
    #[inline]
    pub const fn new() -> Self {
        Self {
            lock: SemaMutex::new(),
            cond: Condvar::new(),
            state: UnsafeCell::new(State {
                readers: 0,
                writer_active: false,
                write_waiters: 0,
            }),
        }
    }

    #[inline]
    fn read(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        // prefer writers: if a writer is active or waiting, block readers
        while state.writer_active || state.write_waiters != 0 {
            unsafe { self.cond.wait(&self.lock) };
        }

        state.readers += 1;
        unsafe { self.lock.unlock() };
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        let ok = !state.writer_active && state.write_waiters == 0;

        if ok {
            state.readers += 1;
        }

        unsafe { self.lock.unlock() };
        ok
    }

    #[inline]
    fn write(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        // Wait until no readers and no active writer
        while state.writer_active || state.readers != 0 {
            unsafe { self.cond.wait(&self.lock) };
        }

        state.write_waiters -= 1;
        state.writer_active = true;

        unsafe { self.lock.unlock() };
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        let ok = !state.writer_active && state.readers == 0;

        if ok {
            state.writer_active = true;
        }

        unsafe { self.lock.unlock() };
        ok
    }

    #[inline]
    #[track_caller]
    unsafe fn read_unlock(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.readers > 0, "`read_unlock` without reader");
        let readers = {
            let r = state.readers - 1;
            state.readers = r;
            r
        };

        // If last reader and writers are waiting, wake one writer.
        if readers == 0 && state.write_waiters > 0 {
            self.cond.notify_one();
        }
        unsafe { self.lock.unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn write_unlock(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.writer_active, "`write_unlock` without writer");
        state.writer_active = false;

        // Prefer writers: wake one writer if any waiting, otherwise wake all readers
        if state.write_waiters > 0 {
            self.cond.notify_one();
        } else {
            self.cond.notify_all();
        }

        unsafe { self.lock.unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn downgrade(&self) {
        self.lock.lock();

        // SAFETY: self.lock is locked
        let state = unsafe { self.state() };

        debug_assert!(state.writer_active, "downgrade without writer");
        state.writer_active = false;
        state.readers = state.readers.saturating_add(1);

        // Allow readers to proceed. Writers remain blocked until readers drain.
        self.cond.notify_all();
        unsafe { self.lock.unlock() };
    }
}

impl SemaRwLock {
    /// Accesses to this state MUST only happen while `self.lock` is held.
    unsafe fn state(&self) -> &mut State {
        unsafe { &mut *self.state.get() }
    }
}

unsafe impl Sync for SemaRwLock {}

impl crate::private::Sealed for SemaRwLock {}
impl RawRwLock for SemaRwLock {
    const NEW: Self = Self::new();

    #[inline]
    fn read(&self) {
        self.read()
    }

    #[inline]
    fn try_read(&self) -> bool {
        self.try_read()
    }

    #[inline]
    fn write(&self) {
        self.write();
    }

    #[inline]
    fn try_write(&self) -> bool {
        self.try_write()
    }

    #[inline]
    unsafe fn read_unlock(&self) {
        unsafe { self.read_unlock() };
    }

    #[inline]
    unsafe fn write_unlock(&self) {
        unsafe { self.write_unlock() };
    }

    #[inline]
    #[track_caller]
    unsafe fn downgrade(&self) {
        unsafe { self.downgrade() };
    }
}
