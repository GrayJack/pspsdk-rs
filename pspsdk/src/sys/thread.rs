//! Thread operations and management.
use core::ffi::c_void;

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{
    mem::MemoryPartitionId, time::SystemClock, SceError, SceIntoOkValue, SceRawUid, SceResult,
    SceResultOk, SceSize, SceUid,
};

pub use crate::sys::usersystemlib::{
    sceKernelGetTlsAddr, sceKernelLockLwMutex, sceKernelLockLwMutexCB, sceKernelReferLwMutexStatus,
    sceKernelTryLockLwMutex, sceKernelUnlockLwMutex,
};

/// The thread UID, created with [`sceKernelCreateThread`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ThreadId(SceRawUid);

/// The thread entry function.
///
/// The returns value will be the end result of the function of the status of
/// [`sceKernelExitThread`] or [`sceKernelExitDeleteThread`].
pub type ThreadEntryFn = unsafe extern "C" fn(args: SceSize, argp: *mut c_void) -> SceResult<u32>;

/// Attributes for threads.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ThreadAttributes {
    /// Thread runs in User mode.
    ///
    /// This is done automatically if the thread creating it is in user mode.
    UserMode = 0x80000000,
    /// Thread runs in USB/WLAN mode.
    UsbWlanMode = 0xA0000000,
    /// Thread runs in VSH mode.
    VshMode = 0xC0000000,
    /// Thread runs in Application mode.
    AppMode = 0xB0000000,
    /// Thread runs in MS mode.
    MsMode = 0x90000000,

    /// Specifies that the thread memory area will not be filled with `0xFF` on creation.
    NoFillStack = 0x00100000,
    /// Specifies that the thread memory area should be cleared to zero when deleted.
    ClearStack = 0x00200000,
    /// Specifies that the stack area is allocated from the lower addresses in memory, not the
    /// higher ones.
    LowStack = 0x00400000,

    /// Specifies that the scratchpad memory is available.
    ///
    /// # Firmware Version
    /// This is not usable on PSP firmware version 1.00.
    UseScratchSRAM = 0x00008000,
    /// Specifies that the VFPU is available.
    UseVFPU = 0x00004000,
    /// Specifies that the FPU cannot be used.
    NeverUseFPU = 0x00002000,
}

/// The possible states/status that a PSP thread can be.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("PspThreadStatus"))]
pub enum ThreadState {
    /// The thread is in running state.
    #[doc(alias("PSP_THREAD_RUNNING"))]
    Run   = 0x01,
    /// The thread is ready to execute.
    #[doc(alias("PSP_THREAD_READY"))]
    Ready = 0x02,
    /// The thread is waiting to resume.
    #[doc(alias("PSP_THREAD_WAITING"))]
    Wait  = 0x04,
    /// The thread is suspended.
    #[doc(alias("PSP_THREAD_SUSPEND"))]
    Suspend = 0x08,
    /// The thread is dormant.
    #[doc(alias("PSP_THREAD_STOPPED"))]
    Dormant = 0x10,
    /// The thread is dead.
    ///
    /// Killed by the thread manager. Stack overflow often causes this state.
    #[doc(alias("PSP_THREAD_KILLED"))]
    Dead  = 0x20,

    /// The thread is in both wait and suspend states.
    WaitSuspend = Wait | Suspend,
}

/// The possible kinds of wait state of the thread.
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ThreadWaitKind {
    #[default]
    #[doc(hidden)]
    Unknown = 0x00,
    Sleep = 0x01,
    Delay = 0x02,
    Semaphore = 0x03,
    EventFlag = 0x04,
    MessageBox = 0x05,
    VariablePoolLength = 0x06,
    FixedPoolLength = 0x07,
    MessagePipe = 0x08,
    WaitThreadEnd = 0x09,
    ReleaseThreadEventHandler = 0x0A,
    DeleteCallback = 0x0B,
    Mutex = 0x0C,
    LightweightMutex = 0x0D,
    TLSPool = 0x0E,

    SleepCallback = 0x101,
    DelayCallback = 0x102,
    SemaphoreCallback = 0x103,
    EventFlagCallback = 0x104,
    MessageBoxCallback = 0x105,
    VariablePoolLengthCallback = 0x106,
    FixedPoolLengthCallback = 0x107,
    MessagePipeCallback = 0x108,
    WaitThreadEndCallback = 0x109,
    MutexCallback = 0x10C,
    LightweightMutexCallback = 0x10D,
}

/// Extra options used when creating threads.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelThreadOptParam")]
pub struct ThreadOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The memory partition ID to allocate the thread stack.
    pub stack_mem_partition: MemoryPartitionId,
}

/// The status information for a thread.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelThreadInfo")]
#[allow(unpredictable_function_pointer_comparisons)]
pub struct ThreadInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the thread.
    pub name: [u8; 32],
    /// The thread attributes.
    pub attr: ThreadAttributes,
    /// The thread current status.
    pub status: ThreadState,
    /// The thread entry function.
    pub entry: Option<ThreadEntryFn>,
    /// The thread stack pointer.
    pub stack: *mut c_void,
    /// The thread stack size.
    pub stack_size: SceSize,
    /// The pointer to the gp register.
    pub gp_reg: *mut c_void,
    /// The thread initial priority.
    pub init_priority: u32,
    /// The thread current priority.
    pub curr_priority: u32,
    /// The wait kind when the thread is in a [`Wait`](ThreadState::Wait) state.
    pub wait_kind: ThreadWaitKind,
    /// The wait target UID.
    pub wait_id: SceUid,
    /// The wake-up count.
    pub wakeup_count: u32,
    /// The exit status of the thread.
    pub exit_status: u32,
    /// The number of the clock cycles run.
    pub run_clocks: SystemClock,
    /// The number of times the CPU execution rights was taken away from the thread because of
    /// interrupts.
    pub intr_preempt_count: u32,
    /// The number of times the CPU execution rights was taken from the thread because other
    /// threads.
    pub thread_preempt_count: u32,
    /// The number of times the thread released the CPU execute right because of a service
    /// call.
    pub release_count: u32,
    /// A indicator that a callback was notified, but the callback function was not executed yet.
    pub notify_callback: u32,
}

/// Statistics about a running thread.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelThreadRunStatus")]
pub struct ThreadRunStatus {
    /// The size of this structure.
    pub size: SceSize,
    /// The thread current status.
    pub status: u32,
    /// The thread current priority.
    pub curr_priority: u32,
    /// The wait kind when the thread is in a [`Wait`](ThreadState::Wait) state.
    pub wait_kind: ThreadWaitKind,
    /// The wait target UID.
    pub wait_id: SceUid,
    /// The wake-up count.
    pub wakeup_count: u32,
    /// The number of the clock cycles run.
    pub run_clocks: SystemClock,
    /// The number of times the CPU execution rights was taken away from the thread because of
    /// interrupts.
    pub intr_preempt_count: u32,
    /// The number of times the CPU execution rights was taken from the thread because other
    /// threads.
    pub thread_preempt_count: u32,
    /// The number of times the thread released the CPU execute right because of a service
    /// call.
    pub release_count: u32,
    /// A indicator that a callback was notified, but the callback function was not executed yet.
    pub notify_callback: u32,
}

/// The semaphore UID, created with [`sceKernelCreateSema`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SemaId(SceUid);

/// Semaphore options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelSemaOptParam")]
pub struct SemaphoreOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// Attributes for semaphore creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SemaphoreAttributes {
    /// Uses FIFO logic in the wait queue.
    #[default]
    WaitByFIFO = 0x000,
    /// Uses thread priority logic in the wait queue.
    WaitByPriority = 0x100,
}

/// The information of the current state of a semaphore.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelSemaInfo")]
pub struct SemaphoreInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the semaphore.
    pub name: [u8; 32],
    /// The semaphore attributes.
    pub attr: SemaphoreAttributes,
    /// The initial value the semaphore was created.
    pub init_val: i32,
    /// The current value the semaphore has set.
    pub curr_val: i32,
    /// The maximum value the semaphore has set.
    pub max_val: i32,
    /// The number of threads waiting on the semaphore.
    pub num_wait_threads: u32,
}

/// The event flag UID, created with [`sceKernelCreateEventFlag`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EventFlagId(SceUid);

/// Event Flag options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelEventFlagOptParam")]
pub struct EventFlagOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// Attributes for event flag creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EventFlagAttributes {
    /// The event flag is only permitted to be waited upon by a single thread.
    #[default]
    WaitSingle = 0x000,
    /// The event flag is permitted to be waited upon by multiple threads.
    WaitMultiple = 0x200,
}

/// The event flag wait kinds.
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EventFlagWaitKinds {
    /// Wait for all bits in the pattern to be set.
    #[default]
    And = 0x00,
    /// Wait for one or more bits in the pattern to be set.
    Or  = 0x01,
    /// Clear all bits when the condition matches.
    ClearAll = 0x10,
    /// Clear the wait pattern when the condition matches.
    ClearPat = 0x20,
}

/// The information of the current state of a event flag.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventFlagInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the event flag.
    pub name: [u8; 32],
    /// The attributes for of the event flag.
    pub attr: EventFlagAttributes,
    /// The initial patter of the event flag.
    pub init_pattern: u32,
    /// The current pattern of the event flag.
    pub curr_pattern: u32,
    /// The number of threads waiting on the event flag.
    pub num_wait_threads: u32,
}

/// The mutex UID, created with [`sceKernelCreateMutex`].
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MutexId(SceUid);

/// Mutex options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelMutexOptParam")]
pub struct MutexOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// Attributes for mutex creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MutexAttributes {
    /// Uses FIFO logic in the wait queue.
    #[default]
    WaitByFIFO = 0x000,
    /// Uses thread priority logic in the wait queue.
    WaitByPriority = 0x100,
    /// Allows reentrant locks on a mutex by the thread that acquired the ID.
    ReentrantLock = 0x200,
}

/// The information of the current state of a mutex.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelMutexInfo")]
pub struct MutexInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the mutex.
    pub name: [u8; 32],
    /// The attributes for of the mutex.
    pub attr: MutexAttributes,
    /// The initial lock count value of the mutex.
    pub init_count: i32,
    /// The current lock count of the mutex.
    pub curr_count: i32,
    /// The current owner of the mutex.
    pub curr_owner: SceUid,
    /// The number of threads waiting on the mutex.
    pub num_wait_threads: u32,
}

/// The lightweight mutex UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LwMutexId(SceUid);

/// The work area for the lightweight mutex to work on.
///
/// The instances of this type **must** live in the PSP user RAM partition to work with the
/// functions related to lightweight mutex (i.e. either live in the data section of a user PRX or
/// PBP or be allocated in that memory partition).
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias = "SceKernelLwMutexWork")]
pub struct LwMutexWorkArea {
    /// The locking count
    pub lock_count: i32,
    /// The locking thread.
    pub lock_thread: ThreadId,
    /// The lightweight mutex attribute
    pub attr: MutexAttributes,
    /// The number of threads waiting on the mutex.
    pub num_wait_threads: u32,
    /// The lightweight mutex UID.
    pub uid: LwMutexId,
    /// Padding,
    pub pad: [u32; 3],
}

/// Lightweight mutex options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelLwMutexOptParam")]
pub struct LwMutexOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// The information of the current state of a lightweight mutex.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelLwMutexInfo")]
pub struct LwMutexInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the lightweight mutex.
    pub name: [u8; 32],
    /// The attributes for of the lightweight mutex.
    pub attr: MutexAttributes,
    /// The lightweight mutex UID.
    pub uid: LwMutexId,
    /// The address of the lightweight mutex work area.
    pub work_addr: *mut LwMutexWorkArea,
    /// The initial lock count value of the lightweight mutex.
    pub init_count: i32,
    /// The current lock count of the lightweight mutex.
    pub curr_count: i32,
    /// The current owner of the lightweight mutex.
    pub curr_owner: SceUid,
    /// The number of threads waiting on the lightweight mutex.
    pub num_wait_threads: u32,
}

/// The message box UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MsgBoxId(SceUid);

/// Message box options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelMbxOptParam")]
pub struct MsgBoxOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// The header of the a message package with user data.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelMsgPacket")]
pub struct MsgPacket {
    /// A pointer to the next package.
    pub next: *mut MsgPacket,
    /// The priority of the message.
    pub msg_priority: u8,
    /// Reserved.
    pub dummy: [u8; 3],
}

/// Attributes for message box creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MsgBoxAttributes {
    /// Uses FIFO logic in the thread wait queue.
    #[default]
    WaitByFIFO = 0x000,
    /// Uses thread priority logic in the thread wait queue.
    WaitByPriority = 0x100,
    /// Uses FIFO logic in the message queue.
    MsgByFIFO = 0x000,
    /// Uses thread priority logic in the message queue.
    MsgByPriority = 0x400,
}

/// The information of the current state of a message box.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelMbxInfo")]
pub struct MsgBoxInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the message box.
    pub name: [u8; 32],
    /// The attributes for of the message box.
    pub attr: MsgBoxAttributes,
    /// The number of threads waiting on the message box.
    pub num_wait_threads: u32,
    /// The number of remaining messages to be received of the message box.
    pub num_messages: u32,
    /// The top message of the message box.
    pub top_msg: *mut MsgPacket,
}

/// The message pipe UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MsgPipeId(SceUid);

/// Message pipe options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelMppOptParam")]
pub struct MsgPipeOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// Attributes for message pipe creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MsgPipeAttributes {
    /// Uses FIFO logic in the sender thread wait queue.
    #[default]
    SenderWaitByFIFO = 0x0000,
    /// Uses thread priority logic in the sender thread wait queue.
    SenderWaitByPriority = 0x0100,
    /// Uses FIFO logic in the receiver thread wait queue.
    ReceiverWaitByFIFO = 0x0000,
    /// Uses thread priority logic in the receiver thread wait queue.
    ReceiverWaitByPriority = 0x1000,
    /// Allocates a message box closest to memory bottom (i.e. High address).
    MemBottom = 0x4000,
    BothWaitByFIFO = SenderWaitByFIFO | ReceiverWaitByFIFO,
    BothWaitByPriority = SenderWaitByPriority | ReceiverWaitByPriority,
}

/// Possible wait strategies for message pipe.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MsgPipeWaitKind {
    /// On send, waits until all data is received. While on receive, waits until the receive buffer
    /// is full.
    #[default]
    Entire = 0x00,
    /// On send, waits until one byte can be send. While on receive, waits until one byte can be
    /// received.
    Imediate = 0x01,
}

/// The information of the current state of a message pipe.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelMppInfo")]
pub struct MsgPipeInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the message pipe.
    pub name: [u8; 32],
    /// The attributes for of the message pipe.
    pub attr: MsgPipeAttributes,
    /// The size of the message pipe buffer.
    pub buf_size: SceSize,
    /// The unused size of the message pipe buffer.
    pub buf_free_size: SceSize,
    /// The number of send threads waiting on the message pipe.
    pub num_send_wait_threads: u32,
    /// The number of receiver threads waiting on the message pipe.
    pub num_recv_wait_threads: u32,
}

/// The variable-sized memory pool UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VplId(SceUid);

/// Variable-sized memory pool options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelVplOptParam")]
pub struct VplOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// Attributes for variable-sized memory pool creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum VplAttributes {
    /// Uses FIFO logic in the thread wait queue.
    #[default]
    WaitByFIFO = 0x0000,
    /// Uses thread priority logic in the thread wait queue.
    WaitByPriority = 0x0100,
    /// Threads with smaller memory requirements may be serviced ahead of queued threads with
    /// larger requirements.
    ThreadPass = 0x0200,
    /// Allocates a variable-sized memory pool closest to memory bottom (i.e. High address).
    MemBottom = 0x4000,
}

/// The information of the current state of a variable-sized memory pool.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelVplInfo")]
pub struct VplInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the memory pool.
    pub name: [u8; 32],
    /// The attributes for of the memory pool.
    pub attr: VplAttributes,
    /// The size of the memory pool buffer.
    pub pool_size: SceSize,
    /// The unused size of the memory pool buffer.
    pub pool_free_size: SceSize,
    /// The number of threads waiting on the memory pool.
    pub num_wait_threads: u32,
}

/// The fixed-sized memory pool UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FplId(SceUid);

/// Fixed-sized memory pool options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelFplOptParam")]
pub struct FplOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The alignment to use for the memory pool.
    ///
    /// Zero defaults to 4 bytes aligned (?).
    pub alignment: SceSize,
}

/// Attributes for fixed-sized memory pool creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FplAttributes {
    /// Uses FIFO logic in the thread wait queue.
    #[default]
    WaitByFIFO = 0x0000,
    /// Uses thread priority logic in the thread wait queue.
    WaitByPriority = 0x0100,
    /// Allocates a fixed-sized memory pool closest to memory bottom (i.e. High address).
    MemBottom = 0x4000,
}

/// The information of the current state of a fixed-sized memory pool.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelFplInfo")]
pub struct FplInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the memory pool.
    pub name: [u8; 32],
    /// The attributes for of the memory pool.
    pub attr: FplAttributes,
    /// The size of one block in bytes for the memory pool.
    pub block_size: SceSize,
    /// The number of blocks of the memory pool
    pub num_blocks: SceSize,
    /// The number of unused blocks in the memory pool.
    pub free_blocks: SceSize,
    /// The number of threads waiting on the memory pool.
    pub num_wait_threads: u32,
}

/// The user TLS pool UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TlsPoolId(SceUid);

/// TLS memory pool options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelTlsplOptParam")]
pub struct TlsPoolOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The alignment to use for the memory pool.
    ///
    /// Zero defaults to 4 bytes aligned (?).
    pub alignment: SceSize,
}

/// Attributes for TLS memory pool creation.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TlsPoolAttributes {
    /// Uses FIFO logic in the thread wait queue.
    #[default]
    WaitByFIFO = 0x0000,
    /// Uses thread priority logic in the thread wait queue.
    WaitByPriority = 0x0100,
    /// Allocates a TLS memory pool closest to memory bottom (i.e. High address).
    MemBottom = 0x4000,
}

/// The information of the current state of a TLS memory pool.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelTlsplInfo")]
pub struct TlsPoolInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the memory pool.
    pub name: [u8; 32],
    /// The attributes for of the memory pool.
    pub attr: TlsPoolAttributes,
    /// The size of one block in bytes for the memory pool.
    pub block_size: SceSize,
    /// The number of blocks of the memory pool
    pub num_blocks: SceSize,
    /// The number of unused blocks in the memory pool.
    pub free_blocks: SceSize,
    /// The number of threads waiting on the memory pool.
    pub num_wait_threads: u32,
}

/// The alarm UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AlarmId(SceUid);

/// The alarm entry function.
///
/// # Parameters
///
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
///
/// # Return Value
///
/// Returns in how much time the handler must be called again (`>= 1`), or zero if the alarm should
/// be deleted.
#[doc(alias("SceKernelAlarmHandler"))]
pub type AlarmHandler = unsafe extern "C" fn(common: *mut c_void) -> u32;

/// The information of the current state of a alarm timer.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelAlarmInfo")]
pub struct AlarmInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The schedule time to call the registered [`AlarmHandler`].
    pub schedule: SystemClock,
    /// The alarm registered handler function.
    pub handler: Option<AlarmHandler>,
    /// The argument passed to `handler`.
    pub common: *mut c_void,
}

/// The virtual timer UID.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VirtualTimerId(SceUid);

/// The virtual timer entry function.
///
/// # Parameters
///
/// - `id`: The virtual timer UID.
/// - `schedule` **[[In parameter]]**: A reference to a time as the schedule time to execute the
///   `handler`.
/// - `actual` **[[InOut parameter]]** A reference to the actual start time.
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
///
/// # Return Value
///
/// Returns the microseconds that the handler will be called again (`>= 1`), or zero if the handler
/// is cancelled.
#[doc(alias("SceKernelVTimerHandler"))]
pub type VirtualTimerHandler = unsafe extern "C" fn(
    id: VirtualTimerId,
    schedule: *mut SystemClock,
    actual: *mut SystemClock,
    common: *mut c_void,
) -> u32;

/// The virtual timer entry function.
///
/// # Parameters
///
/// - `id`: The virtual timer UID.
/// - `schedule` **[[In parameter]]**: A reference to a time as the schedule time to execute the
///   `handler`.
/// - `actual` **[[InOut parameter]]** A reference to the actual start time.
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
///
/// # Return Value
///
/// Returns the microseconds that the handler will be called again (`>= 1`), or zero if the handler
/// is cancelled.
#[doc(alias("SceKernelVTimerHandlerWide"))]
pub type VirtualTimerHandlerWide = unsafe extern "C" fn(
    id: VirtualTimerId,
    schedule: *mut u64,
    actual: *mut u64,
    common: *mut c_void,
) -> u32;

/// Virtual Timer options.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceKernelVTimerOptParam")]
pub struct VirtualTimerOptions {
    /// The size of this structure.
    pub size: SceSize,
}

/// The possible running states of a virtual timer.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum VirtualTimerState {
    /// The timer is/was not running.
    #[default]
    NotRunning = 0,
    /// The timer is/was running.
    Running = 1,
}

/// The information of the current state of a alarm timer.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelVTimerInfo")]
pub struct VirtualTimerInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the virtual timer.
    pub name: [u8; 32],
    /// The current running state of the virtual timer.
    pub state: VirtualTimerState,
    /// The base time of the virtual timer.
    pub base: SystemClock,
    /// The current timer of the virtual timer.
    pub current: SystemClock,
    /// The schedule time to call the registered [`VirtualTimerHandler`].
    pub schedule: SystemClock,
    /// The virtual timer registered handler function.
    pub handler: Option<VirtualTimerHandler>,
    /// The argument passed to `handler`.
    pub common: *mut c_void,
}

/// The callback UID, created with [`sceKernelCreateCallback`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CallbackId(SceUid);

/// The termination state of a `CallbackFunction`.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CallbackTermState {
    /// Function had a normal termination.
    #[default]
    NormalTermination = 0x00,
    /// The function self deleted the callback deleted.
    DeletedCallback = 0x01,
}

/// The callback entry function.
///
/// # Parameters
///
/// - `count`: The number of times [`sceKernelNotifyCallback`] was called before this callback
///   functions was called.
/// - `arg`: The argument from [`sceKernelNotifyCallback`].
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with this function.
///
/// # Return Value
///
/// The termination state of the callback on success, error value otherwise.
#[doc(alias = "SceKernelCallbackFunction")]
pub type CallbackFunction =
    unsafe extern "C" fn(count: u32, arg: u32, common: *mut c_void) -> CallbackTermState;

/// The information of the current state of a callback.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelCallbackInfo")]
pub struct CallbackInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the callback.
    pub name: [u8; 32],
    /// The thread UID to be notified by the callback.
    pub thread_id: ThreadId,
    /// The registered callback entry function.
    pub entry: Option<CallbackFunction>,
    /// The number of times the callback notify was delayed without calling the entry function.
    pub notify_count: u32,
    /// The argument for the callback.
    pub notify_arg: i32,
}

/// The possible return values of [`sceKernelCheckCallback`].
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CallbackCheckStatus {
    /// No callback reported.
    #[default]
    NoCallback = 0x00,

    /// Callback reported and called.
    CallbackCalled = 0x01,
}

#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SystemStatus {
    /// Thread executing with dispatch and interrupt enabled.
    #[default]
    AllEnabled = 0x00,
    /// Dispatch is disabled.
    DisabledDispatch = 0x01,
    /// Interrupts are disabled.
    DisabledInterrupt = 0x03,
    /// Thread-independent execution happening.
    NoThread = 0x04,
}

/// The information of the current state of the system.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelSystemStatus")]
pub struct SystemInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The current status of the system.
    pub status: SystemStatus,
    /// The number of CPU clocks when thread are not executing.
    pub idle_clocks: SystemClock,
    /// The number of times transitioned out of idle.
    pub comes_out_of_idle_count: u32,
    /// The number of times the system switch contexts for threads.
    pub thread_switch_count: u32,
    /// The number of times the system switch contexts with VFPU.
    pub vfpu_switch_count: u32,
}

/// The kinds of possible thread ID
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias = "SceKernelIdListType")]
pub enum ThreadIdKind {
    #[default]
    Any   = 1,
    Semaphore = 2,
    EventFlag = 3,
    MessageBox = 4,
    VariablepMemoryPool = 5,
    FixeMemoryPool = 6,
    MessagePipe = 7,
    Callback = 8,
    ThreadEventHandler = 9,
    Alarm = 10,
    VirtualTimer = 11,
    Mutex = 12,
    LightweightMutex = 13,
    TlsMemoryPool = 14,
    SleepThread = 64,
    DelayThread = 65,
    SuspendThread = 66,
    DormantThread = 67,
}

/// The thread event UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ThreadEventId(SceUid);

/// The thread events that an thread event handler may be called.
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ThreadEvents {
    /// When a thread is created.
    Create = 0x01,
    /// When a thread is started.
    Start  = 0x02,
    /// When a thread terminates.
    Exit   = 0x04,
    /// When a thread is deleted.
    Delete = 0x08,

    /// On all events.
    All    = Create | Start | Exit | Delete,
}

/// The termination state of a `ThreadEventHandler`.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ThreadEventHandlerTermState {
    /// Function had a normal termination.
    #[default]
    NormalTermination = 0x00,
    /// The event handler will be released.
    Release = 0x01,
}

/// The thread event entry function.
///
/// # Parameters
///
/// - `kind`: The kind of event. One of [`ThreadEvents`].
/// - `id`: The thread UID of the thread that caused the thread event.
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with this function.
///
/// # Return Value
///
/// The termination state of the thread event on success, error value otherwise.
#[doc(alias = "SceKernelThreadEventHandler")]
pub type ThreadEventHandler = unsafe extern "C" fn(
    kind: ThreadEvents,
    id: ThreadId,
    common: *mut c_void,
) -> ThreadEventHandlerTermState;

/// The information of the current state of a thread event.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias = "SceKernelThreadEventHandlerInfo")]
pub struct ThreadEventInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the callback.
    pub name: [u8; 32],
    /// The thread UID of the target thread.
    pub thread_id: ThreadId,
    /// The thread events which the handler is executed.
    pub event_mask: ThreadEvents,
    /// The registered thread event handler function.
    pub handler: Option<ThreadEventHandler>,
    /// The argument passed to `handler`.
    pub common: *mut c_void,
}

/// Function of a ExtendStack functions
pub type ExtendStackFunc = unsafe extern "C" fn(common: *mut c_void) -> SceResult<u32>;

/// The kernel TLS UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct KtlsId(SceUid);

/// The KTLS allocation function.
///
/// # Parameters
///
/// - `size`: The allocation size.
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with this function.
///
/// # Return Value
///
/// Returns unknown value on success, error value otherwise.
pub type KtlsAllocFunc = unsafe extern "C" fn(size: SceSize, common: *mut c_void) -> SceResult<u32>;

#[psp_stub(libname = "ThreadManForUser", flags = 0x4009, use_crate)]
unsafe extern "C" {
    /// Create a thread.
    ///
    /// This function does not directly run a thread, it simply returns a thread UID which can be
    /// used as a handle to start the thread later. See [`sceKernelStartThread`].
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new thread. Only used for debug.
    /// - `entry`: The thread function to run when started.
    /// - `init_priority`: The initial priority of the thread. Less if higher priority.
    /// - `stack_size`: The size of the initial stack for the thread.
    /// - `attr`: The thread attributes.
    /// - `options` **[[In parameter]]**: The options configuring the thread behavior. If [`None`],
    ///   the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the UID of the created thread on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x446D8DE6)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateThread(
        name: *const u8, entry: ThreadEntryFn, init_priority: i32, stack_size: SceSize,
        attr: ThreadAttributes, options: Option<&ThreadOptions>,
    ) -> SceResult<ThreadId>;

    /// Deletes a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9FA03CD3)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelDeleteThread(id: ThreadId) -> SceResult<()>;

    /// Starts the execution of a created thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `arg_len`: The length of the data pointed by `argp`, in bytes.
    /// - `argp` **[[Inout parameter]]**: A pointer to the arguments.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF475845D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelStartThread(
        id: ThreadId, arg_len: SceSize, argp: *const c_void,
    ) -> SceResult<()>;

    /// Exits a thread.
    ///
    /// After exiting the thread, it is marked [`Dormant`](ThreadState::Dormant) and all locked
    /// mutexes are unlocked.
    ///
    /// # Parameters
    ///
    /// - `status`: The exit status.
    ///
    /// # Return Value
    ///
    /// Error value on error, no_returns on success;
    #[nid(0xAA73C935)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelExitThread(status: u32) -> SceResult<()>;

    /// Exits and deletes a thread.
    ///
    /// After exiting the thread, it is marked [`Dormant`](ThreadState::Dormant) and all locked
    /// mutexes are unlocked.
    ///
    /// # Parameters
    ///
    /// - `status`: The exit status.
    ///
    /// # Return Value
    ///
    /// Error value on error, no_returns on success;
    #[nid(0x809CE29B)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelExitDeleteThread(status: u32) -> SceResult<()>;

    /// Forcibly terminates a thread.
    ///
    /// This terminates a thread even when with a wait status, but the thread will be put in a
    /// [`Dormant`](ThreadState::Dormant) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x616403BA)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelTerminateThread(id: ThreadId) -> SceResult<()>;

    /// Forcibly terminates and deletes a thread.
    ///
    /// This terminates a thread even when with a wait status, but the thread will be put in a
    /// [`Dormant`](ThreadState::Dormant) state.
    ///
    /// If the thread is already in a [`Dormant`](ThreadState::Dormant) state, it just deletes the
    /// thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x383F7BCC)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelTerminateDeleteThread(id: ThreadId) -> SceResult<()>;

    /// Suspends the dispatch thread.
    ///
    /// # Return Value
    ///
    /// Returns the current state of the dispatch thread, error value on error.
    #[nid(0x3AD58B8C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSuspendDispatchThread() -> SceResult<ThreadState>;

    /// Resumes the dispatch thread.
    ///
    /// # Parameters
    ///
    /// - `state`: The state of the dispatch thread (from [`sceKernelSuspendDispatchThread`])
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x27E22EC2)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelResumeDispatchThread(state: ThreadState) -> SceResult<()>;

    /// Changes the thread attributes of the calling thread.
    ///
    /// # Parameters
    ///
    /// - `clear_attr`: The attributes to be removed.
    /// - `set_attr`: The attributes to be set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEA748E31)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelChangeCurrentThreadAttr(
        clear_attr: ThreadAttributes, set_attr: ThreadAttributes,
    ) -> SceResult<()>;

    /// Gets the current priority of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the current priority of the calling thread on success, error value otherwise.
    #[nid(0x94AA61EE)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetThreadCurrentPriority() -> SceResult<i32>;

    /// Gets the thread UID of the calling thread.
    ///
    /// # Returns Value
    ///
    /// Returns the thread UID of the calling thread on success, error value otherwise.
    #[nid(0x293B45B8)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetThreadId() -> SceResult<ThreadId>;

    /// Makes the calling thread to enter in a [`Wait`](ThreadState::Wait) state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9ACE131E)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSleepThread() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`](ThreadState::Wait) state but service any
    /// callbacks as necessary.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x82826F70)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSleepThreadCB() -> SceResult<()>;

    /// Wakes a thread that was previously put in the [`Wait`](ThreadState::Wait) state.
    ///
    /// If the specified thread was not put in the [`Wait`](ThreadState::Wait) state by
    /// [`sceKernelSleepThread`] or [`sceKernelSleepThreadCB`], the function will only increment the
    /// wake-up request count.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD59EAD2F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWakeupThread(id: ThreadId) -> SceResult<()>;

    /// Transfers the wake-up request count of the calling thread to another thread.
    ///
    /// The calling thread will have its wake-up request count reset to zero and be set to
    /// [`Wait`](ThreadState::Wait) state.
    ///
    /// # Parameters
    ///
    /// - `donate_id`: The thread UID to receive the wake-up request count donation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.80
    #[nid(0x1AF94D03)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDonateWakeupThread(donate_id: ThreadId) -> SceResult<()>;

    /// Forces a thread to get out of a [`Wait`](ThreadState::Wait) state.
    ///
    /// If the specified thread is in the [`Wait | Suspend`](ThreadState) double state, the
    /// thread will be put in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x2C34E053)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReleaseWaitThread(id: ThreadId) -> SceResult<()>;

    /// Cancels all the wake-up requests from a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID. The [`ThreadId::CALLING`] can be used to specify the calling thread
    ///   UID.
    ///
    /// # Return Value
    ///
    /// Returns the number of wake-up requests cancelled on success, error value otherwise.
    #[nid(0xFCCFAD26)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelWakeupThread(id: ThreadId) -> SceResult<u32>;

    /// Puts a thread in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// If the specified thread is already in a [`Wait`](ThreadState::Wait) state, it will be put in
    /// a [`Wait | Suspend`](ThreadState) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9944F31F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSuspendThread(id: ThreadId) -> SceResult<()>;

    /// Resumes a thread that is in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// If the specified thread is already in a [`Wait | Suspend`](ThreadState) state, it
    /// will be put back in a [`Wait`](ThreadState::Wait) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x75156E8F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelResumeThread(id: ThreadId) -> SceResult<()>;

    /// Put the calling thread in a [`Wait`](ThreadState::Wait) state until a thread if finished or
    /// the timeout is reached.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// Returns the result of the specified thread entry function on success, or error value
    /// otherwise.
    #[nid(0x278C0DF5)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWaitThreadEnd(id: ThreadId, timeout: Option<&mut u32>) -> SceResult<u32>;

    /// Put the calling thread in a [`Wait`](ThreadState::Wait) state until a thread if finished or
    /// the timeout is reached, but service any callbacks as necessary.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// Returns the result of the specified thread entry function on success, or error value
    /// otherwise.
    #[nid(0x840E8133)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWaitThreadEndCB(id: ThreadId, timeout: Option<&mut u32>)
        -> SceResult<u32>;

    /// Delays the calling thread by a specified number of microseconds.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// # Parameters
    ///
    /// - `delay`: The time in microseconds of how much time the thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCEADEB47)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDelayThread(delay: u32) -> SceResult<()>;

    /// Delays the calling thread by a specified number of microseconds, but service any callbacks
    /// as necessary.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Parameters
    ///
    /// - `delay`: The time in microseconds of how much time the thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x68DA9E36)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDelayThreadCB(delay: u32) -> SceResult<()>;

    /// Delays the calling thread by a specified number of [`SystemClock`]s.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// # Parameters
    ///
    /// - `delay` **[[In parameter]]**: A reference to the time in sysclocks of how much time the
    ///   thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBD123D9E)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDelaySysClockThread(delay: &SystemClock) -> SceResult<()>;

    /// Delays the calling thread by a specified number of [`SystemClock`]s, but service any
    /// callbacks as necessary.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Parameters
    ///
    /// - `delay` **[[In parameter]]**: A reference to the time in sysclocks of how much time the
    ///   thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1181E963)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDelaySysClockThreadCB(delay: &SystemClock) -> SceResult<()>;

    /// Rotate thread ready queue at a given priority.
    ///
    /// # Parameters
    ///
    /// - `priority`: The priority of the queue
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x912354A7)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelRotateThreadReadyQueue(priority: u32) -> SceResult<()>;

    /// Gets the exit status of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// Returns the exit status on success, error value otherwise.
    #[nid(0x3B183E26)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetThreadExitStatus(id: ThreadId) -> SceResult<u32>;

    /// Gets the remaining free size of the calling thread stack (?)
    ///
    /// # Return Value
    ///
    /// The remaining free size of the calling thread stack (probably in bytes).
    #[nid(0xD13BDE95)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCheckThreadStack() -> SceSize;

    /// Gets the unused thread stack size of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// Returns the unused thread stack size in bytes of a given thread on success, error value
    /// otherwise.
    #[nid(0x52089CA1)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetThreadStackFreeSize(id: ThreadId) -> SceResult<SceSize>;

    /// Get the status information of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `info`  **[[InOut parameter]]**: A reference to a [`ThreadInfo`] to receive the thread
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x17C1684E)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferThreadStatus(id: ThreadId, info: &mut ThreadInfo) -> SceResult<()>;

    /// Gets the current runtime status of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `run_status` **[[InOut parameter]]**: A reference to [`ThreadRunStatus`] to receive the
    ///   runtime thread information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xFFC36A14)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferThreadRunStatus(
        id: ThreadId, run_status: &mut ThreadRunStatus,
    ) -> SceResult<()>;

    /// Creates a new semaphore
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new semaphore. Only used for debug.
    /// - `attr`: The attribute for the semaphore. It changes the wait queue behavior.
    /// - `init_val`: The initial value of the semaphore.
    /// - `max_val`: The maximum value of the semaphore.
    /// - `options` **[[In parameter]]**: The options configuring the semaphore behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Returns Value
    ///
    /// Returns the semaphore UID on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0xD6DA4BA1)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateSema(
        name: *const u8, attr: SemaphoreAttributes, init_val: i32, max_val: i32,
        options: Option<&SemaphoreOptions>,
    ) -> SceResult<SemaId>;

    /// Deletes a semaphore.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x28B6489C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteSema(id: SemaId) -> SceResult<()>;

    /// Sends a signal to a semaphore.
    ///
    /// If the semaphore has reached the maximum value, and the signal overflows that maximum, an
    /// error is returned.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `signal`: The amount to signal the semaphore (i.e. if `2` then increment the semaphore by
    ///   `2`).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3F53E640)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSignalSema(id: SemaId, signal: i32) -> SceResult<()>;

    /// Locks a semaphore until a target value or a timeout is reached.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `target_value`: The target value to wait.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4E3A1105)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWaitSema(
        id: SemaId, target_value: i32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;


    /// Locks a semaphore until a target value or a timeout is reached, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `target_value`: The target value to wait.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6D212BAC)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWaitSemaCB(
        id: SemaId, target_value: i32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Polls a semaphore if the target value has reached.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `target_value`: The target value to test.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x58B1F937)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelPollSema(id: SemaId, target_value: i32) -> SceResult<()>;

    /// Cancels the wait of a semaphore.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `set_value`: The value to set in the semaphore. If set to `-1`, the initial value of the
    ///   semaphore is used.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified semaphore.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8FFDF9A2)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelSema(
        id: SemaId, set_val: i32, num_wait_threads: &mut u32,
    ) -> SceResult<()>;

    /// Gets the current state of a semaphore.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBC6FEBC5)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferSemaStatus(id: SemaId, info: &mut SemaphoreInfo) -> SceResult<()>;

    /// Creates a event flag.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new event flag. Only used for debug.
    /// - `attr`: The attribute for the event flag.
    /// - `init_bitmask`: The initial bitmask value for the event flag.
    /// - `options` **[[In parameter]]**: The options configuring the event flag behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the event flag UID on success, error value otherwise.
    #[nid(0x55C20A00)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateEventFlag(
        name: *const u8, attr: EventFlagAttributes, init_bitmask: u32,
        options: Option<&EventFlagOptions>,
    ) -> SceResult<EventFlagId>;

    /// Deletes a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEF9E4C70)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteEventFlag(id: EventFlagId) -> SceResult<()>;

    /// Sets a bit pattern on a even flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1FB15A32)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSetEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

    /// Clears a bit pattern of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to clean.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x812346E4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelClearEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

    /// Waits for a bit pattern of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to wait for.
    /// - `wait_kind`: The kind of wait to use.
    /// - `out_bits` **[[Out parameter]]**: A reference to receive the bit pattern that was matched.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x402FCF22)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWaitEventFlag(
        id: EventFlagId, bit_pat: u32, wait_kind: EventFlagWaitKinds, out_bits: &mut u32,
        timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Waits for a bit pattern of a event flag, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to wait for.
    /// - `wait_kind`: The kind of wait to use.
    /// - `out_bits` **[[Out parameter]]**: A reference to receive the bit pattern that was matched.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x328C546A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelWaitEventFlagCB(
        id: EventFlagId, bit_pat: u32, wait_kind: EventFlagWaitKinds, out_bits: &mut u32,
        timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Polls for a bit pattern of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to poll for.
    /// - `wait_kind`: The kind of wait to use.
    /// - `out_bits` **[[Out parameter]]**: A reference to receive the bit pattern that was matched.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x30FD48F0)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelPollEventFlag(
        id: EventFlagId, bit_pat: u32, wait_kind: EventFlagWaitKinds, out_bits: &mut u32,
    ) -> SceResult<()>;

    /// Cancels the wait of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `set_pat`: The bit patter to set in the event flag.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified semaphore.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCD203292)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelEventFlag(id: EventFlagId, set_pat: u32, num_wait_threads: &mut u32);

    /// Gets the current state of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA66B0120)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferEventFlagStatus(
        id: EventFlagId, info: &mut EventFlagInfo,
    ) -> SceResult<()>;

    /// Creates a new mutex.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new mutex. Only used for debug.
    /// - `attr`: The attribute for the mutex.
    /// - `init_count`: The initial count value for the mutex.
    /// - `options` **[[In parameter]]**: The options configuring the mutex behavior. If [`None`],
    ///   the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the mutex UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xB7D098C6)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateMutex(
        name: *const u8, attr: MutexAttributes, init_count: i32, options: Option<&MutexOptions>,
    ) -> SceResult<MutexId>;

    /// Deletes a mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xF8170FBE)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteMutex(id: MutexId) -> SceResult<()>;

    /// Locks a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xB011B11F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelLockMutex(
        id: MutexId, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Locks a mutex a number of times, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x5BF4DD27)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelLockMutexCB(
        id: MutexId, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to lock a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x0DDCD2C9)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelTryLockMutex(id: MutexId, lock_count: u32) -> SceResult<()>;

    /// Unlocks a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `unlock_count`: The number of times to unlock a mutex after resource acquisition. It must
    ///   be `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x6B30100F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelUnlockMutex(id: MutexId, unlock_count: u32) -> SceResult<()>;

    /// Cancels the wait state of threads waiting on a mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `new_lock_count`: The new lock count to set on the mutex.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified mutex.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x87D9223C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelMutex(id: MutexId, new_lock_count: u32, numWaitThreads: &mut u32);

    /// Gets the current state of a mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`MutexInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xA9C2CB9A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferMutexStatus(id: MutexId, info: &mut MutexInfo) -> SceResult<()>;

    /// Creates a new lightweight mutex.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[Out parameter]]**: A reference to a lightweight mutex work area to be
    ///   initialized/populated.
    /// - `name` **[[In parameter]]**: The name assigned to the new lightweight mutex. Only used for
    ///   debug.
    /// - `attr`: The attribute for the lightweight mutex.
    /// - `init_count`: The initial count value for the lightweight mutex.
    /// - `options` **[[In parameter]]**: The options configuring the lightweight mutex behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[eabi(i5)]
    #[nid(0x19CFF145)]
    pub unsafe fn sceKernelCreateLwMutex(
        work_area: *mut LwMutexWorkArea, name: *const u8, attr: MutexAttributes, init_count: i32,
        options: Option<&LwMutexOptions>,
    ) -> SceResult<()>;

    /// Deletes a lightweight mutex.
    ///
    ///  # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x60107536)]
    pub safe fn sceKernelDeleteLwMutex(work_area: &mut LwMutexWorkArea) -> SceResult<()>;

    /// Locks a lightweight mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x7CFF8CF3)]
    pub safe fn _sceKernelLockLwMutex(
        work_area: &mut LwMutexWorkArea, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Locks a lightweight mutex a number of times, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x31327F19)]
    pub safe fn _sceKernelLockLwMutexCB(
        work_area: &mut LwMutexWorkArea, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to lock a lightweight mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x71040D5C)]
    pub safe fn _sceKernelTryLockLwMutex(
        work_area: &mut LwMutexWorkArea, lock_count: u32,
    ) -> SceResult<()>;

    /// Unlocks a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `work_area` **[[InOut parameter]]**: A reference to a lightweight mutex work area.
    /// - `unlock_count`: The number of times to unlock a mutex after resource acquisition. It must
    ///   be `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0xBEED3A47)]
    pub safe fn _sceKernelUnlockLwMutex(
        work_area: &mut LwMutexWorkArea, unlock_count: u32,
    ) -> SceResult<()>;

    /// Gets the current state of a lightweight mutex by its UID.
    ///
    /// # Parameters
    ///
    /// - `id`: The lightweight mutex UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`LwMutexInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x4C145944)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferLwMutexStatusByID(
        id: LwMutexId, info: &mut LwMutexInfo,
    ) -> SceResult<()>;

    /// Creates a new message box.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new message box. Only used for
    ///   debug.
    /// - `attr`: The message box attributes.
    /// - `options` **[[In parameter]]**: The options configuring the message box behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the UID of the created message box on success, error value otherwise.
    #[nid(0x8125221D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateMbx(
        name: *const u8, attr: MsgBoxAttributes, options: Option<&mut MsgBoxOptions>,
    ) -> SceResult<MsgBoxId>;

    /// Deletes a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x86255ADA)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelDeleteMbx(id: MsgBoxId) -> SceResult<()>;

    /// Sends a message to a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[In parameter]]**: A pointer to a message to be forwarded to the receiver.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE9B3061E)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSendMbx(id: MsgBoxId, msg: *mut MsgPacket) -> SceResult<()>;

    /// Waits to receive a message to a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[Out parameter]]**: A pointer to a pointer to receive the message.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x18260574)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelReceiveMbx(
        id: MsgBoxId, msg: *mut *mut MsgPacket, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Waits to receive a message to a message box, but service any callbacks as necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[Out parameter]]**: A pointer to a pointer to receive the message.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF3986382)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelReceiveMbxCB(
        id: MsgBoxId, msg: *mut *mut MsgPacket, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Polls if the message has arrived in a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[Out parameter]]**: A pointer to a pointer to receive the message.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0D81716A)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelPollMbx(id: MsgBoxId, msg: *mut *mut MsgPacket) -> SceResult<()>;

    /// Cancels the wait of a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified semaphore.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x87D4DD36)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelReceiveMbx(
        id: MsgBoxId, num_wait_threads: &mut u32,
    ) -> SceResult<()>;

    /// Gets the current state of a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA8E8C846)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferMbxStatus(id: MsgBoxId, info: &mut MsgBoxInfo) -> SceResult<()>;

    /// Creates a new message pipe.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new message pipe. Only used for
    ///   debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the message pipe.
    /// - `buf_size`: The size of the message pipe buffer. Zero is allowed.
    /// - `options` **[[In parameter]]**: The options configuring the message pipe behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the message pipe UID on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x7C0DC2A0)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateMsgPipe(
        name: *const u8, partition_id: MemoryPartitionId, attr: MsgPipeAttributes,
        buf_size: SceSize, options: Option<&SemaphoreOptions>,
    ) -> SceResult<MsgPipeId>;

    /// Deletes a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF0B7DA1C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteMsgPipe(id: MsgPipeId) -> SceResult<()>;

    /// Sends a message to a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[In parameter]]**: A pointer to data to send as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x876DBFAD)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSendMsgPipe(
        id: MsgPipeId, msg_buf: *const c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Sends a message to a message pipe, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[In parameter]]**: A pointer to data to send as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x7C41F2C2)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSendMsgPipeCB(
        id: MsgPipeId, msg_buf: *const c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;


    /// Tries to send a message to a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[In parameter]]**: A pointer to data to send as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x884C9F90)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelTrySendMsgPipe(
        id: MsgPipeId, msg_buf: *const c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32,
    ) -> SceResult<()>;

    /// Waits to receive a message from a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[Out parameter]]**: A pointer to data buffer to receive as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x74829B76)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelReceiveMsgPipe(
        id: MsgPipeId, msg_buf: *mut c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Waits to receive a message from a message pipe, but service any callbacks as necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[Out parameter]]**: A pointer to data buffer to receive as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xFBFA697D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelReceiveMsgPipeCB(
        id: MsgPipeId, msg_buf: *mut c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to receive a message from a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[Out parameter]]**: A pointer to data buffer to receive as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0xDF52098F)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelTryReceiveMsgPipe(
        id: MsgPipeId, msg_buf: *mut c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32,
    ) -> SceResult<()>;

    /// Cancels the wait of message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `num_send_wait_threads` **[[Out parameter]]**: A reference to receive the number of sender
    ///   threads that were waiting on the specified message pipe.
    /// - `num_recv_wait_threads` **[[Out parameter]]**: A reference to receive the number of
    ///   receiver threads that were waiting on the specified message pipe.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x349B864D)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelMsgPipe(
        id: MsgPipeId, num_send_wait_threads: &mut u32, num_recv_wait_threads: &mut u32,
    ) -> SceResult<()>;

    /// Gets the current state of a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`MsgPipeInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x33BE4024)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferMsgPipeStatus(id: MsgPipeId, info: &mut MsgPipeInfo)
        -> SceResult<()>;

    /// Creates a new variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new VPL. Only used for debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the variable-sized memory pool.
    /// - `size`: The size of the memory pool, in bytes.
    /// - `options` **[[In parameter]]**: The options configuring the variable-sized memory pool
    ///   behavior. If [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the variable-sized memory pool UID on success, error value otherwise.
    #[nid(0x56C039B5)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateVpl(
        name: *const u8, partition_id: MemoryPartitionId, attr: VplAttributes, size: SceSize,
        options: Option<&VplOptions>,
    ) -> SceResult<VplId>;

    /// Deletes a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x89B3D48C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteVpl(id: VplId) -> SceResult<()>;

    /// Allocates a memory block from a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `size`: The size to allocate.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBED27435)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelAllocateVpl(
        id: VplId, size: SceSize, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Allocates a memory block from a variable-sized memory pool, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `size`: The size to allocate.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEC0A693F)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelAllocateVplCB(
        id: VplId, size: SceSize, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to allocates a memory block from a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `size`: The size to allocate.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xAF36D708)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelTryAllocateVpl(
        id: VplId, size: SceSize, mem_block: *mut *mut c_void,
    ) -> SceResult<()>;

    /// Deallocates a memory block from a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `mem_block` **[[In parameter]]**: A pointer of the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelFreeVpl(id: VplId, mem_block: *mut c_void) -> SceResult<()>;

    /// Cancels the wait of a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified ID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1D371B8A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelVpl(id: VplId, num_wait_threads: &mut u32) -> SceResult<()>;

    /// Gets the current state of a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x39810265)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferVplStatus(id: VplId, info: &mut VplInfo) -> SceResult<()>;

    /// Creates a new fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new FPL. Only used for debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the fixed-sized memory pool.
    /// - `block_size`: The size of a memory block to use, in bytes.
    /// - `num_blocks`: The number of blocks to allocate.
    /// - `options` **[[In parameter]]**: The options configuring the fixed-sized memory pool
    ///   behavior. If [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the fixed-sized memory pool UID on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xC07BB470)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateFpl(
        name: *const u8, partition_id: MemoryPartitionId, attr: FplAttributes, block_size: SceSize,
        num_blocks: SceSize, options: Option<&FplOptions>,
    ) -> SceResult<FplId>;

    /// Deletes a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xED1410E0)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteFpl(id: FplId) -> SceResult<()>;

    /// Allocates a memory block from a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD979E9BF)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelAllocateFpl(
        id: FplId, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Allocates a memory block from a fixed-sized memory pool, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE7282CB6)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelAllocateFplCB(
        id: FplId, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to allocates a memory block from a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x623AE665)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelTryAllocateFpl(id: FplId, mem_block: *mut *mut c_void) -> SceResult<()>;

    /// Deallocates a memory block from a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[In parameter]]**: A pointer of the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF6414A71)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelFreeFpl(id: FplId, mem_block: *mut c_void) -> SceResult<()>;

    /// Cancels the wait of a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified ID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA8AA591F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelFpl(id: FplId, num_wait_threads: &mut u32) -> SceResult<()>;

    /// Gets the current state of a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD8199E4C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferFplStatus(id: FplId, info: &mut FplInfo) -> SceResult<()>;

    /// Creates a new user TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new TLP pool. Only used for debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the TLP memory pool.
    /// - `block_size`: The size of a memory block to use, in bytes.
    /// - `num_blocks`: The number of blocks to allocate.
    /// - `options` **[[In parameter]]**: The options configuring the TLP memory pool behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the user TLS memory pool UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[eabi(i6)]
    #[nid(0x8DAFF657)]
    pub unsafe fn sceKernelCreateTlspl(
        name: *const u8, partition_id: MemoryPartitionId, attr: TlsPoolAttributes,
        block_size: SceSize, num_blocks: SceSize, options: Option<&TlsPoolOptions>,
    ) -> SceResult<TlsPoolId>;

    /// Deletes a user TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The user TLS memory pool UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x32BF938E)]
    pub safe fn sceKernelDeleteTlspl(id: TlsPoolId) -> SceResult<()>;

    /// Allocates the TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The user TLS memory pool UID.
    /// - `tls_addr` **[[Out parameter]]**: A pointer to the pointer to receive the TLS head
    ///   address.
    /// - `unk`: Unknown. Pass 0.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x65F54FFB)]
    pub unsafe fn _sceKernelAllocateTlspl(
        id: TlsPoolId, tls_addr: *mut *mut c_void, unk: u32,
    ) -> SceResult<()>;

    /// Gets the current state of a user TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The user TLS memory pool UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x721067F3)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferTlsplStatus(id: TlsPoolId, info: &mut TlsPoolInfo) -> SceResult<()>;

    /// Gets the system time.
    ///
    /// # Parameters
    ///
    /// - `clock`  **[[Out parameter]]**: A reference to [`SystemClock`] to receive the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDB738F35)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetSystemTime(clock: &mut SystemClock) -> SceResult<()>;

    /// Gets the system time as raw wide integer.
    ///
    /// # Return Value
    ///
    /// Returns the system time.
    #[nid(0x82BC5777)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetSystemTimeWide() -> u64;

    /// Gets the low part of the system time.
    ///
    /// # Return Value
    ///
    /// Returns the system time low bits.
    #[nid(0x369ED59D)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetSystemTimeLow() -> u32;

    /// Creates an alarm.
    ///
    /// # Parameters
    ///
    /// - `microsec`: The microseconds until the `handler` is called.
    /// - `handler` **[[In parameter]]**: The function pointer set as entry point.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the alarm handler.
    ///
    /// # Return Value
    ///
    /// Returns the alarm UID on success, error value otherwise.
    #[nid(0x6652B8CA)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSetAlarm(
        microsec: u32, handler: AlarmHandler, common: *mut c_void,
    ) -> SceResult<AlarmId>;

    /// Creates an alarm using [`SystemClock`].
    ///
    /// # Parameters
    ///
    /// - `clock` **[[In parameter]]**: A reference to a [`SystemClock`] as the time until the
    ///   `handler` is called.
    /// - `handler` **[[In parameter]]**: The function pointer set as entry point.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the alarm handler.
    ///
    /// # Return Value
    ///
    /// Returns the alarm UID on success, error value otherwise.
    #[nid(0xB2C25152)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSetSysClockAlarm(
        clock: &SystemClock, handler: AlarmHandler, common: *mut c_void,
    ) -> SceResult<AlarmId>;

    /// Cancels an alarm.
    ///
    /// # Parameters
    ///
    /// - `id`: The alarm UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7E65B999)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelAlarm(id: AlarmId) -> SceResult<()>;

    /// Gets the current state of a alarm.
    ///
    /// # Parameters
    ///
    /// - `id`: The alarm UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDAA3F564)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferAlarmStatus(id: AlarmId, info: &mut AlarmInfo) -> SceResult<()>;

    /// Converts time in microseconds to [`SystemClock`].
    ///
    /// # Parameters
    ///
    /// - `microsec`: The time in microseconds.
    /// - `clock` **[[Out parameter]]**: A reference to [`SystemClock`] to receive the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x110DEC9A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelUSec2SysClock(microsec: u32, clock: &mut SystemClock) -> SceResult<()>;

    /// Converts time in [`SystemClock`] to seconds and microseconds.
    ///
    /// # Parameters
    ///
    /// - `clock` **[[In parameter]]**: A reference to [`SystemClock`] to give the time information.
    /// - `sec` **[[Out parameter]]**: A reference to receive the seconds information.
    /// - `microsec` **[[Out parameter]]**: A reference to receive the microseconds information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBA6B92E2)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSysClock2USec(
        clock: &SystemClock, sec: &mut u32, microsec: &mut u32,
    ) -> SceResult<()>;

    /// Converts time in microseconds to raw system clock.
    ///
    /// # Parameters
    ///
    /// - `microsec`: The time in microseconds.
    ///
    /// # Return Value
    ///
    /// Returns the clock in raw format.
    #[nid(0xC8CD158C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelUSec2SysClockWide(microsec: u32) -> u64;

    /// Converts raw system time to seconds and microseconds.
    ///
    /// # Parameters
    ///
    /// - `raw_clock`: The raw system clock.
    /// - `sec` **[[Out parameter]]**: A reference to receive the seconds information.
    /// - `microsec` **[[Out parameter]]**: A reference to receive the microseconds information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE1619D7C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSysClock2USecWide(
        raw_clock: u64, sec: &mut u32, microsec: &mut u32,
    ) -> SceResult<()>;

    /// Creates a new virtual timer.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new virtual timer. Only used for
    ///   debug.
    /// - `options` **[[In parameter]]**: The options configuring the virtual timer behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Returns Value
    ///
    /// Returns the virtual timer UID on success, error value otherwise.
    #[nid(0x20FFF560)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateVTimer(
        name: *const u8, options: Option<&VirtualTimerOptions>,
    ) -> SceResult<VirtualTimerId>;

    /// Deletes a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x328F9E52)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteVTimer(id: VirtualTimerId) -> SceResult<()>;

    /// Gets the base time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `base` **[[Out parameter]]**: A reference to receive the base time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB3A59970)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetVTimerBase(id: VirtualTimerId, base: &mut SystemClock)
        -> SceResult<()>;

    /// Gets the base time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns base time in raw format on success, `0xFFFFFFFFFFFFFFFF` (`-1 as u64`) on error.
    #[nid(0xB7C18B77)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetVTimerBaseWide(id: VirtualTimerId) -> u64;

    /// Gets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `time` **[[Out parameter]]**: A reference to receive the current time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x034A921F)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetVTimerTime(id: VirtualTimerId, time: &mut SystemClock)
        -> SceResult<()>;

    /// Gets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns current time in raw format on success, `0xFFFFFFFFFFFFFFFF` (`-1 as u64`) on error.
    #[nid(0xC0B3FFD2)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetVTimerTimeWide(id: VirtualTimerId) -> u64;

    /// Sets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `time` **[[InOut parameter]]**: A reference to the time to set and also to receive the
    ///   previous set value of the current time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x542AD630)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSetVTimerTime(id: VirtualTimerId, time: &mut SystemClock)
        -> SceResult<()>;

    /// Sets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `time`: The raw time to set.
    ///
    /// # Return Value
    ///
    /// Returns the previous current time in raw format on success, `0xFFFFFFFFFFFFFFFF` (`-1 as
    /// u64`) on error.
    #[nid(0xFB6425C3)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelSetVTimerTimeWide(id: VirtualTimerId, time: u64) -> u64;

    /// Starts a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns the previous timer state on success, error value otherwise.
    #[nid(0xC68D9437)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelStartVTimer(id: VirtualTimerId) -> SceResult<VirtualTimerState>;

    /// Stops a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns the previous timer state on success, error value otherwise.
    #[nid(0xD0AEEE87)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelStopVTimer(id: VirtualTimerId) -> SceResult<VirtualTimerState>;

    /// Sets the virtual time handler and the schedule to execute it.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `schedule` **[[In parameter]]**: A reference to a time as the schedule time to execute the
    ///   `handler`.
    /// - `handler` **[[In parameter]]**: The function pointer of the handler to set.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD8B299AE)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSetVTimerHandler(
        id: VirtualTimerId, schedule: &mut SystemClock, handler: VirtualTimerHandler,
        common: *mut c_void,
    ) -> SceResult<()>;

    /// Sets the virtual time handler and the schedule to execute it.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `schedule`: The time as the schedule time to execute the `handler` in raw format.
    /// - `handler` **[[In parameter]]**: The function pointer of the handler to set.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x53B00E9A)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSetVTimerHandlerWide(
        id: VirtualTimerId, schedule: u64, handler: VirtualTimerHandlerWide, common: *mut c_void,
    ) -> SceResult<()>;

    /// Cancels the handler set for a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD2D615EF)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelVTimerHandler(id: VirtualTimerId) -> SceResult<()>;

    /// Gets the current state of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`VirtualTimerInfo`] to receive the virtual
    ///   timer information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5F32BEAA)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferVTimerStatus(
        id: VirtualTimerId, info: &mut VirtualTimerInfo,
    ) -> SceResult<()>;

    /// Creates a new callback.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new callback. Only used for debug.
    /// - `entry` **[[In parameter]]**: The function pointer of the entry function to set.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
    ///
    /// # Return Value
    ///
    /// Returns the callback UID on success, error value otherwise.
    #[nid(0xE81CAF8F)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelCreateCallback(
        name: *const u8, entry: CallbackFunction, common: *mut c_void,
    ) -> SceResult<CallbackId>;

    /// Deletes a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEDBA5844)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelDeleteCallback(id: CallbackId) -> SceResult<()>;

    /// Notifies a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    /// - `arg`: The argument to be passed to the callback function.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xC11BA8C4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelNotifyCallback(id: CallbackId, arg: i32) -> SceResult<()>;

    /// Cancels all notifications that were reported to a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBA4051D6)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCancelCallback(id: CallbackId) -> SceResult<()>;

    /// Gets the notification count of a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// Returns the number of notification send to the callback on success, error value otherwise.
    #[nid(0x2A3D44FF)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetCallbackCount(id: CallbackId) -> SceResult<u32>;

    /// Checks if a callback by the calling thread has been notified.
    ///
    /// If it was notified, the callback is called.
    ///
    /// # Return Value
    ///
    /// Returns the check status on success, error value otherwise.
    #[nid(0x349D6D6C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelCheckCallback() -> SceResult<CallbackCheckStatus>;

    /// Gets the current state of a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x730ED8BC)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferCallbackStatus(
        id: CallbackId, info: &mut CallbackInfo,
    ) -> SceResult<()>;

    /// Gets the current state of the system.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A reference to [`SystemInfo`] to receive the system
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x627E6F3A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferSystemStatus(info: &mut SystemInfo) -> SceResult<()>;

    /// Gets a list of UIDs from threadman module.
    ///
    /// # Parameters
    ///
    /// - `kind`: The kind of resource to get in the list. Use [`ThreadIdKind::Any`] to get all of
    ///   them.
    /// - `buf` **[[Out parameter]]**: A pointer to a buffer list to receive the UIDs.
    /// - `buf_size`: The size of the `buf`.
    /// - `id_count` **[[Out parameter]]**: A optional reference to receive the number of UIDs of
    ///   the specified `kind` in the system.
    ///
    /// # Return Value
    ///
    /// Returns the number of UIDs added to the buffer on success, error value otherwise.
    #[nid(0x94416130)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelGetThreadmanIdList(
        kind: ThreadIdKind, buf: *mut SceUid, buf_size: SceSize, id_cound: Option<&mut u32>,
    ) -> SceResult<u32>;

    /// Gets the threadman UID kind of an ID.
    ///
    /// # Parameters
    ///
    /// - `id`: The UID to get the kind.
    ///
    /// # Return Value
    ///
    /// Returns the UID kind on success, error value otherwise.
    #[nid(0x57CF62DD)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetThreadmanIdType(id: SceUid) -> SceResult<ThreadIdKind>;

    /// Registers a thread event handler.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new thread event. Only used for
    ///   debug.
    /// - `thread_id`: The thread UID of the thread which the handler should be called. The
    ///   [`ThreadId::CALLING`] can be used to specify the calling thread UID and the
    ///   [`ThreadId::ALL_USER`] can be used to set to be called for all user threads.
    /// - `event_mask`: The thread events which the handler should be called.
    /// - `handler` **[[In parameter]]**: A function pointer to the entry function for the handler.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the thread event
    ///   handler.
    ///
    /// # Return Value
    ///
    /// Returns the thread event UID on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x0C106E53)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelRegisterThreadEventHandler(
        name: *const u8, thread_id: ThreadId, event_mask: ThreadEvents,
        handler: ThreadEventHandler, common: *mut c_void,
    ) -> SceResult<ThreadEventId>;

    /// Releases a thread event handler.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread event UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x72F3C145)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReleaseThreadEventHandler(id: ThreadEventId) -> SceResult<()>;

    /// Gets the current state of a thread event handler.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread event UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`ThreadEventInfo`] to receive the thread
    ///   event handler information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x369EEB6B)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelReferThreadEventHandlerStatus(
        id: ThreadEventId, info: &mut ThreadEventInfo,
    ) -> SceResult<()>;

    /// Temporarily extends the thread stack an executes a function with extended stack size.
    ///
    /// # Parameters
    ///
    /// - `stack_size`: The stack size (in bytes) to temporarily extend.
    /// - `func` **[[In parameter]]**: A pointer to the function to call with extended stack.
    /// - `common` **[[InOut parameter]]** A pointer to memory shared with the `func`.
    ///
    /// # Return Value
    ///
    /// Returns the result of the `func` on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0xBC80EC7C)]
    pub safe fn sceKernelExtendThreadStack(
        stack_size: SceSize, func: ExtendStackFunc, common: *mut c_void,
    ) -> SceResult<u32>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "ThreadManForKernel", flags = 0x0009, use_crate)]
unsafe extern "C" {
    /// Create a thread.
    ///
    /// This function does not directly run a thread, it simply returns a thread UID which can be
    /// used as a handle to start the thread later. See [`sceKernelStartThread`].
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new thread. Only used for debug.
    /// - `entry`: The thread function to run when started.
    /// - `init_priority`: The initial priority of the thread. Less if higher priority.
    /// - `stack_size`: The size of the initial stack for the thread.
    /// - `attr`: The thread attributes.
    /// - `options` **[[In parameter]]**: The options configuring the thread behavior. If [`None`],
    ///   the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the UID of the created thread on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x446D8DE6)]
    pub unsafe fn sceKernelCreateThread(
        name: *const u8, entry: ThreadEntryFn, init_priority: i32, stack_size: SceSize,
        attr: ThreadAttributes, options: Option<&ThreadOptions>,
    ) -> SceResult<ThreadId>;

    /// Deletes a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9FA03CD3)]
    pub unsafe fn sceKernelDeleteThread(id: ThreadId) -> SceResult<()>;

    /// Starts the execution of a created thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `arg_len`: The length of the data pointed by `argp`, in bytes.
    /// - `argp` **[[Inout parameter]]**: A pointer to the arguments.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF475845D)]
    pub unsafe fn sceKernelStartThread(
        id: ThreadId, arg_len: SceSize, argp: *const c_void,
    ) -> SceResult<()>;

    /// Exits a thread.
    ///
    /// After exiting the thread, it is marked [`Dormant`](ThreadState::Dormant) and all locked
    /// mutexes are unlocked.
    ///
    /// # Parameters
    ///
    /// - `status`: The exit status.
    ///
    /// # Return Value
    ///
    /// Error value on error, no_returns on success;
    #[nid(0xAA73C935)]
    pub safe fn sceKernelExitThread(status: u32) -> SceResult<!>;

    /// Exits and deletes a thread.
    ///
    /// After exiting the thread, it is marked [`Dormant`](ThreadState::Dormant) and all locked
    /// mutexes are unlocked.
    ///
    /// # Parameters
    ///
    /// - `status`: The exit status.
    ///
    /// # Return Value
    ///
    /// Error value on error, no_returns on success;
    #[nid(0x809CE29B)]
    pub safe fn sceKernelExitDeleteThread(status: u32) -> SceResult<!>;

    /// Forcibly terminates a thread.
    ///
    /// This terminates a thread even when with a wait status, but the thread will be put in a
    /// [`Dormant`](ThreadState::Dormant) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x616403BA)]
    pub unsafe fn sceKernelTerminateThread(id: ThreadId) -> SceResult<()>;

    /// Forcibly terminates and deletes a thread.
    ///
    /// This terminates a thread even when with a wait status, but the thread will be put in a
    /// [`Dormant`](ThreadState::Dormant) state.
    ///
    /// If the thread is already in a [`Dormant`](ThreadState::Dormant) state, it just deletes the
    /// thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x383F7BCC)]
    pub unsafe fn sceKernelTerminateDeleteThread(id: ThreadId) -> SceResult<()>;

    /// Suspends the dispatch thread.
    ///
    /// # Return Value
    ///
    /// Returns the current state of the dispatch thread, error value on error.
    #[nid(0x3AD58B8C)]
    pub safe fn sceKernelSuspendDispatchThread() -> SceResult<ThreadState>;

    /// Resumes the dispatch thread.
    ///
    /// # Parameters
    ///
    /// - `state`: The state of the dispatch thread (from [`sceKernelSuspendDispatchThread`])
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x27E22EC2)]
    pub safe fn sceKernelResumeDispatchThread(state: ThreadState) -> SceResult<()>;

    /// Changes the thread attributes of the calling thread.
    ///
    /// # Parameters
    ///
    /// - `clear_attr`: The attributes to be removed.
    /// - `set_attr`: The attributes to be set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEA748E31)]
    pub safe fn sceKernelChangeCurrentThreadAttr(
        clear_attr: ThreadAttributes, set_attr: ThreadAttributes,
    ) -> SceResult<()>;

    /// Changes the thread current priority of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `priority`: The priority to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x71BC9871)]
    pub safe fn sceKernelChangeThreadPriority(id: ThreadId, priority: i32) -> SceResult<()>;

    /// Gets the current priority of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the current priority of the calling thread on success, error value otherwise.
    #[nid(0x94AA61EE)]
    pub safe fn sceKernelGetThreadCurrentPriority() -> SceResult<i32>;

    /// Gets the thread UID of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the thread UID of the calling thread on success, error value otherwise.
    #[nid(0x293B45B8)]
    pub safe fn sceKernelGetThreadId() -> SceResult<ThreadId>;

    /// Makes the calling thread to enter in a [`Wait`](ThreadState::Wait) state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9ACE131E)]
    pub safe fn sceKernelSleepThread() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`](ThreadState::Wait) state but service any
    /// callbacks as necessary.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x82826F70)]
    pub safe fn sceKernelSleepThreadCB() -> SceResult<()>;

    /// Wakes a thread that was previously put in the [`Wait`](ThreadState::Wait) state.
    ///
    /// If the specified thread was not put in the [`Wait`](ThreadState::Wait) state by
    /// [`sceKernelSleepThread`] or [`sceKernelSleepThreadCB`], the function will only increment the
    /// wake-up request count.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD59EAD2F)]
    pub safe fn sceKernelWakeupThread(id: ThreadId) -> SceResult<()>;

    /// Transfers the wake-up request count of the calling thread to another thread.
    ///
    /// The calling thread will have its wake-up request count reset to zero and be set to
    /// [`Wait`](ThreadState::Wait) state.
    ///
    /// # Parameters
    ///
    /// - `donate_id`: The thread UID to receive the wake-up request count donation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.80
    #[nid(0x1AF94D03)]
    pub safe fn sceKernelDonateWakeupThread(donate_id: ThreadId) -> SceResult<()>;

    /// Forces a thread to get out of a [`Wait`](ThreadState::Wait) state.
    ///
    /// If the specified thread is in the [`Wait | Suspend`](ThreadState) state, the
    /// thread will be put in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x2C34E053)]
    pub safe fn sceKernelReleaseWaitThread(id: ThreadId) -> SceResult<()>;

    /// Cancels all the wake-up requests from a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID. The [`ThreadId::CALLING`] can be used to specify the calling thread
    ///   UID.
    ///
    /// # Return Value
    ///
    /// Returns the number of wake-up requests cancelled on success, error value otherwise.
    #[nid(0xFCCFAD26)]
    pub safe fn sceKernelCancelWakeupThread(id: ThreadId) -> SceResult<u32>;

    /// Puts a thread in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// If the specified thread is already in a [`Wait`](ThreadState::Wait) state, it will be put in
    /// a [`Wait | Suspend`](ThreadState) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9944F31F)]
    pub safe fn sceKernelSuspendThread(id: ThreadId) -> SceResult<()>;

    /// Resumes a thread that is in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// If the specified thread is already in a [`Wait | Suspend`](ThreadState) state, it
    /// will be put back in a [`Wait`](ThreadState::Wait) state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x75156E8F)]
    pub safe fn sceKernelResumeThread(id: ThreadId) -> SceResult<()>;

    /// Put the calling thread in a [`Wait`](ThreadState::Wait) state until a thread if finished or
    /// the timeout is reached.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// Returns the result of the specified thread entry function on success, or error value
    /// otherwise.
    #[nid(0x278C0DF5)]
    pub safe fn sceKernelWaitThreadEnd(id: ThreadId, timeout: Option<&mut u32>) -> SceResult<u32>;

    /// Put the calling thread in a [`Wait`](ThreadState::Wait) state until a thread if finished or
    /// the timeout is reached, but service any callbacks as necessary.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// Returns the result of the specified thread entry function on success, or error value
    /// otherwise.
    #[nid(0x840E8133)]
    pub safe fn sceKernelWaitThreadEndCB(id: ThreadId, timeout: Option<&mut u32>)
        -> SceResult<u32>;

    /// Delays the calling thread by a specified number of microseconds.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// # Parameters
    ///
    /// - `delay`: The time in microseconds of how much time the thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCEADEB47)]
    pub safe fn sceKernelDelayThread(delay: u32) -> SceResult<()>;

    /// Delays the calling thread by a specified number of microseconds, but service any callbacks
    /// as necessary.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Parameters
    ///
    /// - `delay`: The time in microseconds of how much time the thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x68DA9E36)]
    pub safe fn sceKernelDelayThreadCB(delay: u32) -> SceResult<()>;

    /// Delays the calling thread by a specified number of [`SystemClock`]s.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// # Parameters
    ///
    /// - `delay` **[[In parameter]]**: A reference to the time in sysclocks of how much time the
    ///   thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBD123D9E)]
    pub safe fn sceKernelDelaySysClockThread(delay: &SystemClock) -> SceResult<()>;

    /// Delays the calling thread by a specified number of [`SystemClock`]s, but service any
    /// callbacks as necessary.
    ///
    /// The calling thread is put in a [`Wait`](ThreadState::Wait) state for the duration of the
    /// delay.
    ///
    /// If a callback notification is received while the calling thread is in a
    /// [`Wait`](ThreadState::Wait) state, the thread will get out of the `Wait` state, execute the
    /// necessary callbacks and return to a `Wait` state.
    ///
    /// # Parameters
    ///
    /// - `delay` **[[In parameter]]**: A reference to the time in sysclocks of how much time the
    ///   thread should stop execution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1181E963)]
    pub safe fn sceKernelDelaySysClockThreadCB(delay: &SystemClock) -> SceResult<()>;

    /// Rotate thread ready queue at a given priority.
    ///
    /// # Parameters
    ///
    /// - `priority`: The priority of the queue
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x912354A7)]
    pub safe fn sceKernelRotateThreadReadyQueue(priority: u32) -> SceResult<()>;

    /// Gets the exit status of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// Returns the exit status on success, error value otherwise.
    #[nid(0x3B183E26)]
    pub safe fn sceKernelGetThreadExitStatus(id: ThreadId) -> SceResult<u32>;

    /// Gets the remaining free size of the calling thread stack (?)
    ///
    /// # Return Value
    ///
    /// The remaining free size of the calling thread stack (probably in bytes).
    #[nid(0xD13BDE95)]
    pub safe fn sceKernelCheckThreadStack() -> SceSize;

    /// Gets the unused thread stack size of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    ///
    /// # Return Value
    ///
    /// Returns the unused thread stack size in bytes of a given thread on success, error value
    /// otherwise.
    #[nid(0x52089CA1)]
    pub safe fn sceKernelGetThreadStackFreeSize(id: ThreadId) -> SceResult<SceSize>;

    /// Gets the status information of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `info` **[[InOut parameter]]**: A reference to a [`ThreadInfo`] to receive the thread
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x17C1684E)]
    pub safe fn sceKernelReferThreadStatus(id: ThreadId, info: &mut ThreadInfo) -> SceResult<()>;

    /// Gets the current runtime status of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread UID.
    /// - `run_status` **[[InOut parameter]]**: A reference to [`ThreadRunStatus`] to receive the
    ///   runtime thread information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xFFC36A14)]
    pub safe fn sceKernelReferThreadRunStatus(
        id: ThreadId, run_status: &mut ThreadRunStatus,
    ) -> SceResult<()>;

    /// Gets if the calling thread is a user mode thread.
    ///
    /// # Return Value
    ///
    /// Returns if the thread is user mode on success, error value otherwise.
    #[nid(0x85A2A5BF)]
    pub safe fn sceKernelIsUserModeThread() -> SceResult<bool>;

    /// Puts all user mode threads in the system in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8FD9F70C)]
    pub safe fn sceKernelSuspendAllUserThreads() -> SceResult<()>;

    /// Gets the user level of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the user level of the calling thread on success, error value otherwise.
    #[nid(0xF6427665)]
    pub safe fn sceKernelGetUserLevel() -> SceResult<u32>;

    /// Creates a new semaphore
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new semaphore. Only used for debug.
    /// - `attr`: The attribute for the semaphore. It changes the wait queue behavior.
    /// - `init_val`: The initial value of the semaphore.
    /// - `max_val`: The maximum value of the semaphore.
    /// - `options` **[[In parameter]]**: The options configuring the semaphore behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Returns Value
    ///
    /// Returns the semaphore UID on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0xD6DA4BA1)]
    pub unsafe fn sceKernelCreateSema(
        name: *const u8, attr: SemaphoreAttributes, init_val: i32, max_val: i32,
        options: Option<&SemaphoreOptions>,
    ) -> SceResult<SemaId>;

    /// Deletes a semaphore.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x28B6489C)]
    pub safe fn sceKernelDeleteSema(id: SemaId) -> SceResult<()>;

    /// Sends a signal to a semaphore.
    ///
    /// If the semaphore has reached the maximum value, and the signal overflows that maximum, an
    /// error is returned.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `signal`: The amount to signal the semaphore (i.e. if `2` then increment the semaphore by
    ///   `2`).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3F53E640)]
    pub safe fn sceKernelSignalSema(id: SemaId, signal: i32) -> SceResult<()>;

    /// Locks a semaphore until a target value or a timeout is reached.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `target_value`: The target value to wait.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4E3A1105)]
    pub safe fn sceKernelWaitSema(
        id: SemaId, target_value: i32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;


    /// Locks a semaphore until a target value or a timeout is reached, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `target_value`: The target value to wait.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6D212BAC)]
    pub safe fn sceKernelWaitSemaCB(
        id: SemaId, target_value: i32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Polls a semaphore if the target value has reached.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `target_value`: The target value to test.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x58B1F937)]
    pub safe fn sceKernelPollSema(id: SemaId, target_value: i32) -> SceResult<()>;

    /// Cancels the wait of a semaphore.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `set_value`: The value to set in the semaphore. If set to `-1`, the initial value of the
    ///   semaphore is used.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified semaphore.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8FFDF9A2)]
    pub safe fn sceKernelCancelSema(
        id: SemaId, set_val: i32, num_wait_threads: &mut u32,
    ) -> SceResult<()>;

    /// Gets the current state of a semaphore.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBC6FEBC5)]
    pub safe fn sceKernelReferSemaStatus(id: SemaId, info: &mut SemaphoreInfo) -> SceResult<()>;

    /// Creates a event flag.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new event flag. Only used for debug.
    /// - `attr`: The attribute for the event flag.
    /// - `init_bitmask`: The initial bitmask value for the event flag.
    /// - `options` **[[In parameter]]**: The options configuring the event flag behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the event flag UID on success, error value otherwise.
    #[nid(0x55C20A00)]
    pub unsafe fn sceKernelCreateEventFlag(
        name: *const u8, attr: EventFlagAttributes, init_bitmask: u32,
        options: Option<&EventFlagOptions>,
    ) -> SceResult<EventFlagId>;

    /// Deletes a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEF9E4C70)]
    pub safe fn sceKernelDeleteEventFlag(id: EventFlagId) -> SceResult<()>;

    /// Sets a bit pattern on a even flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1FB15A32)]
    pub safe fn sceKernelSetEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

    /// Clears a bit pattern of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to clean.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x812346E4)]
    pub safe fn sceKernelClearEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

    /// Waits for a bit pattern of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to wait for.
    /// - `wait_kind`: The kind of wait to use.
    /// - `out_bits` **[[Out parameter]]**: A reference to receive the bit pattern that was matched.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x402FCF22)]
    pub safe fn sceKernelWaitEventFlag(
        id: EventFlagId, bit_pat: u32, wait_kind: EventFlagWaitKinds, out_bits: &mut u32,
        timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Waits for a bit pattern of a event flag, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to wait for.
    /// - `wait_kind`: The kind of wait to use.
    /// - `out_bits` **[[Out parameter]]**: A reference to receive the bit pattern that was matched.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x328C546A)]
    pub safe fn sceKernelWaitEventFlagCB(
        id: EventFlagId, bit_pat: u32, wait_kind: EventFlagWaitKinds, out_bits: &mut u32,
        timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Polls for a bit pattern of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `bit_pat`: The bit patter to poll for.
    /// - `wait_kind`: The kind of wait to use.
    /// - `out_bits` **[[Out parameter]]**: A reference to receive the bit pattern that was matched.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x30FD48F0)]
    pub safe fn sceKernelPollEventFlag(
        id: EventFlagId, bit_pat: u32, wait_kind: EventFlagWaitKinds, out_bits: &mut u32,
    ) -> SceResult<()>;

    /// Cancels the wait of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `set_pat`: The bit patter to set in the event flag.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified semaphore.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCD203292)]
    pub safe fn sceKernelCancelEventFlag(id: EventFlagId, set_pat: u32, num_wait_threads: &mut u32);

    /// Gets the current state of a event flag.
    ///
    /// # Parameters
    ///
    /// - `id`: The event flag UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA66B0120)]
    pub safe fn sceKernelReferEventFlagStatus(
        id: EventFlagId, info: &mut EventFlagInfo,
    ) -> SceResult<()>;

    /// Creates a new mutex.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new mutex. Only used for debug.
    /// - `attr`: The attribute for the mutex.
    /// - `init_count`: The initial count value for the mutex.
    /// - `options` **[[In parameter]]**: The options configuring the mutex behavior. If [`None`],
    ///   the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the mutex UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xB7D098C6)]
    pub unsafe fn sceKernelCreateMutex(
        name: *const u8, attr: MutexAttributes, init_count: i32, options: Option<&MutexOptions>,
    ) -> SceResult<MutexId>;

    /// Deletes a mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xF8170FBE)]
    pub safe fn sceKernelDeleteMutex(id: MutexId) -> SceResult<()>;

    /// Locks a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xB011B11F)]
    pub safe fn sceKernelLockMutex(
        id: MutexId, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Locks a mutex a number of times, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x5BF4DD27)]
    pub safe fn sceKernelLockMutexCB(
        id: MutexId, lock_count: u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to lock a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to lock a mutex after resource acquisition. It must be
    ///   `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x0DDCD2C9)]
    pub safe fn sceKernelTryLockMutex(id: MutexId, lock_count: u32) -> SceResult<()>;

    /// Unlocks a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `unlock_count`: The number of times to unlock a mutex after resource acquisition. It must
    ///   be `> 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x6B30100F)]
    pub safe fn sceKernelUnlockMutex(id: MutexId, unlock_count: u32) -> SceResult<()>;

    /// Cancels the wait state of threads waiting on a mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `new_lock_count`: The new lock count to set on the mutex.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified mutex.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0x87D9223C)]
    pub safe fn sceKernelCancelMutex(id: MutexId, new_lock_count: u32, numWaitThreads: &mut u32);

    /// Gets the current state of a mutex.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`MutexInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xA9C2CB9A)]
    pub safe fn sceKernelReferMutexStatus(id: MutexId, info: &mut MutexInfo) -> SceResult<()>;

    /// Gets the current state of a lightweight mutex by its UID.
    ///
    /// # Parameters
    ///
    /// - `id`: The lightweight mutex UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`LwMutexInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.95.
    #[nid(0x4C145944)]
    pub safe fn sceKernelReferLwMutexStatusByID(
        id: LwMutexId, info: &mut LwMutexInfo,
    ) -> SceResult<()>;

    /// Creates a new message box.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new message box. Only used for
    ///   debug.
    /// - `attr`: The message box attributes.
    /// - `options` **[[In parameter]]**: The options configuring the message box behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the UID of the created message box on success, error value otherwise.
    #[nid(0x8125221D)]
    pub unsafe fn sceKernelCreateMbx(
        name: *const u8, attr: MsgBoxAttributes, options: Option<&mut MsgBoxOptions>,
    ) -> SceResult<MsgBoxId>;

    /// Deletes a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x86255ADA)]
    pub unsafe fn sceKernelDeleteMbx(id: MsgBoxId) -> SceResult<()>;

    /// Sends a message to a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[In parameter]]**: A pointer to a message to be forwarded to the receiver.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE9B3061E)]
    pub unsafe fn sceKernelSendMbx(id: MsgBoxId, msg: *mut MsgPacket) -> SceResult<()>;

    /// Waits to receive a message to a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[Out parameter]]**: A pointer to a pointer to receive the message.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x18260574)]
    pub unsafe fn sceKernelReceiveMbx(
        id: MsgBoxId, msg: *mut *mut MsgPacket, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Waits to receive a message to a message box, but service any callbacks as necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[Out parameter]]**: A pointer to a pointer to receive the message.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF3986382)]
    pub unsafe fn sceKernelReceiveMbxCB(
        id: MsgBoxId, msg: *mut *mut MsgPacket, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Polls if the message has arrived in a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `msg` **[[Out parameter]]**: A pointer to a pointer to receive the message.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0D81716A)]
    pub unsafe fn sceKernelPollMbx(id: MsgBoxId, msg: *mut *mut MsgPacket) -> SceResult<()>;

    /// Cancels the wait of a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified semaphore.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x87D4DD36)]
    pub safe fn sceKernelCancelReceiveMbx(
        id: MsgBoxId, num_wait_threads: &mut u32,
    ) -> SceResult<()>;

    /// Gets the current state of a message box.
    ///
    /// # Parameters
    ///
    /// - `id`: The message box UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA8E8C846)]
    pub safe fn sceKernelReferMbxStatus(id: MsgBoxId, info: &mut MsgBoxInfo) -> SceResult<()>;

    /// Creates a new message pipe.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new message pipe. Only used for
    ///   debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the message pipe.
    /// - `buf_size`: The size of the message pipe buffer. Zero is allowed.
    /// - `options` **[[In parameter]]**: The options configuring the message pipe behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the message pipe UID on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x7C0DC2A0)]
    pub unsafe fn sceKernelCreateMsgPipe(
        name: *const u8, partition_id: MemoryPartitionId, attr: MsgPipeAttributes,
        buf_size: SceSize, options: Option<&SemaphoreOptions>,
    ) -> SceResult<MsgPipeId>;

    /// Deletes a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF0B7DA1C)]
    pub safe fn sceKernelDeleteMsgPipe(id: MsgPipeId) -> SceResult<()>;

    /// Sends a message to a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[In parameter]]**: A pointer to data to send as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x876DBFAD)]
    pub unsafe fn sceKernelSendMsgPipe(
        id: MsgPipeId, msg_buf: *const c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Sends a message to a message pipe, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[In parameter]]**: A pointer to data to send as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x7C41F2C2)]
    pub unsafe fn sceKernelSendMsgPipeCB(
        id: MsgPipeId, msg_buf: *const c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;


    /// Tries to send a message to a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[In parameter]]**: A pointer to data to send as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x884C9F90)]
    pub unsafe fn sceKernelTrySendMsgPipe(
        id: MsgPipeId, msg_buf: *const c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32,
    ) -> SceResult<()>;

    /// Waits to receive a message from a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[Out parameter]]**: A pointer to data buffer to receive as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0x74829B76)]
    pub unsafe fn sceKernelReceiveMsgPipe(
        id: MsgPipeId, msg_buf: *mut c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Waits to receive a message from a message pipe, but service any callbacks as necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[Out parameter]]**: A pointer to data buffer to receive as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xFBFA697D)]
    pub unsafe fn sceKernelReceiveMsgPipeCB(
        id: MsgPipeId, msg_buf: *mut c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to receive a message from a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `msg_buf` **[[Out parameter]]**: A pointer to data buffer to receive as a message.
    /// - `msg_size`: The size of `msg_buf`.
    /// - `wait_kind`: The wait strategy to use.
    /// - `data_send_size` **[[Out parameter]]**: A reference to receive the number of bytes send.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0xDF52098F)]
    pub unsafe fn sceKernelTryReceiveMsgPipe(
        id: MsgPipeId, msg_buf: *mut c_void, msg_size: SceSize, wait_kind: MsgPipeWaitKind,
        data_send_size: &mut u32,
    ) -> SceResult<()>;

    /// Cancels the wait of message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The message pipe UID.
    /// - `num_send_wait_threads` **[[Out parameter]]**: A reference to receive the number of sender
    ///   threads that were waiting on the specified message pipe.
    /// - `num_recv_wait_threads` **[[Out parameter]]**: A reference to receive the number of
    ///   receiver threads that were waiting on the specified message pipe.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x349B864D)]
    pub safe fn sceKernelCancelMsgPipe(
        id: MsgPipeId, num_send_wait_threads: &mut u32, num_recv_wait_threads: &mut u32,
    ) -> SceResult<()>;

    /// Gets the current state of a message pipe.
    ///
    /// # Parameters
    ///
    /// - `id`: The semaphore UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`MsgPipeInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x33BE4024)]
    pub safe fn sceKernelReferMsgPipeStatus(id: MsgPipeId, info: &mut MsgPipeInfo)
        -> SceResult<()>;

    /// Creates a new variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new VPL. Only used for debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the variable-sized memory pool.
    /// - `size`: The size of the memory pool, in bytes.
    /// - `options` **[[In parameter]]**: The options configuring the variable-sized memory pool
    ///   behavior. If [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the variable-sized memory pool UID on success, error value otherwise.
    #[nid(0x56C039B5)]
    pub unsafe fn sceKernelCreateVpl(
        name: *const u8, partition_id: MemoryPartitionId, attr: VplAttributes, size: SceSize,
        options: Option<&VplOptions>,
    ) -> SceResult<VplId>;

    /// Deletes a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x89B3D48C)]
    pub safe fn sceKernelDeleteVpl(id: VplId) -> SceResult<()>;

    /// Allocates a memory block from a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `size`: The size to allocate.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBED27435)]
    pub unsafe fn sceKernelAllocateVpl(
        id: VplId, size: SceSize, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Allocates a memory block from a variable-sized memory pool, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `size`: The size to allocate.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEC0A693F)]
    pub unsafe fn sceKernelAllocateVplCB(
        id: VplId, size: SceSize, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to allocates a memory block from a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `size`: The size to allocate.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xAF36D708)]
    pub unsafe fn sceKernelTryAllocateVpl(
        id: VplId, size: SceSize, mem_block: *mut *mut c_void,
    ) -> SceResult<()>;

    /// Deallocates a memory block from a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `mem_block` **[[In parameter]]**: A pointer of the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    pub unsafe fn sceKernelFreeVpl(id: VplId, mem_block: *mut c_void) -> SceResult<()>;

    /// Cancels the wait of a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified ID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1D371B8A)]
    pub safe fn sceKernelCancelVpl(id: VplId, num_wait_threads: &mut u32) -> SceResult<()>;

    /// Gets the current state of a variable-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The variable-sized memory pool UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x39810265)]
    pub safe fn sceKernelReferVplStatus(id: VplId, info: &mut VplInfo) -> SceResult<()>;

    /// Creates a new fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new FPL. Only used for debug.
    /// - `partition_id`: The memory partition ID to use for allocations.
    /// - `attr`: The attribute for the fixed-sized memory pool.
    /// - `block_size`: The size of a memory block to use, in bytes.
    /// - `num_blocks`: The number of blocks to allocate.
    /// - `options` **[[In parameter]]**: The options configuring the fixed-sized memory pool
    ///   behavior. If [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the fixed-sized memory pool UID on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xC07BB470)]
    pub unsafe fn sceKernelCreateFpl(
        name: *const u8, partition_id: MemoryPartitionId, attr: FplAttributes, block_size: SceSize,
        num_blocks: SceSize, options: Option<&FplOptions>,
    ) -> SceResult<FplId>;

    /// Deletes a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xED1410E0)]
    pub safe fn sceKernelDeleteFpl(id: FplId) -> SceResult<()>;

    /// Allocates a memory block from a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD979E9BF)]
    pub unsafe fn sceKernelAllocateFpl(
        id: FplId, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Allocates a memory block from a fixed-sized memory pool, but service any callbacks as
    /// necessary.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    /// - `timeout` **[[InOut parameter]]**: Timeout in microseconds (?). If a timeout is specified
    ///   and the specified thread finished before the timeout, the remaining timeout is set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE7282CB6)]
    pub unsafe fn sceKernelAllocateFplCB(
        id: FplId, mem_block: *mut *mut c_void, timeout: Option<&mut u32>,
    ) -> SceResult<()>;

    /// Tries to allocates a memory block from a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[Out parameter]]**: A pointer to receive the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x623AE665)]
    pub unsafe fn sceKernelTryAllocateFpl(id: FplId, mem_block: *mut *mut c_void) -> SceResult<()>;

    /// Deallocates a memory block from a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `mem_block` **[[In parameter]]**: A pointer of the address of the allocated data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF6414A71)]
    pub unsafe fn sceKernelFreeFpl(id: FplId, mem_block: *mut c_void) -> SceResult<()>;

    /// Cancels the wait of a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `num_wait_threads` **[[Out parameter]]**: A reference to receive the number of threads
    ///   that were waiting on the specified ID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA8AA591F)]
    pub safe fn sceKernelCancelFpl(id: FplId, num_wait_threads: &mut u32) -> SceResult<()>;

    /// Gets the current state of a fixed-sized memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The fixed-sized memory pool UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD8199E4C)]
    pub safe fn sceKernelReferFplStatus(id: FplId, info: &mut FplInfo) -> SceResult<()>;

    /// Gets the current state of a user TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The user TLS memory pool UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x721067F3)]
    pub safe fn sceKernelReferTlsplStatus(id: TlsPoolId, info: &mut TlsPoolInfo) -> SceResult<()>;

    /// Gets the system time.
    ///
    /// # Parameters
    ///
    /// - `clock` **[[Out parameter]]**: A reference to [`SystemClock`] to receive the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDB738F35)]
    pub safe fn sceKernelGetSystemTime(clock: &mut SystemClock) -> SceResult<()>;

    /// Gets the system time as raw wide integer.
    ///
    /// # Return Value
    ///
    /// Returns the system time.
    #[nid(0x82BC5777)]
    pub safe fn sceKernelGetSystemTimeWide() -> u64;

    /// Gets the low part of the system time.
    ///
    /// # Return Value
    ///
    /// Returns the system time low bits.
    #[nid(0x369ED59D)]
    pub safe fn sceKernelGetSystemTimeLow() -> u32;

    /// Creates an alarm.
    ///
    /// # Parameters
    ///
    /// - `microsec`: The microseconds until the `handler` is called.
    /// - `handler` **[[In parameter]]**: The function pointer set as entry point.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the alarm handler.
    ///
    /// # Return Value
    ///
    /// Returns the alarm UID on success, error value otherwise.
    #[nid(0x6652B8CA)]
    pub unsafe fn sceKernelSetAlarm(
        microsec: u32, handler: AlarmHandler, common: *mut c_void,
    ) -> SceResult<AlarmId>;

    /// Creates an alarm using [`SystemClock`].
    ///
    /// # Parameters
    ///
    /// - `clock` **[[In parameter]]**: A reference to a [`SystemClock`] as the time until the
    ///   `handler` is called.
    /// - `handler` **[[In parameter]]**: The function pointer set as entry point.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the alarm handler.
    ///
    /// # Return Value
    ///
    /// Returns the alarm UID on success, error value otherwise.
    #[nid(0xB2C25152)]
    pub unsafe fn sceKernelSetSysClockAlarm(
        clock: &SystemClock, handler: AlarmHandler, common: *mut c_void,
    ) -> SceResult<AlarmId>;

    /// Cancels an alarm.
    ///
    /// # Parameters
    ///
    /// - `id`: The alarm UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7E65B999)]
    pub safe fn sceKernelCancelAlarm(id: AlarmId) -> SceResult<()>;

    /// Gets the current state of a alarm.
    ///
    /// # Parameters
    ///
    /// - `id`: The alarm UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`SemaphoreInfo`] to receive the semaphore
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDAA3F564)]
    pub safe fn sceKernelReferAlarmStatus(id: AlarmId, info: &mut AlarmInfo) -> SceResult<()>;

    /// Converts time in microseconds to [`SystemClock`].
    ///
    /// # Parameters
    ///
    /// - `microsec`: The time in microseconds.
    /// - `clock` **[[Out parameter]]**: A reference to [`SystemClock`] to receive the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x110DEC9A)]
    pub safe fn sceKernelUSec2SysClock(microsec: u32, clock: &mut SystemClock) -> SceResult<()>;

    /// Converts time in [`SystemClock`] to seconds and microseconds.
    ///
    /// # Parameters
    ///
    /// - `clock` **[[In parameter]]**: A reference to [`SystemClock`] to give the time information.
    /// - `sec` **[[Out parameter]]**: A reference to receive the seconds information.
    /// - `microsec` **[[Out parameter]]**: A reference to receive the microseconds information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBA6B92E2)]
    pub safe fn sceKernelSysClock2USec(
        clock: &SystemClock, sec: &mut u32, microsec: &mut u32,
    ) -> SceResult<()>;

    /// Converts time in microseconds to raw system clock.
    ///
    /// # Parameters
    ///
    /// - `microsec`: The time in microseconds.
    ///
    /// # Return Value
    ///
    /// Returns the clock in raw format.
    #[nid(0xC8CD158C)]
    pub safe fn sceKernelUSec2SysClockWide(microsec: u32) -> u64;

    /// Converts raw system time to seconds and microseconds.
    ///
    /// # Parameters
    ///
    /// - `raw_clock`: The raw system clock.
    /// - `sec` **[[Out parameter]]**: A reference to receive the seconds information.
    /// - `microsec` **[[Out parameter]]**: A reference to receive the microseconds information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE1619D7C)]
    pub safe fn sceKernelSysClock2USecWide(
        raw_clock: u64, sec: &mut u32, microsec: &mut u32,
    ) -> SceResult<()>;

    /// Creates a new virtual timer.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new virtual timer. Only used for
    ///   debug.
    /// - `options` **[[In parameter]]**: The options configuring the virtual timer behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Returns Value
    ///
    /// Returns the virtual timer UID on success, error value otherwise.
    #[nid(0x20FFF560)]
    pub unsafe fn sceKernelCreateVTimer(
        name: *const u8, options: Option<&VirtualTimerOptions>,
    ) -> SceResult<VirtualTimerId>;

    /// Deletes a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x328F9E52)]
    pub safe fn sceKernelDeleteVTimer(id: VirtualTimerId) -> SceResult<()>;

    /// Gets the base time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `base` **[[Out parameter]]**: A reference to receive the base time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB3A59970)]
    pub safe fn sceKernelGetVTimerBase(id: VirtualTimerId, base: &mut SystemClock)
        -> SceResult<()>;

    /// Gets the base time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns base time in raw format on success, `0xFFFFFFFFFFFFFFFF` (`-1 as u64`) on error.
    #[nid(0xB7C18B77)]
    pub safe fn sceKernelGetVTimerBaseWide(id: VirtualTimerId) -> u64;

    /// Gets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `time` **[[Out parameter]]**: A reference to receive the current time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x034A921F)]
    pub safe fn sceKernelGetVTimerTime(id: VirtualTimerId, time: &mut SystemClock)
        -> SceResult<()>;

    /// Gets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns current time in raw format on success, `0xFFFFFFFFFFFFFFFF` (`-1 as u64`) on error.
    #[nid(0xC0B3FFD2)]
    pub safe fn sceKernelGetVTimerTimeWide(id: VirtualTimerId) -> u64;

    /// Sets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `time` **[[InOut parameter]]**: A reference to the time to set and also to receive the
    ///   previous set value of the current time.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x542AD630)]
    pub safe fn sceKernelSetVTimerTime(id: VirtualTimerId, time: &mut SystemClock)
        -> SceResult<()>;

    /// Sets the current time of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `time`: The raw time to set.
    ///
    /// # Return Value
    ///
    /// Returns the previous current time in raw format on success, `0xFFFFFFFFFFFFFFFF` (`-1 as
    /// u64`) on error.
    #[nid(0xFB6425C3)]
    pub safe fn sceKernelSetVTimerTimeWide(id: VirtualTimerId, time: u64) -> u64;

    /// Starts a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns the previous timer state on success, error value otherwise.
    #[nid(0xC68D9437)]
    pub safe fn sceKernelStartVTimer(id: VirtualTimerId) -> SceResult<VirtualTimerState>;

    /// Stops a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// Returns the previous timer state on success, error value otherwise.
    #[nid(0xD0AEEE87)]
    pub safe fn sceKernelStopVTimer(id: VirtualTimerId) -> SceResult<VirtualTimerState>;

    /// Sets the virtual time handler and the schedule to execute it.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `schedule` **[[In parameter]]**: A reference to a time as the schedule time to execute the
    ///   `handler`.
    /// - `handler` **[[In parameter]]**: The function pointer of the handler to set.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD8B299AE)]
    pub unsafe fn sceKernelSetVTimerHandler(
        id: VirtualTimerId, schedule: &mut SystemClock, handler: VirtualTimerHandler,
        common: *mut c_void,
    ) -> SceResult<()>;

    /// Sets the virtual time handler and the schedule to execute it.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `schedule`: The time as the schedule time to execute the `handler` in raw format.
    /// - `handler` **[[In parameter]]**: The function pointer of the handler to set.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x53B00E9A)]
    pub unsafe fn sceKernelSetVTimerHandlerWide(
        id: VirtualTimerId, schedule: u64, handler: VirtualTimerHandlerWide, common: *mut c_void,
    ) -> SceResult<()>;

    /// Cancels the handler set for a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD2D615EF)]
    pub safe fn sceKernelCancelVTimerHandler(id: VirtualTimerId) -> SceResult<()>;

    /// Gets the current state of a virtual timer.
    ///
    /// # Parameters
    ///
    /// - `id`: The virtual timer UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`VirtualTimerInfo`] to receive the virtual
    ///   timer information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5F32BEAA)]
    pub safe fn sceKernelReferVTimerStatus(
        id: VirtualTimerId, info: &mut VirtualTimerInfo,
    ) -> SceResult<()>;

    /// Creates a new callback.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new callback. Only used for debug.
    /// - `entry` **[[In parameter]]**: The function pointer of the entry function to set.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the handler.
    ///
    /// # Return Value
    ///
    /// Returns the callback UID on success, error value otherwise.
    #[nid(0xE81CAF8F)]
    pub unsafe fn sceKernelCreateCallback(
        name: *const u8, entry: CallbackFunction, common: *mut c_void,
    ) -> SceResult<CallbackId>;

    /// Deletes a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEDBA5844)]
    pub safe fn sceKernelDeleteCallback(id: CallbackId) -> SceResult<()>;

    /// Notifies a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    /// - `arg`: The argument to be passed to the callback function.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xC11BA8C4)]
    pub safe fn sceKernelNotifyCallback(id: CallbackId, arg: i32) -> SceResult<()>;

    /// Cancels all notifications that were reported to a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBA4051D6)]
    pub safe fn sceKernelCancelCallback(id: CallbackId) -> SceResult<()>;

    /// Gets the notification count of a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    ///
    /// # Return Value
    ///
    /// Returns the number of notification send to the callback on success, error value otherwise.
    #[nid(0x2A3D44FF)]
    pub safe fn sceKernelGetCallbackCount(id: CallbackId) -> SceResult<u32>;

    /// Checks if a callback by the calling thread has been notified.
    ///
    /// If it was notified, the callback is called.
    ///
    /// # Return Value
    ///
    /// Returns the check status on success, error value otherwise.
    #[nid(0x349D6D6C)]
    pub safe fn sceKernelCheckCallback() -> SceResult<CallbackCheckStatus>;

    /// Gets the current state of a callback.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`CallbackInfo`] to receive the callback
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x730ED8BC)]
    pub safe fn sceKernelReferCallbackStatus(
        id: CallbackId, info: &mut CallbackInfo,
    ) -> SceResult<()>;

    /// Gets the current state of the system.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A reference to [`SystemInfo`] to receive the system
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x627E6F3A)]
    pub safe fn sceKernelReferSystemStatus(info: &mut SystemInfo) -> SceResult<()>;

    /// Gets a list of UIDs from threadman module.
    ///
    /// # Parameters
    ///
    /// - `kind`: The kind of resource to get in the list. Use [`ThreadIdKind::Any`] to get all of
    ///   them.
    /// - `buf` **[[Out parameter]]**: A pointer to a buffer list to receive the UIDs.
    /// - `buf_size`: The size of the `buf`.
    /// - `id_count` **[[Out parameter]]**: A optional reference to receive the number of UIDs of
    ///   the specified `kind` in the system.
    ///
    /// # Return Value
    ///
    /// Returns the number of UIDs added to the buffer on success, error value otherwise.
    #[nid(0x94416130)]
    pub unsafe fn sceKernelGetThreadmanIdList(
        kind: ThreadIdKind, buf: *mut SceUid, buf_size: SceSize, id_cound: Option<&mut u32>,
    ) -> SceResult<u32>;

    /// Gets the threadman UID kind of an ID.
    ///
    /// # Parameters
    ///
    /// - `id`: The UID to get the kind.
    ///
    /// # Return Value
    ///
    /// Returns the UID kind on success, error value otherwise.
    #[nid(0x57CF62DD)]
    pub safe fn sceKernelGetThreadmanIdType(id: SceUid) -> SceResult<ThreadIdKind>;

    /// Registers a thread event handler.
    ///
    /// # Parameters
    ///
    /// - `name` **[[In parameter]]**: The name assigned to the new thread event. Only used for
    ///   debug.
    /// - `thread_id`: The thread UID of the thread which the handler should be called. The
    ///   [`ThreadId::CALLING`] can be used to specify the calling thread UID and the
    ///   [`ThreadId::ALL_USER`] can be used to set to be called for all user threads.
    /// - `event_mask`: The thread events which the handler should be called.
    /// - `handler` **[[In parameter]]**: A function pointer to the entry function for the handler.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with the thread event
    ///   handler.
    ///
    /// # Return Value
    ///
    /// Returns the thread event UID on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x0C106E53)]
    pub unsafe fn sceKernelRegisterThreadEventHandler(
        name: *const u8, thread_id: ThreadId, event_mask: ThreadEvents,
        handler: ThreadEventHandler, common: *mut c_void,
    ) -> SceResult<ThreadEventId>;

    /// Releases a thread event handler.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread event UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x72F3C145)]
    pub safe fn sceKernelReleaseThreadEventHandler(id: ThreadEventId) -> SceResult<()>;

    /// Gets the current state of a thread event handler.
    ///
    /// # Parameters
    ///
    /// - `id`: The thread event UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`ThreadEventInfo`] to receive the thread
    ///   event handler information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x369EEB6B)]
    pub safe fn sceKernelReferThreadEventHandlerStatus(
        id: ThreadEventId, info: &mut ThreadEventInfo,
    ) -> SceResult<()>;

    /// Gets the available stack space on a kernel thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The kernel thread UID.
    ///
    /// # Return Value
    ///
    /// Returns the available stack size in bytes on success, error value otherwise.
    #[nid(0xD890B370)]
    pub safe fn sceKernelGetThreadKernelStackFreeSize(id: ThreadId) -> SceResult<u32>;

    /// Checks the calling kernel thread stack.
    ///
    /// # Return Value
    ///
    /// Returns the check status of the calling kernel thread stack on success, error value
    /// otherwise.
    #[nid(0x4FE44D5E)]
    pub safe fn sceKernelCheckThreadKernelStack() -> SceResult<i32>;

    /// Temporarily extends the kernel thread stack an executes a function with extended stack size.
    ///
    /// # Parameters
    ///
    /// - `stack_size`: The stack size (in bytes) to temporarily extend.
    /// - `func` **[[In parameter]]**: A pointer to the function to call with extended stack.
    /// - `common` **[[InOut parameter]]** A pointer to memory shared with the `func`.
    ///
    /// # Return Value
    ///
    /// Returns the result of the `func` on success, error value otherwise.
    #[nid(0xBC31C1B9)]
    pub safe fn sceKernelExtendKernelStack(
        stack_size: SceSize, func: ExtendStackFunc, common: *mut c_void,
    ) -> SceResult<u32>;

    /// Gets the system status flag.
    ///
    /// # Return Value
    ///
    /// Returns the system status flag.
    #[nid(0xFCB5EB49)]
    pub safe fn sceKernelGetSystemStatusFlag() -> u32;

    /// Setup the KTLS allocator.
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the allocator.
    /// - `func` **[[In parameter]]**: A function pointer to the allocation function.
    /// - `common` **[[InOut parameter]]** A pointer to memory shared with the `func`.
    ///
    /// # Return Value
    ///
    /// Returns the KTLS UID on success, error value otherwise.
    #[nid(0x04E72261)]
    pub unsafe fn sceKernelAllocateKTLS(
        id: SceUid, func: KtlsAllocFunc, common: *mut c_void,
    ) -> SceResult<KtlsId>;

    /// Deallocates a KTLS allocator.
    ///
    /// # Parameters
    ///
    /// - `id`: The KTLS allocator UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD198B811)]
    pub safe fn sceKernelFreeKTLS(id: KtlsId) -> SceResult<()>;

    /// Gets the KTLS of the calling thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The KTLS allocator UID.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the KTLS on success, null pointer on error.
    #[nid(0xA249EAAE)]
    pub safe fn sceKernelGetKTLS(id: KtlsId) -> *mut c_void;

    /// Gets the KTLS of a thread.
    ///
    /// # Parameters
    ///
    /// - `id`: The KTLS allocator UID.
    /// - `thread_id`: The thread UID to get the KTLS. The [`ThreadId::CALLING`] can be used to
    ///   specify the calling thread UID.
    /// - `mode`: Unknown purpose. Reverse engineer show it being passed `0` and `1`.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the KTLS on success, null pointer on error.
    #[nid(0x3AD875C3)]
    pub safe fn sceKernelGetThreadKTLS(id: KtlsId, thread_id: ThreadId, mode: u32) -> *mut c_void;
}


impl ThreadId {
    /// Special value to use on some functions for all user-threads.
    pub const ALL_USER: Self = unsafe { Self::from_raw_unchecked(0xFFFFFFF0) };
    /// Represent the calling thread UID in some functions.
    pub const CALLING: Self = unsafe { Self::from_raw_unchecked(0) };

    /// Create a new thread ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for ThreadId {}
unsafe impl SceResultOk for ThreadId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe {
            SceUid::handle_ok_value(ok_value).map(|id| Self::from_raw_unchecked(id.to_inner()))
        }
    }
}
unsafe impl SceIntoOkValue for ThreadId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl SemaId {
    /// Create a new semaphore ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new semaphore ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for SemaId {}
unsafe impl SceResultOk for SemaId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for SemaId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl EventFlagId {
    /// Create a new event flag ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new event flag ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for EventFlagId {}
unsafe impl SceResultOk for EventFlagId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for EventFlagId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl CallbackId {
    /// Create a new callback ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new callback ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for CallbackId {}
unsafe impl SceResultOk for CallbackId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for CallbackId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl crate::private::Sealed for ThreadState {}
unsafe impl SceResultOk for ThreadState {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        Ok(Self::from_bits_retain(ok_value))
    }
}
unsafe impl SceIntoOkValue for ThreadState {
    fn into_ok_value(self) -> u32 {
        self.bits()
    }
}

impl MutexId {
    /// Create a new mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for MutexId {}
unsafe impl SceResultOk for MutexId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for MutexId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for ThreadOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            stack_mem_partition: Default::default(),
        }
    }
}

impl Default for ThreadInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            status: Default::default(),
            entry: Default::default(),
            stack: Default::default(),
            stack_size: Default::default(),
            gp_reg: Default::default(),
            init_priority: Default::default(),
            curr_priority: Default::default(),
            wait_kind: Default::default(),
            wait_id: Default::default(),
            wakeup_count: Default::default(),
            exit_status: Default::default(),
            run_clocks: Default::default(),
            intr_preempt_count: Default::default(),
            thread_preempt_count: Default::default(),
            release_count: Default::default(),
            notify_callback: Default::default(),
        }
    }
}

impl Default for ThreadRunStatus {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            status: Default::default(),
            curr_priority: Default::default(),
            wait_kind: Default::default(),
            wait_id: Default::default(),
            wakeup_count: Default::default(),
            run_clocks: Default::default(),
            intr_preempt_count: Default::default(),
            thread_preempt_count: Default::default(),
            release_count: Default::default(),
            notify_callback: Default::default(),
        }
    }
}

impl Default for SemaphoreOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
        }
    }
}

impl Default for SemaphoreInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            init_val: Default::default(),
            curr_val: Default::default(),
            max_val: Default::default(),
            num_wait_threads: Default::default(),
        }
    }
}

impl Default for EventFlagOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
        }
    }
}

impl Default for EventFlagInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            init_pattern: Default::default(),
            curr_pattern: Default::default(),
            num_wait_threads: Default::default(),
        }
    }
}

impl Default for MutexOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
        }
    }
}

impl Default for MutexInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            init_count: Default::default(),
            curr_count: Default::default(),
            curr_owner: Default::default(),
            num_wait_threads: Default::default(),
        }
    }
}

impl LwMutexId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for LwMutexId {}
unsafe impl SceResultOk for LwMutexId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for LwMutexId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl LwMutexWorkArea {
    #[inline]
    pub const fn default_new() -> Self {
        Self {
            lock_count: 0,
            lock_thread: unsafe { ThreadId::from_raw_unchecked(0) },
            attr: MutexAttributes::from_bits_retain(0),
            num_wait_threads: 0,
            uid: unsafe { LwMutexId::from_raw_unchecked(0) },
            pad: [0; 3],
        }
    }
}

impl Default for LwMutexInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            uid: Default::default(),
            work_addr: Default::default(),
            init_count: Default::default(),
            curr_count: Default::default(),
            curr_owner: Default::default(),
            num_wait_threads: Default::default(),
        }
    }
}

impl MsgBoxId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for MsgBoxId {}
unsafe impl SceResultOk for MsgBoxId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for MsgBoxId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for MsgBoxInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            num_wait_threads: Default::default(),
            num_messages: Default::default(),
            top_msg: Default::default(),
        }
    }
}

impl MsgPipeId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for MsgPipeId {}
unsafe impl SceResultOk for MsgPipeId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for MsgPipeId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for MsgPipeOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
        }
    }
}

impl Default for MsgPipeInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            buf_size: Default::default(),
            buf_free_size: Default::default(),
            num_send_wait_threads: Default::default(),
            num_recv_wait_threads: Default::default(),
        }
    }
}

impl VplId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for VplId {}
unsafe impl SceResultOk for VplId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for VplId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for VplOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
        }
    }
}

impl FplId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for FplId {}
unsafe impl SceResultOk for FplId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for FplId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for FplOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            alignment: Default::default(),
        }
    }
}

impl TlsPoolId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for TlsPoolId {}
unsafe impl SceResultOk for TlsPoolId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for TlsPoolId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for TlsPoolOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            alignment: Default::default(),
        }
    }
}

impl Default for TlsPoolInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            attr: Default::default(),
            block_size: Default::default(),
            num_blocks: Default::default(),
            free_blocks: Default::default(),
            num_wait_threads: Default::default(),
        }
    }
}

impl AlarmId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for AlarmId {}
unsafe impl SceResultOk for AlarmId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for AlarmId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for AlarmInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            schedule: Default::default(),
            handler: Default::default(),
            common: Default::default(),
        }
    }
}

impl VirtualTimerId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for VirtualTimerId {}
unsafe impl SceResultOk for VirtualTimerId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for VirtualTimerId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for VirtualTimerOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
        }
    }
}

impl Default for VirtualTimerInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            state: Default::default(),
            base: Default::default(),
            current: Default::default(),
            schedule: Default::default(),
            handler: Default::default(),
            common: Default::default(),
        }
    }
}

impl crate::private::Sealed for VirtualTimerState {}
unsafe impl SceResultOk for VirtualTimerState {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0x00 => Ok(Self::NotRunning),
            0x01 => Ok(Self::Running),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for VirtualTimerState {
    fn into_ok_value(self) -> u32 {
        match self {
            VirtualTimerState::NotRunning => 0x00,
            VirtualTimerState::Running => 0x01,
        }
    }
}

impl crate::private::Sealed for CallbackTermState {}
unsafe impl SceResultOk for CallbackTermState {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0x00 => Ok(Self::NormalTermination),
            0x01 => Ok(Self::DeletedCallback),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for CallbackTermState {
    fn into_ok_value(self) -> u32 {
        match self {
            CallbackTermState::NormalTermination => 0x00,
            CallbackTermState::DeletedCallback => 0x01,
        }
    }
}

impl Default for CallbackInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            thread_id: Default::default(),
            entry: Default::default(),
            notify_count: Default::default(),
            notify_arg: Default::default(),
        }
    }
}

impl crate::private::Sealed for CallbackCheckStatus {}
unsafe impl SceResultOk for CallbackCheckStatus {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0x00 => Ok(Self::NoCallback),
            0x01 => Ok(Self::CallbackCalled),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            status: Default::default(),
            idle_clocks: Default::default(),
            comes_out_of_idle_count: Default::default(),
            thread_switch_count: Default::default(),
            vfpu_switch_count: Default::default(),
        }
    }
}

impl crate::private::Sealed for ThreadIdKind {}
unsafe impl SceResultOk for ThreadIdKind {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            1 => Ok(Self::Any),
            2 => Ok(Self::Semaphore),
            3 => Ok(Self::EventFlag),
            4 => Ok(Self::MessageBox),
            5 => Ok(Self::VariablepMemoryPool),
            6 => Ok(Self::FixeMemoryPool),
            7 => Ok(Self::MessagePipe),
            8 => Ok(Self::Callback),
            9 => Ok(Self::ThreadEventHandler),
            10 => Ok(Self::Alarm),
            11 => Ok(Self::VirtualTimer),
            12 => Ok(Self::Mutex),
            13 => Ok(Self::LightweightMutex),
            14 => Ok(Self::TlsMemoryPool),
            64 => Ok(Self::SleepThread),
            65 => Ok(Self::DelayThread),
            66 => Ok(Self::SuspendThread),
            67 => Ok(Self::DormantThread),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for ThreadIdKind {
    fn into_ok_value(self) -> u32 {
        self as u32
    }
}

impl Default for ThreadEventInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            name: Default::default(),
            thread_id: Default::default(),
            event_mask: Default::default(),
            handler: Default::default(),
            common: Default::default(),
        }
    }
}

impl KtlsId {
    /// Create a new lightweight mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for KtlsId {}
unsafe impl SceResultOk for KtlsId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for KtlsId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl ThreadAttributes {
    /// Creates a `ThreadAttributes` with the default value for the main thread.
    #[inline]
    pub const fn main_default() -> Self {
        cfg_select! {
            any(prx, feature = "kernel") => Self::from_bits_retain(0),
            eboot => Self::UserMode.union(Self::UseVFPU),
            _ => Self::UserMode,
        }
    }
}
