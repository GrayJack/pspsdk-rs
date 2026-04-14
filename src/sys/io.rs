use core::{ffi::c_void, mem, ptr::NonNull};

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{
    thread::{CallbackId, EventFlagId, SemaId, ThreadId},
    time::DateTime,
    SceError, SceResult, SceResultOk, SceSize, SceUid,
};

/// File descriptor UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(SceUid);

/// Flags that control how a file is opened and its subsequent behavior.
///
/// The flags can roughly be divided in 3 groups:
/// - **File access mode flags:** These flags specify the access allowed in the file.
/// - **Open mode flags:** These flags control the behavior of the open operations.
///     - These flags can only take effect in open operations. Usually ignored or error on other
///       operations.
/// - **I/O operation mode flags:** These flags control the behavior of the I/O operations (read,
///   write, etc)
///     - These flags are set by open operations.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum FileFlags {
    /// File access mode flag. Marks that the file is open for reads only.
    #[doc(alias("SCE_O_RDONLY", "SCE_FREAD", "PSP_O_RDONLY"))]
    ReadOnly = 0x0001,
    /// File access mode flag. Marks that the file is open for writes only.
    #[doc(alias("SCE_O_WRONLY", "SCE_FWRITE", "PSP_O_WRONLY"))]
    WriteOnly = 0x0002,
    /// File access mode flag. Marks that the file is open for reads and writes.
    #[doc(alias("SCE_O_RDWR", "PSP_O_RDWR"))]
    ReadWrite = 0x0003,
    /// I/O operation mode flag. Non-blocking mode.
    ///
    /// Not used/reserved.
    #[doc(alias("SCE_O_NBLOCK", "SCE_FNBLOCK", "PSP_O_NBLOCK"))]
    NonBlock = 0x0004,
    /// I/O operation mode flag. Directory operation mode.
    ///
    /// This flag is only used internally by [`sceIoDopen`].
    #[doc(alias("SCE_O_DIR", "SCE_FDIRO", "PSP_O_DIR"))]
    DirectoryMode = 0x0008,
    /// Unknown mode flag. Read locked (non-shared) (?).
    ///
    /// Not used/reserved.
    #[doc(alias("SCE_FRLOCK"))]
    ReadLocked = 0x0010,
    /// Unknown mode flag. Write locked (non-shared) (?).
    ///
    /// Not used/reserved.
    #[doc(alias("SCE_FWLOCK"))]
    WriteLocked = 0x0020,
    /// I/O operation mode flag. Append mode.
    ///
    /// If this flags is set, the file offset is set to the end of the file prior to each write
    /// operation on the file.
    #[doc(alias("SCE_O_APPEND", "SCE_FAPPEND", "PSP_O_APPEND"))]
    AppendMode = 0x0100,
    /// Open mode flag. Create file if it does not exist.
    ///
    /// This flag has no effect if set and the file exist, unless the [`Exclusive`] flags is also
    /// set, on which case the open operation will fail if the file exists.
    ///
    /// [`Exclusive`]: Self::Exclusive
    #[doc(alias("SCE_O_CREAT", "SCE_FCREAT", "PSP_O_CREAT"))]
    CreateFile = 0x0200,
    /// Open mode flag. Truncate existing file to zero length.
    ///
    /// This flag only has effect on regular files.
    ///
    /// If this flag is set and the file is a regular file, truncates the file to a length of zero
    /// (`0`).
    #[doc(alias("SCE_O_TRUNC", "SCE_FTRUNC", "PSP_O_TRUNC"))]
    Truncate = 0x0400,
    /// Open mode flag. Exclusive file creation.
    ///
    /// If this flag is set with the [`CreateFile`], an open operation will fail if the file
    /// already exists.
    ///
    /// [`CreateFile`]: Self::CreateFile
    #[doc(alias("SCE_O_EXCL", "SCE_EXCL", "PSP_O_EXCL"))]
    Exclusive = 0x0800,
    /// Unknown mode flag. Scan type (?).
    ///
    /// Not used/reserved.
    #[doc(alias("SCE_FSCAN"))]
    Scan   = 0x1000,
    /// Unknown mode flag. Remote command entry (?).
    ///
    /// Not used/reserved.
    #[doc(alias("SCE_FRCOM"))]
    RemoteCommand = 0x2000,
    /// Open mode flag. Do not use device buffer and console interrupt.
    ///
    /// Not used/reserved.
    #[doc(alias("SCE_FNBUF"))]
    NoBuffer = 0x4000,
    /// Open mode flag. No wait (?)
    ///
    /// It is set in some titles I/O operations, but no reference to it on the reversed code on I/O
    /// functions.
    #[doc(alias("SCE_O_NOWAIT", "SCE_FASYNC", "PSP_O_NOWAIT"))]
    NoWait = 0x8000,
    /// I/O operation mode flag. Exclusive access mode.
    ///
    /// When set, only the the same thread that opened the file descriptor or the async thread of
    /// the file descriptor can execute operations on the file.
    #[doc(alias("SCE_O_FDEXCL", "SCE_FFDEXCL"))]
    ExclusiveAccess = 0x01000000,
    /// I/O operation mode flag. Power locked mode.
    ///
    /// When set, all power tick timer will be locked from triggering while the file descriptor is
    /// opened.
    #[doc(alias("SCE_O_PWLOCK", "SCE_FPWLOCK"))]
    PowerLock = 0x02000000,
    /// Open mode flag. Encrypted mode.
    ///
    /// The file uses Kernel/DNAS/NPDRM-encryption.
    #[doc(alias("SCE_O_ENCRYPTED", "SCE_FENCRYPTED"))]
    Encrypted = 0x04000000,
    /// Open mode flag. Global File descriptor mode.
    ///
    /// Attempts to open the file and save the UID in the system global File
    /// descriptor UID list.
    ///
    /// When set, the return value for the [`sceIoOpen`] and [`sceIoOpenAsync`] will
    /// be the index in the list. The real UID then can be retrieved with
    /// [`sceIoGetUID`].
    GlobalFdIndex = 0x08000000,
    /// I/O operation mode flag. DRM protected mode.
    ///
    /// When set, specifies that the opened file descriptor references a file that is DRM
    /// protected.
    ///
    /// Trying to open DRM protected files without this flag will cause errors. Other checks
    /// probably happen on the DRM specific functions that require this flag and more.
    #[doc(alias("SCE_O_FGAMEDATA", "SCE_FGAMEDATA"))]
    DrmProtected = 0x40000000,
}

/// Typed representation of file modes, combining file type and permissions.
///
/// This bitflag type represents the possible file types and permissions that can be associated
/// with a file.
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceMode"))]
pub enum Mode {
    /// Permission flag. Owner read permission.
    #[doc(alias = "S_IRUSR")]
    OwnerRead = 0x0100,
    /// Permission flag. Owner write permission.
    #[doc(alias = "S_IWUSR")]
    OwnerWrite = 0x0080,
    /// Permission flag. Owner execute/search permission.
    #[doc(alias = "S_IXUSR")]
    OwnerExec = 0x0040,
    /// Permission flag. Owner read, write, and execute/search permissions.
    #[doc(alias = "S_IRWXU")]
    OwnerAll = 0x01C0,
    /// Permission flag. Group read permission.
    #[doc(alias = "S_IRGRP")]
    GroupRead = 0x0020,
    /// Permission flag. Group write permission.
    #[doc(alias = "S_IWGRP")]
    GroupWrite = 0x0010,
    /// Permission flag. Group execute/search permission.
    #[doc(alias = "S_IXGRP")]
    GroupExec = 0x0008,
    /// Permission flag. Group read, write, and execute/search permissions.
    #[doc(alias = "S_IRWXG")]
    GroupAll = 0x0038,
    /// Permission flag. Others read permission.
    #[doc(alias = "S_IROTH")]
    OtherRead = 0x0004,
    /// Permission flag. Others write permission.
    #[doc(alias = "S_IWOTH")]
    OtherWrite = 0x0002,
    /// Permission flag. Others execute/search permission.
    #[doc(alias = "S_IXOTH")]
    OtherExec = 0x0001,
    /// Permission flag. Others read, write, and execute/search permissions.
    #[doc(alias = "S_IRWXO")]
    OtherAll = 0x0007,

    /// Special attribute flag. Set-user-ID on execution.
    #[doc(alias = "S_ISUID")]
    IsSetUid = 0x0800,
    /// Special attribute flag. Set-group-ID on execution.
    #[doc(alias = "S_ISGID")]
    IsSetGid = 0x0400,
    /// Special attribute flag. On directories, restricted deletion flag.
    #[doc(alias = "S_ISVTX")]
    IsRestrictDeletion = 0x0200,

    /// File kind flag. File type of Directory file.
    #[doc(alias = "S_IFDIR")]
    IsDir  = 0x1000,
    /// File kind flag. File type of Regular file.
    #[doc(alias = "S_IFREG")]
    IsRegular = 0x2000,
    /// File kind flag. File type of symbolic Link file.
    #[doc(alias = "S_IFLNK")]
    IsLink = 0x4000,
    /// Bitmask for the file type on [`Mode`].
    #[doc(alias = "S_IFMT")]
    FileKind = 0xF000,
}

/// Possible methods to seek within an file object.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Whence {
    /// Seek offset from the start of the file.
    #[doc(alias("SCE_SEEK_SET", "SEEK_SET"))]
    Start = 0,
    /// Seek offset from the current internal position of the file.
    #[doc(alias("SCE_SEEK_CUR", "SEEK_CUR"))]
    Current = 1,
    /// Seek offset from the end of the file.
    #[doc(alias("SCE_SEEK_END", "SEEK_END"))]
    End   = 2,
}

/// A single directory entry.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceIoDirent"))]
pub struct Dirent {
    /// The file status
    #[doc(alias("d_stat"))]
    pub stat: Stat,
    /// The file name.
    #[doc(alias("d_name"))]
    pub name: [u8; 256],
    /// Device specific data.
    #[doc(alias("d_private"))]
    pub private: *mut DirentFatPrivate,
    pub dummy: u32,
}

/// Private information of a directory (specific to FAT filesystem, which is the only one the PSP
/// has support anyway).
#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirentFatPrivate {
    pub size: SceSize,
    pub short_name: [u8; 13],
    _padding: [u8; 3],
    pub long_name: [u8; 1024],
}

/// The status information of a file.
#[repr(C)]
#[doc(alias("SceIoStat"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stat {
    /// The file permissions.
    pub mode: Mode,
    /// The file status attributes.
    pub attr: StatAttribute,
    /// The size of the file in bytes.
    pub size: u64,
    /// Creation time.
    pub creation_time: DateTime,
    /// Access time.
    pub access_time: DateTime,
    /// Modification time.
    pub modification_time: DateTime,
    /// Device specific data.
    pub private: [u32; 6],
}

/// Attributes of the file [`Stat`].
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StatAttribute {
    /// Symlink.
    Link = 0x0008,
    /// Directory.
    Directory = 0x0010,
    /// Regular file.
    RegularFile = 0x0020,
    /// Hidden read permission.
    ReadPermission = 0x0004,
    /// Hidden write permission.
    WritePermission = 0x0002,
    /// Hidden execution permission.
    ExecPermission = 0x0001,
}

/// What field of stat to change on [`sceIoChstat`].
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChangeStatFlag {
    /// Changes [`Stat::mode`].
    Mode = 0x0001,
    /// Changes [`Stat::attr`].
    Attribute = 0x0002,
    /// Changes [`Stat::size`].
    Size = 0x0004,
    /// Changes [`Stat::creation_time`].
    CreationTime = 0x0008,
    /// Changes [`Stat::access_time`].
    AccessTime = 0x0010,
    /// Changes [`Stat::modification_time`].
    ModificationTime = 0x0020,
    /// Changes [`Stat::private`].
    Private = 0x0040,
}

/// Mount flags used on [`sceIoAssign`].
#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssignFlag {
    /// Mounts as read/write enabled.
    ReadWrite = 0x00,
    /// Mounts as read-only.
    ReadOnly = 0x01,
    /// Mount in ROBUST mode.
    Robust = 0x02,
    /// Set an error if there is anything abnormal in the file system when mounting.
    ErrorCheck = 0x04,
}

/// Drive device information to add with [`sceIoAddDrv`].
#[repr(C)]
#[doc(alias("PspIoDrv", "SceIoDeviceTable"))]
pub struct DriveDevice {
    /// The name of the device to add.
    pub name: *const u8,
    /// The device kind.
    pub kind: DeviceKind,
    /// Unknown.
    ///
    /// ## Known values
    /// - `0x001`: Unknown
    /// - `0x800`: Unknown
    pub unk: u32,
    /// The device short description.
    pub description: *const u8,
    /// A pointer to a filled out functions table.
    pub drive_funcs: NonNull<DriveFunctionTable>,
}

/// Device information.
///
/// Mainly used for the parameter for the [`DriveFunctionTable`] function pointers.
#[repr(C)]
#[doc(alias("PspIoDrvFileArg", "SceIoIob"))]
pub struct IoBlockInfo {
    /// The file flags
    pub flags: FileFlags,
    /// The file system number.
    ///
    /// E.g.: If a file is opened as `host5:/myfile.txt` this field will be `5`.
    pub fs_unit: u32,
    /// A pointer to the device entry data.
    ///
    /// This pointer should never be null in the context of [`DriveFunctionTable`] functions being
    /// called, but it can be before that.
    pub dev_entry: *mut DeviceEntry,
    /// The device attribute.
    ///
    /// It contains the device kind.
    pub dev_attr: DeviceAttribute,
    /// A pointer to a setter defined argument.
    ///
    /// If written by the driver will preserve across calls.
    ///
    /// The actual data types used is device dependent.
    pub private_data: *mut c_void,
    pub cwd: *mut Cwd,
    /// The current file position offset.
    pub file_pos: i64,
    /// The thread UID of (?).
    pub thread_id: ThreadId,
    /// Currently unknown
    pub unk: u32,
    /// Raw UID of the opened file
    pub uid: FileId,
    /// The thread UID of the thread that opened the file.
    pub open_thread_id: ThreadId,
    /// If the file is opened in user mode.
    pub is_user_mode: bool,
    /// If the file was opened with [`FileFlags::PowerLock`].
    pub is_power_locked: bool,
    /// Unknown. It is a boolean of some sort, async related.
    pub unk2: u8,
    /// The async task priority
    pub async_priority: u8,
    /// The thread UID of the thread for the file async operations.
    pub async_thread_id: ThreadId,
    /// The semaphore UID for the file async operations.
    pub async_sema: SemaId,
    /// The event flag UID for the file async operations.
    pub async_event_flag: EventFlagId,
    /// The callback UID for the file async operations.
    pub async_callback: CallbackId,
    /// A pointer to the parameter to pass to `async_callback`.
    pub async_callback_argp: *mut c_void,
    /// Unused.
    pub unused: u32,
    /// The current `k1` register value for async operations
    pub k1: u32,
    /// The async return value of operations.
    pub async_return: i64,
    /// Arguments for the command to use.
    pub async_args: [i32; 6],
    /// The async command to execute.
    pub async_cmd: u32,
    /// The current user level of
    pub user_level: u32,
    pub hook: IoHook,
    pub unk3: u32,
    pub new_path: *mut u8,
    /// The return address.
    pub return_addr: SceSize,
}

/// Device attributes.
///
/// It hold the device kind and other things.
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceAttribute {
    /// Device kind. A device that does nothing.
    DummyDevice = 0x00,
    /// Device kind. A physical character device.
    CharacterDevice = 0x01,
    /// Device kind. A physical block device.
    ///
    /// E.g. A physical UMD drive.
    BlockDevice = 0x04,
    /// Device kind. A filesystem logical device.
    FilesystemDevice = 0x10,
    /// Device kind. A device alias device.
    ///
    /// A device that is simply an alias for another device.
    DeviceAliasDevice = 0x20,
    /// Device kind. A mount point alias device.
    ///
    /// A device that is simply an alias for an mount point.
    MountPointAliasDevice = 0x30,

    /// Device trait. The device has a block alias.
    HasBlockAlias = 0x100,
}

/// Type to maintain the device driver pointers.
#[repr(C)]
#[doc(alias("PspIoDrvFuncs", "SceIoDeviceFunction"))]
#[rustfmt::skip]
pub struct DriveFunctionTable {
    pub init: Option<unsafe extern "C" fn(entry: *mut DeviceEntry) -> SceResult<()>>,
    pub exit: Option<unsafe extern "C" fn(entry: *mut DeviceEntry) -> SceResult<()>>,
    pub open: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8, flags: FileFlags, mode: Mode) -> SceResult<FileId>>,
    pub close: Option<unsafe extern "C" fn(block: *mut IoBlockInfo) -> SceResult<()>>,
    pub read: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, buf: *mut u8, buf_size: SceSize) -> SceResult<SceSize>>,
    pub write: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, buf: *const u8, buf_size: SceSize) -> SceResult<SceSize>>,
    pub lseek: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, offset: i64, whence: Whence) -> i64>,
    pub ioctl: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8, out_size: SceSize) -> SceResult<u32>>,
    pub remove: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8) -> SceResult<()>>,
    pub mkdir: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8, mode: Mode) -> SceResult<()>>,
    pub rmdir: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8) -> SceResult<()>>,
    pub dopen: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, dir_path: *const u8) -> SceResult<FileId>>,
    pub dclose: Option<unsafe extern "C" fn(block: *mut IoBlockInfo) -> SceResult<()>>,
    pub dread: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, entry_info: &mut Dirent) -> SceResult<u32>>,
    pub get_stat: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8, stat: &mut Stat) -> SceResult<()>>,
    pub change_stat: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8, stat: &Stat, change_bits: ChangeStatFlag) -> SceResult<()>>,
    pub rename: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, old_name: *const u8, new_name: *const u8) -> SceResult<()>>,
    pub chdir: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, path: *const u8) -> SceResult<()>>,
    pub mount: Option<unsafe extern "C" fn(block: *mut IoBlockInfo) -> SceResult<()>>,
    pub unmount: Option<unsafe extern "C" fn(block: *mut IoBlockInfo) -> SceResult<()>>,
    pub devctl: Option<unsafe extern "C" fn(block: *mut IoBlockInfo, dev: *const u8, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8, out_size: SceSize) -> SceResult<u32>>,
    pub cancel: Option<unsafe extern "C" fn(block: *mut IoBlockInfo) -> SceResult<()>>,
}

/// Structure passed to the drive function [`DriveFunctionTable::init`] and
/// [`DriveFunctionTable::exit`].
#[repr(C)]
#[doc(alias("PspIoDrvArg", "SceIoDeviceEntry"))]
pub struct DeviceEntry {
    /// A pointer to the original driver which was added.
    pub drv: *mut DriveDevice,
    /// A pointer to a setter defined argument.
    ///
    /// If written by the driver will preserve across calls.
    ///
    /// The actual data types used is device dependent.
    pub private_data: *mut c_void,
    /// The number of file descriptor UID related to the device.
    pub user_fd_count: SceSize,
}

/// Possible device kinds.
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceKind {
    /// A device that does nothing.
    Dummy = 0x00,
    /// A physical character device.
    Character = 0x01,
    /// A physical block device.
    ///
    /// E.g. A physical UMD drive.
    Block = 0x04,
    /// A filesystem logical device.
    Filesystem = 0x10,
    /// A device alias device.
    ///
    /// A device that is simply an alias for another device.
    DeviceAlias = 0x20,
    /// A mount point alias device.
    ///
    /// A device that is simply an alias for an mount point.
    MountPointAlias = 0x30,
}

/// Current working directory.
#[repr(C)]
#[doc(alias = "SceIoCwd")]
pub struct Cwd {
    pub next: *mut Cwd,
    pub path_name: *mut u8,
    pub entry: *mut DeviceEntry,
    pub cwd_private: *mut c_void,
    pub ref_count: SceSize,
}

#[repr(C)]
#[doc(alias = "SceIoHook")]
pub struct IoHook {
    // FIXME: *mut SceIoHookArg
    pub arg: *mut c_void,
    pub io_block: *mut IoBlockInfo,
    pub func_table: *mut DriveFunctionTable,
}

#[psp_stub(libname = "IoFileMgrForUser", flags = 0x4001, use_crate)]
extern "C" {
    /// Opens a file with the corresponding file access attributes.
    ///
    /// The function only supports full path.
    ///
    /// If the flag [`FileFlags::CreateFile`] is set in `flags`, the third parameter is used if the
    /// file needs to be created, otherwise it is ignored.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    ///
    /// # Result Value
    ///
    /// Returns the open file descriptor ID on success, error value otherwise.
    #[nid(0x109F50BC)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoOpen(path: *const u8, flags: FileFlags, mode: Mode) -> SceResult<FileId>;

    /// Opens a file with the corresponding file access attributes asynchronously.
    ///
    /// The function only supports full path.
    ///
    /// If the flag [`FileFlags::CreateFile`] is set in `flags`, the third parameter is used if the
    /// file needs to be created, otherwise it is ignored.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    ///
    /// # Result Value
    ///
    /// Returns the open file descriptor ID on success, error value otherwise.
    #[nid(0x89AA9906)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoOpenAsync(
        path: *const u8, flags: FileFlags, mode: Mode,
    ) -> SceResult<FileId>;

    /// Closes a open file and deletes the associated file descriptor ID.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x810C4BC3)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoClose(fd: FileId) -> SceResult<()>;

    /// Closes a open file and deletes the associated file descriptor ID asynchronously.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xFF5940B6)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoCloseAsync(fd: FileId) -> SceResult<()>;

    /// Reads data from the file associated to the file descriptor UID to a buffer.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the data read.
    /// - `buf_size`: The size of the buffer.
    ///
    /// # Returns Value
    ///
    /// Returns the number of bytes read on success, error value otherwise.
    #[nid(0x6A638D83)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoRead(fd: FileId, buf: *mut u8, buf_size: SceSize) -> SceResult<SceSize>;

    /// Reads data from the file associated to the file descriptor UID to a buffer asynchronously.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the data read.
    /// - `buf_size`: The size of the buffer.
    ///
    /// # Returns Value
    ///
    /// Returns the number of bytes read on success, error value otherwise.
    #[nid(0xA0B5A7C2)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoReadAsync(fd: FileId, buf: *mut u8, buf_size: SceSize)
        -> SceResult<SceSize>;

    /// Writes data to the file associated to the file descriptor UID.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `data` **[[In parameter]]**: A pointer to a buffer data to write.
    /// - `data_size`: The size of `data` buffer.
    ///
    /// # Return Value
    ///
    /// Returns the number of bytes written on success, error value otherwise.
    #[nid(0x42EC03AC)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoWrite(fd: FileId, data: *const u8, data_size: SceSize)
        -> SceResult<SceSize>;

    /// Writes data to the file associated to the file descriptor UID asynchronously.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `data` **[[In parameter]]**: A pointer to a buffer data to write.
    /// - `data_size`: The size of `data` buffer.
    ///
    /// # Return Value
    ///
    /// Returns the number of bytes written on success, error value otherwise.
    #[nid(0x0FACAB19)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoWriteAsync(
        fd: FileId, data: *const u8, data_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Repositions the file offset of the file descriptor.
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// The position of the internal file offset on success, or one of the raw [`SceError`] constant
    /// values on error.
    #[eabi(i_ii_i_rii)]
    #[nid(0x27EB27B8)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoLseek(fd: FileId, offset: i64, whence: Whence) -> i64;

    /// Repositions the file offset of the file descriptor asynchronously.
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// The position of the internal file offset on success, or one of the raw [`SceError`] constant
    /// values on error.
    #[eabi(i_ii_i_rii)]
    #[nid(0x71B19E77)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoLseekAsync(fd: FileId, offset: i64, whence: Whence) -> i64;

    /// Repositions the file offset of the file descriptor. (32-bit mode).
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// Returns the position of the internal file offset on success, error value otherwise.
    #[nid(0x68963324)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoLseek32(fd: FileId, offset: i32, whence: Whence) -> SceResult<SceSize>;

    /// Repositions the file offset of the file descriptor. (32-bit mode).
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// Returns the position of the internal file offset on success, error value otherwise.
    #[nid(0x1B385D8F)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoLseek32Async(fd: FileId, offset: i32, whence: Whence) -> SceResult<SceSize>;

    /// Removes a file associated to a given path.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to remove.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF27A9C51)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoRemove(path: *const u8) -> SceResult<()>;

    /// Creates a directory file.
    ///
    /// # Parameters
    ///
    /// - `dir_path` **[[In parameter]]**: The path to create the directory file.
    /// - `mode`: The permission mode to set on the created file.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x06A70004)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoMkdir(dir_path: *const u8, mode: Mode) -> SceResult<()>;

    /// Removes a directory file associated to a given path.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to remove.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1117C65F)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoRmdir(path: *const u8) -> SceResult<()>;

    /// Changes the current directory.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to change.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x55F4717D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoChdir(path: *const u8) -> SceResult<()>;

    /// Changes the file name to a new name.
    ///
    /// # Parameters
    ///
    /// - `old_name` **[[In parameter]]**: The old name for the file.
    /// - `new_name` **[[In parameter]]**: The new name for the file.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x779103A0)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoRename(old_name: *const u8, new_name: *const u8) -> SceResult<()>;

    /// Opens a directory.
    ///
    /// # Parameters
    ///
    /// - `dir_path` **[[In parameter]]**: The directory path to open.
    ///
    /// # Result Value
    ///
    /// Returns the open file descriptor ID on success, error value otherwise.
    #[nid(0xB29DDF9C)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoDopen(dir_path: *const u8) -> SceResult<FileId>;

    /// Reads an entry from a opened file descriptor UID associated to a file opened with
    /// [`sceIoDopen`].
    ///
    /// # Parameters
    ///
    /// - `dir_fd`: The directory file descriptor UID.
    /// - `entry_info` **[[Out parameter]]**: A reference to the structure to receive the directory
    ///   entry information.
    ///
    /// # Return Value
    ///
    /// Returns, on success, `0` if there is no more directory entries left, `> 0` if there are more
    /// directory entries to go. On error, a error value is returned.
    #[nid(0xE3EB004C)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoDread(dir_fd: FileId, entry_info: &mut Dirent) -> SceResult<u32>;

    /// Closes an directory opened with [`sceIoDopen`].
    ///
    /// # Parameters
    ///
    /// - `dir_fd`: The directory file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEB092469)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoDclose(dir_fd: FileId) -> SceResult<()>;

    /// Gets the status of a file.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file.
    /// - `stat` **[[Out parameter]]**: A reference to receive the status data from the file.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xACE946E8)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoGetstat(path: *const u8, stat: &mut Stat) -> SceResult<()>;

    /// Changes the status of a file.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file.
    /// - `stat` **[[In parameter]]**: A reference to the status data to modify the file stat.
    /// - `change_bits`: The bitflags of what field of stat to change.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB8A740F4)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoChstat(
        path: *const u8, stat: &Stat, change_bits: ChangeStatFlag,
    ) -> SceResult<()>;

    /// Assigns one I/O device to another.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name to assign.
    /// - `block_dev` **[[In parameter]]**: The block device name to assign from.
    /// - `fs` **[[In parameter]]**: The filesystem device name to map the block device to `dev`.
    /// - `flags`: The mounting modes flags.
    /// - `unk1`: Unknown. Set it to [`null`](core::ptr::null_mut).
    /// - `unk2`: Unknown. Set it to `0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xB2A628C1)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoAssign(
        dev: *const u8, block_dev: *const u8, fs: *const u8, flags: AssignFlag, unk1: *mut c_void,
        unk2: u32,
    ) -> SceResult<()>;

    /// Unassign an I/O device.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name to unassign.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6D08A871)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoUnassign(dev: *const u8) -> SceResult<()>;

    /// Performs an Device Control command on a device.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name.
    /// - `cmd`: The command to send to the device.
    /// - `in_buf` **[[In parameter]]**: A pointer to a data block buffer to send to the device. If
    ///   [`null`](core::ptr::null_mut) it sends no data.
    /// - `in_size`: The size of the `in_buf`.
    /// - `out_buf` **[[Out parameter]]**: A pointer to a data block buffer to receive data from the
    ///   device. If [`null`](core::ptr::null_mut) it receives no data.
    /// - `out_size`: The size of the `out_buf`.
    ///
    /// /// # Return Value
    ///
    /// `Ok` value on success (meaning depends on the command), error value otherwise.
    #[eabi(i6)]
    #[nid(0x54F5FB11)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoDevctl(
        dev: *const u8, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8,
        out_size: SceSize,
    ) -> SceResult<u32>;

    /// Performs an I/O Control command on a file.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `cmd`: The command to send to the file.
    /// - `in_buf` **[[In parameter]]**: A pointer to a data block buffer to send to the file. If
    ///   [`null`](core::ptr::null_mut) it sends no data.
    /// - `in_size`: The size of the `in_buf`.
    /// - `out_buf` **[[Out parameter]]**: A pointer to a data block buffer to receive data from the
    ///   file. If [`null`](core::ptr::null_mut) it receives no data.
    /// - `out_size`: The size of the `out_buf`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success (meaning depends on the command), error value otherwise.
    #[eabi(i6)]
    #[nid(0x63632449)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoIoctl(
        fd: FileId, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8,
        out_size: SceSize,
    ) -> SceResult<u32>;

    /// Performs an I/O Control command on a file asynchronously.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `cmd`: The command to send to the device.
    /// - `in_buf` **[[In parameter]]**: A pointer to a data block buffer to send to the device. If
    ///   [`null`](core::ptr::null_mut) it sends no data.
    /// - `in_size`: The size of the `in_buf`.
    /// - `out_buf` **[[Out parameter]]**: A pointer to a data block buffer to receive data from the
    ///   device. If [`null`](core::ptr::null_mut) it receives no data.
    /// - `out_size`: The size of the `out_buf`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success (meaning depends on the command), error value otherwise.
    #[eabi(i6)]
    #[nid(0xE95A012B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoIoctlAsync(
        fd: FileId, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8,
        out_size: SceSize,
    ) -> SceResult<u32>;

    /// Waits for a file asynchronous action completion.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE23EEC33)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoWaitAsync(fd: FileId, result: &mut i64) -> SceResult<()>;

    /// Waits for a file asynchronous action completion with callbacks.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x35DBD746)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoWaitAsyncCB(fd: FileId, result: &mut i64) -> SceResult<()>;

    /// Polls a file asynchronous task completion.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3251EA56)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoPollAsync(fd: FileId, result: &mut i64) -> SceResult<()>;

    /// Gets the completions status of a file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `pool`: If `0` then waits for the status, otherwise it polls the file descriptor.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCB05F8D6)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoGetAsyncStat(fd: FileId, poll: u32, result: &mut i64) -> SceResult<()>;

    /// Cancels a file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE8BC6571)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoCancel(fd: FileId) -> SceResult<()>;

    /// Changes the priority level of a file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `priority`: The priority level.
    ///   - On user request, it must be `-1` or in the range of `7` to `120`.
    ///   - On kernel request, it must be `-1` or in the range of `0` to `127`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB293727F)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoChangeAsyncPriority(fd: FileId, priority: i32) -> SceResult<()>;

    /// Sets a callback for the file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `callback_id`: The callback UID. It is created with [`sceKernelCreateCallback`].
    /// - `argp` **[[In-out parameter]]**: A pointer to the arguments to pass to the callback.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA12A0514)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoSetAsyncCallback(
        fd: FileId, callback_id: CallbackId, argp: *mut c_void,
    ) -> SceResult<()>;

    /// Gets the device attribute of the opened file descriptor.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Returns Value
    ///
    /// Returns the device attribute on success, error value otherwise.
    #[nid(0x08BD7374)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceIoGetDevType(fd: FileId) -> SceResult<DeviceAttribute>;

    /// Gets the list of opened file descriptor UIDs.
    ///
    /// This function filters by caller permission, so callers only receive the file descriptors
    /// they have access.
    ///
    /// # Parameters
    ///
    /// - `fd_list` **[[Out parameter]]**: A pointer to a list to receive the information.
    /// - `fd_list_size`: The total size of `fd_list`.
    /// - `count` **[[Out parameter]]**: A pointer to receive how many file descriptor are open. If
    ///   `null` this information is not retrieved.
    ///
    /// # Return Value
    ///
    /// Returns the number of file descriptor UID added to the list on success, error value
    /// otherwise.
    #[nid(0x5C2BE2CC)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceIoGetFdList(
        fd_list: *mut FileId, fd_list_size: SceSize, count: *mut SceSize,
    ) -> SceResult<SceSize>;
}

// FIXME: Missing functions
//
// int sceIoGetFdDebugInfo(int fd, SceIoFdDebugInfo *outInfo);
// int sceIoAddHook(SceIoHookType *hook);
#[cfg(feature = "kernel")]
#[psp_stub(libname = "IoFileMgrForKernel", flags = 0x0001, use_crate)]
extern "C" {
    /// Opens a file with the corresponding file access attributes.
    ///
    /// The function only supports full path.
    ///
    /// If the flag [`FileFlags::CreateFile`] is set in `flags`, the third parameter is used if the
    /// file needs to be created, otherwise it is ignored.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    ///
    /// # Result Value
    ///
    /// Returns the open file descriptor ID on success, error value otherwise.
    #[nid(0x109F50BC)]
    pub unsafe fn sceIoOpen(path: *const u8, flags: FileFlags, mode: Mode) -> SceResult<FileId>;

    /// Opens a file with the corresponding file access attributes asynchronously.
    ///
    /// The function only supports full path.
    ///
    /// If the flag [`FileFlags::CreateFile`] is set in `flags`, the third parameter is used if the
    /// file needs to be created, otherwise it is ignored.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    ///
    /// # Result Value
    ///
    /// Returns the open file descriptor ID on success, error value otherwise.
    #[nid(0x89AA9906)]
    pub unsafe fn sceIoOpenAsync(
        path: *const u8, flags: FileFlags, mode: Mode,
    ) -> SceResult<FileId>;

    /// Reopens an existing file descriptor UID.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    /// - `fd`: The old/closed file descriptor UID.
    ///
    /// # Return Value
    ///
    /// Returns the reopened file descriptor UID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x947D7A06 }
        else if cfg!(feature = "psp_630") { 0x421B8EB4 }
        else if cfg!(feature = "psp_600") { 0xF0AD3C72 }
        else if cfg!(feature = "psp_570") { 0x593C32F4 }
        else if cfg!(feature = "psp_500") { 0x30DA4775 }
        else if cfg!(feature = "psp_420") { 0x9F096EDF }
        else if cfg!(feature = "psp_395") { 0xD6569E5A }
        else if cfg!(feature = "psp_380") { 0xCB1F53B3 }
        else { 0x3C54E908 }
    )]
    pub unsafe fn sceIoReopen(
        path: *const u8, flags: FileFlags, mode: Mode, fd: FileId,
    ) -> SceResult<FileId>;

    /// Closes a open file and deletes the associated file descriptor ID.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x810C4BC3)]
    pub unsafe fn sceIoClose(fd: FileId) -> SceResult<()>;

    /// Closes a open file and deletes the associated file descriptor ID asynchronously.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xFF5940B6)]
    pub unsafe fn sceIoCloseAsync(fd: FileId) -> SceResult<()>;

    /// Reads data from the file associated to the file descriptor UID to a buffer.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the data read.
    /// - `buf_size`: The size of the buffer.
    ///
    /// # Returns Value
    ///
    /// Returns the number of bytes read on success, error value otherwise.
    #[nid(0x6A638D83)]
    pub unsafe fn sceIoRead(fd: FileId, buf: *mut u8, buf_size: SceSize) -> SceResult<SceSize>;

    /// Reads data from the file associated to the file descriptor UID to a buffer asynchronously.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `buf` **[[Out parameter]]**: A pointer to the buffer to receive the data read.
    /// - `buf_size`: The size of the buffer.
    ///
    /// # Returns Value
    ///
    /// Returns the number of bytes read on success, error value otherwise.
    #[nid(0xA0B5A7C2)]
    pub unsafe fn sceIoReadAsync(fd: FileId, buf: *mut u8, buf_size: SceSize)
        -> SceResult<SceSize>;

    /// Writes data to the file associated to the file descriptor UID.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `data` **[[In parameter]]**: A pointer to a buffer data to write.
    /// - `data_size`: The size of `data` buffer.
    ///
    /// # Return Value
    ///
    /// Returns the number of bytes written on success, error value otherwise.
    #[nid(0x42EC03AC)]
    pub unsafe fn sceIoWrite(fd: FileId, data: *const u8, data_size: SceSize)
        -> SceResult<SceSize>;

    /// Writes data to the file associated to the file descriptor UID asynchronously.
    ///
    /// # Parameter
    ///
    /// - `fd`: The file descriptor UID.
    /// - `data` **[[In parameter]]**: A pointer to a buffer data to write.
    /// - `data_size`: The size of `data` buffer.
    ///
    /// # Return Value
    ///
    /// Returns the number of bytes written on success, error value otherwise.
    #[nid(0x0FACAB19)]
    pub unsafe fn sceIoWriteAsync(
        fd: FileId, data: *const u8, data_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Repositions the file offset of the file descriptor.
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// The position of the internal file offset on success, or one of the raw [`SceError`] constant
    /// values on error.
    #[eabi(i_ii_i_rii)]
    #[nid(0x27EB27B8)]
    pub fn sceIoLseek(fd: FileId, offset: i64, whence: Whence) -> i64;

    /// Repositions the file offset of the file descriptor asynchronously.
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// The position of the internal file offset on success, or one of the raw [`SceError`] constant
    /// values on error.
    #[eabi(i_ii_i_rii)]
    #[nid(0x71B19E77)]
    pub fn sceIoLseekAsync(fd: FileId, offset: i64, whence: Whence) -> i64;

    /// Repositions the file offset of the file descriptor. (32-bit mode).
    ///
    /// # Parameters
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// Returns the position of the internal file offset on success, error value otherwise.
    #[nid(0x68963324)]
    pub fn sceIoLseek32(fd: FileId, offset: i32, whence: Whence) -> SceResult<SceSize>;

    /// Repositions the file offset of the file descriptor asynchronously. (32-bit mode).
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `offset`: The relative offset from the start position determined by `whence`.
    /// - `whence`: The start position options to use for the relative `offset`.
    ///
    /// # Return Value
    ///
    /// Returns the position of the internal file offset on success, error value otherwise.
    #[nid(0x1B385D8F)]
    pub fn sceIoLseek32Async(fd: FileId, offset: i32, whence: Whence) -> SceResult<SceSize>;

    /// Removes a file associated to a given path.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to remove.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF27A9C51)]
    pub unsafe fn sceIoRemove(path: *const u8) -> SceResult<()>;

    /// Creates a directory file.
    ///
    /// # Parameters
    ///
    /// - `dir_path` **[[In parameter]]**: The path to create the directory file.
    /// - `mode`: The permission mode to set on the created file.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x06A70004)]
    pub unsafe fn sceIoMkdir(dir_path: *const u8, mode: Mode) -> SceResult<()>;

    /// Removes a directory file associated to a given path.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to remove.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1117C65F)]
    pub unsafe fn sceIoRmdir(path: *const u8) -> SceResult<()>;

    /// Changes the current directory.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to change.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x55F4717D)]
    pub unsafe fn sceIoChdir(path: *const u8) -> SceResult<()>;

    /// Changes the file name to a new name.
    ///
    /// # Parameters
    ///
    /// - `old_name` **[[In parameter]]**: The old name for the file.
    /// - `new_name` **[[In parameter]]**: The new name for the file.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x779103A0)]
    pub unsafe fn sceIoRename(old_name: *const u8, new_name: *const u8) -> SceResult<()>;

    /// Opens a directory.
    ///
    /// # Parameters
    ///
    /// - `dir_path` **[[In parameter]]**: The directory path to open.
    ///
    /// # Result Value
    ///
    /// Returns the open file descriptor ID on success, error value otherwise.
    #[nid(0xB29DDF9C)]
    pub unsafe fn sceIoDopen(dir_path: *const u8) -> SceResult<FileId>;

    /// Reads an entry from a opened file descriptor UID associated to a file opened with
    /// [`sceIoDopen`].
    ///
    /// # Parameters
    ///
    /// - `dir_fd`: The file descriptor UID.
    /// - `entry_info` **[[Out parameter]]**: A reference to the structure to receive the directory
    ///   entry information.
    ///
    /// # Return Value
    ///
    /// Returns, on success, `0` if there is no more directory entries left, `> 0` if there are more
    /// directory entries to go. On error, a error value is returned.
    #[nid(0xE3EB004C)]
    pub fn sceIoDread(dir_fd: FileId, entry_info: &mut Dirent) -> SceResult<u32>;

    /// Closes an directory opened with [`sceIoDopen`].
    ///
    /// # Parameters
    ///
    /// - `dir_fd`: The directory file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEB092469)]
    pub unsafe fn sceIoDclose(dir_fd: FileId) -> SceResult<()>;

    /// Gets the status of a file.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file.
    /// - `stat` **[[Out parameter]]**: A reference to receive the status data from the file.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xACE946E8)]
    pub unsafe fn sceIoGetstat(path: *const u8, stat: &mut Stat) -> SceResult<()>;

    /// Changes the status of a file.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file.
    /// - `stat` **[[In parameter]]**: A reference to the status data to modify the file stat.
    /// - `change_bits`: The bitflags of what field of stat to change.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB8A740F4)]
    pub unsafe fn sceIoChstat(
        path: *const u8, stat: &Stat, change_bits: ChangeStatFlag,
    ) -> SceResult<()>;

    /// Assigns one I/O device to another.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name to assign.
    /// - `block_dev` **[[In parameter]]**: The block device name to assign from.
    /// - `fs` **[[In parameter]]**: The filesystem device name to map the block device to `dev`.
    /// - `flags`: The mounting modes flags.
    /// - `unk1`: Unknown. Set it to [`null`](core::ptr::null_mut).
    /// - `unk2`: Unknown. Set it to `0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xB2A628C1)]
    pub unsafe fn sceIoAssign(
        dev: *const u8, block_dev: *const u8, fs: *const u8, flags: AssignFlag, unk1: *mut c_void,
        unk2: u32,
    ) -> SceResult<()>;

    /// Unassign an I/O device.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name to unassign.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6D08A871)]
    pub unsafe fn sceIoUnassign(dev: *const u8) -> SceResult<()>;

    /// Performs an Device Control command on a device.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name.
    /// - `cmd`: The command to send to the device.
    /// - `in_buf` **[[In parameter]]**: A pointer to a data block buffer to send to the device. If
    ///   [`null`](core::ptr::null_mut) it sends no data.
    /// - `in_size`: The size of the `in_buf`.
    /// - `out_buf` **[[Out parameter]]**: A pointer to a data block buffer to receive data from the
    ///   device. If [`null`](core::ptr::null_mut) it receives no data.
    /// - `out_size`: The size of the `out_buf`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success (meaning depends on the command), error value otherwise.
    #[eabi(i6)]
    #[nid(0x54F5FB11)]
    pub unsafe fn sceIoDevctl(
        dev: *const u8, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8,
        out_size: SceSize,
    ) -> SceResult<u32>;

    /// Performs an I/O Control command on a file.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `cmd`: The command to send to the file.
    /// - `in_buf` **[[In parameter]]**: A pointer to a data block buffer to send to the file. If
    ///   [`null`](core::ptr::null_mut) it sends no data.
    /// - `in_size`: The size of the `in_buf`.
    /// - `out_buf` **[[Out parameter]]**: A pointer to a data block buffer to receive data from the
    ///   file. If [`null`](core::ptr::null_mut) it receives no data.
    /// - `out_size`: The size of the `out_buf`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success (meaning depends on the command), error value otherwise.
    #[eabi(i6)]
    #[nid(0x63632449)]
    pub unsafe fn sceIoIoctl(
        fd: FileId, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8,
        out_size: SceSize,
    ) -> SceResult<i32>;

    /// Performs an I/O Control command on a file asynchronously.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `cmd`: The command to send to the device.
    /// - `in_buf` **[[In parameter]]**: A pointer to a data block buffer to send to the device. If
    ///   [`null`](core::ptr::null_mut) it sends no data.
    /// - `in_size`: The size of the `in_buf`.
    /// - `out_buf` **[[Out parameter]]**: A pointer to a data block buffer to receive data from the
    ///   device. If [`null`](core::ptr::null_mut) it receives no data.
    /// - `out_size`: The size of the `out_buf`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i6)]
    #[nid(0xE95A012B)]
    pub unsafe fn sceIoIoctlAsync(
        fd: FileId, cmd: u32, in_buf: *mut u8, in_size: SceSize, out_buf: *mut u8,
        out_size: SceSize,
    ) -> SceResult<u32>;

    /// Synchronize the file data on the device.
    ///
    /// # Parameters
    ///
    /// - `dev` **[[In parameter]]**: The device name to sync.
    /// - `unk`: Unknown value.
    ///
    /// # Return Value
    ///
    /// Returns an unknown value on success, error value otherwise.
    #[nid(0xAB96437F)]
    pub unsafe fn sceIoSync(dev: *const u8, unk: u32) -> SceResult<u32>;

    /// Waits for a file asynchronous action completion.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE23EEC33)]
    pub fn sceIoWaitAsync(fd: FileId, result: &mut i64) -> SceResult<()>;

    /// Waits for a file asynchronous action completion with callbacks.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x35DBD746)]
    pub fn sceIoWaitAsyncCB(fd: FileId, result: &mut i64) -> SceResult<()>;

    /// Polls a file asynchronous task completion.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3251EA56)]
    pub fn sceIoPollAsync(fd: FileId, result: &mut i64) -> SceResult<()>;

    /// Gets the completions status of a file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `pool`: If `0` then waits for the status, otherwise it polls the file descriptor.
    /// - `return` **[[Out parameter]]**: A reference to the result of the async action.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCB05F8D6)]
    pub fn sceIoGetAsyncStat(fd: FileId, poll: u32, result: &mut i64) -> SceResult<()>;

    /// Cancels a file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE8BC6571)]
    pub fn sceIoCancel(fd: FileId) -> SceResult<()>;

    /// Changes the priority level of a file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `priority`: The priority level.
    ///   - On user request, it must be `-1` or in the range of `7` to `120`.
    ///   - On kernel request, it must be `-1` or in the range of `0` to `127`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB293727F)]
    pub fn sceIoChangeAsyncPriority(fd: FileId, priority: i32) -> SceResult<()>;

    /// Sets a callback for the file asynchronous task.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `callback_id`: The callback UID. It is created with [`sceKernelCreateCallback`].
    /// - `argp` **[[In-out parameter]]**: A pointer to the arguments to pass to the callback.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// [`sceKernelCreateCallback`]: crate::sys::thread::sceKernelCreateCallback
    #[nid(0xA12A0514)]
    pub unsafe fn sceIoSetAsyncCallback(
        fd: FileId, callback_id: CallbackId, argp: *mut c_void,
    ) -> SceResult<()>;

    /// Gets the device attribute of the opened file descriptor.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Returns Value
    ///
    /// Returns the device attribute on success, error value otherwise.
    #[nid(0x08BD7374)]
    pub fn sceIoGetDevType(fd: FileId) -> SceResult<DeviceAttribute>;

    /// Gets the real file descriptor UID from global list of UID.
    ///
    /// # Parameters
    ///
    /// - `fd`: A file descriptor UID returned from a [`sceIoOpen`]/[`sceIoOpenAsync`] with the
    ///   [`FileFlags::GlobalFdIndex`] set.
    ///
    /// # Return Value
    ///
    /// Returns the real file descriptor UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.50.
    #[nid(if cfg!(feature = "psp_660") { 0xDCCD6185 }
        else if cfg!(feature = "psp_630") { 0x64ED84C9 }
        else if cfg!(feature = "psp_600") { 0x4F663EBF }
        else if cfg!(feature = "psp_570") { 0xB83F536C }
        else if cfg!(feature = "psp_500") { 0x10656D20 }
        else if cfg!(feature = "psp_420") { 0x6C2EBE4E }
        else if cfg!(feature = "psp_395") { 0x450EDC8A }
        else if cfg!(feature = "psp_380") { 0xE5A18603 }
        else { 0x9B86630B }
    )]
    pub fn sceIoGetUID(fd: FileId) -> SceResult<FileId>;

    /// Gets the list of opened file descriptor UIDs.
    ///
    /// This function filters by caller permission, so callers only receive the file descriptors
    /// they have access.
    ///
    /// # Parameters
    ///
    /// - `fd_list` **[[Out parameter]]**: A pointer to a list to receive the information.
    /// - `fd_list_size`: The total size of `fd_list`.
    /// - `count` **[[Out parameter]]**: A pointer to receive how many file descriptor are open. If
    ///   `null` this information is not retrieved.
    ///
    /// # Return Value
    ///
    /// Returns the number of file descriptor UID added to the list on success, error value
    /// otherwise.
    #[nid(0x5C2BE2CC)]
    pub unsafe fn sceIoGetFdList(
        fd_list: *mut FileId, fd_list_size: SceSize, count: *mut SceSize,
    ) -> SceResult<SceSize>;

    /// Adds a new I/O driver to the system.
    ///
    /// # Parameters
    ///
    /// - `drv` **[[In parameter]]**: A pointer to a filled out driver structure.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8E982A74)]
    pub unsafe fn sceIoAddDrv(drv: *const DriveDevice) -> SceResult<()>;

    /// Removes a I/O driver from the system.
    ///
    /// This functions fails if the driver doesn't have a [`DriveFunctionTable::exit`]
    /// implementation.
    ///
    /// # Parameters
    ///
    /// - `drv_name` **[[In parameter]]**: The name of the driver to delete (null terminated).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xC7F35804)]
    pub unsafe fn sceIoDelDrv(drv_name: *const u8) -> SceResult<()>;

    /// Forcedly removes a I/O driver from the system.
    ///
    /// This doesn't execute the driver [`DriveFunctionTable::exit`] function like [`sceIoDelDrv`].
    ///
    /// # Parameters
    ///
    /// - `drv_name` **[[In parameter]]**: The name of the driver to delete (null terminated).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.80.
    #[nid(if cfg!(feature = "psp_660") { 0x76DA16E3 }
        // else if cfg!(feature = "psp_630") { 0x5216CE3F }
        // else if cfg!(feature = "psp_600") { 0x5216CE3F }
        else if cfg!(feature = "psp_570") { 0x1D0D4B29 }
        else if cfg!(feature = "psp_500") { 0x5BC52506 }
        else if cfg!(feature = "psp_420") { 0x1F54AE78 }
        else if cfg!(feature = "psp_395") { 0x72AC9C0E }
        else if cfg!(feature = "psp_380") { 0xC27B252F }
        else { 0x5216CE3F }
    )]
    pub unsafe fn sceIoTerminateFd(drv_name: *const u8) -> SceResult<()>;

    /// Gets the user level out of I/O block.
    ///
    /// # Parameters
    ///
    /// - `block` **[[In parameter]]**: A reference of the device block you want to get.
    ///
    /// # Return Value
    ///
    /// Returns the block user level.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.52.
    #[nid(if cfg!(feature = "psp_660") { 0x49356C12 }
        else if cfg!(feature = "psp_630") { 0x804DFCE6 }
        else if cfg!(feature = "psp_600") { 0x6CE0E5F0 }
        else if cfg!(feature = "psp_570") { 0xBD7D5591 }
        else if cfg!(feature = "psp_500") { 0xFFE9FFDE }
        else if cfg!(feature = "psp_420") { 0x6A90F545 }
        else if cfg!(feature = "psp_395") { 0x993395FE }
        else if cfg!(feature = "psp_380") { 0x128BD999 }
        else { 0xBD17474F }
    )]
    pub fn sceIoGetIobUserLevel(block: &IoBlockInfo) -> u32;

    /// Checks for the validity of a file descriptor.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    /// - `unk`: Unknown, permission related. Checked against a system object if `(obj.attr & unk)
    ///   != 0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.70.
    #[nid(if cfg!(feature = "psp_660") { 0x2B6A9B21 }
        else if cfg!(feature = "psp_630") { 0x13A4DEB0 }
        else if cfg!(feature = "psp_600") { 0xF2990AC6 }
        else if cfg!(feature = "psp_570") { 0xB3411C4F }
        else if cfg!(feature = "psp_500") { 0x5B00C8FB }
        else if cfg!(feature = "psp_420") { 0x2B8EC797 }
        else if cfg!(feature = "psp_395") { 0x0D6CC391 }
        else if cfg!(feature = "psp_380") { 0x588C6903 }
        else { 0x30E8ABB3 }
    )]
    pub fn sceIoValidateFd(fd: FileId, unk: u32) -> SceResult<()>;

    /// Gets the current working directory for a thread.
    ///
    /// # Parameters
    ///
    /// - `thread_id`: The thread UID of the thread.
    /// - `path_buf` **[[Out parameter]]**: A pointer to a buffer to receive the data.
    /// - `path_buf_size`: The size of `path_buf`.
    ///
    /// # Return Value
    ///
    /// Returns the number of characters written to `path_buf` on success, an error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1FC0620B }
        else if cfg!(feature = "psp_630") { 0x74482CE3 }
        else if cfg!(feature = "psp_600") { 0x0AB86E5D }
        else if cfg!(feature = "psp_570") { 0x2912ADE8 }
        else if cfg!(feature = "psp_500") { 0x957305E4 }
        else if cfg!(feature = "psp_420") { 0x34ADDEA9 }
        else if cfg!(feature = "psp_395") { 0x636BB28B }
        else if cfg!(feature = "psp_380") { 0x0B281A72 }
        else { 0x411106BA }
    )]
    pub unsafe fn sceIoGetThreadCwd(
        thread_id: ThreadId, path_buf: *mut u8, path_buf_size: SceSize,
    ) -> SceResult<SceSize>;

    /// Sets the current working directory for a thread.
    ///
    /// # Parameters
    ///
    /// - `thread_id`: The thread UID of the thread.
    /// - `dir_path` **[[In parameter]]**: The directory path to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xC30581F4 }
        else if cfg!(feature = "psp_630") { 0xBCC4C96F }
        else if cfg!(feature = "psp_600") { 0x9D5CDC31 }
        else if cfg!(feature = "psp_570") { 0x8C15C11E }
        else if cfg!(feature = "psp_500") { 0xA72FAEB4 }
        else if cfg!(feature = "psp_420") { 0xC87784D8 }
        else if cfg!(feature = "psp_395") { 0xCA3B4391 }
        else if cfg!(feature = "psp_380") { 0xA16D8C5C }
        else { 0xCB0A151F }
    )]
    pub unsafe fn sceIoChangeThreadCwd(thread_id: ThreadId, dir_path: *const u8) -> SceResult<()>;
}

// FIXME (Low-priority): Add missing itens
// - (fdgetc, 0xD2B2A2A7)
// - (fdgets, 0x11A5127A)
// - (fdprintf, 0x2CCF071A)
// - (fdputc, 0x4F78930A)
// - (fdputs, 0x36B23B8B)
// - (getchar, 0x7E338487)
// - (gets, 0xBFF7E760)
// - (printf, 0xCAB439DF)
// - (putchar, 0xD768752A)
// - (puts, 0xD97C8CB9)
#[psp_stub(libname = "StdioForUser", flags = 0x4001, use_crate)]
extern "C" {
    /// Function to get the current standard in file ID
    ///
    /// # Return Value
    ///
    /// The stdin file ID.
    #[nid(0x172D316E)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelStdin() -> FileId;

    /// Function to get the current standard out file ID
    ///
    /// # Return Value
    ///
    /// The stdout file ID.
    #[nid(0xA6BAB2E9)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelStdout() -> FileId;

    /// Function to get the current standard error file ID
    ///
    /// # Return Value
    ///
    /// The stderr file ID.
    #[nid(0xF78BA90A)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelStderr() -> FileId;

    /// Register a file descriptor as PIPE to stdout.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(0x432D8F5C)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelRegisterStdoutPipe(fd: FileId) -> SceResult<()>;

    /// Register a file descriptor as PIPE to stderr.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(0x6F797E03)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelRegisterStderrPipe(fd: FileId) -> SceResult<()>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "StdioForKernel", flags = 0x0001, use_crate)]
extern "C" {
    /// Function to get the current standard in file ID
    ///
    /// # Return Value
    ///
    /// The stdin file ID.
    #[nid(0x172D316E)]
    pub fn sceKernelStdin() -> FileId;

    /// Function to get the current standard out file ID
    ///
    /// # Return Value
    ///
    /// The stdout file ID.
    #[nid(0xA6BAB2E9)]
    pub fn sceKernelStdout() -> FileId;

    /// Function to get the current standard error file ID
    ///
    /// # Return Value
    ///
    /// The stderr file ID.
    #[nid(0xF78BA90A)]
    pub fn sceKernelStderr() -> FileId;

    /// Register a file descriptor as PIPE to stdout.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(0x432D8F5C)]
    pub fn sceKernelRegisterStdoutPipe(fd: FileId) -> SceResult<()>;

    /// Register a file descriptor as PIPE to stderr.
    ///
    /// # Parameters
    ///
    /// - `fd`: The file descriptor UID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(0x6F797E03)]
    pub fn sceKernelRegisterStderrPipe(fd: FileId) -> SceResult<()>;

    /// Reopens the standard Out.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    ///
    /// # Return Value
    ///
    /// Returns the reopened file descriptor UID on success, error value otherwise.
    #[nid(0x98220F3E)]
    pub unsafe fn sceKernelStdoutReopen(
        path: *const u8, flags: FileFlags, mode: Mode,
    ) -> SceResult<FileId>;

    /// Reopens the standard Error.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the file to open.
    /// - `flags`: The flags with file access attributes.
    /// - `mode`: The permission mode to use. If [`FileFlags::CreateFile`] is set in `flags`, it is
    ///   ignored otherwise.
    ///
    /// # Return Value
    ///
    /// Returns the reopened file descriptor UID on success, error value otherwise.
    #[nid(0xFB5380C5)]
    pub unsafe fn sceKernelStderrReopen(
        path: *const u8, flags: FileFlags, mode: Mode,
    ) -> SceResult<FileId>;

    /// Resets the standard Out.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(0x2D8551AB)]
    pub fn sceKernelStdoutReset() -> SceResult<()>;

    /// Resets the standard Error.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(0x9662BF86)]
    pub fn sceKernelStderrReset() -> SceResult<()>;
}

impl FileId {
    /// Create a new file descriptor ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new file descriptor ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for FileId {}
unsafe impl SceResultOk for FileId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

impl DeviceAttribute {
    pub const fn device_kind(self) -> DeviceKind {
        // Safety: the device kind is hold in the last 8bits and will always be one of the device
        // kind values
        unsafe { mem::transmute(self.and(Self::from_bits_retain(0xFF)).bits()) }
    }
}

impl crate::private::Sealed for DeviceAttribute {}
unsafe impl SceResultOk for DeviceAttribute {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        Ok(Self::from_bits_retain(ok_value))
    }
}
