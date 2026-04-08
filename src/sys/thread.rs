use core::{ffi::c_void, mem::MaybeUninit};

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{
    mem::MemoryPartitionId, time::SystemClock, SceError, SceResult, SceResultOk, SceSize, SceUid,
};

/// The thread UID, created with [`sceKernelCreateThread`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ThreadId(SceUid);

/// The semaphore UID, created with [`sceKernelCreateSema`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SemaId(SceUid);

/// The event flag UID, created with [`sceKernelCreateEventFlag`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EventFlagId(SceUid);

/// The callback UID, created with [`sceKernelCreateCallback`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CallbackId(SceUid);

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

/// The possible states that a PSP thread can be.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ThreadState {
    Run   = 0x01,
    Ready = 0x02,
    Wait  = 0x04,
    Suspend = 0x08,
    Dormant = 0x10,
    Dead  = 0x20,

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
    TLSPL = 0x0E,

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
pub struct ThreadInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the thread.
    pub name: [u8; 32],
    /// The thread attributes.
    pub attr: ThreadAttributes,
    /// The thread current status.
    pub status: u32,
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
    /// Allows recursive locks on a mutex by the thread that acquired the it.
    RecursiveLock = 0x200,
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
    /// Allocates a variable-sized memory pool  closest to memory bottom (i.e. High address).
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
#[doc(alias("SceKernelAlarmHandler"))]
pub type AlarmHandler = unsafe extern "C" fn(common: *mut c_void) -> SceResult<u32>;

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

#[psp_stub(libname = "ThreadManForUser", flags = 0x4001)]
extern "C" {
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
        id: ThreadId, arg_len: SceSize, argp: *mut c_void,
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
    pub fn sceKernelExitThread(status: u32) -> SceResult<()>;

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
    pub fn sceKernelExitDeleteThread(status: u32) -> SceResult<()>;

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
    pub fn sceKernelSuspendDispatchThread() -> SceResult<ThreadState>;

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
    pub fn sceKernelResumeDispatchThread(state: ThreadState) -> SceResult<()>;

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
    pub fn sceKernelChangeCurrentThreadAttr(
        clear_attr: ThreadAttributes, set_attr: ThreadAttributes,
    ) -> SceResult<()>;

    /// Gets the current priority of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the current priority of the calling thread on success, error value otherwise.
    #[nid(0x94AA61EE)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelGetThreadCurrentPriority() -> SceResult<i32>;

    /// Gets the thread UID of the calling thread.
    ///
    /// # Returns Value
    ///
    /// Returns the thread UID of the calling thread on success, error value otherwise.
    #[nid(0x293B45B8)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelGetThreadId() -> SceResult<ThreadId>;

    /// Makes the calling thread to enter in a [`Wait`](ThreadState::Wait) state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9ACE131E)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelSleepThread() -> SceResult<()>;

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
    pub fn sceKernelSleepThreadCB() -> SceResult<()>;

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
    pub fn sceKernelWakeupThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelDonateWakeupThread(donate_id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelReleaseWaitThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelCancelWakeupThread(id: ThreadId) -> SceResult<u32>;

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
    pub fn sceKernelSuspendThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelResumeThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelWaitThreadEnd(id: ThreadId, timeout: Option<&mut u32>) -> SceResult<u32>;

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
    pub fn sceKernelWaitThreadEndCB(id: ThreadId, timeout: Option<&mut u32>) -> SceResult<u32>;

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
    pub fn sceKernelDelayThread(delay: u32) -> SceResult<()>;

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
    pub fn sceKernelDelayThreadCB(delay: u32) -> SceResult<()>;

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
    pub fn sceKernelDelaySysClockThread(delay: &SystemClock) -> SceResult<()>;

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
    pub fn sceKernelDelaySysClockThreadCB(delay: &SystemClock) -> SceResult<()>;

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
    pub fn sceKernelRotateThreadReadyQueue(priority: u32) -> SceResult<()>;

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
    pub fn sceKernelGetThreadExitStatus(id: ThreadId) -> SceResult<u32>;

    /// Gets the remaining free size of the calling thread stack (?)
    ///
    /// # Return Value
    ///
    /// The remaining free size of the calling thread stack (probably in bytes).
    #[nid(0xD13BDE95)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelCheckThreadStack() -> SceSize;

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
    pub fn sceKernelGetThreadStackFreeSize(id: ThreadId) -> SceResult<SceSize>;

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
    pub fn sceKernelReferThreadStatus(id: ThreadId, info: &mut ThreadInfo) -> SceResult<()>;

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
    pub fn sceKernelReferThreadRunStatus(
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
    pub fn sceKernelDeleteSema(id: SemaId) -> SceResult<()>;

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
    pub fn sceKernelSignalSema(id: SemaId, signal: i32) -> SceResult<()>;

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
    pub fn sceKernelWaitSema(
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
    pub fn sceKernelWaitSemaCB(
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
    pub fn sceKernelPollSema(id: SemaId, target_value: i32) -> SceResult<()>;

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
    pub fn sceKernelCancelSema(
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
    pub fn sceKernelReferSemaStatus(id: SemaId, info: &mut SemaphoreInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteEventFlag(id: EventFlagId) -> SceResult<()>;

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
    pub fn sceKernelSetEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

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
    pub fn sceKernelClearEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

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
    pub fn sceKernelWaitEventFlag(
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
    pub fn sceKernelWaitEventFlagCB(
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
    pub fn sceKernelPollEventFlag(
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
    pub fn sceKernelCancelEventFlag(id: EventFlagId, set_pat: u32, num_wait_threads: &mut u32);

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
    pub fn sceKernelReferEventFlagStatus(
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
    pub fn sceKernelDeleteMutex(id: MutexId) -> SceResult<()>;

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
    pub fn sceKernelLockMutex(
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
    pub fn sceKernelLockMutexCB(
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
    pub fn sceKernelTryLockMutex(id: MutexId, lock_count: u32) -> SceResult<()>;

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
    pub fn sceKernelUnlockMutex(id: MutexId, unlock_count: u32) -> SceResult<()>;

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
    pub fn sceKernelCancelMutex(id: MutexId, new_lock_count: u32, numWaitThreads: &mut u32);

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
    pub fn sceKernelReferMutexStatus(id: MutexId, info: &mut MutexInfo) -> SceResult<()>;

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
        work_area: &mut MaybeUninit<LwMutexWorkArea>, name: *const u8, attr: MutexAttributes,
        init_count: i32, options: Option<&LwMutexOptions>,
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
    pub fn sceKernelDeleteLwMutex(work_area: &mut LwMutexWorkArea) -> SceResult<()>;

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
    pub fn _sceKernelLockLwMutex(
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
    pub fn _sceKernelLockLwMutexCB(
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
    pub fn _sceKernelTryLockLwMutex(
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
    pub fn _sceKernelUnlockLwMutex(
        work_area: &mut LwMutexWorkArea, unlock_count: u32,
    ) -> SceResult<()>;

    /// Gets the current state of a lightweight mutex by its UID.
    ///
    /// # Parameters
    ///
    /// - `id`: The lightweight mutex UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`MutexInfo`] to receive the semaphore
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
    pub fn sceKernelReferLwMutexStatusByID(id: LwMutexId, info: &mut LwMutexInfo) -> SceResult<()>;

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
    pub fn sceKernelCancelReceiveMbx(id: MsgBoxId, num_wait_threads: &mut u32) -> SceResult<()>;

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
    pub fn sceKernelReferMbxStatus(id: MsgBoxId, info: &mut MsgBoxInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteMsgPipe(id: MsgPipeId) -> SceResult<()>;

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
    pub fn sceKernelCancelMsgPipe(
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
    pub fn sceKernelReferMsgPipeStatus(id: MsgPipeId, info: &mut MsgPipeInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteVpl(id: VplId) -> SceResult<()>;

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
    pub fn sceKernelCancelVpl(id: VplId, num_wait_threads: &mut u32) -> SceResult<()>;

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
    pub fn sceKernelReferVplStatus(id: VplId, info: &mut VplInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteFpl(id: FplId) -> SceResult<()>;

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
    pub fn sceKernelCancelFpl(id: FplId, num_wait_threads: &mut u32) -> SceResult<()>;

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
    pub fn sceKernelReferFplStatus(id: FplId, info: &mut FplInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteTlspl(id: TlsPoolId) -> SceResult<()>;

    /// Gets the address of a user TLS memory pool.
    ///
    /// # Parameters
    ///
    /// - `id`: The user TLS memory pool UID.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the user TLS memory pool on success, null value on error.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0xFA835CDE)]
    pub fn sceKernelGetTlsAddr(id: TlsPoolId) -> *mut c_void;

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
    pub fn sceKernelReferTlsplStatus(id: TlsPoolId, info: &mut TlsPoolInfo) -> SceResult<()>;

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
    pub fn sceKernelGetSystemTime(clock: &mut SystemClock) -> SceResult<()>;

    /// Gets the system time as raw wide integer.
    ///
    /// # Return Value
    ///
    /// Returns the system time.
    #[nid(0x82BC5777)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelGetSystemTimeWide() -> u64;

    /// Gets the low part of the system time.
    ///
    /// # Return Value
    ///
    /// Returns the system time low bits.
    #[nid(0x369ED59D)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelGetSystemTimeLow() -> u32;

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
    pub fn sceKernelSetAlarm(
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
    pub fn sceKernelSetSysClockAlarm(
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
    pub fn sceKernelCancelAlarm(id: AlarmId) -> SceResult<()>;

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
    pub fn sceKernelReferAlarmStatus(id: AlarmId, info: &mut AlarmInfo) -> SceResult<()>;

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
    pub fn sceKernelUSec2SysClock(microsec: u32, clock: &mut SystemClock) -> SceResult<()>;

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
    pub fn sceKernelSysClock2USec(
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
    pub fn sceKernelUSec2SysClockWide(microsec: u32) -> u64;

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
    pub fn sceKernelSysClock2USecWide(
        raw_clock: u64, sec: &mut u32, microsec: &mut u32,
    ) -> SceResult<()>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "ThreadManForKernel", flags = 0x0001)]
extern "C" {
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
        id: ThreadId, arg_len: SceSize, argp: *mut c_void,
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
    pub fn sceKernelExitThread(status: u32) -> SceResult<()>;

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
    pub fn sceKernelExitDeleteThread(status: u32) -> SceResult<()>;

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
    pub fn sceKernelSuspendDispatchThread() -> SceResult<ThreadState>;

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
    pub fn sceKernelResumeDispatchThread(state: ThreadState) -> SceResult<()>;

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
    pub fn sceKernelChangeCurrentThreadAttr(
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
    pub fn sceKernelChangeThreadPriority(id: ThreadId, priority: i32) -> SceResult<()>;

    /// Gets the current priority of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the current priority of the calling thread on success, error value otherwise.
    #[nid(0x94AA61EE)]
    pub fn sceKernelGetThreadCurrentPriority() -> SceResult<i32>;

    /// Gets the thread UID of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the thread UID of the calling thread on success, error value otherwise.
    #[nid(0x293B45B8)]
    pub fn sceKernelGetThreadId() -> SceResult<ThreadId>;

    /// Makes the calling thread to enter in a [`Wait`](ThreadState::Wait) state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9ACE131E)]
    pub fn sceKernelSleepThread() -> SceResult<()>;

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
    pub fn sceKernelSleepThreadCB() -> SceResult<()>;

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
    pub fn sceKernelWakeupThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelDonateWakeupThread(donate_id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelReleaseWaitThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelCancelWakeupThread(id: ThreadId) -> SceResult<u32>;

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
    pub fn sceKernelSuspendThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelResumeThread(id: ThreadId) -> SceResult<()>;

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
    pub fn sceKernelWaitThreadEnd(id: ThreadId, timeout: Option<&mut u32>) -> SceResult<u32>;

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
    pub fn sceKernelWaitThreadEndCB(id: ThreadId, timeout: Option<&mut u32>) -> SceResult<u32>;

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
    pub fn sceKernelDelayThread(delay: u32) -> SceResult<()>;

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
    pub fn sceKernelDelayThreadCB(delay: u32) -> SceResult<()>;

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
    pub fn sceKernelDelaySysClockThread(delay: &SystemClock) -> SceResult<()>;

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
    pub fn sceKernelDelaySysClockThreadCB(delay: &SystemClock) -> SceResult<()>;

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
    pub fn sceKernelRotateThreadReadyQueue(priority: u32) -> SceResult<()>;

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
    pub fn sceKernelGetThreadExitStatus(id: ThreadId) -> SceResult<u32>;

    /// Gets the remaining free size of the calling thread stack (?)
    ///
    /// # Return Value
    ///
    /// The remaining free size of the calling thread stack (probably in bytes).
    #[nid(0xD13BDE95)]
    pub fn sceKernelCheckThreadStack() -> SceSize;

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
    pub fn sceKernelGetThreadStackFreeSize(id: ThreadId) -> SceResult<SceSize>;

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
    pub fn sceKernelReferThreadStatus(id: ThreadId, info: &mut ThreadInfo) -> SceResult<()>;

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
    pub fn sceKernelReferThreadRunStatus(
        id: ThreadId, run_status: &mut ThreadRunStatus,
    ) -> SceResult<()>;

    /// Gets if the calling thread is a user mode thread.
    ///
    /// # Return Value
    ///
    /// Returns if the thread is user mode on success, error value otherwise.
    #[nid(0x85A2A5BF)]
    pub fn sceKernelIsUserModeThread() -> SceResult<bool>;

    /// Puts all user mode threads in the system in a [`Suspend`](ThreadState::Suspend) state.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8FD9F70C)]
    pub fn sceKernelSuspendAllUserThreads() -> SceResult<()>;

    /// Gets the user level of the calling thread.
    ///
    /// # Return Value
    ///
    /// Returns the user level of the calling thread on success, error value otherwise.
    #[nid(0xF6427665)]
    pub fn sceKernelGetUserLevel() -> SceResult<u32>;

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
    pub fn sceKernelDeleteSema(id: SemaId) -> SceResult<()>;

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
    pub fn sceKernelSignalSema(id: SemaId, signal: i32) -> SceResult<()>;

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
    pub fn sceKernelWaitSema(
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
    pub fn sceKernelWaitSemaCB(
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
    pub fn sceKernelPollSema(id: SemaId, target_value: i32) -> SceResult<()>;

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
    pub fn sceKernelCancelSema(
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
    pub fn sceKernelReferSemaStatus(id: SemaId, info: &mut SemaphoreInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteEventFlag(id: EventFlagId) -> SceResult<()>;

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
    pub fn sceKernelSetEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

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
    pub fn sceKernelClearEventFlag(id: EventFlagId, bit_pat: u32) -> SceResult<()>;

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
    pub fn sceKernelWaitEventFlag(
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
    pub fn sceKernelWaitEventFlagCB(
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
    pub fn sceKernelPollEventFlag(
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
    pub fn sceKernelCancelEventFlag(id: EventFlagId, set_pat: u32, num_wait_threads: &mut u32);

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
    pub fn sceKernelReferEventFlagStatus(
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
    pub fn sceKernelDeleteMutex(id: MutexId) -> SceResult<()>;

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
    pub fn sceKernelLockMutex(
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
    pub fn sceKernelLockMutexCB(
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
    pub fn sceKernelTryLockMutex(id: MutexId, lock_count: u32) -> SceResult<()>;

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
    pub fn sceKernelUnlockMutex(id: MutexId, unlock_count: u32) -> SceResult<()>;

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
    pub fn sceKernelCancelMutex(id: MutexId, new_lock_count: u32, numWaitThreads: &mut u32);

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
    pub fn sceKernelReferMutexStatus(id: MutexId, info: &mut MutexInfo) -> SceResult<()>;

    /// Gets the current state of a lightweight mutex by its UID.
    ///
    /// # Parameters
    ///
    /// - `id`: The lightweight mutex UID.
    /// - `info` **[[InOut parameter]]**: A reference to [`MutexInfo`] to receive the semaphore
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
    pub fn sceKernelReferLwMutexStatusByID(id: LwMutexId, info: &mut LwMutexInfo) -> SceResult<()>;

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
    pub fn sceKernelCancelReceiveMbx(id: MsgBoxId, num_wait_threads: &mut u32) -> SceResult<()>;

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
    pub fn sceKernelReferMbxStatus(id: MsgBoxId, info: &mut MsgBoxInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteMsgPipe(id: MsgPipeId) -> SceResult<()>;

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
    pub fn sceKernelCancelMsgPipe(
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
    pub fn sceKernelReferMsgPipeStatus(id: MsgPipeId, info: &mut MsgPipeInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteVpl(id: VplId) -> SceResult<()>;

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
    pub fn sceKernelCancelVpl(id: VplId, num_wait_threads: &mut u32) -> SceResult<()>;

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
    pub fn sceKernelReferVplStatus(id: VplId, info: &mut VplInfo) -> SceResult<()>;

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
    pub fn sceKernelDeleteFpl(id: FplId) -> SceResult<()>;

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
    pub fn sceKernelCancelFpl(id: FplId, num_wait_threads: &mut u32) -> SceResult<()>;

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
    pub fn sceKernelReferFplStatus(id: FplId, info: &mut FplInfo) -> SceResult<()>;

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
    pub fn sceKernelReferTlsplStatus(id: TlsPoolId, info: &mut TlsPoolInfo) -> SceResult<()>;

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
    pub fn sceKernelGetSystemTime(clock: &mut SystemClock) -> SceResult<()>;

    /// Gets the system time as raw wide integer.
    ///
    /// # Return Value
    ///
    /// Returns the system time.
    #[nid(0x82BC5777)]
    pub fn sceKernelGetSystemTimeWide() -> u64;

    /// Gets the low part of the system time.
    ///
    /// # Return Value
    ///
    /// Returns the system time low bits.
    #[nid(0x369ED59D)]
    pub fn sceKernelGetSystemTimeLow() -> u32;

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
    pub fn sceKernelSetAlarm(
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
    pub fn sceKernelSetSysClockAlarm(
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
    pub fn sceKernelCancelAlarm(id: AlarmId) -> SceResult<()>;

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
    pub fn sceKernelReferAlarmStatus(id: AlarmId, info: &mut AlarmInfo) -> SceResult<()>;

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
    pub fn sceKernelUSec2SysClock(microsec: u32, clock: &mut SystemClock) -> SceResult<()>;

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
    pub fn sceKernelSysClock2USec(
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
    pub fn sceKernelUSec2SysClockWide(microsec: u32) -> u64;

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
    pub fn sceKernelSysClock2USecWide(
        raw_clock: u64, sec: &mut u32, microsec: &mut u32,
    ) -> SceResult<()>;
}


impl ThreadId {
    /// Represent the calling thread UID in some functions.
    pub const CALLING: Self = unsafe { Self::new_unchecked(0) };

    /// Create a new thread ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for ThreadId {}
unsafe impl SceResultOk for ThreadId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

impl SemaId {
    /// Create a new semaphore ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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

impl EventFlagId {
    /// Create a new event flag ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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

impl CallbackId {
    /// Create a new callback ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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

impl crate::private::Sealed for ThreadState {}
unsafe impl SceResultOk for ThreadState {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        Ok(Self::from_bits_retain(ok_value))
    }
}

impl MutexId {
    /// Create a new mutex ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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

impl LwMutexWorkArea {
    #[inline]
    pub const fn default_new() -> Self {
        Self {
            lock_count: 0,
            lock_thread: unsafe { ThreadId::new_unchecked(0) },
            attr: MutexAttributes::from_bits_retain(0),
            num_wait_threads: 0,
            uid: unsafe { LwMutexId::new_unchecked(0) },
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
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
