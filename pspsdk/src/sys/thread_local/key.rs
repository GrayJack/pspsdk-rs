#![allow(unused, reason = "FW version dependant")]
use core::{
    mem,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

use crate::{
    allocators::MemoryPartitionId,
    sync::RwLock,
    sys::{
        sync::SemaRwLock,
        thread::{TlsPoolAttributes, TlsPoolId},
    },
};

#[cfg(fw_has_thread_local)]
use crate::sys::thread::{sceKernelCreateTlspl, sceKernelDeleteTlspl};

mod racy;

use pspsdk_macros::psp_fw_select;
pub(super) use racy::LazyKey;

const MAX_KEYS: usize = cfg_select! {
    not(fw_has_thread_local) => 0,
    fw_has_thread_local => 128,
};
const MAX_THREADS: usize = 500;

#[repr(C, align(8))]
struct TlsBlock {
    slots: [*mut u8; MAX_KEYS],
}

static DTORS: RwLock<[Option<unsafe extern "C" fn(*mut u8)>; MAX_KEYS], SemaRwLock> =
    RwLock::new_with([None; MAX_KEYS], SemaRwLock::new());

pub type Key = usize;

static NEXT_KEY: AtomicUsize = AtomicUsize::new(1);
static PSP_TLS_POOL: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    cfg_select! {
        not(fw_has_thread_local) => {},
        fw_has_thread_local => {
            // No-op if already initialized
            if PSP_TLS_POOL.load(Ordering::Acquire) != 0 {
                return;
            }

            let pool = unsafe {
                crate::sys::thread::sceKernelCreateTlspl(
                    c"SDK_TLS_POOL".as_ptr().cast(),
                    MemoryPartitionId::MainUser,
                    TlsPoolAttributes::default(),
                    size_of::<TlsBlock>(),
                    MAX_THREADS,
                    None,
                )
            };

            let Some(pool) = pool.ok() else {
                crate::rtabort!("failed to initialize TLS Pool");
            };

            match PSP_TLS_POOL.compare_exchange(
                0,
                pool.to_inner(),
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {},
                Err(_) => {
                    // Another thread initialized it first.
                    let _ = crate::sys::thread::sceKernelDeleteTlspl(pool);
                },
            }
        },
    }
}

/// Safety: Must be called only once
pub unsafe fn cleanup() {
    cfg_select! {
        not(fw_has_thread_local) => {},
        fw_has_thread_local => {
            let tls_id = get_pool_id();
            if tls_id.to_inner() == 0 {
                return;
            }

            let _res = crate::sys::thread::sceKernelDeleteTlspl(tls_id);
            PSP_TLS_POOL.store(0, Ordering::Release);
        },
    }
}

#[inline]
pub fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    init();

    cfg_select! {
        not(fw_has_thread_local) => 1,
        fw_has_thread_local => {
            let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
            if key >= MAX_KEYS {
                crate::rtabort!("out of TLS keys");
            }

            let mut guard = DTORS.write();
            let slot = guard.get_mut(key - 1).unwrap_or_else(|| {
                crate::eprintln!("failed on create");
                fail()
            });

            *slot = dtor;

            key
        },
    }
}

#[cold]
fn fail() -> ! {
    crate::rtabort!("Unexpected TLS failure")
}

fn get_pool_id() -> TlsPoolId {
    let raw_id = PSP_TLS_POOL.load(Ordering::Acquire);
    if raw_id == 0 {
        init();
        unsafe { mem::transmute::<u32, TlsPoolId>(PSP_TLS_POOL.load(Ordering::Acquire)) }
    } else {
        unsafe { mem::transmute::<u32, TlsPoolId>(raw_id) }
    }
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    cfg_select! {
        not(fw_has_thread_local) => {},
        fw_has_thread_local => {
            let addr = crate::sys::get_tls_addr(get_pool_id()).unwrap_or_else(|| fail());

            let block = addr.cast::<TlsBlock>();

            let slot = unsafe { (*block).slots.get_mut(key - 1).unwrap_or_else(|| fail()) };

            *slot = value
        },
    }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    cfg_select! {
        not(fw_has_thread_local) => fail(),
        fw_has_thread_local => {
            let addr = crate::sys::get_tls_addr(get_pool_id()).unwrap_or_else(|| fail());

            let block = addr.cast::<TlsBlock>();

            let slot = unsafe { (*block).slots.get(key - 1).unwrap_or_else(|| fail()) };

            *slot
        },
    }
}

#[inline]
pub unsafe fn destroy(_key: Key) {
    crate::eprintln!("key::get called");
    // A Rust key is only an index into the shared table.
    //
    // Do not delete the PSP TLS pool here: other Rust TLS keys still use it.
}

pub unsafe fn run_dtors() {
    loop {
        let mut ran_dtor = false;
        let key_count = NEXT_KEY.load(Ordering::Acquire);

        for key in 1..key_count {
            let value = unsafe { get(key) };

            if value.is_null() || value.addr() <= 2 {
                continue;
            }

            let dtor = {
                let dtors = DTORS.read();
                dtors[key - 1]
            };

            if let Some(dtor) = dtor {
                ran_dtor = true;
                unsafe {
                    dtor(value);
                }
            }
        }

        if !ran_dtor {
            break;
        }
    }
}
