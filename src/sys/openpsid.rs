//! PS ID operations.
#![allow(unused_imports)]

use pspsdk_macros::psp_stub;

use crate::sys::SceError;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("PspOpenPSID", "SceOpenPSID"))]
pub struct OpenPSID {
    pub data: [u8; 16],
}


#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceProductCode"))]
pub struct ProductCode {
    pub unk: [u8; 2],
}

#[psp_stub(libname = "sceOpenPSID", flags = 0x4001, version = (0x00, 0x11), use_crate)]
unsafe extern "C" {
    /// Get the open PS ID.
    ///
    /// # Parameters
    ///
    /// - `openpsid` **[[Out parameter]]**: A reference to a [`OpenPSID`] to receive the data.
    ///
    /// # Return Values
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xC69BEBCE)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceOpenPSIDGetOpenPSID(openpsid: &mut OpenPSID) -> Result<(), SceError>;

    /// Get the product code.
    ///
    /// # Parameters
    ///
    /// - `product_code` **[[Out parameter]]**: A reference to a [`ProductCode`] to receive the
    ///   data.
    ///
    /// # Return Values
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB29330DE)]
    pub safe fn sceOpenPSIDGetProductCode(product_code: &mut ProductCode) -> Result<(), SceError>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceOpenPSID_driver", flags = 0x0001, version = (0x00, 0x11), use_crate)]
unsafe extern "C" {
    /// Get the open PS ID.
    ///
    /// # Parameters
    ///
    /// - `openpsid` **[[Out parameter]]**: A reference to a [`OpenPSID`] to receive the data.
    ///
    /// # Return Values
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xC69BEBCE)]
    pub safe fn sceOpenPSIDGetOpenPSID(openpsid: &mut OpenPSID) -> Result<(), SceError>;
}
