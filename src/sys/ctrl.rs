use core::ffi::c_void;

use bitflag_attr::bitflag;

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceResultOk, SceSize};

/// The callback function used by [`sceCtrlSetSpecialButtonCallback`].
///
/// # Parameters
///
/// - `curr_buttons`: The current buttons.
/// - `last_buttons`: The last buttons.
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with this function.
#[doc(alias("SceKernelButtonCallbackFunction"))]
pub type SpecialButtonCallback =
    unsafe extern "C" fn(curr_buttons: PadButtons, last_buttons: PadButtons, common: *mut c_void);

/// A valid sampling cycle value in microseconds.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SamplingCycle(u32);

/// A valid idle cancel threshold value.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdleCancelThreshold(i32);

/// The slot for a rapid-fire setup.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RapidFireSlot(pattern_type!(u32 is 0..=15));

/// The slot for a special button callback setup.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SpecialButtonCallbackSlot(pattern_type!(u8 is 0..=3));

/// The slot for a button emulation setup.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ButtonEmulationSlot(pattern_type!(u8 is 0..=3));

/// Controller input sampling mode.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
#[doc(alias("SceCtrlPadInputMode", "PspCtrlMode"))]
pub enum InputMode {
    /// Only digital data is included.
    #[doc(alias("PSP_CTRL_MODE_DIGITAL", "SCE_CTRL_INPUT_DIGITAL_ONLY"))]
    DigitalOnly = 0x0,
    /// Digital and analog data is considered.
    #[default]
    #[doc(alias("PSP_CTRL_MODE_ANALOG", "SCE_CTRL_INPUT_DIGITAL_ANALOG"))]
    DigitalAndAnalog = 0x1,
}

/// Flags representing digital controller button.
///
/// Each flag corresponds to a different button and can be used to extract button states from
/// [`CtrlData`] and [`LatchData`] structures. Flags can be combined using bitwise OR operation to
/// check for multiple key states at once.
///
/// ## Note
/// The following buttons can only be read at a kernel-level: [`PadButtons::Home`],
/// [`PadButtons::WlanUp`], [`PadButtons::Remote`], [`PadButtons::VolumeUp`],
/// [`PadButtons::VolumeDown`], [`PadButtons::Screen`], [`PadButtons::Note`],
/// [`PadButtons::Disc`], and [`PadButtons::MemoryStick`]
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
#[doc(alias("CtrlButtons", "SceCtrlPadButtons", "PspCtrlButtons"))]
pub enum PadButtons {
    /// Select button.
    #[doc(alias("SCE_CTRL_SELECT", "PSP_CTRL_SELECT"))]
    Select = 0x00000001,
    /// L3 button.
    #[doc(alias("SCE_CTRL_L3", "PSP_CTRL_L3"))]
    L3     = 0x00000002,
    /// R3 button.
    #[doc(alias("SCE_CTRL_R3", "PSP_CTRL_R3"))]
    R3     = 0x00000004,
    /// Start button.
    #[doc(alias("SCE_CTRL_START", "PSP_CTRL_START"))]
    Start  = 0x00000008,
    /// Up D-Pad button.
    #[doc(alias("SCE_CTRL_UP", "PSP_CTRL_UP"))]
    Up     = 0x00000010,
    /// Right D-Pad button.
    #[doc(alias("SCE_CTRL_RIGHT", "PSP_CTRL_RIGHT"))]
    Right  = 0x00000020,
    /// Down D-Pad button.
    #[doc(alias("SCE_CTRL_DOWN", "PSP_CTRL_DOWN"))]
    Down   = 0x00000040,
    /// Left D-Pad button.
    #[doc(alias("SCE_CTRL_LEFT", "PSP_CTRL_LEFT"))]
    Left   = 0x00000080,
    /// Left trigger.
    ///
    /// This accounts for the L2 trigger as well.
    #[doc(alias("SCE_CTRL_LTRIGGER", "PSP_CTRL_LTRIGGER"))]
    LTrigger = 0x00000100,
    /// Right trigger.
    ///
    /// This accounts for the R2 trigger as well.
    #[doc(alias("SCE_CTRL_RTRIGGER", "PSP_CTRL_RTRIGGER"))]
    RTrigger = 0x00000200,
    /// L1 trigger.
    #[doc(alias("SCE_CTRL_L1TRIGGER", "PSP_CTRL_L1"))]
    L1Trigger = 0x00000400,
    /// R1 trigger.
    #[doc(alias("SCE_CTRL_R1TRIGGER", "PSP_CTRL_R1"))]
    R1Trigger = 0x00000800,
    /// L2 trigger.
    #[doc(alias("PSP_CTRL_L2"))]
    L2Trigger = LTrigger,
    /// R2 trigger.
    #[doc(alias("PSP_CTRL_R2"))]
    R2Trigger = RTrigger,
    /// Triangle button.
    #[doc(alias("SCE_CTRL_TRIANGLE", "PSP_CTRL_TRIANGLE"))]
    Triangle = 0x00001000,
    /// Circle button.
    #[doc(alias("SCE_CTRL_CIRCLE", "PSP_CTRL_CIRCLE"))]
    Circle = 0x00002000,
    /// Cross button.
    #[doc(alias("SCE_CTRL_CROSS", "PSP_CTRL_CROSS"))]
    Cross  = 0x00004000,
    /// Square button.
    #[doc(alias("SCE_CTRL_SQUARE", "PSP_CTRL_SQUARE"))]
    Square = 0x00008000,
    /// If this bit is set, then controller input is being intercepted by the
    /// system software or another application.  For example, this is the case
    /// when the PSP's HOME menu or the exit dialog is being shown.
    #[doc(alias("SCE_CTRL_INTERCEPTED"))]
    Intercepted = 0x00010000,
    /// Home button.
    #[doc(alias("PSP_CTRL_HOME"))]
    Home   = Intercepted,
    /// Hold button.
    #[doc(alias("SCE_CTRL_HOLD", "PSP_CTRL_HOLD"))]
    Hold   = 0x00020000,
    /// W-LAN switch up.
    #[doc(alias("SCE_CTRL_WLAN_UP", "PSP_CTRL_WLAN_UP"))]
    WlanUp = 0x00040000,
    /// Remote hold position.
    #[doc(alias("SCE_CTRL_REMOTE", "PSP_CTRL_REMOTE"))]
    Remote = 0x00080000,
    /// Volume up button.
    #[doc(alias("SCE_CTRL_VOLUP", "PSP_CTRL_VOLUP"))]
    VolumeUp = 0x00100000,
    /// Volume down button.
    #[doc(alias("SCE_CTRL_VOLDOWN", "PSP_CTRL_VOLDOWN"))]
    VolumeDown = 0x00200000,
    /// Screen button.
    #[doc(alias("SCE_CTRL_SCREEN", "PSP_CTRL_SCREEN"))]
    Screen = 0x00400000,
    /// Music Note button.
    #[doc(alias("SCE_CTRL_NOTE", "PSP_CTRL_NOTE"))]
    Note   = 0x00800000,
    /// Disc is present.
    #[doc(alias("SCE_CTRL_DISC", "PSP_CTRL_DISC"))]
    Disc   = 0x01000000,
    /// Memory Stick is present.
    #[doc(alias("SCE_CTRL_MS", "PSP_CTRL_MS"))]
    MemoryStick = 0x02000000,
    Unk10000000 = 0x10000000,
    Unk20000000 = 0x20000000,
}

/// Structure used to obtain input data from the controller.
///
/// An extended variant exist that also gather more data related to wireless controller:
/// [`CtrlDataExt`]
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
#[doc(alias("SceCtrlData"))]
pub struct CtrlData {
    /// The time stamp of the time during which sampling was performed in microseconds.
    ///
    /// It can be used to get the time period of a button pressing event.
    pub timestamp: u32,
    /// The currently pressed button.
    pub buttons: PadButtons,
    /// The left analog stick X-axis offset (`[0, 0xFF]`. Left = `0`, Right = `0xFF`).
    pub l_x: u8,
    /// The left analog stick y-axis offset (`[0, 0xFF]`. Up = `0`, Down = `0xFF`).
    pub l_y: u8,
    /// The right analog stick X-axis offset (`[0, 0xFF]`. Left = `0`, Right = `0xFF`).
    ///
    /// This is valid when using DualShock 3 on PSP GO, a PS VITA system, through hardware/software
    /// hacking, or system emulation.
    pub r_x: u8,
    /// The right analog stick y-axis offset (`[0, 0xFF]`. Up = `0`, Down = `0xFF`).
    ///
    /// This is valid when using DualShock 3 on PSP GO, a PS VITA system, through hardware/software
    /// hacking, or system emulation.
    pub r_y: u8,
    /// Reserved bytes unused by the PSP firmware.
    pub reserved: [u8; 4],
}

/// Structure used to obtain input extended data from the controller.
///
/// In addition to PSP controller state it can contain input state of external input devices such as
/// a wireless controller.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
#[doc(alias("SceCtrlData2"))]
pub struct CtrlDataExt {
    /// The time stamp of the time during which sampling was performed in microseconds.
    ///
    /// It can be used to get the time period of a button pressing event.
    pub timestamp: u32,
    /// The currently pressed button.
    pub buttons: PadButtons,
    /// The left analog stick X-axis offset (`[0, 0xFF]`. Left = `0`, Right = `0xFF`).
    pub l_x: u8,
    /// The left analog stick y-axis offset (`[0, 0xFF]`. Up = `0`, Down = `0xFF`).
    pub l_y: u8,
    /// The right analog stick X-axis offset (`[0, 0xFF]`. Left = `0`, Right = `0xFF`).
    ///
    /// This is valid when using DualShock 3 on PSP GO, a PS VITA system, through hardware/software
    /// hacking, or system emulation.
    pub r_x: u8,
    /// The right analog stick y-axis offset (`[0, 0xFF]`. Up = `0`, Down = `0xFF`).
    ///
    /// This is valid when using DualShock 3 on PSP GO, a PS VITA system, through hardware/software
    /// hacking, or system emulation.
    pub r_y: u8,
    /// Reserved bytes unused by the PSP firmware.
    pub reserved: [u8; 4],
    /// D-Pad Left button pressure sensitivity.
    pub dpad_left_sense: u16,
    /// D-Pad Right button pressure sensitivity.
    pub dpad_right_sense: u16,
    /// D-Pad Up button pressure sensitivity.
    pub dpad_up_sense: u16,
    /// D-Pad Down button pressure sensitivity.
    pub dpad_down_sense: u16,
    /// Triangle button pressure sensitivity.
    pub triangle_sense: u16,
    /// Circle button pressure sensitivity.
    pub circle_sense: u16,
    /// Cross button pressure sensitivity.
    pub cross_sense: u16,
    /// Square button pressure sensitivity.
    pub square_sense: u16,
    /// L1 pressure sensitivity.
    pub l1_sense: u16,
    /// R1 pressure sensitivity.
    pub r1_sense: u16,
    /// L2 pressure sensitivity.
    pub l2_sense: u16,
    /// R2 pressure sensitivity.
    pub r2_sense: u16,
    /// DS3 sixaxis. The return value for tilting the x-axis.
    pub tilt_x: i32,
    /// DS3 sixaxis. The return value for tilting the y-axis.
    pub tilt_y: i32,
}

/// Represents a controller latch data.
///
/// With each sampling cycle, the controller service compares the new pressed & released button
/// states with the previously collected pressed button states. This comparison will result in the
/// following possible states for each button:
///
/// - `make`: The button has just been pressed with its prior state being the `released` state.
///   Transition from `released` state to `pressed` state.
/// - `press`: The button is currently in the `pressed` state.
/// - `break` (named `breakk` due to Rust keywords): The button has just been released with its
///   prior state being the `pressed` state. Transition from `pressed` state to `release` state.
/// - `release`: The button is currently in the `released` state.
///
/// It is possible for a button to (briefly) be in two states at the same time. Valid combinations
/// are as follows:
///
/// - `make & press`
/// - `break & release`
///
/// In other words, if a button is in the `make` state, then it is also in the `press` state.
/// However, this is not the case for the inverse. A button in the `press` state does not need to be
/// in the `make` state.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
#[doc(alias("SceCtrlLatch"))]
pub struct LatchData {
    /// Button transitioned to `pressed` state.
    pub make: PadButtons,
    /// Button transitioned to `released` state.
    pub breakk: PadButtons,
    /// Button is in the `pressed` state.
    pub press: PadButtons,
    /// Button is in the `released` state.
    pub release: PadButtons,
}

/// Specifies the kind of input data to be obtained.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
#[doc(alias("SceCtrlPort"))]
pub enum ControllerPort {
    /// Input is only obtained from the PSP integrated controller.
    #[default]
    Psp = 0,
    /// Input is obtained from the PSP integrated controller and a connected DualShock3 controller.
    PspAndDs3 = 1,
    /// Input is obtained from the PSP integrated controller and an unknown connected external
    /// device.
    PspAndUnknown = 2,
}

/// Button mask setting that specifies the type of input data to be obtained.
#[repr(u8)]
#[doc(alias("SceCtrlButtonMaskMode", "SceCtrlPadButtonMaskMode"))]
pub enum ButtonMaskMode {
    /// No mask for the specified buttons.
    ///
    /// Button input is normally recognized.
    #[doc(alias("SCE_CTRL_MASK_NO_MASK"))]
    NoMask = 0,
    /// The specified buttons are ignored.
    ///
    /// That means even if these buttons are pressed by the user they won't be shown as pressed
    /// internally.
    ///
    /// You can only block user buttons for applications running in User Mode.
    #[doc(alias("SCE_CTRL_MASK_IGNORE_BUTTONS"))]
    IgnoreButtons = 1,
    /// The specified buttons show up as being pressed, even if the user does not press them.
    ///
    /// You can only turn ON user buttons for applications running in User Mode.
    #[doc(alias("SCE_CTRL_MASK_APPLY_BUTTONS"))]
    ApplyButtons = 2,
}

/// Type used to copy external input data into PSP internal controller buffers.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceCtrlInputDataTransferHandler"))]
pub struct InputDataTransferHandle {
    /// Unknown. Always set to `0xC` by Sony.
    pub unk: u32,
    /// A pointer to a transfer function to copy input data into a PSP internal controller buffer.
    ///
    /// Check [`InputDataTransferFn`] for more information on the fn pointer.
    pub copy_input_data: Option<InputDataTransferFn>,
}

/// Represents the function pointer of [`InputDataTransferHandle`].
///
/// # Parameters
///
/// - `src`: A pointer to buffer containing the Controller input data to copy to the PSP's
///   controller buffers.
/// - `dest`: A pointer to the PSP Controller input data format.
///
/// # Return Value
///
/// A positive value on success, error value otherwise.
pub type InputDataTransferFn =
    unsafe extern "C" fn(src: *mut c_void, dest: *mut CtrlDataExt) -> SceResult<u32>;

#[psp_stub(libname = "sceCtrl", flags = 0x4001, use_crate)]
extern "C" {
    /// Sets the controller input sampling mode.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    ///
    /// # Return Value
    ///
    /// Returns the previous input sampling mode on success, error value otherwise
    #[nid(0x1F4011E6)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlSetSamplingMode(mode: InputMode) -> SceResult<InputMode>;


    /// Gets the currently set input sampling mode.
    ///
    /// # Parameters
    ///
    /// - `mode` **[[Out parameter]]**: A reference to receive the input mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDA6B76A1)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlGetSamplingMode(mode: &mut InputMode) -> SceResult<InputMode>;

    /// Sets the update frequency of the internal controller buffer.
    ///
    /// The default update interval is the VBlank interrupt (approximately `60` times per second).
    ///
    /// # Parameters
    ///
    /// - `cycle`: The new interval between two samplings of controller attributes. If set to
    ///   [SamplingCycle::VBLANK_INTERRUPT_UPDATE] enables the VBlank-Interrupt-Update process. To
    ///   set an own interval for updating the internal controller buffers, `cycle` has to in the
    ///   range of `[5555, 20000]` microseconds (the range from about 180Hz to 50Hz).
    ///
    /// # Return Value
    ///
    /// Returns the previous sampling cycle value set on success, error value otherwise.
    #[nid(0x6A2774F3)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlSetSamplingCycle(cycle: SamplingCycle) -> SceResult<SamplingCycle>;

    /// Gets the currently set sampling cycle value.
    ///
    /// # Parameters
    ///
    /// - `cycle` **[[Out parameter]]**: A reference to receive the sampling cycle value.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x02BAAD91)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlGetSamplingCycle(cycle: &mut SamplingCycle) -> SceResult<()>;

    /// Sets analog stick movement threshold values for cancelling the idle timer.
    ///
    /// In case [InputMode::DigitalOnly] is set as the input mode for the controller, analog stick
    /// movements will not result in cancelling the idle timer.
    ///
    /// # Parameters
    ///
    /// - `unhold_threshold`: Movement needed by the analog stick to reset the idle timer when HOLD
    ///   mode is inactive.
    /// - `hold_threshold`: Movement needed by the analog stick to reset the idle timer when HOLD
    ///   mode is active.
    ///
    /// Set between `[1,128]` to specify the movement on either axis.
    ///
    /// Set to [`IdleCancelThreshold::ALWAYS_CANCEL`] for idle timer to be canceled even if the
    /// analog stick is not moved (that is, the idle timer itself stops running).
    ///
    /// Set to [`IdleCancelThreshold::NO_CANCEL`] for analog stick movement to not cancel the idle
    /// timer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(0xA7144800)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlSetIdleCancelThreshold(
        unhold_threshold: IdleCancelThreshold, hold_threshold: IdleCancelThreshold,
    ) -> SceResult<()>;

    /// Gets the currently set idle timer cancel movement threshold values for the analog stick.
    ///
    /// # Parameters
    ///
    /// - `unhold_threshold`: A reference to receive the movement needed by the analog stick to
    ///   reset the idle timer when HOLD mode is inactive.
    /// - `hold_threshold`: A reference to receive the movement needed by the analog stick to reset
    ///   the idle timer when HOLD mode is active.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(0x687660FA)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlGetIdleCancelThreshold(
        unhold_threshold: &mut IdleCancelThreshold, hold_threshold: &mut IdleCancelThreshold,
    ) -> SceResult<()>;

    /// Retrieves controller state data by polling (positive logic, i.e. state data will be `1` when
    /// the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(0x3A622550)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlPeekBufferPositive(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data by polling (negative logic, i.e. state data will be `0` when
    /// the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(0xC152080A)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlPeekBufferNegative(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data by blocking (positive logic, i.e. state data will be `1`
    /// when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(0x1F803938)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlReadBufferPositive(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data by blocking (negative logic, i.e. state data will be `0`
    /// when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(0x60B81F86)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlReadBufferNegative(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Gets the latest latch data from the controller service.
    ///
    /// This function reads the latch data collected by the controller service. At each sampling
    /// interval, the controller service compares the new `pressed`/`released` button states with
    /// the previously sampled `pressed` button states and stores that comparison as latch data.
    ///
    /// Compared to [`sceCtrlReadLatch`], calling this API will not result in clearing the internal
    /// latch data. As such, the data returned is the accumulated latch data since the last time
    /// [`sceCtrlReadLatch`] was called. Consequently, the returned data should not be relied on
    /// whether a button is currently in a pressed or released state.
    ///
    /// # Parameters
    ///
    /// - `latch_data` **[[Out parameter]]**: A pointer to receive the latch data.
    ///
    /// # Return Value
    ///
    /// Returns the number of times the controller service performed sampling since the last time
    /// [`sceCtrlReadLatch`] was called on success, error value otherwise.
    #[nid(0xB1D0E5CD)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlPeekLatch(latch_data: *mut LatchData) -> SceResult<u32>;

    /// Reads new latch data from the controller service.
    ///
    /// This function reads the most recent latch data collected by the controller service. At each
    /// sampling interval, the controller service compares the new `pressed`/`released` button
    /// states with the previously sampled `pressed` button states and stores that comparison as
    /// latch data.
    ///
    /// Compared to [`sceCtrlPeekLatch`], calling this API will result in clearing the internal
    /// latch data. As such, calling code might have to explicitly wait for the controller service
    /// to update its collected latch data.
    ///
    /// # Parameters
    ///
    /// - `latch_data` **[[Out parameter]]**: A pointer to receive the latch data.
    ///
    /// # Return Value
    ///
    /// Returns the number of times the controller service performed sampling since the last time
    /// [`sceCtrlReadLatch`] was called on success, error value otherwise.
    #[nid(0x0B588501)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlReadLatch(latch_data: *mut LatchData) -> SceResult<u32>;

    /// Disables a rapid-fire button event.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot of the event to clear.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA68FD260)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlClearRapidFire(slot: RapidFireSlot) -> SceResult<()>;

    /// Sets a rapid-fire event for one or more buttons.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot of the event to set.
    /// - `mask`: The comparison mask of the button operation for rapid-fire trigger. In order for
    ///   the `trigger` buttons to trigger the event, they need to be included in these buttons.
    /// - `trigger`: The buttons which will start the rapid fire event for the specified `target`
    ///   buttons when being pressed.
    /// - `target`: The buttons for which the rapid-fire event will be applied to. User mode buttons
    ///   only. `make` and `breakk` define the rapid-fire cycle.
    /// - `delay`: The dead time of rapid-fire trigger (sampling count). Specifies the rapid-fire
    ///   start timing. It will only be applied for the first ON period of a (not cancelled)
    ///   rapid-fire event. Valid value in the range of `[0,63]`.
    /// - `make`: The press time for the `target` buttons.  This "ON-time" is set after `delay` was
    ///   applied and the `trigger` buttons were turned OFF. It will be applied for as long as the
    ///   same rapid fire event is called without a break (i.e. pressing of a different PSP button).
    ///   Valid value in the range of `[0,63]`. If set to `0`, the `target` button(s) will be turned
    ///   ON for one sampling count.
    /// - `breakk`: The release time for `target` buttons. This "OFF-time" is set after `delay` was
    ///   applied. It will be applied as long as the same rapid fire event is called without a break
    ///   (i.e. the pressing of a different PSP button). Valid value in the range of `[0,63]`. If
    ///   set to `0`, the `target` button will be turned OFF for `64` consecutive sampling counts.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i7)]
    #[nid(0x6841BE1A)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlSetRapidFire(
        slot: RapidFireSlot, mask: PadButtons, trigger: PadButtons, target: PadButtons, delay: u32,
        make: u32, breakk: u32,
    ) -> SceResult<()>;

    /// Sets a number of VBlanks which will be waited for when the PSP device is being suspended.
    ///
    /// # Parameters
    ///
    /// - `suspend_samples`: The number of VBlanks. Valid value in the range in `[0,300]`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0x348D99D4)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlSetSuspendingExtraSamples(suspend_samples: u16) -> SceResult<()>;

    /// Gets the number of VBlanks which will be waited for when the PSP device is being suspended.
    ///
    /// # Return Value
    ///
    /// Returns the number of VBlanks to wait for.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0xAF5960F3)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceCtrlGetSuspendingExtraSamples() -> u16;

    /// Retrieves controller state data on a specified controller port by polling (positive logic,
    /// i.e. state data will be `1` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x5A36B1C2)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlPeekBufferPositive2(
        port: ControllerPort, data_buf: *mut CtrlDataExt, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data on a specified controller port by polling (negative logic,
    /// i.e. state data will be `0` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x239A6BA7)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlPeekBufferNegative2(
        port: ControllerPort, data_buf: *mut CtrlDataExt, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data on a specified controller port by blocking (positive logic,
    /// i.e. state data will be `1` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x1098030B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlReadBufferPositive2(
        port: ControllerPort, data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data on a specified controller port by blocking (negative logic,
    /// i.e. state data will be `0` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(0x7C3675AB)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceCtrlReadBufferNegative2(
        port: ControllerPort, data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceCtrl_driver", flags = 0x0001, use_crate)]
extern "C" {
    /// Sets the controller input sampling mode.
    ///
    /// # Parameters
    ///
    /// - `mode`: The mode to set.
    ///
    /// # Return Value
    ///
    /// Returns the previous input sampling mode on success, error value otherwise
    #[nid(if cfg!(feature = "psp_660") { 0xF6E94EA3 }
        else if cfg!(feature = "psp_630") { 0x6CB49301 }
        else if cfg!(feature = "psp_600") { 0x262DD0DC }
        else if cfg!(feature = "psp_570") { 0xFDD3101B }
        else if cfg!(feature = "psp_500") { 0x87B0677D }
        else if cfg!(feature = "psp_420") { 0x734DA399 }
        else if cfg!(feature = "psp_395") { 0x9A0923A5 }
        else if cfg!(feature = "psp_380") { 0xFE76947F }
        else if cfg!(feature = "psp_370") { 0x28E71A16 }
        else { 0x1F4011E6 }
    )]
    pub fn sceCtrlSetSamplingMode(mode: InputMode) -> SceResult<InputMode>;


    /// Gets the currently set input sampling mode.
    ///
    /// # Parameters
    ///
    /// - `mode` **[[Out parameter]]**: A reference to receive the input mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xF8EC18BD }
        else if cfg!(feature = "psp_630") { 0x410833B7 }
        else if cfg!(feature = "psp_600") { 0xC44ED01B }
        else if cfg!(feature = "psp_570") { 0x879E0B76 }
        else if cfg!(feature = "psp_500") { 0x6D4BEA72 }
        else if cfg!(feature = "psp_420") { 0x27FDCC55 }
        else if cfg!(feature = "psp_395") { 0x4CEB8CC4 }
        else if cfg!(feature = "psp_380") { 0xC432938A }
        else if cfg!(feature = "psp_370") { 0xD7F23B0B }
        else { 0xDA6B76A1 }
    )]
    pub fn sceCtrlGetSamplingMode(mode: &mut InputMode) -> SceResult<InputMode>;

    /// Sets the update frequency of the internal controller buffer.
    ///
    /// The default update interval is the VBlank interrupt (approximately `60` times per second).
    ///
    /// # Parameters
    ///
    /// - `cycle`: The new raw interval between two samplings of controller attributes. If set to
    ///   [SamplingCycle::VBLANK_INTERRUPT_UPDATE] enables the VBlank-Interrupt-Update process. To
    ///   set an own interval for updating the internal controller buffers, `cycle` has to in the
    ///   range of `[5555, 20000]` microseconds (the range from about 180Hz to 50Hz).
    ///
    /// # Return Value
    ///
    /// Returns the previous sampling cycle value set on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x83B15A81 }
        else if cfg!(feature = "psp_630") { 0x855C255D }
        else if cfg!(feature = "psp_600") { 0xE4758982 }
        else if cfg!(feature = "psp_570") { 0x31EEA2AF }
        else if cfg!(feature = "psp_500") { 0x8AAECD34 }
        else if cfg!(feature = "psp_420") { 0x1566C499 }
        else if cfg!(feature = "psp_395") { 0xA69EC68F }
        else if cfg!(feature = "psp_380") { 0x13C3008D }
        else if cfg!(feature = "psp_370") { 0x6D74BF08 }
        else { 0x6A2774F3 }
    )]
    pub fn sceCtrlSetSamplingCycle(cycle: SamplingCycle) -> SceResult<SamplingCycle>;

    /// Gets the currently set interval of the internal controller data buffers.
    ///
    /// # Parameters
    ///
    /// - `cycle` **[[Out parameter]]**: A reference to receive the sampling cycle value.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x501E0C70 }
        else if cfg!(feature = "psp_630") { 0x4E972A76 }
        else if cfg!(feature = "psp_600") { 0xFE15E7BA }
        else if cfg!(feature = "psp_570") { 0xE38FAADA }
        else if cfg!(feature = "psp_500") { 0xA0390EE8 }
        else if cfg!(feature = "psp_420") { 0xE1BDEAFE }
        else if cfg!(feature = "psp_395") { 0xD53B2287 }
        else if cfg!(feature = "psp_380") { 0x74EC15B5 }
        else if cfg!(feature = "psp_370") { 0x8FE1D531 }
        else { 0x02BAAD91 }
    )]
    pub fn sceCtrlGetSamplingCycle(cycle: &mut SamplingCycle) -> SceResult<()>;

    /// Sets analog stick movement threshold values for cancelling the idle timer.
    ///
    /// In case [InputMode::DigitalOnly] is set as the input mode for the controller, analog stick
    /// movements will not result in cancelling the idle timer.
    ///
    /// # Parameters
    ///
    /// - `unhold_threshold`: Movement needed by the analog stick to reset the idle timer when HOLD
    ///   mode is inactive.
    /// - `hold_threshold`: Movement needed by the analog stick to reset the idle timer when HOLD
    ///   mode is active.
    ///
    /// Set between `[1,128]` to specify the movement on either axis.
    ///
    /// Set to [`IdleCancelThreshold::ALWAYS_CANCEL`] for idle timer to be canceled even if the
    /// analog stick is not moved (that is, the idle timer itself stops running).
    ///
    /// Set to [`IdleCancelThreshold::NO_CANCEL`] for analog stick movement to not cancel the idle
    /// timer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(if cfg!(feature = "psp_660") { 0x37533267 }
        else if cfg!(feature = "psp_630") { 0x84CEAE74 }
        else if cfg!(feature = "psp_600") { 0x32999159 }
        else if cfg!(feature = "psp_570") { 0x41C2182B }
        else if cfg!(feature = "psp_500") { 0x03A81798 }
        else if cfg!(feature = "psp_420") { 0x35B0900D }
        else if cfg!(feature = "psp_395") { 0x7E5029E2 }
        else if cfg!(feature = "psp_380") { 0x50B54185 }
        else if cfg!(feature = "psp_370") { 0x6F3B46FB }
        else { 0xA7144800 }
    )]
    pub fn sceCtrlSetIdleCancelThreshold(
        unhold_threshold: IdleCancelThreshold, hold_threshold: IdleCancelThreshold,
    ) -> SceResult<()>;

    /// Gets the currently set idle timer cancel movement threshold values for the analog stick.
    ///
    /// # Parameters
    ///
    /// - `unhold_threshold`: A reference to receive the movement needed by the analog stick to
    ///   reset the idle timer when HOLD mode is inactive.
    /// - `hold_threshold`: A reference to receive the movement needed by the analog stick to reset
    ///   the idle timer when HOLD mode is active.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[nid(if cfg!(feature = "psp_660") { 0xE54253E7 }
        else if cfg!(feature = "psp_630") { 0x390D1A49 }
        else if cfg!(feature = "psp_600") { 0xD821DF71 }
        else if cfg!(feature = "psp_570") { 0xB88D61F5 }
        else if cfg!(feature = "psp_500") { 0xE3F78BF9 }
        else if cfg!(feature = "psp_420") { 0xD0EBBD96 }
        else if cfg!(feature = "psp_395") { 0x9A77FAE5 }
        else if cfg!(feature = "psp_380") { 0x82EA38BC }
        else if cfg!(feature = "psp_370") { 0xF3630971 }
        else { 0x687660FA }
    )]
    pub fn sceCtrlGetIdleCancelThreshold(
        unhold_threshold: &mut IdleCancelThreshold, hold_threshold: &mut IdleCancelThreshold,
    ) -> SceResult<()>;

    /// Retrieves controller state data by polling (positive logic, i.e. state data will be `1` when
    /// the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x2BA616AF }
        else if cfg!(feature = "psp_630") { 0x18654FC0 }
        else if cfg!(feature = "psp_600") { 0x3CA6922B }
        else if cfg!(feature = "psp_570") { 0x73167FA0 }
        else if cfg!(feature = "psp_500") { 0x6B247CCE }
        else if cfg!(feature = "psp_420") { 0x4D4F6778 }
        else if cfg!(feature = "psp_395") { 0x591B3F36 }
        else if cfg!(feature = "psp_380") { 0xD65D4E9A }
        else if cfg!(feature = "psp_370") { 0xC4AAD55F }
        else { 0x3A622550 }
    )]
    pub unsafe fn sceCtrlPeekBufferPositive(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data by polling (negative logic, i.e. state data will be `0` when
    /// the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE6085C33 }
        else if cfg!(feature = "psp_630") { 0x02DD57CF }
        else if cfg!(feature = "psp_600") { 0x2468E1F3 }
        else if cfg!(feature = "psp_570") { 0xDDD34FB2 }
        else if cfg!(feature = "psp_500") { 0xCC2D8C39 }
        else if cfg!(feature = "psp_420") { 0x6D59F7C6 }
        else if cfg!(feature = "psp_395") { 0x5E774B29 }
        else if cfg!(feature = "psp_380") { 0x22E8F160 }
        else if cfg!(feature = "psp_370") { 0x8182D8A0 }
        else { 0xC152080A }
    )]
    pub unsafe fn sceCtrlPeekBufferNegative(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data by blocking (positive logic, i.e. state data will be `1`
    /// when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xBE30CED0 }
        else if cfg!(feature = "psp_630") { 0x9F3038AC }
        else if cfg!(feature = "psp_600") { 0xD073ECA4 }
        else if cfg!(feature = "psp_570") { 0x8811A4C5 }
        else if cfg!(feature = "psp_500") { 0x919215D7 }
        else if cfg!(feature = "psp_420") { 0x670447FA }
        else if cfg!(feature = "psp_395") { 0xBA664B5E }
        else if cfg!(feature = "psp_380") { 0xAD0510F6 }
        else if cfg!(feature = "psp_370") { 0x454455AC }
        else { 0x1F803938 }
    )]
    pub unsafe fn sceCtrlReadBufferPositive(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data by blocking (negative logic, i.e. state data will be `0`
    /// when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// # Parameters
    ///
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x3A6A612A }
        else if cfg!(feature = "psp_630") { 0xEB5F1D7A }
        else if cfg!(feature = "psp_600") { 0x5E758582 }
        else if cfg!(feature = "psp_570") { 0xC0870C3C }
        else if cfg!(feature = "psp_500") { 0xF54317C4 }
        else if cfg!(feature = "psp_420") { 0xD1C22195 }
        else if cfg!(feature = "psp_395") { 0xAEDB3C0D }
        else if cfg!(feature = "psp_380") { 0xA7A89B5F }
        else if cfg!(feature = "psp_370") { 0xFF847C31 }
        else { 0x60B81F86 }
    )]
    pub unsafe fn sceCtrlReadBufferNegative(
        data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Gets the latest latch data from the controller service.
    ///
    /// This function reads the latch data collected by the controller service. At each sampling
    /// interval, the controller service compares the new `pressed`/`released` button states with
    /// the previously sampled `pressed` button states and stores that comparison as latch data.
    ///
    /// Compared to [`sceCtrlReadLatch`], calling this API will not result in clearing the internal
    /// latch data. As such, the data returned is the accumulated latch data since the last time
    /// [`sceCtrlReadLatch`] was called. Consequently, the returned data should not be relied on
    /// whether a button is currently in a pressed or released state.
    ///
    /// # Parameters
    ///
    /// - `latch_data` **[[Out parameter]]**: A pointer to receive the latch data.
    ///
    /// # Return Value
    ///
    /// Returns the number of times the controller service performed sampling since the last time
    /// [`sceCtrlReadLatch`] was called on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x637CB76C }
        else if cfg!(feature = "psp_630") { 0x6574DC7C }
        else if cfg!(feature = "psp_600") { 0x8440BBFA }
        else if cfg!(feature = "psp_570") { 0xA9ECA14E }
        else if cfg!(feature = "psp_500") { 0x655A18BE }
        else if cfg!(feature = "psp_420") { 0x6EABD658 }
        else if cfg!(feature = "psp_395") { 0x05FEDF78 }
        else if cfg!(feature = "psp_380") { 0x4281FFBA }
        else if cfg!(feature = "psp_370") { 0xEA6EDF43 }
        else { 0xB1D0E5CD }
    )]
    pub unsafe fn sceCtrlPeekLatch(latch_data: *mut LatchData) -> SceResult<u32>;

    /// Reads new latch data from the controller service.
    ///
    /// This function reads the most recent latch data collected by the controller service. At each
    /// sampling interval, the controller service compares the new `pressed`/`released` button
    /// states with the previously sampled `pressed` button states and stores that comparison as
    /// latch data.
    ///
    /// Compared to [`sceCtrlPeekLatch`], calling this API will result in clearing the internal
    /// latch data. As such, calling code might have to explicitly wait for the controller service
    /// to update its collected latch data.
    ///
    /// # Parameters
    ///
    /// - `latch_data` **[[Out parameter]]**: A pointer to receive the latch data.
    ///
    /// # Return Value
    ///
    /// Returns the number of times the controller service performed sampling since the last time
    /// [`sceCtrlReadLatch`] was called on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x7F7C4E0A }
        else if cfg!(feature = "psp_630") { 0xD883CAF9 }
        else if cfg!(feature = "psp_600") { 0x821F0F79 }
        else if cfg!(feature = "psp_570") { 0xB94FDBB5 }
        else if cfg!(feature = "psp_500") { 0x5068A531 }
        else if cfg!(feature = "psp_420") { 0xDE5A92FE }
        else if cfg!(feature = "psp_395") { 0x336402C3 }
        else if cfg!(feature = "psp_380") { 0xA2F57E56 }
        else if cfg!(feature = "psp_370") { 0x7DF19E59 }
        else { 0x0B588501 }
    )]
    pub unsafe fn sceCtrlReadLatch(latch_data: *mut LatchData) -> SceResult<u32>;

    /// Disables a rapid-fire button event.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot of the event to clear.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x994488EC }
        else if cfg!(feature = "psp_630") { 0xFAF675CB }
        else if cfg!(feature = "psp_600") { 0xF7E92CE5 }
        else if cfg!(feature = "psp_570") { 0xA2A83819 }
        else if cfg!(feature = "psp_500") { 0xF17D609C }
        else if cfg!(feature = "psp_420") { 0x8C2D25CA }
        else if cfg!(feature = "psp_395") { 0x35D6FE23 }
        else if cfg!(feature = "psp_380") { 0x3AF3B1D7 }
        else if cfg!(feature = "psp_370") { 0xEF06B8B2 }
        else { 0xA68FD260 }
    )]
    pub fn sceCtrlClearRapidFire(slot: RapidFireSlot) -> SceResult<()>;

    /// Sets a rapid-fire event for one or more buttons.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot of the event to set.
    /// - `mask`: The comparison mask of the button operation for rapid-fire trigger. In order for
    ///   the `trigger` buttons to trigger the event, they need to be included in these buttons.
    /// - `trigger`: The buttons which will start the rapid fire event for the specified `target`
    ///   buttons when being pressed.
    /// - `target`: The buttons for which the rapid-fire event will be applied to. User mode buttons
    ///   only. `make` and `breakk` define the rapid-fire cycle.
    /// - `delay`: The dead time of rapid-fire trigger (sampling count). Specifies the rapid-fire
    ///   start timing. It will only be applied for the first ON period of a (not cancelled)
    ///   rapid-fire event. Valid value in the range of `[0,63]`.
    /// - `make`: The press time for the `target` buttons.  This "ON-time" is set after `delay` was
    ///   applied and the `trigger` buttons were turned OFF. It will be applied for as long as the
    ///   same rapid fire event is called without a break (i.e. pressing of a different PSP button).
    ///   Valid value in the range of `[0,63]`. If set to `0`, the `target` button(s) will be turned
    ///   ON for one sampling count.
    /// - `breakk`: The release time for `target` buttons. This "OFF-time" is set after `delay` was
    ///   applied. It will be applied as long as the same rapid fire event is called without a break
    ///   (i.e. the pressing of a different PSP button). Valid value in the range of `[0,63]`. If
    ///   set to `0`, the `target` button will be turned OFF for `64` consecutive sampling counts.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i7)]
    #[nid(if cfg!(feature = "psp_660") { 0x89438C13 }
        else if cfg!(feature = "psp_630") { 0xE96A4D84 }
        else if cfg!(feature = "psp_600") { 0xA2FF2213 }
        else if cfg!(feature = "psp_570") { 0x61AE3D4F }
        else if cfg!(feature = "psp_500") { 0xF83A8920 }
        else if cfg!(feature = "psp_420") { 0x08D37981 }
        else if cfg!(feature = "psp_395") { 0x909A48A9 }
        else if cfg!(feature = "psp_380") { 0xB5C7A22F }
        else if cfg!(feature = "psp_370") { 0xCE223F52 }
        else { 0x6841BE1A }
    )]
    pub fn sceCtrlSetRapidFire(
        slot: RapidFireSlot, mask: PadButtons, trigger: PadButtons, target: PadButtons, delay: u32,
        make: u32, breakk: u32,
    ) -> SceResult<()>;

    /// Sets a number of VBlanks which will be waited for when the PSP device is being suspended.
    ///
    /// # Parameters
    ///
    /// - `suspend_samples`: The number of VBlanks. Valid value in the range in `[0,300]`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0x547F89D3 }
        else if cfg!(feature = "psp_630") { 0x53E67075 }
        else if cfg!(feature = "psp_600") { 0xB7D6332B }
        else if cfg!(feature = "psp_570") { 0xDCD38F30 }
        else if cfg!(feature = "psp_500") { 0x189BA4F3 }
        else if cfg!(feature = "psp_420") { 0xE5452A46 }
        else if cfg!(feature = "psp_395") { 0x23A99CBF }
        else if cfg!(feature = "psp_380") { 0xDFDEC8DE }
        else if cfg!(feature = "psp_370") { 0x37B6B9E9 }
        else { 0x348D99D4 }
    )]
    pub fn sceCtrlSetSuspendingExtraSamples(suspend_samples: u16) -> SceResult<()>;

    /// Gets the number of VBlanks which will be waited for when the PSP device is being suspended.
    ///
    /// # Return Value
    ///
    /// Returns the number of VBlanks to wait for.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(if cfg!(feature = "psp_660") { 0xBC8D1A3B }
        else if cfg!(feature = "psp_630") { 0x525D27AC }
        else if cfg!(feature = "psp_600") { 0xD2EC6240 }
        else if cfg!(feature = "psp_570") { 0xEF6E93C4 }
        else if cfg!(feature = "psp_500") { 0x46C2F764 }
        else if cfg!(feature = "psp_420") { 0x6CDA528C }
        else if cfg!(feature = "psp_395") { 0xFDDEF6C8 }
        else if cfg!(feature = "psp_380") { 0x2159E716 }
        else if cfg!(feature = "psp_370") { 0xBCE989DD }
        else { 0xAF5960F3 }
    )]
    pub fn sceCtrlGetSuspendingExtraSamples() -> u16;

    /// Retrieves controller state data on a specified controller port by polling (positive logic,
    /// i.e. state data will be `1` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// **Note:** [`sceCtrlSetInternalBufferHandler`] must be called before initial use of this API
    /// or its related ones.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(if cfg!(feature = "psp_660") { 0xD4692E77 }
        else if cfg!(feature = "psp_630") { 0x1D75C1D4 }
        else if cfg!(feature = "psp_600") { 0xE8121137 }
        else { 0x5A36B1C2 }
    )]
    pub unsafe fn sceCtrlPeekBufferPositive2(
        port: ControllerPort, data_buf: *mut CtrlDataExt, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data on a specified controller port by polling (negative logic,
    /// i.e. state data will be `0` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It does not
    /// wait for the next update interval to be performed.
    ///
    /// The obtained data will be the latest transferred button data into the internal controller
    /// buffers.
    ///
    /// **Note:** [`sceCtrlSetInternalBufferHandler`] must be called before initial use of this API
    /// or its related ones.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(if cfg!(feature = "psp_660") { 0x41BCD9ED }
        else if cfg!(feature = "psp_630") { 0x6E552572 }
        else if cfg!(feature = "psp_600") { 0x52404C02 }
        else { 0x239A6BA7 }
    )]
    pub unsafe fn sceCtrlPeekBufferNegative2(
        port: ControllerPort, data_buf: *mut CtrlDataExt, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data on a specified controller port by blocking (positive logic,
    /// i.e. state data will be `1` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// **Note:** [`sceCtrlSetInternalBufferHandler`] must be called before initial use of this API
    /// or its related ones.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(if cfg!(feature = "psp_660") { 0x3BD76EDE }
        else if cfg!(feature = "psp_630") { 0x16BB4085 }
        else if cfg!(feature = "psp_600") { 0x1A5393EC }
        else { 0x1098030B }
    )]
    pub unsafe fn sceCtrlReadBufferPositive2(
        port: ControllerPort, data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Retrieves controller state data on a specified controller port by blocking (negative logic,
    /// i.e. state data will be `0` when the button is pressed).
    ///
    /// This function obtains button data stored in the internal controller buffers. It waits for
    /// the next update interval before obtaining the data.
    ///
    /// The read data is the newest transferred data into the internal controller buffers.
    ///
    /// **Note:** [`sceCtrlSetInternalBufferHandler`] must be called before initial use of this API
    /// or its related ones.
    ///
    /// # Parameters
    ///
    /// - `port`: The controller port to get the data.
    /// - `data_buf` **[[Out parameter]]**: Buffer to receive the controller data.
    /// - `data_buf_size`: The size of the `data_buf` buffer. Valid range of `[1; 64]`.
    ///
    /// # Return Value
    ///
    /// Returns the number of the buffer items set on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    #[nid(if cfg!(feature = "psp_660") { 0x7ABDEBAA }
        else if cfg!(feature = "psp_630") { 0x4870C6AF }
        else if cfg!(feature = "psp_600") { 0xF8508E92 }
        else { 0x7C3675AB }
    )]
    pub unsafe fn sceCtrlReadBufferNegative2(
        port: ControllerPort, data_buf: *mut CtrlData, data_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Sets a button mask mode for one or more buttons.
    ///
    /// You can only mask user mode buttons in user applications. Masking of kernel mode buttons is
    /// ignored as well as buttons used in kernel mode applications.
    ///
    /// # Parameters
    ///
    /// - `buttons`: The button value for which the button mask mode will be applied.
    /// - `mask_mode`: The type of the button mask.
    ///
    /// # Return Value
    ///
    /// Returns the previous set mask mode for the given buttons.
    #[nid(if cfg!(feature = "psp_660") { 0xF8346777 }
        else if cfg!(feature = "psp_630") { 0x5B15473C }
        else if cfg!(feature = "psp_600") { 0xF3132A07 }
        else if cfg!(feature = "psp_570") { 0xDEB5B38A }
        else if cfg!(feature = "psp_500") { 0xC24FF87A }
        else if cfg!(feature = "psp_420") { 0x23F7B668 }
        else if cfg!(feature = "psp_395") { 0x29A5082C }
        else if cfg!(feature = "psp_380") { 0x270345D5 }
        else if cfg!(feature = "psp_370") { 0xDB6F93CB }
        else { 0x7CA723DC }
    )]
    pub fn sceCtrlSetButtonIntercept(
        buttons: PadButtons, mask_mode: ButtonMaskMode,
    ) -> ButtonMaskMode;

    /// Gets the button mask settings applied to PSP buttons.
    ///
    /// # Parameters
    ///
    /// - `buttons`: The buttons to check.
    ///
    /// # Return Value
    ///
    /// Returns the current set mask mode for the given buttons.
    #[nid(if cfg!(feature = "psp_660") { 0x1809B9FC }
        else if cfg!(feature = "psp_630") { 0x33AB5BDB }
        else if cfg!(feature = "psp_600") { 0x063D8197 }
        else if cfg!(feature = "psp_570") { 0x6136E0EE }
        else if cfg!(feature = "psp_500") { 0x09236627 }
        else if cfg!(feature = "psp_420") { 0xD119E479 }
        else if cfg!(feature = "psp_395") { 0xE3870772 }
        else if cfg!(feature = "psp_380") { 0xC2D874D7 }
        else if cfg!(feature = "psp_370") { 0xDB3CD94C }
        else { 0x5E77BC8A }
    )]
    pub fn sceCtrlGetButtonIntercept(buttons: PadButtons) -> ButtonMaskMode;

    /// Registers a button callback.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot used to register the callback.
    /// - `button_mask`: The button values which will be checked for being pressed.
    /// - `callback`: The callback function handling the button callbacks.
    /// - `common` **[[InOut parameter]]**: A pointer to memory shared with `callback`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xDF53E160 }
        else if cfg!(feature = "psp_630") { 0x5D8CE0B2 }
        else if cfg!(feature = "psp_600") { 0xEB6CDD17 }
        else if cfg!(feature = "psp_570") { 0x8FF0A8DF }
        else if cfg!(feature = "psp_500") { 0x2037F8E7 }
        else if cfg!(feature = "psp_420") { 0x38F11D5C }
        else if cfg!(feature = "psp_395") { 0xEB97D7AA }
        else if cfg!(feature = "psp_380") { 0x02DC2E6C }
        else if cfg!(feature = "psp_370") { 0x0B7AC633 }
        else { 0x5C56C779 }
    )]
    pub fn sceCtrlSetSpecialButtonCallback(
        slot: SpecialButtonCallbackSlot, button_mask: PadButtons, callback: SpecialButtonCallback,
        common: *mut c_void,
    ) -> SceResult<()>;

    /// Sets the buttons which, when being pressed, reset the idle timer.
    ///
    /// # Parameters
    ///
    /// - `make`: The buttons which will reset the timer if pressed. If you keep pressing one or
    ///   more of these buttons after resetting the timer, they will not reset the timer anymore.
    ///   You will have to release the buttons first, before they can reset it again. In case HOLD
    ///   mode is active, pressing these buttons will not reset the timer.
    /// - `press`: The buttons which will reset the timer if pressed. As long as you press one of
    ///   these buttons, the timer is resetted. In case HOLD mode is active, pressing these buttons
    ///   will not reset the timer.
    /// - `hold_mode_make`: The buttons which will reset the timer if pressed while HOLD mode is
    ///   active. If you keep pressing these buttons after resetting the timer, they will not reset
    ///   it anymore. You will have to release the buttons first, before they can reset the timer
    ///   again.
    /// - `hold_mode_press`: The buttons which will reset the timer if pressed while HOLD mode is
    ///   active. As long as you press one of these buttons, the timer is resetted.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x6A1DF4CB }
        else if cfg!(feature = "psp_630") { 0x0D627B90 }
        else if cfg!(feature = "psp_600") { 0x0DCB5BD2 }
        else if cfg!(feature = "psp_570") { 0x1C3A13B2 }
        else if cfg!(feature = "psp_500") { 0x0A218F56 }
        else if cfg!(feature = "psp_420") { 0x7BB2710E }
        else if cfg!(feature = "psp_395") { 0x77BABAF9 }
        else if cfg!(feature = "psp_380") { 0xB0DE19EC }
        else if cfg!(feature = "psp_370") { 0x312646B8 }
        else { 0xA88E8D22 }
    )]
    pub fn sceCtrlSetIdleCancelKey(
        make: PadButtons, press: PadButtons, hold_mode_make: PadButtons,
        hold_mode_press: PadButtons,
    ) -> SceResult<()>;

    /// Gets the different cancel-idle-timer buttons.
    ///
    /// # Parameters
    ///
    /// - `make` **[[Out parameter]]**: A reference to receive the buttons resetting the idle timer
    ///   when being pressed a new time (not already being pressed). Not used when the system is in
    ///   HOLD mode.
    /// - `press` **[[Out parameter]]**: A reference to receive the buttons resetting the idle timer
    ///   when being pressed. Not used when the system is in HOLD mode.
    /// - `hold_mode_make` **[[Out parameter]]**: A reference to receive the buttons resetting the
    ///   idle timer when being pressed a new time (not already being pressed). Used when the system
    ///   is in HOLD mode.
    /// - `hold_mode_press` **[[Out parameter]]**: A reference to receive the buttons resetting the
    ///   idle timer when being pressed a new time. Used when the system is in HOLD mode.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x6A1DF4CB }
        else if cfg!(feature = "psp_630") { 0x0D627B90 }
        else if cfg!(feature = "psp_600") { 0x0DCB5BD2 }
        else if cfg!(feature = "psp_570") { 0x1C3A13B2 }
        else if cfg!(feature = "psp_500") { 0x0A218F56 }
        else if cfg!(feature = "psp_420") { 0x7BB2710E }
        else if cfg!(feature = "psp_395") { 0x77BABAF9 }
        else if cfg!(feature = "psp_380") { 0xB0DE19EC }
        else if cfg!(feature = "psp_370") { 0x312646B8 }
        else { 0xA88E8D22 }
    )]
    pub fn sceCtrlGetIdleCancelKey(
        make: Option<&mut PadButtons>, press: Option<&mut PadButtons>,
        hold_mode_make: Option<&mut PadButtons>, hold_mode_press: Option<&mut PadButtons>,
    ) -> SceResult<()>;

    /// Sets up internal controller buffers to receive external input data.
    ///
    /// Each input mode has its own set of buffers. These buffers are of type [`CtrlDataExt`].
    ///
    /// **Note:** This function has to be called initially in order to obtain external input data
    /// via the corresponding Peek/Read functions.
    ///
    /// # Parameters
    ///
    /// - `external_port`: The external controller port to set.
    /// - `transfer_handler` **[[In parameter]]**: A pointer the handle structure containing a
    ///   function to copy the `input_source` into the PSP's controller buffers.
    /// - `input_src` **[[InOut parameter]]**: A pointer to the buffer containing the Controller
    ///   input data to copy to the PSP's controller buffers. It is passed as the source argument to
    ///   the given [transfer function](InputDataTransferFn).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 5.70.
    ///
    /// # Function Name Warning
    ///
    /// This function name was never cracked, this name is a ludic name given to facilitate its
    /// usage in different firmware versions.
    #[nid(if cfg!(feature = "psp_660") { 0xE467BEC8 }
        else if cfg!(feature = "psp_630") { 0x65698764 }
        else if cfg!(feature = "psp_600") { 0xCB81B5DB }
        else { 0xFE6B7860 }
    )]
    pub unsafe fn sceCtrlSetInternalBufferHandler(
        external_port: ControllerPort, transfer_handle: *mut InputDataTransferHandle,
        input_src: *mut c_void,
    ) -> SceResult<()>;


    /// Emulates values for the analog pad's X- and Y-axis.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot used to set the custom values. If multiple slots are used, their settings
    ///   are combined.
    /// - `x`: The new emulated value for the X-axis.
    /// - `y`: The new emulated value for the Y-axis.
    /// - `make`: The duration of the emulation, measured in sampling counts.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware 5.70.
    #[nid(if cfg!(feature = "psp_660") { 0xDB76878D }
        else if cfg!(feature = "psp_630") { 0x094EF1BB }
        else if cfg!(feature = "psp_600") { 0xA759DB6A }
        else { 0xD8329216 }
    )]
    pub fn sceCtrlSetAnalogEmulation(
        slot: ButtonEmulationSlot, x: u8, y: u8, make: u32,
    ) -> SceResult<()>;

    /// Emulates buttons for the digital pad.
    ///
    /// # Parameters
    ///
    /// - `slot`: The slot used to set the custom values. If multiple slots are used, their settings
    ///   are combined.
    /// - `user_buttons`: The emulated user buttons. You cannot emulate kernel buttons and the
    ///   emulated buttons will only be applied for applications running in user mode.
    /// - `kernel_buttons`: The emulated buttons (both user and kernel buttons are valid). The
    ///   emulated buttons will only be applied for applications running in kernel mode.
    /// - `make`: The duration of the emulation, measured in sampling counts.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware 5.70.
    #[nid(if cfg!(feature = "psp_660") { 0x5130DAE3 }
        else if cfg!(feature = "psp_630") { 0x094EF1BB }
        else if cfg!(feature = "psp_600") { 0xA7D5A6BA }
        else { 0x010C3A1A }
    )]
    pub fn sceCtrlSetButtonEmulation(
        slot: ButtonEmulationSlot, user_buttons: PadButtons, kernel_buttons: PadButtons, make: u32,
    ) -> SceResult<()>;
}


impl crate::private::Sealed for InputMode {}
unsafe impl SceResultOk for InputMode {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0 => Ok(Self::DigitalOnly),
            1 => Ok(Self::DigitalAndAnalog),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}


impl SamplingCycle {
    /// When the sampling is set to this value, the input sampling occurs at the start of each
    /// VBlank period.
    pub const VBLANK_INTERRUPT_UPDATE: Self = unsafe { Self::new_unchecked(0) };

    /// Creates a new sampling cycle if it is one of the valid values.
    pub const fn new(raw: u32) -> Result<Self, SceError> {
        match raw {
            0 | 5555..=20000 => Ok(unsafe { Self::new_unchecked(raw) }),
            _ => Err(SceError::INVALID_VALUE),
        }
    }

    /// Creates a new sampling cycle without checking for it's validity.
    ///
    /// # Safety
    /// The value must be `0` or in the range of `[5555,20000]` (or, alternatively `[180Hz, 50Hz]`).
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(raw)
    }

    /// Gets the inner value of the sampling cycle value.
    #[inline]
    pub const fn to_inner(self) -> u32 {
        self.0
    }
}

impl crate::private::Sealed for SamplingCycle {}
unsafe impl SceResultOk for SamplingCycle {
    #[inline]
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        Self::new(ok_value)
    }
}

impl IdleCancelThreshold {
    /// When idle cancel threshold is set to this value, the idle timer is canceled even if the
    /// analog stick is not moved (i.e., the idle timer itself stops running).
    pub const ALWAYS_CANCEL: Self = unsafe { Self::new_unchecked(0) };
    /// The center value of the analog pad
    const CENTER: Self = unsafe { Self::new_unchecked(0x80) };
    /// When idle cancel threshold is set to this value, no cancellation the idle timer happens no
    /// matter the analog stick movement.
    pub const NO_CANCEL: Self = unsafe { Self::new_unchecked(-1) };

    /// Creates a new idle cancel threshold if it is one of the valid values.
    pub const fn new(raw: i32) -> Result<Self, SceError> {
        match raw {
            -1..=128 => Ok(unsafe { Self::new_unchecked(raw) }),
            _ => Err(SceError::INVALID_VALUE),
        }
    }

    /// Creates a new idle cancel threshold without checking for it's validity.
    ///
    /// # Safety
    /// The value must be `0` or in the range of `[5555,20000]` (or, alternatively `[180Hz, 50Hz]`).
    #[inline]
    pub const unsafe fn new_unchecked(raw: i32) -> Self {
        Self(raw)
    }

    /// Gets the inner value of the idle cancel threshold value.
    #[inline]
    pub const fn to_inner(self) -> i32 {
        self.0
    }
}

impl Default for IdleCancelThreshold {
    #[inline]
    fn default() -> Self {
        Self::CENTER
    }
}

crate::impl_ranged_ty!(RapidFireSlot);

impl RapidFireSlot {
    /// Rapid fire slot #00.
    pub const SLOT_00: Self = unsafe { Self::new_unchecked(0) };
    /// Rapid fire slot #01.
    pub const SLOT_01: Self = unsafe { Self::new_unchecked(1) };
    /// Rapid fire slot #02.
    pub const SLOT_02: Self = unsafe { Self::new_unchecked(2) };
    /// Rapid fire slot #03.
    pub const SLOT_03: Self = unsafe { Self::new_unchecked(3) };
    /// Rapid fire slot #04.
    pub const SLOT_04: Self = unsafe { Self::new_unchecked(4) };
    /// Rapid fire slot #05.
    pub const SLOT_05: Self = unsafe { Self::new_unchecked(5) };
    /// Rapid fire slot #06.
    pub const SLOT_06: Self = unsafe { Self::new_unchecked(6) };
    /// Rapid fire slot #07.
    pub const SLOT_07: Self = unsafe { Self::new_unchecked(7) };
    /// Rapid fire slot #08.
    pub const SLOT_08: Self = unsafe { Self::new_unchecked(8) };
    /// Rapid fire slot #09.
    pub const SLOT_09: Self = unsafe { Self::new_unchecked(9) };
    /// Rapid fire slot #10.
    pub const SLOT_10: Self = unsafe { Self::new_unchecked(10) };
    /// Rapid fire slot #11.
    pub const SLOT_11: Self = unsafe { Self::new_unchecked(11) };
    /// Rapid fire slot #12.
    pub const SLOT_12: Self = unsafe { Self::new_unchecked(12) };
    /// Rapid fire slot #13.
    pub const SLOT_13: Self = unsafe { Self::new_unchecked(13) };
    /// Rapid fire slot #14.
    pub const SLOT_14: Self = unsafe { Self::new_unchecked(14) };
    /// Rapid fire slot #15.
    pub const SLOT_15: Self = unsafe { Self::new_unchecked(15) };

    /// Create a new `RapidFireSlot` structure from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible SceUid
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=15 = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new `RapidFireSlot` structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for RapidFireSlot {}
impl core::fmt::Debug for RapidFireSlot {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("RapidFireSlot").field(&self.to_inner()).finish()
    }
}

crate::impl_ranged_ty!(SpecialButtonCallbackSlot);
impl SpecialButtonCallbackSlot {
    /// Special button callback slot #00.
    pub const SLOT_00: Self = unsafe { Self::new_unchecked(0) };
    /// Special button callback slot #01.
    pub const SLOT_01: Self = unsafe { Self::new_unchecked(1) };
    /// Special button callback slot #02.
    pub const SLOT_02: Self = unsafe { Self::new_unchecked(2) };
    /// Special button callback slot #03.
    pub const SLOT_03: Self = unsafe { Self::new_unchecked(3) };

    /// Create a new `RapidFireSlot` structure from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible SceUid
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u8) -> Option<Self> {
        if let 0..=3 = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new `RapidFireSlot` structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u8) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub const fn to_inner(self) -> u8 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for SpecialButtonCallbackSlot {}
impl core::fmt::Debug for SpecialButtonCallbackSlot {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("SpecialButtonCallbackSlot").field(&self.to_inner()).finish()
    }
}

crate::impl_ranged_ty!(ButtonEmulationSlot);
impl ButtonEmulationSlot {
    /// Button emulation slot #00.
    pub const SLOT_00: Self = unsafe { Self::new_unchecked(0) };
    /// Button emulation slot #01.
    pub const SLOT_01: Self = unsafe { Self::new_unchecked(1) };
    /// Button emulation slot #02.
    pub const SLOT_02: Self = unsafe { Self::new_unchecked(2) };
    /// Button emulation slot #03.
    pub const SLOT_03: Self = unsafe { Self::new_unchecked(3) };

    /// Create a new `ButtonEmulationSlot` structure from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible SceUid
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u8) -> Option<Self> {
        if let 0..=3 = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new `ButtonEmulationSlot` structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u8) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub const fn to_inner(self) -> u8 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for ButtonEmulationSlot {}
impl core::fmt::Debug for ButtonEmulationSlot {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("ButtonEmulationSlot").field(&self.to_inner()).finish()
    }
}

impl InputDataTransferHandle {
    /// Creates a new `InputDataTransferHandle` with the specified `func`
    #[inline]
    pub const fn new(func: InputDataTransferFn) -> Self {
        Self {
            unk: 0xC,
            copy_input_data: Some(func),
        }
    }
}

impl Default for InputDataTransferHandle {
    #[inline]
    fn default() -> Self {
        Self {
            unk: 0xC,
            copy_input_data: Default::default(),
        }
    }
}
