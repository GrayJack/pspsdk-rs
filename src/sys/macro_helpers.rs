#[cfg(target_os = "psp")]
use crate::sys::library::SceStubLibraryEntry;

/// A "function" stub.
///
/// This is a very dirty trick for LTO. Essentially, the PSP OS takes the
/// address of a stub and inserts 2 instructions (8 bytes) which are `jr $ra`
/// and `syscall xxx`. Traditionally, the C/C++ toolchain stores the initial
/// data here as just an empty function that immediately returns (8 bytes,
/// `jr $ra; nop`). However, we use it to store 2 references: to the NID and to
/// the library stub.
///
/// This results in LTO builds compiling in the NID and library stubs whenever a
/// function is called, as this struct references them. Thus, this struct
/// definition creates a dependency between the function call, and the NID + lib
/// stub. As mentioned earlier, these two addresses (8 bytes) are replaced with
/// two instructions (also 8 bytes) at runtime, so they are not actually called
/// as a function. With this method, nothing has to be marked `#[used]`, so LLVM
/// can automatically remove unreferenced NIDs and library stubs during LTO.
/// Compiling with LTO then only links the functions that are called, and no
/// more.
#[cfg(target_os = "psp")]
#[derive(Copy, Clone)]
pub(crate) struct Stub {
    // These are never read, but need to be written into as static items.
    #[allow(dead_code)]
    pub(crate) lib_addr: &'static SceStubLibraryEntry,
    #[allow(dead_code)]
    pub(crate) nid_addr: &'static u32,
}

/// Calculate the padded length for a library name.
///
/// The name is padded on the end with zeroes. Must be at least one and a
/// multiple of 4.
#[cfg(target_os = "psp")]
pub const fn lib_name_bytes_len(name: &str) -> usize {
    let name_len = name.len();
    name_len + (4 - name_len % 4)
}

/// Convert a library name to a byte array.
///
/// This is intended to be used with `lib_name_bytes_len`.
#[cfg(target_os = "psp")]
pub const fn lib_name_bytes<const T: usize>(name: &str) -> [u8; T] {
    let mut buf = [0; T];

    let name_bytes = name.as_bytes();
    let mut i = 0;

    while i < name_bytes.len() {
        buf[i] = name_bytes[i];
        i += 1;
    }

    buf
}
