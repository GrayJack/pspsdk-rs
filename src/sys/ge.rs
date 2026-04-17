use pspsdk_macros::psp_stub;

use crate::sys::SceSize;


#[psp_stub(libname = "sceGe_user", flags = 0x4001, use_crate)]
extern "C" {
    /// Gets the size of eDRAM.
    ///
    /// # Return Value
    ///
    /// Returns the size of the eDRAM in bytes.
    #[nid(0x1F6752AD)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceGeEdramGetSize() -> SceSize;

    /// Gets the physical eDRAM address.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the start of the eDRAM.
    #[nid(0xE47E40E4)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceGeEdramGetAddr() -> *mut u8;
}

#[psp_stub(libname = "sceGe_driver", flags = 0x0001, use_crate)]
extern "C" {
    /// Gets the size of eDRAM.
    ///
    /// # Return Value
    ///
    /// Returns the size of the eDRAM in bytes.
    #[nid(if cfg!(feature = "psp_660") { 0x5D28A52B }
        else if cfg!(feature = "psp_630") { 0x670F15ED }
        else if cfg!(feature = "psp_600") { 0xF489E74B }
        else if cfg!(feature = "psp_570") { 0xA2A7CB01 }
        else if cfg!(feature = "psp_500") { 0x6663F1BD }
        else if cfg!(feature = "psp_420") { 0x5731D2C6 }
        else if cfg!(feature = "psp_395") { 0xF5423CAA }
        else if cfg!(feature = "psp_380") { 0x8B2CE4CE }
        else if cfg!(feature = "psp_370") { 0xBA035FC8 }
        else { 0x1F6752AD }
    )]
    pub fn sceGeEdramGetSize() -> SceSize;

    /// Gets the physical eDRAM address.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the start of the eDRAM.
    #[nid(if cfg!(feature = "psp_660") { 0x7D662DF2 }
        else if cfg!(feature = "psp_630") { 0x40F72852 }
        else if cfg!(feature = "psp_600") { 0x9919BBE3 }
        else if cfg!(feature = "psp_570") { 0x0A9D78C5 }
        else if cfg!(feature = "psp_500") { 0x6F13CB16 }
        else if cfg!(feature = "psp_420") { 0x67A1FAB5 }
        else if cfg!(feature = "psp_395") { 0x84DF53A6 }
        else if cfg!(feature = "psp_380") { 0x3AC43ABD }
        else if cfg!(feature = "psp_370") { 0xC812D69E }
        else { 0xE47E40E4 }
    )]
    pub fn sceGeEdramGetAddr() -> *mut u8;
}
