//! Display operations.
#![allow(unused_imports)]
use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceSize, SceUid};

/// Display output modes.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Default)]
#[doc(alias("PspDisplayMode"))]
pub enum DisplayMode {
    /// LCD output.
    ///
    /// MAX 480x272 at 59.94005995 Hz.
    #[default]
    Lcd    = 0,
    /// VESA 1A (VGA) output. Devkit-only (?).
    ///
    /// MAX 640x480 at 59.94047618 Hz.
    Vesa1a = 0x1A,
    /// Pseudo VGA output. Devkit-only (?).
    ///
    /// MAX 640x480 at 59.94005995 Hz.
    PseudoVga = 0x60,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("DisplayPixelFormat", "PspDisplayPixelFormats"))]
pub enum PixelFormat {
    /// 16-bit RGB 5:6:5.
    Psm5650 = 0,
    /// 16-bit RGBA 5:5:5:1.
    Psm5551 = 1,
    /// 16-bit RGBA 4:4:4:4.
    Psm4444 = 2,
    /// 32-bit RGBA 8:8:8:8.
    Psm8888 = 3,
}

/// Display change sync strategies.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayUpdateSync {
    /// The display buffer change is effective in the next hsync.
    NextHsync = 0,
    /// The display buffer change is effective in the next vsync.
    NextVsync = 1,
}

/// The display mode during a soft reboot (`*LoadExec*` family of functions).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum DisplayHoldMode {
    /// Turns the display black during soft reboot.
    #[default]
    Black = 0,
    /// Holds the current frame during soft reboot.
    CurrentFrame = 1,
}

/// The display state.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum DisplayState {
    /// The display is disabled.
    #[default]
    Disabled = 0,
    /// The display is enabled.
    Enabled = 1,
}


#[psp_stub(libname = "sceDisplay", flags = 0x4001, use_crate)]
unsafe extern "C" {
    /// Sets the display mode.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    /// - `width`: The width of the screen in pixels.
    /// - `height`: The height of the screen in pixels.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0E20F177)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplaySetMode(
        mode: DisplayMode, width: SceSize, height: SceSize,
    ) -> SceResult<()>;

    /// Gets the current display mode and display proportions.
    ///
    /// # Parameters
    ///
    /// - `mode` **[[Out parameter]]**: A reference to receive the display mode information.
    /// - `width` **[[Out parameter]]**: A reference to receive the width of the screen in pixels.
    /// - `height` **[[Out parameter]]**: A reference to receive the height of the screen in pixels.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDEA197D4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayGetMode(
        mode: &mut DisplayMode, width: &mut SceSize, height: &mut SceSize,
    ) -> SceResult<()>;

    /// Gets the number of frames per second in the current display mode.
    ///
    /// # Return Value
    ///
    /// Returns the current number of frames per second.
    #[nid(0xDBA6C4C4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayGetFramePerSec() -> f32;


    /// Sets the display framebuffer.
    ///
    /// # Parameters
    ///
    /// - `buf_top_addr` **[[In parameter]]**: A pointer to the start of the frame buffer. If null,
    ///   turn the display black.
    /// - `buf_width`: The buffer width. It must be a power of 2 value.
    /// - `pixel_format`: The pixel format of the framebuffer.
    /// - `sync`: The update sync mode of the framebuffer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x289D82FE)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceDisplaySetFrameBuf(
        buf_top_addr: *const u8, buf_width: SceSize, pixel_format: PixelFormat,
        sync: DisplayUpdateSync,
    ) -> SceResult<()>;

    /// Gets the display framebuffer.
    ///
    /// # Parameters
    ///
    /// - `buf_top_addr` **[[Out parameter]]**: A pointer to receive the top address of the
    ///   framebuffer.
    /// - `buf_width` **[[Out parameter]]**: A pointer to receive the buffer width.
    /// - `pixel_format` **[[Out parameter]]**: A pointer to receive the pixel format of the
    ///   framebuffer.
    /// - `sync` **[[Out parameter]]**: A pointer to receive the update sync mode of the
    ///   framebuffer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEEDA2E54)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceDisplayGetFrameBuf(
        buf_top_addr: *mut *const u8, buf_width: &mut SceSize, pixel_format: &mut PixelFormat,
        sync: &mut DisplayUpdateSync,
    ) -> SceResult<()>;

    /// Gets if the framebuffer is current being displayed.
    ///
    /// # Return Value
    ///
    /// Returns `true` if it is being displayed, false otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xB4F378FA)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayIsForeground() -> bool;

    /// Sets the hold mode.
    ///
    /// The hold mode affects how the display behaves during a soft reboot.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7ED59BC4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplaySetHoldMode(mode: DisplayHoldMode) -> SceResult<()>;

    /// Gets the number of vertical blank pulses up to now
    ///
    /// # Return Value
    ///
    /// Returns the number of vertical blank pulses on success, error value otherwise.
    #[nid(0x9C6EAAD7)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayGetVcount() -> SceResult<u32>;

    /// Checks whether VBLANK is active in process.
    ///
    /// # Return Value
    ///
    /// Returns `true` if VBLANK is in process, `false` if it is not, error value otherwise.
    #[nid(0x4D4E10EC)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayIsVblank() -> SceResult<bool>;

    /// Makes the calling thread to enter in a [`Wait`] state until VBLANK is in process.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(0x36CDFADE)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayWaitVblank() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until VBLANK is in process, but
    /// service any callbacks as necessary.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(0x8EB9EC49)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayWaitVblankCB() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the **next**
    /// VBLANK.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(0x984C27E7)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayWaitVblankStart() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the **next**
    /// VBLANK, but service any callbacks as necessary.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(0x46F186C3)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayWaitVblankStartCB() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the next
    /// VBLANK after a number of VSYNC cycles.
    ///
    /// # Parameters
    ///
    /// - `vsync_cycles`: The number of VSYNC cycles to wait.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.00.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(0x40F1469C)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayWaitVblankStartMulti(vsync_cycles: u32) -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the next
    /// VBLANK after a number of VSYNC cycles, but service any callbacks as necessary.
    ///
    /// # Parameters
    ///
    /// - `vsync_cycles`: The number of VSYNC cycles to wait.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.00.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(0x77ED8B3A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayWaitVblankStartMultiCB(vsync_cycles: u32) -> SceResult<()>;

    /// Gets the current HSYNC count.
    ///
    /// # Return Value
    ///
    /// Returns the current HSYNC count on success, error value otherwise.
    #[nid(0x773DD3A3)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayGetCurrentHcount() -> SceResult<u32>;

    /// Gets the accumulated HSYNC count.
    ///
    /// # Return Value
    ///
    /// Returns the accumulated HSYNC count on success, error value otherwise.
    #[nid(0x210EAB3A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayGetAccumulatedHcount() -> SceResult<u32>;

    /// Gets the current brightness level set.
    ///
    /// # Parameters
    ///
    /// - `level`: A reference to receive the brightness level.
    /// - `mode`: A reference to receive the brightness mode (?). Values `0` or `1`, exactly meaning
    ///   unknown. Always returns `0` on `display_01g.prx`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x31C4BAA8)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceDisplayGetBrightness(level: &mut u32, mode: &mut u32) -> SceResult<()>;
}

// FIXME: Add missing
//
// - `sceDisplaySetFrameBufferInternal` (not real cracked name)
// - `sceDisplayGetFrameBufferInternal` (not real cracked name)
#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceDisplay_driver", flags = 0x0001, use_crate)]
unsafe extern "C" {
    /// Sets the display mode.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    /// - `width`: The width of the screen in pixels.
    /// - `height`: The height of the screen in pixels.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0E20F177)]
    pub safe fn sceDisplaySetMode(
        mode: DisplayMode, width: SceSize, height: SceSize,
    ) -> SceResult<()>;

    /// Gets the current display mode and display proportions.
    ///
    /// # Parameters
    ///
    /// - `mode` **[[Out parameter]]**: A reference to receive the display mode information.
    /// - `width` **[[Out parameter]]**: A reference to receive the width of the screen in pixels.
    /// - `height` **[[Out parameter]]**: A reference to receive the height of the screen in pixels.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDEA197D4)]
    pub safe fn sceDisplayGetMode(
        mode: &mut DisplayMode, width: &mut SceSize, height: &mut SceSize,
    ) -> SceResult<()>;

    /// Gets the number of frames per second in the current display mode.
    ///
    /// # Return Value
    ///
    /// Returns the current number of frames per second.
    #[nid(if cfg!(feature = "psp_660") { 0x1EAA0BDC }
        // else if cfg!(feature = "psp_630") { 0xDBA6C4C4 }
        // else if cfg!(feature = "psp_600") { 0xDBA6C4C4 }
        // else if cfg!(feature = "psp_570") { 0xDBA6C4C4 }
        // else if cfg!(feature = "psp_500") { 0xDBA6C4C4 }
        // else if cfg!(feature = "psp_420") { 0xDBA6C4C4 }
        else if cfg!(feature = "psp_395") { 0x4D1414AF }
        else if cfg!(feature = "psp_380") { 0x1EF4432A }
        else if cfg!(feature = "psp_370") { 0x13AA96B7 }
        else { 0xDBA6C4C4 }
    )]
    pub safe fn sceDisplayGetFramePerSec() -> f32;


    /// Sets the display framebuffer.
    ///
    /// # Parameters
    ///
    /// - `buf_top_addr` **[[In parameter]]**: A pointer to the start of the frame buffer. If null,
    ///   turn the display black.
    /// - `buf_width`: The buffer width. It must be a power of 2 value.
    /// - `pixel_format`: The pixel format of the framebuffer.
    /// - `sync`: The update sync mode of the framebuffer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xA38B3F89 }
        // else if cfg!(feature = "psp_630") { 0x289D82FE }
        // else if cfg!(feature = "psp_600") { 0x289D82FE }
        // else if cfg!(feature = "psp_570") { 0x289D82FE }
        // else if cfg!(feature = "psp_500") { 0x289D82FE }
        // else if cfg!(feature = "psp_420") { 0x289D82FE }
        else if cfg!(feature = "psp_395") { 0xC28EFAA7 }
        else if cfg!(feature = "psp_380") { 0x3749CDA0 }
        else if cfg!(feature = "psp_370") { 0x4AB7497F }
        else { 0x289D82FE }
    )]
    pub unsafe fn sceDisplaySetFrameBuf(
        buf_top_addr: *const u8, buf_width: SceSize, pixel_format: PixelFormat,
        sync: DisplayUpdateSync,
    ) -> SceResult<()>;

    /// Gets the display framebuffer.
    ///
    /// # Parameters
    ///
    /// - `buf_top_addr` **[[Out parameter]]**: A pointer to receive the top address of the
    ///   framebuffer.
    /// - `buf_width` **[[Out parameter]]**: A pointer to receive the buffer width.
    /// - `pixel_format` **[[Out parameter]]**: A pointer to receive the pixel format of the
    ///   framebuffer.
    /// - `sync` **[[Out parameter]]**: A pointer to receive the update sync mode of the
    ///   framebuffer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xFBB369FD }
        // else if cfg!(feature = "psp_630") { 0xEEDA2E54 }
        // else if cfg!(feature = "psp_600") { 0xEEDA2E54 }
        // else if cfg!(feature = "psp_570") { 0xEEDA2E54 }
        // else if cfg!(feature = "psp_500") { 0xEEDA2E54 }
        // else if cfg!(feature = "psp_420") { 0xEEDA2E54 }
        else if cfg!(feature = "psp_395") { 0xCFB91094 }
        else if cfg!(feature = "psp_380") { 0x3E954D17 }
        else if cfg!(feature = "psp_370") { 0xE56B11BA }
        else { 0xEEDA2E54 }
    )]
    pub unsafe fn sceDisplayGetFrameBuf(
        buf_top_addr: *mut *const u8, buf_width: &mut SceSize, pixel_format: &mut PixelFormat,
        sync: &mut DisplayUpdateSync,
    ) -> SceResult<()>;

    /// Gets if the framebuffer is current being displayed.
    ///
    /// # Return Value
    ///
    /// Returns `true` if it is being displayed, false otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x99E358F1 }
        // else if cfg!(feature = "psp_630") { 0xB4F378FA }
        // else if cfg!(feature = "psp_600") { 0xB4F378FA }
        // else if cfg!(feature = "psp_570") { 0xB4F378FA }
        // else if cfg!(feature = "psp_500") { 0xB4F378FA }
        // else if cfg!(feature = "psp_420") { 0xB4F378FA }
        else if cfg!(feature = "psp_395") { 0x2D634972 }
        else if cfg!(feature = "psp_380") { 0x6668FCDF }
        else if cfg!(feature = "psp_370") { 0x044FF282 }
        else { 0xB4F378FA }
    )]
    pub safe fn sceDisplayIsForeground() -> bool;

    /// Sets the hold mode.
    ///
    /// The hold mode affects how the display behaves during a soft reboot.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x3552AB11 }
        // else if cfg!(feature = "psp_630") { 0x7ED59BC4 }
        // else if cfg!(feature = "psp_600") { 0x7ED59BC4 }
        // else if cfg!(feature = "psp_570") { 0x7ED59BC4 }
        // else if cfg!(feature = "psp_500") { 0x7ED59BC4 }
        // else if cfg!(feature = "psp_420") { 0x7ED59BC4 }
        else if cfg!(feature = "psp_395") { 0x2E6AA0AB }
        else if cfg!(feature = "psp_380") { 0xB0D487B3 }
        else if cfg!(feature = "psp_370") { 0x906281D5 }
        else { 0x7ED59BC4 }
    )]
    pub safe fn sceDisplaySetHoldMode(mode: DisplayHoldMode) -> SceResult<()>;

    /// Gets the number of vertical blank pulses up to now
    ///
    /// # Return Value
    ///
    /// Returns the number of vertical blank pulses on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1FBE8856 }
        // else if cfg!(feature = "psp_630") { 0x9C6EAAD7 }
        // else if cfg!(feature = "psp_600") { 0x9C6EAAD7 }
        // else if cfg!(feature = "psp_570") { 0x9C6EAAD7 }
        // else if cfg!(feature = "psp_500") { 0x9C6EAAD7 }
        // else if cfg!(feature = "psp_420") { 0x9C6EAAD7 }
        else if cfg!(feature = "psp_395") { 0x6E57C773 }
        else if cfg!(feature = "psp_380") { 0xF5EEEFEF }
        else if cfg!(feature = "psp_370") { 0xE8466BC2 }
        else { 0x9C6EAAD7 }
    )]
    pub safe fn sceDisplayGetVcount() -> SceResult<u32>;

    /// Checks whether VBLANK is active in process.
    ///
    /// # Return Value
    ///
    /// Returns `true` if VBLANK is in process, `false` if it is not, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x572D7804 }
        // else if cfg!(feature = "psp_630") { 0x4D4E10EC }
        // else if cfg!(feature = "psp_600") { 0x4D4E10EC }
        // else if cfg!(feature = "psp_570") { 0x4D4E10EC }
        // else if cfg!(feature = "psp_500") { 0x4D4E10EC }
        // else if cfg!(feature = "psp_420") { 0x4D4E10EC }
        else if cfg!(feature = "psp_395") { 0x93CA8A9B }
        else if cfg!(feature = "psp_380") { 0x93596B96 }
        else if cfg!(feature = "psp_370") { 0x8BE19BF8 }
        else { 0x4D4E10EC }
    )]
    pub safe fn sceDisplayIsVblank() -> SceResult<bool>;

    /// Makes the calling thread to enter in a [`Wait`] state until VBLANK is in process.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(if cfg!(feature = "psp_660") { 0xFE5884EF }
        // else if cfg!(feature = "psp_630") { 0x36CDFADE }
        // else if cfg!(feature = "psp_600") { 0x36CDFADE }
        // else if cfg!(feature = "psp_570") { 0x36CDFADE }
        // else if cfg!(feature = "psp_500") { 0x36CDFADE }
        // else if cfg!(feature = "psp_420") { 0x36CDFADE }
        else if cfg!(feature = "psp_395") { 0xC922270C }
        else if cfg!(feature = "psp_380") { 0xC89E1F1D }
        else if cfg!(feature = "psp_370") { 0x7FBA941A }
        else { 0x36CDFADE }
    )]
    pub safe fn sceDisplayWaitVblank() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until VBLANK is in process, but
    /// service any callbacks as necessary.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(if cfg!(feature = "psp_660") { 0x58E8680E }
        // else if cfg!(feature = "psp_630") { 0x8EB9EC49 }
        // else if cfg!(feature = "psp_600") { 0x8EB9EC49 }
        // else if cfg!(feature = "psp_570") { 0x8EB9EC49 }
        // else if cfg!(feature = "psp_500") { 0x8EB9EC49 }
        // else if cfg!(feature = "psp_420") { 0x8EB9EC49 }
        else if cfg!(feature = "psp_395") { 0x9C67EA53 }
        else if cfg!(feature = "psp_380") { 0x0C92A897 }
        else if cfg!(feature = "psp_370") { 0x16FD49DC }
        else { 0x8EB9EC49 }
    )]
    pub safe fn sceDisplayWaitVblankCB() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the **next**
    /// VBLANK.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(if cfg!(feature = "psp_660") { 0xB0942511 }
        // else if cfg!(feature = "psp_630") { 0x984C27E7 }
        // else if cfg!(feature = "psp_600") { 0x984C27E7 }
        // else if cfg!(feature = "psp_570") { 0x984C27E7 }
        // else if cfg!(feature = "psp_500") { 0x984C27E7 }
        // else if cfg!(feature = "psp_420") { 0x984C27E7 }
        else if cfg!(feature = "psp_395") { 0x9C268F30 }
        else if cfg!(feature = "psp_380") { 0x3A730F7F }
        else if cfg!(feature = "psp_370") { 0xB685BA36 }
        else { 0x984C27E7 }
    )]
    pub safe fn sceDisplayWaitVblankStart() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the **next**
    /// VBLANK, but service any callbacks as necessary.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(if cfg!(feature = "psp_660") { 0xE38CA615 }
        // else if cfg!(feature = "psp_630") { 0x46F186C3 }
        // else if cfg!(feature = "psp_600") { 0x46F186C3 }
        // else if cfg!(feature = "psp_570") { 0x46F186C3 }
        // else if cfg!(feature = "psp_500") { 0x46F186C3 }
        // else if cfg!(feature = "psp_420") { 0x46F186C3 }
        else if cfg!(feature = "psp_395") { 0x661CB78C }
        else if cfg!(feature = "psp_380") { 0x1B0D8989 }
        else if cfg!(feature = "psp_370") { 0x792E8018 }
        else { 0x46F186C3 }
    )]
    pub safe fn sceDisplayWaitVblankStartCB() -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the next
    /// VBLANK after a number of VSYNC cycles.
    ///
    /// # Parameters
    ///
    /// - `vsync_cycles`: The number of VSYNC cycles to wait.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.00.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(if cfg!(feature = "psp_660") { 0xA70066A1 }
        // else if cfg!(feature = "psp_630") { 0x40F1469C }
        // else if cfg!(feature = "psp_600") { 0x40F1469C }
        // else if cfg!(feature = "psp_570") { 0x40F1469C }
        else { 0x40F1469C }
    )]
    pub safe fn sceDisplayWaitVblankStartMulti(vsync_cycles: u32) -> SceResult<()>;

    /// Makes the calling thread to enter in a [`Wait`] state until the the start of the next
    /// VBLANK after a number of VSYNC cycles, but service any callbacks as necessary.
    ///
    /// # Parameters
    ///
    /// - `vsync_cycles`: The number of VSYNC cycles to wait.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.00.
    ///
    /// [`Wait`]: crate::sys::thread::ThreadState::Wait
    #[nid(if cfg!(feature = "psp_660") { 0x113958AE }
        // else if cfg!(feature = "psp_630") { 0x77ED8B3A }
        // else if cfg!(feature = "psp_600") { 0x77ED8B3A }
        // else if cfg!(feature = "psp_570") { 0x77ED8B3A }
        else { 0x77ED8B3A }
    )]
    pub safe fn sceDisplayWaitVblankStartMultiCB(vsync_cycles: u32) -> SceResult<()>;

    /// Gets the current HSYNC count.
    ///
    /// # Return Value
    ///
    /// Returns the current HSYNC count on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xEC80D435 }
        // else if cfg!(feature = "psp_630") { 0x773DD3A3 }
        // else if cfg!(feature = "psp_600") { 0x773DD3A3 }
        // else if cfg!(feature = "psp_570") { 0x773DD3A3 }
        // else if cfg!(feature = "psp_500") { 0x773DD3A3 }
        // else if cfg!(feature = "psp_420") { 0x773DD3A3 }
        else if cfg!(feature = "psp_395") { 0xDB559C60 }
        else if cfg!(feature = "psp_380") { 0x51CE9C76 }
        else if cfg!(feature = "psp_370") { 0x689C0CC2 }
        else { 0x773DD3A3 }
    )]
    pub safe fn sceDisplayGetCurrentHcount() -> SceResult<u32>;

    /// Gets the accumulated HSYNC count.
    ///
    /// # Return Value
    ///
    /// Returns the accumulated HSYNC count on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xF84D16CC }
        // else if cfg!(feature = "psp_630") { 0x210EAB3A }
        // else if cfg!(feature = "psp_600") { 0x210EAB3A }
        // else if cfg!(feature = "psp_570") { 0x210EAB3A }
        // else if cfg!(feature = "psp_500") { 0x210EAB3A }
        // else if cfg!(feature = "psp_420") { 0x210EAB3A }
        else if cfg!(feature = "psp_395") { 0xCAC9C43D }
        else if cfg!(feature = "psp_380") { 0x34662DEA }
        else if cfg!(feature = "psp_370") { 0xA99B94A3 }
        else { 0x210EAB3A }
    )]
    pub safe fn sceDisplayGetAccumulatedHcount() -> SceResult<u32>;

    /// Sets the display brightness to a particular level.
    ///
    /// # Parameters
    ///
    /// - `level`: The brightness level to set. `0-100`.
    /// - `mode`: The brightness mode (?). Values `0` or `1`, exactly meaning unknown. Not used on
    ///   `display_01g.prx`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x60112E07 }
        // else if cfg!(feature = "psp_630") { 0x9E3C6DC6 }
        // else if cfg!(feature = "psp_600") { 0x9E3C6DC6 }
        // else if cfg!(feature = "psp_570") { 0x9E3C6DC6 }
        // else if cfg!(feature = "psp_500") { 0x9E3C6DC6 }
        // else if cfg!(feature = "psp_420") { 0x9E3C6DC6 }
        else if cfg!(feature = "psp_395") { 0x1380A62E }
        else if cfg!(feature = "psp_380") { 0x267BF9F7 }
        else if cfg!(feature = "psp_370") { 0x776ADFDB }
        else { 0x9E3C6DC6 }
    )]
    pub safe fn sceDisplaySetBrightness(level: u32, mode: u32) -> SceResult<()>;

    /// Gets the current brightness level set.
    ///
    /// # Parameters
    ///
    /// - `level` **[[Out parameter]]**: A reference to receive the brightness level.
    /// - `mode` **[[Out parameter]]**: A reference to receive the brightness mode (?). Values `0`
    ///   or `1`, exactly meaning unknown. Always returns `0` on `display_01g.prx`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x0043973F }
        // else if cfg!(feature = "psp_630") { 0x31C4BAA8 }
        // else if cfg!(feature = "psp_600") { 0x31C4BAA8 }
        // else if cfg!(feature = "psp_570") { 0x31C4BAA8 }
        // else if cfg!(feature = "psp_500") { 0x31C4BAA8 }
        // else if cfg!(feature = "psp_420") { 0x31C4BAA8 }
        else if cfg!(feature = "psp_395") { 0xB0B1C399 }
        else if cfg!(feature = "psp_380") { 0x14C854AA }
        else if cfg!(feature = "psp_370") { 0x1CB8CB47 }
        else { 0x31C4BAA8 }
    )]
    pub safe fn sceDisplayGetBrightness(level: &mut u32, mode: &mut u32) -> SceResult<()>;

    /// Sets the display backlight brightness to a particular level.
    ///
    /// # Parameters
    ///
    /// - `level`: The brightness level to set. `0-100`.
    /// - `mode`: The brightness mode (?). Values `0` or `1`, exactly meaning unknown. Not used on
    ///   `display_01g.prx`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[nid(0xE55F0D50)]
    pub safe fn sceDisplaySetBacklightSel(level: u32, mode: u32) -> SceResult<()>;

    /// Gets the display backlight brightness.
    ///
    /// # Parameters
    ///
    /// - `level` **[[Out parameter]]**: A reference to receive the backlight brightness level.
    /// - `mode` **[[Out parameter]]**: A reference to receive the backlight brightness mode (?).
    ///   Values `0` or `1`, exactly meaning unknown. Always returns `0` on `display_01g.prx`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x96CFAC38)]
    pub safe fn sceDisplayGetBacklightSel(level: &mut u32, mode: &mut u32) -> SceResult<()>;

    /// Initializes the `sceDisplay*` library.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    /// This function should be called only once by the system.
    #[nid(0x206276C2)]
    pub unsafe fn sceDisplayInit() -> SceResult<()>;

    /// Deinitilizes the `sceDisplay*` library.
    #[nid(0x7A10289D)]
    pub unsafe fn sceDisplayEnd();

    /// Disables the display.
    ///
    /// # Return Value
    ///
    /// Returns the previous state of the display.
    #[nid(if cfg!(feature = "psp_660") { 0x33B620AF }
        // else if cfg!(feature = "psp_630") { 0x681EE6A7 }
        // else if cfg!(feature = "psp_600") { 0x681EE6A7 }
        // else if cfg!(feature = "psp_570") { 0x681EE6A7 }
        // else if cfg!(feature = "psp_500") { 0x681EE6A7 }
        // else if cfg!(feature = "psp_420") { 0x681EE6A7 }
        else if cfg!(feature = "psp_395") { 0xDB98F049 }
        else if cfg!(feature = "psp_380") { 0x1A4E0C25 }
        else if cfg!(feature = "psp_370") { 0x32B67781 }
        else { 0x681EE6A7 }
    )]
    pub safe fn sceDisplayDisable() -> DisplayState;

    /// Enables the display.
    ///
    /// # Return Value
    ///
    /// Returns the previous state of the display.
    #[nid(if cfg!(feature = "psp_660") { 0x117C3E2C }
        // else if cfg!(feature = "psp_630") { 0x432D133F }
        // else if cfg!(feature = "psp_600") { 0x432D133F }
        // else if cfg!(feature = "psp_570") { 0x432D133F }
        // else if cfg!(feature = "psp_500") { 0x432D133F }
        // else if cfg!(feature = "psp_420") { 0x432D133F }
        else if cfg!(feature = "psp_395") { 0xEB6C2BA3 }
        else if cfg!(feature = "psp_380") { 0x7E67BFCF }
        else if cfg!(feature = "psp_370") { 0x946155FD }
        else { 0x432D133F }
    )]
    pub safe fn sceDisplayEnable() -> DisplayState;
}
