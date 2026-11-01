use core::{ffi::c_void, ptr::NonNull};

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceSize};

/// A valid font handle.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontHandle(NonNull<c_void>);

/// A valid font file handle.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontFile(NonNull<c_void>);

/// A valid font file handle.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontFileHandle(NonNull<c_void>);

/// Font initialization options.
#[repr(C)]
#[derive(Debug)]
#[doc(alias("SceFontLibData"))]
pub struct FontInitOptions {
    /// A pointer to user data.
    user_data: *mut u8,
    /// The maximum number of fonts that are open simultaneously.
    max_num_fonts: u32,
    /// The handle for the cache system.
    // FIXME: Actual type. (SceFontCache on C PSPSDK)
    cache: *mut c_void,
    /// A allocation function.
    alloc_fn: Option<unsafe extern "C" fn(user_data: *mut u8, size: SceSize) -> *mut c_void>,
    /// A deallocation function.
    free_fn: Option<unsafe extern "C" fn(user_data: *mut u8, ptr: *mut c_void) -> *mut c_void>,
    /// A read-only file opening function.
    open_fn: Option<
        unsafe extern "C" fn(
            user_data: *mut u8,
            filename: *const u8,
            error: &mut SceError,
        ) -> Option<FontFileHandle>,
    >,
    /// A file close function.
    close_fn: Option<
        unsafe extern "C" fn(user_data: *mut u8, file_handle: FontFileHandle) -> SceResult<()>,
    >,
    /// A file read function.
    read_fn: Option<
        unsafe extern "C" fn(
            user_data: *mut u8,
            file_handle: FontFileHandle,
            buf: *mut u8,
            size: SceSize,
            num: u32,
            error: &mut SceError,
        ) -> SceSize,
    >,
    /// A file seek function.
    seek_fn: Option<
        unsafe extern "C" fn(
            user_data: *mut u8,
            file_handle: FontFileHandle,
            offset: SceSize,
        ) -> SceResult<()>,
    >,
    /// On error function... seems unused.
    on_error_fn: Option<unsafe extern "C" fn(user_data: *mut u8, unk: i32) -> i32>,
    /// When read operation is done function... seems unused.
    when_done_read_fn: Option<unsafe extern "C" fn(user_data: *mut u8, unk: i32) -> i32>,
}

/// The possible font families.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontFamily", "SceFontFamilyCode"))]
pub enum FontFamily {
    /// System default font family.
    #[default]
    #[doc(alias("SCE_FONT_FAMILY_DEFAULT"))]
    Default = 0,
    /// A font of SanSerif family.
    #[doc(alias("SCE_FONT_FAMILY_SANS_SERIF"))]
    SanSerif = 1,
    /// A font of Serif family.
    #[doc(alias("SCE_FONT_FAMILY_SERIF"))]
    Serif = 2,
    /// A font of Rounded family. Never used.
    #[doc(alias("SCE_FONT_FAMILY_ROUNDED"))]
    Rounded = 3,
}

/// Possible font styles.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontStyles", "SceFontStylesCode"))]
pub enum FontStyles {
    /// System default font style.
    #[default]
    #[doc(alias("SCE_FONT_STYLE_DEFAULT"))]
    Default = 0,
    /// A regular font style.
    #[doc(alias("SCE_FONT_STYLE_REGULAR"))]
    Regular = 1,
    /// A italic font style.
    #[doc(alias("SCE_FONT_STYLE_ITALIC"))]
    Italic = 2,
    /// A thin font style.
    #[doc(alias("SCE_FONT_STYLE_THIN"))]
    Thin   = 3,
    /// A italic and thin font style.
    #[doc(alias("SCE_FONT_STYLE_ITALIC_THIN"))]
    ItalicThin = 4,
    /// A bold font style.
    #[doc(alias("SCE_FONT_STYLE_BOLD"))]
    Bold   = 5,
    /// A italic and bold font style.
    #[doc(alias("SCE_FONT_STYLE_ITALIC_BOLD"))]
    ItalicBold = 6,
    /// A thick font style.
    #[doc(alias("SCE_FONT_STYLE_THICK"))]
    Thick  = 7,
    /// A italic and thick font style.
    #[doc(alias("SCE_FONT_STYLE_ITALIC_THICK"))]
    ItalicThick = 8,
    L      = 101,
    M      = 102,
    DB     = 103,
    B      = 104,
    EB     = 105,
    UB     = 106,
}

/// The possible font languages.
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontLanguageCode", "SceFontLanguage"))]
pub enum FontLanguage {
    /// System default font language.
    #[default]
    #[doc(alias("SCE_FONT_LANGUAGE_DEFAULT"))]
    Default = 0,
    /// Japanese font language.
    #[doc(alias("SCE_FONT_LANGUAGE_JAPANESE"))]
    Japanese = 1,
    /// Latin font language.
    ///
    /// Used for more than one language that uses the Latin script.
    #[doc(alias("SCE_FONT_LANGUAGE_ENGLISH"))]
    Latin  = 2,
    /// Korean font language.
    #[doc(alias("SCE_FONT_LANGUAGE_KOREAN"))]
    Korean = 3,
    /// Chinese font language.
    #[doc(alias("SCE_FONT_LANGUAGE_CHINESE"))]
    Chinese = 4,
    /// Japanese/Korean/Chinese font language.
    #[doc(alias("SCE_FONT_LANGUAGE_JKC"))]
    Jkc    = 5,
}

/// The font sub-style used by some fonts.
#[bitflag(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FontSubStyle {
    /// Typeface with vertical layout.
    VerticalLayout = 0x1,
    /// Font with Pseudo Bold process applied.
    PseudoBold = 0x2,
    /// Font with Pseudo Italic process applied.
    PseudoItalic = 0x4,
}

/// Font possible regions.
///
/// # RE Note
///
/// It seems to be checked to be in 0 to 7 range, but no usage found.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[non_exhaustive]
pub enum FontRegion {
    /// Generic region.
    #[default]
    Generic = 0,
    /// Unknown or unused (?).
    Unk1 = 1,
    /// Unknown or unused (?).
    Unk2 = 2,
    /// Unknown or unused (?).
    Unk3 = 3,
    /// Unknown or unused (?).
    Unk4 = 4,
    /// Unknown or unused (?).
    Unk5 = 5,
    /// Unknown or unused (?).
    Unk6 = 6,
    /// Unknown or unused (?).
    Unk7 = 7,
}

/// The country code for the font vendor.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontVendorCountry"))]
pub enum FontVendorCountry {
    /// A generic vendor country value.
    #[default]
    #[doc(alias("SCE_FONT_VENDOR_COUNTRY_DEFAULT"))]
    Generic = 0,
    /// Japan.
    #[doc(alias("SCE_FONT_VENDOR_COUNTRY_JAPAN"))]
    Japan = 1,
    /// United States of America.
    #[doc(alias("SCE_FONT_VENDOR_COUNTRY_USA"))]
    Usa   = 2,
    /// Korea.
    #[doc(alias("SCE_FONT_VENDOR_COUNTRY_KOREA"))]
    Korea = 3,
}

/// A type that holds a font information.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[doc(alias("SceFontStyle"))]
pub struct FontInfo {
    /// The width in points (`1/72` inch).
    ///
    /// Zero is the system standard.
    pub width: f32,
    /// The height in points (`1/72` inch).
    ///
    /// Zero is the system standard.
    pub height: f32,
    /// The horizontal resolution in dpi.
    ///
    /// Zero is the system standard.
    pub horizontal_dpi: f32,
    /// The vertical resolution in dpi.
    ///
    /// Zero is the system standard.
    pub vertical_dpi: f32,
    /// The font weight value.
    pub weight: f32,
    /// The font family.
    pub family: FontFamily,
    /// The font style.
    pub style: FontStyles,
    /// The font sub-style.
    pub sub_style: FontSubStyle,
    /// The font language.
    pub language: FontLanguage,
    /// The font region.
    pub region: FontRegion,
    /// The font vendor country.
    pub country: FontVendorCountry,
    /// The font name.
    pub name: [u8; 64],
    /// The font filename.
    pub filename: [u8; 64],
    /// The font attributes.
    pub attributes: u32,
    /// The font expiration date. Not Used (?).
    pub expire_date: u32,
}

/// The possible open mode for fonts.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum FontMode {
    /// File based stream.
    #[default]
    FileStream = 0,
    /// Memory based stream.
    MemStream = 1,
}

/// Represents in formation of a glyph image of a character.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontCharacterInfo"))]
pub struct CharacterInfo {
    /// The horizontal size of the character in pixels.
    pub width: u32,
    /// The vertical size of the character in pixels.
    pub height: u32,
    /// The character offset in the `X` axis direction from the origin of the left edge of the
    /// character in pixels.
    pub x: i32,
    /// The character offset in the `Y` axis direction from the origin of the top edge of the
    /// character in pixels.
    pub y: i32,
    /// The character glyph information.
    pub glyph_info: GlyphInfo,
    /// Not used.
    pub reserved: [u8; 4],
}

/// Represents a glyph information for a character.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontCharacterData"))]
pub struct GlyphInfo {
    /// The character width in units of `1/64` pixels.
    pub width: u32,
    /// The character height in units of `1/64` pixels.
    pub height: u32,
    /// The character ascender in units of `1/64` pixels.
    pub ascender: i32,
    /// The character descender in units of `1/64` pixels.
    pub descender: i32,
    /// The bearing value in the `X` axis for horizontal characters in units of `1/64` pixels.
    pub horizontal_bearing_x: i32,
    /// The bearing value in the `Y` axis for horizontal characters in units of `1/64` pixels.
    pub horizontal_bearing_y: i32,
    /// The bearing value in the `Y` axis for vertical characters in units of `1/64` pixels.
    pub vertical_bearing_x: i32,
    /// The bearing value in the `Y` axis for vertical characters in units of `1/64` pixels.
    pub vertical_bearing_y: i32,
    /// The advance in the `X` axis for horizontal characters in units of `1/64` pixels.
    pub horizontal_advance: i32,
    /// The advance in the `Y` axis for vertical characters in units of `1/64` pixels.
    pub vertical_advance: i32,
}

/// Represents the rectangle data used by a character of a font.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontRect"))]
pub struct CharacterRectangle {
    /// The width of the rectangle.
    pub width: u16,
    /// The width of the rectangle.
    pub height: u16,
}

/// The character glyph buffer.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc(alias("SceFontImageBuffer"))]
pub struct GlyphBuffer {
    /// The glyph pixel format.
    pub pixel_format: PixelFormat,
    /// The horizontal position for writing.
    pub x_pos: i32,
    /// The vertical position for writing.
    pub y_pos: i32,
    /// The glyph rectangle information.
    pub size: CharacterRectangle,
    /// The number of bytes per horizontal line of the image buffer.
    pub bytes_per_line: u16,
    /// Unused, set to `0`.
    pub reserved: u16,
    /// The buffer that holds the glyph image.
    ///
    /// # Safety
    /// Make sure to have it allocated at least `bytes_per_line * size.height` bytes.
    pub data_buf: *mut u8,
}

/// Possible font pixel formats.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum PixelFormat {
    /// This formats as 2 pixels in one byte (8 bits).
    ///
    /// The higher half is to be placed at the left side of the screen. The lower half is to be
    /// placed at the right side of the screen.
    ///
    /// `0x0` means minimal brightness, and `0xF` means maximum brightness.
    #[default]
    L4    = 0,
    /// Probably the oposite of `L4`, but not used.
    R4    = 1,
    /// This formats as 1 pixel in one byte (8 bits).
    ///
    /// `0x00` means minimal brightness, and `0xFF` means maximum brightness.
    Byte  = 2,
    /// Probably 24bit pixels, not used.
    Bit24 = 3,
    /// Probably 32bit pixels, not used.
    Bit32 = 4,
}

#[psp_stub(libname = "sceLibFont", flags = 0x4009, use_crate)]
extern "C" {

    /// Initialize a font library.
    ///
    /// # Parameters
    ///
    /// - `options` **[[InOut parameter]]**: A reference to the font initialization options.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the font handle on success, `None` otherwise.
    #[nid(0x67F17ED7)]
    pub safe fn sceFontNewLib(
        options: &FontInitOptions, error: &mut SceError,
    ) -> Option<FontHandle>;

    /// Terminates a font handle.
    ///
    /// This forcibly closes all fonts that remain open, releases memory allocations in relation to
    /// the specified handle instance.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle to terminate.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x574B6FBC)]
    pub safe fn sceFontDoneLib(handle: FontHandle) -> SceResult<()>;

    /// Sets the font resolution.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `horizontal_dpi`: The horizontal dpi resolution.
    /// - `vertical_dpi`: The vertical dpi resolution.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x48293280)]
    pub safe fn sceFontSetResolution(
        handle: FontHandle, horizontal_dpi: f32, vertical_dpi: f32,
    ) -> SceResult<()>;

    /// Gets the number of all available font lists
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the number of available font lists success, zero otherwise.
    #[nid(0x27F6E642)]
    pub safe fn sceFontGetNumFontList(handle: FontHandle, error: &mut SceError) -> u32;

    /// Gets the available fonts.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `infos` **[[Out parameter]]**: A pointer to an array to receive the available fonts
    ///   information.
    /// - `infos_size`: The size of `infos` array.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xBC75D85B)]
    pub unsafe fn sceFontGetFontList(
        handle: FontHandle, infos: *mut FontInfo, infos_size: SceSize,
    ) -> SceResult<()>;

    /// Finds the optimal font based on a font information.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `info` **[[In parameter]]**: A reference to a font information.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the font index on success, zero otherwise.
    #[nid(0x099EF33C)]
    pub safe fn sceFontFindOptimumFont(
        handle: FontHandle, info: &FontInfo, error: &mut SceError,
    ) -> SceSize;

    /// Finds a font that matches a font information.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `info` **[[In parameter]]**: A reference to a font information.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the font index on success, zero otherwise.
    #[nid(0x099EF33C)]
    pub safe fn sceFontFindFont(
        handle: FontHandle, info: &FontInfo, error: &mut SceError,
    ) -> SceSize;

    /// Gets the font information from the font index number.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `info` **[[Out parameter]]**: A reference to receive a font information.
    /// - `index`: The font index to get the the font information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5333322D)]
    pub safe fn sceFontGetFontInfoByIndexNumber(
        handle: FontHandle, info: &mut FontInfo, index: SceSize,
    ) -> SceResult<()>;

    /// Opens a new font file.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `index`: The font index (`0` for default).
    /// - `mode`: The font mode.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the font file handle on success, [`None`] otherwise.
    #[nid(0xA834319D)]
    pub safe fn sceFontOpen(
        handle: FontHandle, index: SceSize, mode: FontMode, error: &mut SceError,
    ) -> Option<FontFile>;

    /// Opens a new local font file.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `path` **[[In parameter]]**: The full path to the font.
    /// - `mode`: The font mode.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the font file handle on success, [`None`] otherwise.
    #[nid(0x57FCB733)]
    pub unsafe fn sceFontOpenUserFile(
        handle: FontHandle, filename: *const u8, mode: FontMode, error: &mut SceError,
    ) -> Option<FontFile>;

    /// Opens a new font file from an address.
    ///
    /// # Parameters
    ///
    /// - `handle`: The font handle.
    /// - `addr` **[[In parameter]]**: The address where the font data is located.
    /// - `size`: The size of the font data.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the font file handle on success, [`None`] otherwise.
    #[nid(0xBB8E7FE6)]
    pub unsafe fn sceFontOpenUserMemory(
        handle: FontHandle, addr: *mut c_void, size: SceSize, error: &mut SceError,
    ) -> Option<FontFile>;

    /// Closes a opened font file.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file to close.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3AEA8CB6)]
    pub safe fn sceFontClose(file: FontFile) -> SceResult<()>;

    /// Flushes a font cache.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file to flush.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x02D7F94B)]
    pub safe fn sceFontFlush(file: FontFile) -> SceResult<()>;

    /// Gets the font information from a font file.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `info` **[[Out parameter]]**: A reference to receive the font information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0DA7535E)]
    pub safe fn sceFontGetFontInfo(file: FontFile, info: &mut FontInfo) -> SceResult<()>;

    /// Gets a character information from a font.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `char_code`: The character code (UCS2) to get the information.
    /// - `info` **[[Out parameter]]**: A reference to receive the character information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDCC80C2F)]
    pub safe fn sceFontGetCharInfo(
        file: FontFile, char_code: u16, info: &mut CharacterInfo,
    ) -> SceResult<()>;

    /// Gets the size of a character imaged rectangle.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `char_code`: The character code (UCS2) to get the information.
    /// - `rectangle` **[[Out parameter]]**: A reference to receive the character rectangle
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5C3E4A9E)]
    pub safe fn sceFontGetCharImageRect(
        file: FontFile, char_code: u16, rectangle: &mut CharacterRectangle,
    ) -> SceResult<()>;

    /// Gets the glyph image of a character of a font.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `char_code`: The character code (UCS2) to get the information.
    /// - `glyph_buff` **[[Out parameter]]**: A reference to receive the character glyph buffer.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x980F4895)]
    pub safe fn sceFontGetCharGlyphImage(
        file: FontFile, char_code: u16, glyph_buff: &mut GlyphBuffer,
    ) -> SceResult<()>;

    /// Gets the clip of glyph image of a character of a font.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `char_code`: The character code (UCS2) to get the information.
    /// - `glyph_buff` **[[Out parameter]]**: A reference to receive the character glyph buffer.
    /// - `clip_x` : The horizontal position of clipping rectangle.
    /// - `clip_y` : The vertical position of clipping rectangle.
    /// - `clip_width` : Clipping rectangle width.
    /// - `clip_height` : Clipping rectangle height.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCA1E6945)]
    pub safe fn sceFontGetCharGlyphImage_Clip(
        file: FontFile, char_code: u16, glyph_buff: &mut GlyphBuffer, clip_x: i32, clip_y: i32,
        clip_width: u32, clip_height: u32,
    ) -> SceResult<()>;

    /// Converts pixel units to points in the horizontal direction.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `pixel`: The value in pixels.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the value in points in the horizontal direction.
    #[nid(0x74B21701)]
    pub safe fn sceFontPixelToPointH(file: FontFile, pixel: f32, error: &mut SceError) -> f32;

    /// Converts pixel units to points in the vertical direction.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `pixel`: The value in pixels.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the value in points in the vertical direction.
    #[nid(0xF8F0752E)]
    pub safe fn sceFontPixelToPointV(file: FontFile, pixel: f32, error: &mut SceError) -> f32;

    /// Converts points in the horizontal direction to pixels.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `point`: The value in points in the horizontal direction.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the value in pixels.
    #[nid(0x472694CD)]
    pub safe fn sceFontPointToPixelH(file: FontFile, point: f32, error: &mut SceError) -> f32;

    /// Converts points in the vertical direction to pixels.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `point`: The value in points in the vertical direction.
    /// - `error` **[[Out parameter]]**: A reference to receive the error value if an error
    ///   happened.
    ///
    /// # Return Value
    ///
    /// Returns the value in pixels.
    #[nid(0x3C4B7E82)]
    pub safe fn sceFontPointToPixelV(file: FontFile, point: f32, error: &mut SceError) -> f32;

    /// Set a fallback character.
    ///
    /// # Parameters
    ///
    /// - `file`: The font file.
    /// - `char_code`: The character code (UCS2) to get the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEE232411)]
    pub safe fn sceFontSetAltCharacterCode(file: FontFile, code: u16) -> SceResult<()>;
}

impl FontHandle {
    /// Create new font ID from raw.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub const fn new(raw: *mut c_void) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(unsafe { NonNull::new_unchecked(raw) }))
        }
    }

    /// Creates new font ID from raw without checks.
    ///
    /// # Safety
    /// The `raw` must not be null.
    pub const unsafe fn new_unchecked(raw: *mut c_void) -> Self {
        Self(unsafe { NonNull::new_unchecked(raw) })
    }
}

impl FontFile {
    /// Create new font ID from raw.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub const fn new(raw: *mut c_void) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(unsafe { NonNull::new_unchecked(raw) }))
        }
    }

    /// Creates new font ID from raw without checks.
    ///
    /// # Safety
    /// The `raw` must not be null.
    pub const unsafe fn new_unchecked(raw: *mut c_void) -> Self {
        Self(unsafe { NonNull::new_unchecked(raw) })
    }
}

impl FontFileHandle {
    /// Create new font ID from raw.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub const fn new(raw: *mut c_void) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(unsafe { NonNull::new_unchecked(raw) }))
        }
    }

    /// Creates new font ID from raw without checks.
    ///
    /// # Safety
    /// The `raw` must not be null.
    pub const unsafe fn new_unchecked(raw: *mut c_void) -> Self {
        Self(unsafe { NonNull::new_unchecked(raw) })
    }
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            horizontal_dpi: Default::default(),
            vertical_dpi: Default::default(),
            weight: Default::default(),
            family: Default::default(),
            style: Default::default(),
            sub_style: Default::default(),
            language: Default::default(),
            region: Default::default(),
            country: Default::default(),
            name: [0; 64],
            filename: [0; 64],
            attributes: Default::default(),
            expire_date: Default::default(),
        }
    }
}
