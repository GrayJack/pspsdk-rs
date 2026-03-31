use core::ffi::c_void;

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{
    mem::MemoryPartitionId, time::SystemClock, SceError, SceResult, SceResultOk, SceSize, SceUid,
};

/// The thread UID, created with [`sceKernelCreateThread`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(SceUid);

/// The semaphore UID, created with [`sceKernelCreateSema`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemaId(SceUid);

/// The event flag UID, created with [`sceKernelCreateEventFlag`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventFlagId(SceUid);

/// The callback UID, created with [`sceKernelCreateCallback`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[repr(C)]
#[derive(Debug, Clone)]
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

    /// Unlock a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to unlock a mutex after resource acquisition. It must be
    ///   `> 0`.
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

    /// Unlock a mutex a number of times.
    ///
    /// # Parameters
    ///
    /// - `id`: The mutex UID.
    /// - `lock_count`: The number of times to unlock a mutex after resource acquisition. It must be
    ///   `> 0`.
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
