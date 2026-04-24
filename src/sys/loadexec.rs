use pspsdk_macros::psp_stub;

use crate::sys::{thread::CallbackId, SceResult, SceSize};

/// The load and exec options.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceKernelLoadExecParam"))]
pub struct LoadExecOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The size of the arg string.
    pub args: SceSize,
    /// A pointer to the arg string.
    pub argv: *mut u8,
    /// A encryption key (?)
    pub key: *const u8,
}

#[psp_stub(libname = "LoadExecForUser", flags = 0x4001, use_crate)]
extern "C" {
    /// Exits the game and reboot back to the XMB/VSH.
    ///
    /// You need to be in a thread in order for this function to work.
    ///
    /// # Return Values
    ///
    /// Returns an error value on error. Otherwise, this functions doesn't return.
    #[nid(0x05572A5F)]
    pub fn sceKernelExitGame() -> SceResult<()>;

    /// Registers a callback that is executed when "Exit Game" is confirmed in the home button menu.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID to register.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4AC57943)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelRegisterExitCallback(id: CallbackId) -> SceResult<()>;

    /// Load and execute a new executable.
    ///
    /// # Parameters
    ///
    /// - `file_path` **[[In parameter]]**: The file path to the executable.
    /// - `options` **[[In parameter]]**: The options configuring the semaphore behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(0xBD2F1094)]
    pub unsafe fn sceKernelLoadExec(
        file_path: *const u8, options: Option<&LoadExecOptions>,
    ) -> SceResult<()>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "LoadExecForKernel", flags = 0x0009, use_crate)]
extern "C" {
    /// Registers a callback that is executed when "Exit Game" is confirmed in the home button menu.
    ///
    /// # Parameters
    ///
    /// - `id`: The callback UID to register.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1F88A490 }
        // else if cfg!(feature = "psp_630") { 0x4AC57943 }
        // else if cfg!(feature = "psp_600") { 0x4AC57943 }
        // else if cfg!(feature = "psp_570") { 0x4AC57943 }
        // else if cfg!(feature = "psp_500") { 0x4AC57943 }
        // else if cfg!(feature = "psp_420") { 0x4AC57943 }
        else if cfg!(feature = "psp_395") { 0x9CB970B4 }
        else if cfg!(feature = "psp_380") { 0xDB7DF065 }
        else { 0x4AC57943 }
    )]
    pub fn sceKernelRegisterExitCallback(id: CallbackId) -> SceResult<()>;
}
