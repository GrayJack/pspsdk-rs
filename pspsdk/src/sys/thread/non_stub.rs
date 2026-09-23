use core::{ffi::CStr, mem::ManuallyDrop, num::NonZero};

use crate::{
    io,
    sys::thread::{sceKernelDelayThreadCB, ThreadId},
    time::{Duration, Instant},
};

pub struct Thread {
    id: ThreadId,
}

impl Thread {
    // pub unsafe fn new(stack: usize, init: Box<ThreadInit>) -> io::Result<Thread> {
    //     todo!()
    // }

    pub fn join(self) {
        todo!()
    }

    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn into_id(self) -> ThreadId {
        ManuallyDrop::new(self).id
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        todo!()
    }
}

pub fn available_parallelism() -> io::Result<NonZero<usize>> {
    Ok(unsafe { NonZero::new_unchecked(1) })
}

pub fn current_os_id() -> Option<u64> {
    todo!()
}

pub fn yield_now() {
    todo!()
}

pub fn set_name(_name: &CStr) {
    todo!()
}

pub fn sleep(dur: Duration) {
    let mut micros =
        dur.as_micros() + if !dur.subsec_nanos().is_multiple_of(1_000) { 1 } else { 0 };

    while micros > 0 {
        let chunk = micros.min(u32::MAX as u128) as u32;
        let res = sceKernelDelayThreadCB(chunk);

        if res.is_ok() {
            micros -= chunk as u128;
        }
    }
}

pub fn sleep_until(deadline: Instant) {
    // The clock source used for `sleep` might not be the same used for `Instant`.
    // Since this function *must not* return before the deadline, we recheck the
    // time after every call to `sleep`. See #149935 for an example of this
    // occurring on older Windows systems.
    while let Some(delay) = deadline.checked_duration_since(Instant::now()) {
        // Sleep for the estimated time remaining until the deadline.
        //
        // If your system has a better way of estimating the delay time or
        // provides a way to sleep until an absolute time, specialize this
        // function for your system.
        sleep(delay);
    }
}
