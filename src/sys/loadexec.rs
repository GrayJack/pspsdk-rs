//! Load, execution and exit management.
use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{thread::CallbackId, SceResult, SceSize};

/// Options for [`sceKernelLoadExec`].
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceKernelLoadExecParam"))]
pub struct LoadExecOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The size of the arg string.
    pub args_size: SceSize,
    /// A pointer to the arg string.
    pub argp: *mut u8,
    /// A encryption key (?)
    pub key: *const u8,
}

/// Options to configure the `sceKernelVSH*` functions behavior.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceKernelLoadExecVSHParam"))]
pub struct LoadExecVshOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The size of the arg string.
    pub args_size: SceSize,
    /// A pointer to the arg string.
    pub argp: *mut u8,
    /// The key, usually `b"game"`, `b"updater"` or `b"vsh"`.
    pub key: *const u8,
    /// The size of the `vshmain` arguments.
    pub vshmain_args_size: SceSize,
    /// The vshmain arguments that will be passed to vshmain after the program has exited.
    pub vshmain_args: *mut c_void,
    /// The btcnf configuration file.
    ///
    /// It has a maximum of 256 characters.
    ///
    /// Usually `"/kd/pspbtcnf_game.txt"` or `"/kd/pspbtcnf.txt"` if not supplied.
    pub configfile: *mut u8,
    /// An unknown string (max. 256 chars) probably used in 2nd stage of loadexec.
    pub unk: *mut u8,
    /// Unknown flag.
    ///
    /// Default value is `0x10000`.
    pub unk2: u32,
}

#[psp_stub(libname = "LoadExecForUser", flags = 0x4009, use_crate)]
unsafe extern "C" {
    /// Exits the game and reboot back to the XMB/VSH.
    ///
    /// You need to be in a thread in order for this function to work.
    ///
    /// # Return Value
    ///
    /// Returns an error value on error. Otherwise, this functions doesn't return.
    #[nid(0x05572A5F)]
    pub safe fn sceKernelExitGame() -> SceResult<!>;

    /// Exits the game and reboot back to the XMB/VSH with a status value.
    ///
    /// You need to be in a thread in order for this function to work.
    ///
    /// # Parameters
    ///
    /// - `status`: The exit status to use.
    ///
    /// # Return Value
    ///
    /// Returns an error value on error. Otherwise, this functions doesn't return.
    #[nid(0x2AC9954B)]
    pub safe fn sceKernelExitGameWithStatus(status: u32) -> SceResult<!>;

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
    pub safe fn sceKernelRegisterExitCallback(id: CallbackId) -> SceResult<()>;

    /// Load and execute a new executable.
    ///
    /// # Parameters
    ///
    /// - `file_path` **[[In parameter]]**: The file path to the executable.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(0xBD2F1094)]
    pub unsafe fn sceKernelLoadExec(
        file_path: *const u8, options: Option<&LoadExecOptions>,
    ) -> SceResult<!>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "LoadExecForKernel", flags = 0x0009, use_crate)]
unsafe extern "C" {
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
    pub safe fn sceKernelRegisterExitCallback(id: CallbackId) -> SceResult<()>;

    /// Unregisters the exit callback.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x24114598 }
        // else if cfg!(feature = "psp_630") { 0xD9739B89 }
        // else if cfg!(feature = "psp_600") { 0xD9739B89 }
        // else if cfg!(feature = "psp_570") { 0xD9739B89 }
        // else if cfg!(feature = "psp_500") { 0xD9739B89 }
        // else if cfg!(feature = "psp_420") { 0xD9739B89 }
        else if cfg!(feature = "psp_395") { 0x5AF87B62 }
        else if cfg!(feature = "psp_380") { 0xF1C99C38 }
        else { 0xD9739B89 }
    )]
    pub safe fn sceKernelUnregisterExitCallback() -> SceResult<()>;

    /// Invokes the exit callback.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1F08547A }
        // else if cfg!(feature = "psp_630") { 0x62A27008 }
        // else if cfg!(feature = "psp_600") { 0x62A27008 }
        // else if cfg!(feature = "psp_570") { 0x62A27008 }
        // else if cfg!(feature = "psp_500") { 0x62A27008 }
        // else if cfg!(feature = "psp_420") { 0x62A27008 }
        else if cfg!(feature = "psp_395") { 0x2EAA8A5A }
        else if cfg!(feature = "psp_380") { 0x860783BF }
        else { 0x62A27008 }
    )]
    pub safe fn sceKernelInvokeExitCallback() -> SceResult<()>;

    /// Executes a new executable from a buffer.
    ///
    /// # Parameters
    ///
    /// - `buf_size`: The size of the `buf`.
    /// - `buf` **[[In parameter]]**: The buffer with the executable.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.00 [and removed on version 1.52].
    #[cfg(feature = "psp_100")]
    #[nid(0x71A1D738)]
    pub unsafe fn sceKernelLoadExecBufferPlain(
        buf_size: SceSize, buf: *const u8, options: Option<&LoadExecOptions>,
    ) -> SceResult<!>;

    /// Restarts the VSH.
    ///
    /// When called in game mode it will have the same effect that [`sceKernelExitGame`].
    ///
    /// # Parameters
    ///
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(if cfg!(feature = "psp_660") { 0x08F7166C }
        // else if cfg!(feature = "psp_630") { 0xA3D5E142 }
        // else if cfg!(feature = "psp_600") { 0xA3D5E142 }
        // else if cfg!(feature = "psp_570") { 0xA3D5E142 }
        // else if cfg!(feature = "psp_500") { 0xA3D5E142 }
        // else if cfg!(feature = "psp_420") { 0xA3D5E142 }
        else if cfg!(feature = "psp_395") { 0xCA8011A2 }
        else if cfg!(feature = "psp_380") { 0x62879AD8 }
        // else if cfg!(feature = "psp_370") { 0xA3D5E142 }
        else { 0xA3D5E142 }
    )]
    pub safe fn sceKernelExitVSHVSH(options: Option<&LoadExecVshOptions>) -> SceResult<!>;

    /// Restart the VSH (to be used by a kernel module).
    ///
    /// When called in game mode it will have the same effect that [`sceKernelExitGame`].
    ///
    /// # Parameters
    ///
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.52.
    #[nid(if cfg!(feature = "psp_660") { 0xC3474C2A }
        // else if cfg!(feature = "psp_630") { 0x6D302D3D }
        // else if cfg!(feature = "psp_600") { 0x6D302D3D }
        // else if cfg!(feature = "psp_570") { 0x6D302D3D }
        // else if cfg!(feature = "psp_500") { 0x6D302D3D }
        // else if cfg!(feature = "psp_420") { 0x6D302D3D }
        else if cfg!(feature = "psp_395") { 0x63D88393 }
        else if cfg!(feature = "psp_380") { 0xAE5AC5D3 }
        // else if cfg!(feature = "psp_370") { 0x6D302D3D }
        else { 0x6D302D3D }
    )]
    pub safe fn sceKernelExitVSHKernel(options: Option<&LoadExecVshOptions>) -> SceResult<!>;

    /// Executes a executable from a disc.
    ///
    /// It is the function used by the firmware to execute the `EBOOT.BIN` from a UMD disc.
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(if cfg!(feature = "psp_660") { 0xD8320A28 }
        // else if cfg!(feature = "psp_630") { 0x1B97BDB3 }
        // else if cfg!(feature = "psp_600") { 0x1B97BDB3 }
        // else if cfg!(feature = "psp_570") { 0x1B97BDB3 }
        // else if cfg!(feature = "psp_500") { 0x1B97BDB3 }
        // else if cfg!(feature = "psp_420") { 0x1B97BDB3 }
        else if cfg!(feature = "psp_395") { 0x6331FC3A }
        else if cfg!(feature = "psp_380") { 0x234B2B47 }
        // else if cfg!(feature = "psp_370") { 0x1B97BDB3 }
        else { 0x1B97BDB3 }
    )]
    pub unsafe fn sceKernelLoadExecVSHDisc(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a updater executable from a disc.
    ///
    /// It is the function used by the firmware to execute an updater from a disc.
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(if cfg!(feature = "psp_660") { 0xD4B49C4B }
        // else if cfg!(feature = "psp_630") { 0x821BE114 }
        // else if cfg!(feature = "psp_600") { 0x821BE114 }
        // else if cfg!(feature = "psp_570") { 0x821BE114 }
        // else if cfg!(feature = "psp_500") { 0x821BE114 }
        // else if cfg!(feature = "psp_420") { 0x821BE114 }
        else if cfg!(feature = "psp_395") { 0x02320D56 }
        else if cfg!(feature = "psp_380") { 0xAF9AB97F }
        // else if cfg!(feature = "psp_370") { 0x821BE114 }
        else { 0x821BE114 }
    )]
    pub unsafe fn sceKernelLoadExecVSHDiscUpdater(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a debug executable from a disc.
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(if cfg!(feature = "psp_660") { 0x1B305B09 }
        // else if cfg!(feature = "psp_630") { 0x7B7C47EF }
        // else if cfg!(feature = "psp_600") { 0x7B7C47EF }
        // else if cfg!(feature = "psp_570") { 0x7B7C47EF }
        // else if cfg!(feature = "psp_500") { 0x7B7C47EF }
        // else if cfg!(feature = "psp_420") { 0x7B7C47EF }
        else if cfg!(feature = "psp_395") { 0x5522A305 }
        else if cfg!(feature = "psp_380") { 0xF33FF989 }
        // else if cfg!(feature = "psp_370") { 0x7B7C47EF }
        else { 0x7B7C47EF }
    )]
    pub unsafe fn sceKernelLoadExecVSHDiscDebug(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a updater executable from a memory stick.
    ///
    /// It is the function used by the firmware to execute an updater from a memory stick.
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(if cfg!(feature = "psp_660") { 0x4FB44D27 }
        // else if cfg!(feature = "psp_630") { 0x31DF42BF }
        // else if cfg!(feature = "psp_600") { 0x31DF42BF }
        // else if cfg!(feature = "psp_570") { 0x31DF42BF }
        // else if cfg!(feature = "psp_500") { 0x31DF42BF }
        // else if cfg!(feature = "psp_420") { 0x31DF42BF }
        else if cfg!(feature = "psp_395") { 0x4D6C5A67 }
        else if cfg!(feature = "psp_380") { 0xB2F14B53 }
        // else if cfg!(feature = "psp_370") { 0x31DF42BF }
        else { 0x31DF42BF }
    )]
    pub unsafe fn sceKernelLoadExecVSHMs1(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a executable from a memory stick.
    ///
    /// It is the function used by the firmware to execute games (and homebrew :P) from a memory
    /// stick.
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(if cfg!(feature = "psp_660") { 0xD940C83C }
        // else if cfg!(feature = "psp_630") { 0x28D0D249 }
        // else if cfg!(feature = "psp_600") { 0x28D0D249 }
        // else if cfg!(feature = "psp_570") { 0x28D0D249 }
        // else if cfg!(feature = "psp_500") { 0x28D0D249 }
        // else if cfg!(feature = "psp_420") { 0x28D0D249 }
        else if cfg!(feature = "psp_395") { 0x5F2B2E14 }
        else if cfg!(feature = "psp_380") { 0x8D157BC7 }
        // else if cfg!(feature = "psp_370") { 0x28D0D249 }
        else { 0x28D0D249 }
    )]
    pub unsafe fn sceKernelLoadExecVSHMs2(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a executable from a memory stick.
    ///
    /// It is the function used by the firmware to execute ... ?
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    #[nid(if cfg!(feature = "psp_660") { 0xCC6A47D2 }
        // else if cfg!(feature = "psp_630") { 0x70901231 }
        // else if cfg!(feature = "psp_600") { 0x70901231 }
        // else if cfg!(feature = "psp_570") { 0x70901231 }
        // else if cfg!(feature = "psp_500") { 0x70901231 }
        // else if cfg!(feature = "psp_420") { 0x70901231 }
        else if cfg!(feature = "psp_395") { 0x9DA0BBBD }
        else if cfg!(feature = "psp_380") { 0x63E0A104 }
        // else if cfg!(feature = "psp_370") { 0x70901231 }
        else { 0x70901231 }
    )]
    pub unsafe fn sceKernelLoadExecVSHMs3(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a App executable from a memory stick.
    ///
    /// It is the function used by the firmware to execute applications (i.e. Comic Reader, Skype).
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.00.
    #[nid(if cfg!(feature = "psp_660") { 0x00745486 }
        // else if cfg!(feature = "psp_630") { 0x061D9514 }
        // else if cfg!(feature = "psp_600") { 0x061D9514 }
        // else if cfg!(feature = "psp_570") { 0x061D9514 }
        // else if cfg!(feature = "psp_500") { 0x061D9514 }
        // else if cfg!(feature = "psp_420") { 0x061D9514 }
        else if cfg!(feature = "psp_395") { 0x71528FED }
        else if cfg!(feature = "psp_380") { 0xFC8050B0 }
        // else if cfg!(feature = "psp_370") { 0x061D9514 }
        else { 0x061D9514 }
    )]
    pub unsafe fn sceKernelLoadExecVSHMs4(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Executes a PS1 executable from a memory stick.
    ///
    /// It is the function used by the firmware to execute PS1 games.
    ///
    /// # Parameters
    ///
    /// - `file` **[[In parameter]]**: The file to execute.
    /// - `options` **[[In parameter]]**: The options configuring the load-exec behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// On error, an error value is returned, otherwise, this functions doesn't return.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.00.
    #[nid(if cfg!(feature = "psp_660") { 0x7CABED9B }
        // else if cfg!(feature = "psp_630") { 0xB7AB08DA }
        // else if cfg!(feature = "psp_600") { 0xB7AB08DA }
        // else if cfg!(feature = "psp_570") { 0xB7AB08DA }
        // else if cfg!(feature = "psp_500") { 0xB7AB08DA }
        // else if cfg!(feature = "psp_420") { 0xB7AB08DA }
        else if cfg!(feature = "psp_395") { 0x2D00AF8E }
        else if cfg!(feature = "psp_380") { 0x58699192 }
        // else if cfg!(feature = "psp_370") { 0xB7AB08DA }
        else { 0xB7AB08DA }
    )]
    pub unsafe fn sceKernelLoadExecVSHMs5(
        file: *const u8, options: Option<&LoadExecVshOptions>,
    ) -> SceResult<!>;

    /// Gets the current exit callback.
    ///
    /// # Return Value
    ///
    /// Returns the current exit callback ID, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0xB57D0DEC }
        // else if cfg!(feature = "psp_630") { 0x659188E1 }
        // else if cfg!(feature = "psp_600") { 0x659188E1 }
        // else if cfg!(feature = "psp_570") { 0x659188E1 }
        // else if cfg!(feature = "psp_500") { 0x659188E1 }
        // else if cfg!(feature = "psp_420") { 0x659188E1 }
        else if cfg!(feature = "psp_395") { 0x6274D0D5 }
        else if cfg!(feature = "psp_380") { 0x753EF37C }
        // else if cfg!(feature = "psp_370") { 0x659188E1 }
        else { 0x659188E1 }
    )]
    pub safe fn sceKernelCheckExitCallback() -> SceResult<CallbackId>;
}

impl Default for LoadExecOptions {
    fn default() -> Self {
        Self {
            size: size_of::<LoadExecOptions>(),
            args_size: Default::default(),
            argp: Default::default(),
            key: Default::default(),
        }
    }
}

impl Default for LoadExecVshOptions {
    fn default() -> Self {
        Self {
            size: size_of::<LoadExecOptions>(),
            args_size: Default::default(),
            argp: Default::default(),
            key: Default::default(),
            vshmain_args_size: Default::default(),
            vshmain_args: Default::default(),
            configfile: Default::default(),
            unk: Default::default(),
            unk2: 0x10000,
        }
    }
}
