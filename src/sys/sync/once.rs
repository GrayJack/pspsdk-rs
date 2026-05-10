use core::{
    cell::Cell,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    sync::{self as public, ExclusiveState},
    sys::{
        thread::{
            sceKernelClearEventFlag, sceKernelCreateEventFlag, sceKernelDeleteEventFlag,
            sceKernelSetEventFlag, sceKernelWaitEventFlag, EventFlagAttributes, EventFlagId,
            EventFlagWaitKinds,
        },
        SceError,
    },
};

const INCOMPLETE: u32 = 0;
const POISONED: u32 = 1;
const RUNNING: u32 = 2;
const COMPLETE: u32 = 3;

const EVENT_COMPLETE: u32 = 1 << 0;
const EVENT_POISONED: u32 = 1 << 1;

const UNINIT_FLAG: u32 = u32::MAX;
const INITIALIZING_FLAG: u32 = u32::MAX - 1;

pub struct OnceState {
    poisoned: bool,
    set_state_to: Cell<u32>,
}

impl OnceState {
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    #[inline]
    pub fn poison(&self) {
        self.set_state_to.set(POISONED);
    }
}

struct CompletionGuard<'a> {
    once: &'a Once,
    set_state_on_drop_to: u32,
}

impl Drop for CompletionGuard<'_> {
    fn drop(&mut self) {
        let new_state = self.set_state_on_drop_to;
        let old_state = self.once.state.swap(new_state, Ordering::Release);

        debug_assert_eq!(old_state, RUNNING, "completion guard dropped outside RUNNING state");
        self.once.signal_waiters(new_state);
    }
}

pub struct Once {
    state: AtomicU32,
    event_flag_id: AtomicU32,
}

impl Once {
    #[inline]
    pub const fn new() -> Once {
        Once {
            state: AtomicU32::new(INCOMPLETE),
            event_flag_id: AtomicU32::new(UNINIT_FLAG),
        }
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }

    #[inline]
    pub(crate) fn state(&mut self) -> ExclusiveState {
        let _ = self.get_event_flag();
        match *self.state.get_mut() {
            INCOMPLETE => ExclusiveState::Incomplete,
            POISONED => ExclusiveState::Poisoned,
            COMPLETE => ExclusiveState::Complete,
            _ => unreachable!("invalid Once state"),
        }
    }

    #[inline]
    pub(crate) fn set_state(&mut self, new_state: ExclusiveState) {
        let _ = self.get_event_flag();
        *self.state.get_mut() = match new_state {
            ExclusiveState::Incomplete => INCOMPLETE,
            ExclusiveState::Poisoned => POISONED,
            ExclusiveState::Complete => COMPLETE,
        };
    }

    #[cold]
    #[track_caller]
    pub fn wait(&self, ignore_poisoning: bool) {
        let Some(flag) = self.get_event_flag() else {
            return;
        };
        let wait_mask = EVENT_COMPLETE | EVENT_POISONED;

        let mut matched_bits = 0;
        loop {
            match self.state.load(Ordering::Acquire) {
                COMPLETE => return,
                POISONED if !ignore_poisoning => {
                    panic!("Once instance has previously been poisoned");
                },
                POISONED => return,
                _ => {
                    let _ = sceKernelWaitEventFlag(
                        flag,
                        wait_mask,
                        EventFlagWaitKinds::Or,
                        &mut matched_bits,
                        None,
                    );
                },
            }
        }
    }

    #[cold]
    #[track_caller]
    pub fn call(&self, ignore_poisoning: bool, f: &mut dyn FnMut(&public::OnceState)) {
        let mut current_state = self.state.load(Ordering::Acquire);

        loop {
            match current_state {
                COMPLETE => return,
                POISONED if !ignore_poisoning => {
                    panic!("Once instance has previously been poisoned");
                },
                INCOMPLETE | POISONED => {
                    if self
                        .state
                        .compare_exchange(
                            current_state,
                            RUNNING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_err()
                    {
                        current_state = self.state.load(Ordering::Acquire);
                        continue;
                    }

                    let Some(flag) = self.get_event_flag() else {
                        continue;
                    };
                    let _ = sceKernelClearEventFlag(flag, EVENT_COMPLETE | EVENT_POISONED);

                    let mut waiter_queue = CompletionGuard {
                        once: self,
                        set_state_on_drop_to: POISONED,
                    };

                    let f_state = public::OnceState {
                        inner: OnceState {
                            poisoned: current_state == POISONED,
                            set_state_to: Cell::new(COMPLETE),
                        },
                    };

                    f(&f_state);
                    waiter_queue.set_state_on_drop_to = f_state.inner.set_state_to.get();
                    return;
                },
                _ => {
                    let flag = self
                        .get_event_flag()
                        .expect("event flag should be initialized at this point");
                    let mut matched_bits = 0;

                    let _ = sceKernelWaitEventFlag(
                        flag,
                        EVENT_COMPLETE | EVENT_POISONED,
                        EventFlagWaitKinds::Or,
                        &mut matched_bits,
                        None,
                    );

                    current_state = self.state.load(Ordering::Acquire);
                },
            }
        }
    }
}


impl Once {
    fn get_event_flag(&self) -> Option<EventFlagId> {
        let mut i = 0;
        while i < 0x10 {
            i += 1;
            match self.event_flag_id.load(Ordering::Acquire) {
                UNINIT_FLAG => {
                    if self
                        .event_flag_id
                        .compare_exchange(
                            UNINIT_FLAG,
                            INITIALIZING_FLAG,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        match self.create_event_flag() {
                            Ok(id) => return Some(id),
                            Err(_) => continue,
                        }
                    }
                },
                INITIALIZING_FLAG => core::hint::spin_loop(),
                raw => return Some(unsafe { EventFlagId::from_raw_unchecked(raw) }),
            }
        }
        None
    }

    #[cold]
    fn create_event_flag(&self) -> Result<EventFlagId, SceError> {
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
                self.event_flag_id.store(id.as_inner(), Ordering::Release);
                Ok(id)
            },
            Err(err) => {
                self.event_flag_id.store(UNINIT_FLAG, Ordering::Release);
                Err(err)
            },
        }
    }

    fn signal_waiters(&self, new_state: u32) {
        let Some(flag) = self.get_event_flag() else {
            return;
        };

        let bit = match new_state {
            COMPLETE => EVENT_COMPLETE,
            POISONED => EVENT_POISONED,
            _ => return,
        };

        let _ = sceKernelSetEventFlag(flag, bit);
    }
}

impl Drop for Once {
    fn drop(&mut self) {
        let raw_id = self.event_flag_id.load(Ordering::Relaxed);

        if raw_id != UNINIT_FLAG && raw_id != INITIALIZING_FLAG {
            let id = unsafe { EventFlagId::from_raw_unchecked(raw_id) };
            let _ = sceKernelDeleteEventFlag(id);
        }
    }
}
