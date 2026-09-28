use core::{
    ffi::{c_void, CStr},
    mem::ManuallyDrop,
    num::NonZero,
};

use alloc::boxed::Box;

use crate::{
    io,
    sys::{
        thread::{
            sceKernelCreateThread, sceKernelDelayThreadCB, sceKernelDeleteThread,
            sceKernelStartThread, sceKernelWaitThreadEndCB, ThreadAttributes, ThreadId,
        },
        SceError, SceResult, SceSize,
    },
    thread::{ThreadInit, ThreadOsId},
    time::{Duration, Instant},
};

pub struct Thread {
    id: ThreadId,
}

impl Thread {
    #[allow(private_interfaces, reason = "Internal API")]
    pub unsafe fn with_attr(
        stack: usize, attr: ThreadAttributes, init: Box<ThreadInit>,
    ) -> io::Result<Thread> {
        extern "C" fn thread_start(args: SceSize, argp: *mut c_void) -> SceResult<u32> {
            if argp.is_null() || args != size_of::<*mut ()>() {
                return SceResult::new(u32::MAX);
            }


            // SAFETY: we are simply recreating the box that was leaked earlier.
            let init_ptr = unsafe { *(argp as *mut *mut ThreadInit) };
            let init: Box<ThreadInit> = unsafe { Box::from_raw(init_ptr) };
            let rust_start = init.init();
            rust_start();

            SceResult::new(0)
        }

        // crate::dbg!(&init.handle);

        let thread_name = init.handle.cname().unwrap_or(c"");


        let thread = unsafe {
            sceKernelCreateThread(
                thread_name.as_ptr().cast(),
                thread_start,
                0x20,
                stack,
                attr,
                None,
            )
            .map_err(Into::<io::Error>::into)?
        };

        let mut init_ptr = Box::into_raw(init);
        let ptr = &raw mut init_ptr;
        let res = unsafe { sceKernelStartThread(thread, size_of_val(&init_ptr), ptr.cast()) };

        // crate::eprintln!("{:?}", &res);

        if let Some(err) = res.err() {
            // The thread failed to start and as a result data was not consumed. Therefore, it is
            // safe to reconstruct the box so that it gets deallocated.
            drop(unsafe { Box::from_raw(init_ptr) });
            let _ = unsafe { sceKernelDeleteThread(thread) };
            Err(err.into())
        } else {
            Ok(Thread { id: thread })
        }
    }

    #[allow(private_interfaces, reason = "Internal API")]
    pub unsafe fn new(stack: usize, init: Box<ThreadInit>) -> io::Result<Thread> {
        let attr = cfg_select! {
            feature = "kernel" => ThreadAttributes::default(),
            all(prx, not(feature = "kernel")) => ThreadAttributes::UserMode,
            pbp => ThreadAttributes::UserMode | ThreadAttributes::UseVFPU,
            _ => ThreadAttributes::default(),
        };
        unsafe { Thread::with_attr(stack, attr, init) }
    }

    pub fn join(self) {
        let id = self.into_id();

        let ret = sceKernelWaitThreadEndCB(id, None);
        assert!(ret.is_ok(), "failed to join thread: {}", unsafe {
            SceError::from_raw_unchecked(ret.as_inner())
        });

        let ret = unsafe { sceKernelDeleteThread(id) };
        assert!(ret.is_ok(), "failed to delete join thread: {}", unsafe {
            SceError::from_raw_unchecked(ret.as_inner())
        });
    }

    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn into_id(self) -> ThreadId {
        ManuallyDrop::new(self).id
    }
}

// PSP threads are detached by default
// impl Drop for Thread {
//     fn drop(&mut self) {
//         // we can not call detach, so just panic if thread spawn without join
//     }
// }

pub fn available_parallelism() -> io::Result<NonZero<usize>> {
    Ok(unsafe { NonZero::new_unchecked(1) })
}

pub fn current_os_id() -> Option<ThreadOsId> {
    super::sceKernelGetThreadId().ok()
}

pub fn yield_now() {
    // Zero is always a valid parameter for this function and it will always succeed.
    let _res = super::sceKernelRotateThreadReadyQueue(0);
}

pub fn set_name(_name: &CStr) {
    // PSP doesn't allow to set the name after creation.
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
