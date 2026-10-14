//! PSP exception handler.

use core::ffi::c_void;

use pspsdk_macros::psp_stub;

#[allow(unused)]
use crate::sys::SceResult;

/// The type of the Non-Maskable Interrupt handler.
pub type NmiHandler = unsafe extern "C" fn();

/// A PSP exception handler internal structure. This is not meant to be used directly.
///
/// But the thing passed to `*ExceptionHandler` functions must instead be considered a custom ABI
/// function.
///
/// Something similar to this:
///
/// ```no_run
/// /// Simple infinite loop exception
/// #[unsafe(naked)]
/// pub unsafe extern "C" fn my_exception(k1: u32) -> u32 {
///     core::arch::naked_asm!(
///         ".set noreorder",
///         ".set noat",
///         "nop", // Empty `next`, filled at runtime if needed.
///         "nop", // `unk` which should be always zero.
///         "loc:",
///         "b loc", // `callback` actual start of the exception logic.
///         "nop",
///     )
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
#[doc(alias("SceExceptionHandler"))]
pub struct ExceptionHandler {
    /// The next exception handler.
    ///
    /// For the `Default Exception Handler`, this needs to be null.
    pub next: *mut ExceptionHandler,
    /// Unknown, always zero.
    pub unk: u32,
    /// The actual start when the exception happens.
    pub callback: Option<*mut c_void>,
}

/// The exceptions kinds in value.
///
/// Many values are unknown, but listed, since it may show up in reversed code, or could be used for
/// patches.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExceptionKind {
    /// Interruption exception.
    Interruption = 1,
    Unknown2 = 2,
    Unknown3 = 3,
    /// Instruction load or fetch exception.
    LoadFetch = 4,
    /// Address store exception.
    AddressStore = 5,
    /// Instruction fetch bus error exception.
    FetchBusError = 6,
    /// Instruction load or store bus error exception.
    LoadStoreBusError = 7,
    /// System call exception.
    Syscall = 8,
    /// Break point exception.
    Breakpoint = 9,
    /// Reserved instruction exception.
    ReservedInstruction = 10,
    /// Unusable coprocessor exception.
    UnusableCpu = 11,
    /// Arithmetic overflow exception.
    Overflow = 12,
    Unknown13 = 13,
    Unknown14 = 14,
    /// Float-point exception.
    FloatPoint = 15,
    Unknown16 = 16,
    Unknown17 = 17,
    Unknown18 = 18,
    Unknown19 = 19,
    Unknown20 = 20,
    Unknown21 = 21,
    Unknown22 = 22,
    /// Watch exception.
    Watch = 23,
    Unknown24 = 24,
    Unknown25 = 25,
    Unknown26 = 26,
    Unknown27 = 27,
    Unknown28 = 28,
    Unknown29 = 29,
    Unknown30 = 30,
    /// Virtual Coherency Exception data (used for NMI handling apparently).
    VCED  = 31,
}

// FIXME: Add `sceExceptionManager_0xD3744BE0` once reversed.
#[psp_stub(libname = "sceExceptionManager", flags = 0x4009, use_crate)]
extern "C" {}

// FIXME: Higher level NMI kind.
#[cfg(feature = "kernel")]
#[psp_stub(libname = "ExceptionManagerForKernel", flags = 0x0009, use_crate)]
extern "C" {
    /// Registers a default exception handler.
    ///
    /// # Parameters
    ///
    /// - `handler` **[[In parameter]]**: The handler to register. It must start with two `nop`
    ///   instructions. More details on the handler ABI on [`ExceptionHandler`] documentation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `handler` must follow the custom ABI for the exception handler. The function does have
    /// some checks, but you must be careful with it anyways.
    #[nid(if cfg!(feature = "psp_660") { 0x0A998599 }
        else if cfg!(feature = "psp_630") { 0x46EA27C3 }
        else if cfg!(feature = "psp_600") { 0x557FBABB }
        else if cfg!(feature = "psp_570") { 0x82FEF48F }
        else if cfg!(feature = "psp_500") { 0xCF57A486 }
        else if cfg!(feature = "psp_420") { 0x63400637 }
        else if cfg!(feature = "psp_395") { 0x08C90CB2 }
        else if cfg!(feature = "psp_380") { 0xC91681F2 }
        else { 0x565C0B0E }
    )]
    pub unsafe fn sceKernelRegisterDefaultExceptionHandler(handler: *mut c_void) -> SceResult<()>;

    /// Releases/Unregisters a handler.
    ///
    /// # Parameters
    ///
    /// - `handler` **[[In parameter]]**: The handler to unregister. It must start with two `nop`
    ///   instructions. More details on the handler ABI on [`ExceptionHandler`] documentation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `handler` must follow the custom ABI for the exception handler. The function does have
    /// some checks, but you must be careful with it anyways.
    #[nid(if cfg!(feature = "psp_660") { 0x409DBB53 }
        else if cfg!(feature = "psp_630") { 0x409632B8 }
        else if cfg!(feature = "psp_600") { 0x85F02133 }
        else if cfg!(feature = "psp_570") { 0xDD89F9C0 }
        else if cfg!(feature = "psp_500") { 0xA966D178 }
        else if cfg!(feature = "psp_420") { 0x4863E952 }
        else if cfg!(feature = "psp_395") { 0x0297D344 }
        else if cfg!(feature = "psp_380") { 0xB203C040 }
        else { 0x291FF031 }
    )]
    pub unsafe fn sceKernelReleaseDefaultExceptionHandler(handler: *mut c_void) -> SceResult<()>;

    /// Gets the currently active default exception handler.
    ///
    /// # Return Value
    ///
    /// A pointer to the handler on success, null value on error.
    #[nid(if cfg!(feature = "psp_660") { 0x27E6A663 }
        else if cfg!(feature = "psp_630") { 0xD061B94E }
        else if cfg!(feature = "psp_600") { 0x014FD1D1 }
        else if cfg!(feature = "psp_570") { 0x056613A8 }
        else if cfg!(feature = "psp_500") { 0x60DFC510 }
        else if cfg!(feature = "psp_420") { 0xAB5F1C05 }
        else if cfg!(feature = "psp_395") { 0x24BC1029 }
        else if cfg!(feature = "psp_380") { 0x366FAF27 }
        else { 0xDF83875E }
    )]
    pub fn sceKernelGetActiveDefaultExceptionHandler() -> *mut ExceptionHandler;

    /// Registers a exception handler for a exception kind.
    ///
    /// # Parameters
    ///
    /// - `kind`: The exception kind to be registered.
    /// - `handler` **[[In parameter]]**: The handler to register. It must start with two `nop`
    ///   instructions. More details on the handler ABI on [`ExceptionHandler`] documentation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `handler` must follow the custom ABI for the exception handler. The function does have
    /// some checks, but you must be careful with it anyways.
    #[nid(if cfg!(feature = "psp_660") { 0xFBC2353A }
        else if cfg!(feature = "psp_630") { 0x06372550 }
        else if cfg!(feature = "psp_600") { 0x56198384 }
        else if cfg!(feature = "psp_570") { 0xAC135CA2 }
        else if cfg!(feature = "psp_500") { 0xF937D843 }
        else if cfg!(feature = "psp_420") { 0xC3CC6551 }
        else if cfg!(feature = "psp_395") { 0x5DF89943 }
        else if cfg!(feature = "psp_380") { 0x2686BD23 }
        else { 0x3FB264FC }
    )]
    pub unsafe fn sceKernelRegisterExceptionHandler(
        kind: ExceptionKind, handler: *mut c_void,
    ) -> SceResult<()>;

    /// Registers a exception handler for a exception kind with a priority.
    ///
    /// # Parameters
    ///
    /// - `kind`: The exception kind to be registered.
    /// - `priority`: The priority value to use. 0 to 3.
    /// - `handler` **[[In parameter]]**: The handler to register. It must start with two `nop`
    ///   instructions. More details on the handler ABI on [`ExceptionHandler`] documentation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `handler` must follow the custom ABI for the exception handler. The function does have
    /// some checks, but you must be careful with it anyways.
    #[nid(if cfg!(feature = "psp_660") { 0x7C11759D }
        else if cfg!(feature = "psp_630") { 0x7D995AE8 }
        else if cfg!(feature = "psp_600") { 0x71286CF4 }
        else if cfg!(feature = "psp_570") { 0x37117D98 }
        else if cfg!(feature = "psp_500") { 0x792C424C }
        else if cfg!(feature = "psp_420") { 0x613DF9BA }
        else if cfg!(feature = "psp_395") { 0x49681E68 }
        else if cfg!(feature = "psp_380") { 0x224D8BA0 }
        else { 0x5A837AD4 }
    )]
    pub unsafe fn sceKernelRegisterPriorityExceptionHandler(
        kind: ExceptionKind, priority: u32, handler: *mut c_void,
    ) -> SceResult<()>;

    /// Releases/Unregisters a handler of a exception kind.
    ///
    /// # Parameters
    ///
    /// - `kind`: The exception kind to be released.
    /// - `handler` **[[In parameter]]**: The handler to unregister. It must start with two `nop`
    ///   instructions. More details on the handler ABI on [`ExceptionHandler`] documentation.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `handler` must follow the custom ABI for the exception handler. The function does have
    /// some checks, but you must be careful with it anyways.
    #[nid(if cfg!(feature = "psp_660") { 0x54B0CAD2 }
        else if cfg!(feature = "psp_630") { 0x51763A88 }
        else if cfg!(feature = "psp_600") { 0xC0BC60E9 }
        else if cfg!(feature = "psp_570") { 0xB512DDF4 }
        else if cfg!(feature = "psp_500") { 0xE1F6B00B }
        else if cfg!(feature = "psp_420") { 0x168FA4A9 }
        else if cfg!(feature = "psp_395") { 0x00A80A02 }
        else if cfg!(feature = "psp_380") { 0xBA2F9E68 }
        else { 0x1AA6CFFA }
    )]
    pub unsafe fn sceKernelReleaseExceptionHandler(
        kind: ExceptionKind, handler: *mut c_void,
    ) -> SceResult<()>;


    /// Registers a Non-Maskable Interrupt handler for a Non-Maskable Interrupt kind.
    ///
    /// # Parameters
    ///
    /// - `kind`: The Non-Maskable Interrupt kind to be registered.
    /// - `handler` **[[In parameter]]**: The handler to register.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x15ADC862)]
    pub unsafe fn sceKernelRegisterNmiHandler(kind: u32, handler: NmiHandler) -> SceResult<()>;

    /// Releases/Unregisters a Non-Maskable Interrupt handler for a Non-Maskable Interrupt kind.
    ///
    /// # Parameters
    ///
    /// - `kind`: The Non-Maskable Interrupt kind to be released.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB15357C9)]
    pub unsafe fn sceKernelReleaseNmiHandler(kind: u32) -> SceResult<()>;
}
