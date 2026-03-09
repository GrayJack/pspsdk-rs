use pspsdk_macros::psp_stub;

use crate::sys::SceError;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceOpenPSID {
    pub data: [u8; 16],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceProductCode {
    pub unk: [u8; 2],
}

#[psp_stub(libname = "sceOpenPSID", flags = 0x4001, version = (0x00, 0x11))]
extern "C" {
    /// Get the open PS ID.
    ///
    /// # Parameters
    ///
    /// - `openpsid` **[[Out parameter]]**: A reference to a [`SceOpenPSID`] to receive the data.
    ///
    /// # Return Values
    ///
    /// [`Ok`] value on success, error value otherwise.
    #[cfg(not(feature = "kernel"))]
    pub fn sceOpenPSIDGetOpenPSID(openpsid: &mut SceOpenPSID) -> Result<(), SceError>;

    /// Get the product code.
    ///
    /// # Parameters
    ///
    /// - `product_code` **[[Out parameter]]**: A reference to a [`SceProductCode`] to receive the
    ///   data.
    ///
    /// # Return Values
    ///
    /// [`Ok`] value on success, error value otherwise.
    pub fn sceOpenPSIDGetProductCode(product_code: &mut SceProductCode) -> Result<(), SceError>;
}

#[psp_stub(libname = "sceOpenPSID_driver", flags = 0x0001, version = (0x00, 0x11))]
#[cfg(feature = "kernel")]
extern "C" {
    /// Get the open PS ID.
    ///
    /// # Parameters
    ///
    /// - `openpsid` **[[Out parameter]]**: A reference to a [`SceOpenPSID`] to receive the data.
    ///
    /// # Return Values
    ///
    /// [`Ok`] value on success, error value otherwise.
    pub fn sceOpenPSIDGetOpenPSID(openpsid: &mut SceOpenPSID) -> Result<(), SceError>;
}
