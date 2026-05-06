/// Type representing return errors from the PSP APIs, official or otherwise.
///
/// It's valid range is `(0x80000001, 0xFFFFFFFF]`.
#[repr(transparent)]
// #[derive(Clone, Copy)]
// pub struct SceError(pattern_type!(u32 is 0x80000001..=0xFFFFFFFF));
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceError(u32);

// crate::impl_ranged_ty!(SceError);

/// The source facility of an error value.
#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ErrorFacility {
    Null   = 0x000,
    Errno  = 0x001,
    Kernel = 0x002,
    Registry = 0x008,
    Vsh    = 0x010,
    Utility = 0x011,
    SysFile = 0x012,
    MsApp  = 0x013,
    Umd    = 0x021,
    MemoryStick = 0x022,
    Flash  = 0x023,
    Usb    = 0x024,
    Syscon = 0x025,
    Audio  = 0x026,
    Lflash = 0x027,
    Lfatfs = 0x028,
    Sircs  = 0x029,
    Irda   = 0x02A,
    Power  = 0x02B,
    AudioRouting = 0x02C,
    MediaSync = 0x02D,
    Periph = 0x03F,
    Network = 0x041,
    Sas    = 0x042,
    Http   = 0x043,
    Wave   = 0x044,
    Snd    = 0x045,
    Font   = 0x046,
    P3DA   = 0x047,
    Magicgate = 0x050,
    Cphio  = 0x051,
    OpenPsId = 0x052,
    Dnas   = 0x053,
    Mtp    = 0x054,
    Np     = 0x055,
    GameUpdate = 0x056,
    Fmac   = 0x057,
    Face   = 0x058,
    Library = 0x05F,
    Mpeg   = 0x061,
    Avc    = 0x062,
    Atrac  = 0x063,
    Asf    = 0x064,
    Jpeg   = 0x065,
    Avi    = 0x066,
    MP3    = 0x067,
    G729   = 0x068,
    Aac    = 0x069,
    Codec  = 0x07F,
    #[default]
    Other,
}

impl SceError {
    /// Create a new Error structure from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible SceError
    /// values used by the PSP OS and homebrews, returning an [`None`] otherwise. That means
    /// that this function allows the creation of custom error codes outside of the range used in by
    /// the PSP OS.
    ///
    /// # Note
    /// This type has as constants all the OS error codes. We encourage to use them when possible.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0x80000001..=0xFFFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new Error structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }

    /// Get the facility of the error.
    pub const fn facility(self) -> ErrorFacility {
        match (self.as_inner() >> 16) & 0xFF {
            0x000 => ErrorFacility::Null,
            0x001 => ErrorFacility::Errno,
            0x002 => ErrorFacility::Kernel,
            0x008 => ErrorFacility::Registry,
            0x010 => ErrorFacility::Vsh,
            0x011 => ErrorFacility::Utility,
            0x012 => ErrorFacility::SysFile,
            0x013 => ErrorFacility::MsApp,
            0x021 => ErrorFacility::Umd,
            0x022 => ErrorFacility::MemoryStick,
            0x023 => ErrorFacility::Flash,
            0x024 => ErrorFacility::Usb,
            0x025 => ErrorFacility::Syscon,
            0x026 => ErrorFacility::Audio,
            0x027 => ErrorFacility::Lflash,
            0x028 => ErrorFacility::Lfatfs,
            0x029 => ErrorFacility::Sircs,
            0x02A => ErrorFacility::Irda,
            0x02B => ErrorFacility::Power,
            0x02C => ErrorFacility::AudioRouting,
            0x02D => ErrorFacility::MediaSync,
            0x03F => ErrorFacility::Periph,
            0x041 => ErrorFacility::Network,
            0x042 => ErrorFacility::Sas,
            0x043 => ErrorFacility::Http,
            0x044 => ErrorFacility::Wave,
            0x045 => ErrorFacility::Snd,
            0x046 => ErrorFacility::Font,
            0x047 => ErrorFacility::P3DA,
            0x050 => ErrorFacility::Magicgate,
            0x051 => ErrorFacility::Cphio,
            0x052 => ErrorFacility::OpenPsId,
            0x053 => ErrorFacility::Dnas,
            0x054 => ErrorFacility::Mtp,
            0x055 => ErrorFacility::Np,
            0x056 => ErrorFacility::GameUpdate,
            0x057 => ErrorFacility::Fmac,
            0x058 => ErrorFacility::Face,
            0x05F => ErrorFacility::Library,
            0x061 => ErrorFacility::Mpeg,
            0x062 => ErrorFacility::Avc,
            0x063 => ErrorFacility::Atrac,
            0x064 => ErrorFacility::Asf,
            0x065 => ErrorFacility::Jpeg,
            0x066 => ErrorFacility::Avi,
            0x067 => ErrorFacility::MP3,
            0x068 => ErrorFacility::G729,
            0x069 => ErrorFacility::Aac,
            0x07F => ErrorFacility::Codec,
            _ => ErrorFacility::Other,
        }
    }
}

macro_rules! __err_def {
    ($($err:ident = $val:expr;)+) => {
        impl SceError {
            $(pub const $err: Self = unsafe {Self($val)};)+
        }
    };
}

// Generic Error
__err_def!(
    NOT_INIT = 0x80000001;
    WRONG_VERSION = 0x80000002;
    NOT_IMPL= 0x80000003;
    NOT_SUPPORTED = 0x80000004;
    ALREADY = 0x80000020;
    BUSY = 0x80000021;
    OUT_OF_MEMORY = 0x80000022;
    PRIV_REQUIRED = 0x80000023;
    TIMEOUT = 0x80000024;
    NOT_FOUND = 0x80000025;
    ILLEGAL_CONTEXT = 0x80000030;
    CPUDI = 0x80000031;
    THREAD = 0x80000040;
    SEMAPHORE = 0x80000041;
    EVENT_FLAG = 0x80000042;
    TIMER = 0x80000043;
    ALARM = 0x80000044;
    INVALID_ID = 0x80000100;
    INVALID_NAME = 0x80000101;
    INVALID_INDEX = 0x80000102;
    INVALID_POINTER = 0x80000103;
    INVALID_SIZE = 0x80000104;
    INVALID_FLAG = 0x80000105;
    INVALID_COMMAND = 0x80000106;
    INVALID_MODE = 0x80000107;
    INVALID_FORMAT = 0x80000108;
    INVALID_VALUE = 0x800001FE;
    INVALID_ARGUMENT = 0x800001FF;
    NO_ENTRY = 0x80000202;
    BAD_FILE = 0x80000209;
    ACCESS_ERROR = 0x8000020D;
    FILE_EXIST = 0x80000211;
    INVAL = 0x80000216;
    MFILE = 0x80000218;
    NO_SPACE = 0x8000021C;
    DRIVER_FUNC = 0x800002FF;
);

// Standard errors
__err_def!(
    OPERATION_NOT_PERMITTED = 0x80010001;
    FILE_NOT_FOUND = 0x80010002;
    SEARCH = 0x80010003;
    INTERRUPTED = 0x80010004;
    IO = 0x80010005;
    RR_DEVICE_IO = 0x80010006;
    ARG_LIST_TOO_LONG = 0x80010007;
    NOT_EXEC = 0x80010008;
    INVALID_FILE_DESCRIPTOR = 0x80010009;
    CHILD_PROC_NOT_FOUND = 0x8001000A;
    RESOURCE_UNAVAILABLE = 0x8001000B;
    NO_MEMORY = 0x8001000C;
    NO_PERM = 0x8001000D;
    FILE_INVALID_ADDR = 0x8001000E;
    NOT_BLOCK_DEV = 0x8001000F;
    DEVICE_BUSY = 0x80010010;
    FILE_ALREADY_EXISTS = 0x80010011;
    CROSS_DEV_LINK = 0x80010012;
    DEVICE_NOT_FOUND = 0x80010013;
    NOT_A_DIRECTORY = 0x80010014;
    IS_DIRECTORY = 0x80010015;
    STD_INVALID_ARGUMENT = 0x80010016;
    TOO_MANY_OPEN_SYSTEM_FILES = 0x80010018;
    NOT_TTY = 0x80010019;
    TEXT_FILE_BUSY = 0x8001001A;
    FILE_IS_TOO_BIG = 0x8001001B;
    DEVICE_NO_FREE_SPACE = 0x8001001C;
    NOT_SEEKABLE = 0x8001001D;
    READ_ONLY = 0x8001001E;
    TOO_MANY_LINKS = 0x8001001F;
    BROKEN_PIPE = 0x80010020;
    MATH_DOMAIN = 0x80010021;
    MATH_RANGE = 0x80010022;
    NO_MESSAGE = 0x80010023;
    ID_DELETED = 0x80010024;
    CHANNEL_RANGE = 0x80010025;
    ASYNC = 0x80010026;
    HALTED = 0x80010027;
    RESETED = 0x80010028;
    LINK_RANGE = 0x80010029;
    NOT_ATTACHED = 0x8001002A;
    DEADLOCK = 0x8001002D;
    NOT_LOCKED = 0x8001002E;
    INVALID_FILE_FMT = 0x8001002F;
    NOT_SUPPORTED_OP = 0x80010030;
    BAD_EXCHANGE = 0x80010032;
    INVALID_SLOT = 0x80010037;
    FILE_DEADLOCK = 0x80010038;
    INVALID_FONT = 0x80010039; // Not sure, but only happens when error opening fonts so far
    NOT_STREAM_DEV = 0x8001003C;
    NO_DATA_FOUND = 0x8001003D;
    FILE_PROTOCOL = 0x80010047;
    DIRECTORY_IS_NOT_EMPTY = 0x8001005A;
    NAME_TOO_LONG = 0x8001005B;
    TOO_MANY_SYMBOLIC_LINKS = 0x8001005C;
    CONNECTION_RESET = 0x80010068;
    NO_FREE_BUF_SPACE = 0x80010069;
    SHUTDOWN = 0x8001006E;
    CONNECTION_REFUSED = 0x8001006F;
    ADDR_IN_USE = 0x80010070 ;
    CONNECTION_ABORTED = 0x80010071 ;
    NETWORK_UNREACHABLE = 0x80010072;
    NETWORK_DOWN = 0x80010073;
    TIMEDOUT = 0x80010074 ;
    HOST_DOWN = 0x80010075;
    HOST_UNREACHABLE = 0x80010076;
    IN_PROGRESS = 0x80010077;
    STD_ALREADY = 0x80010078;
    INVALID_PROTOCOL = 0x8001007B ;
    INVALID_SOCKET_TYPE = 0x8001007C ;
    ADDR_NOT_AVAILABLE = 0x8001007D;
    IS_ALREADY_CONNECTED = 0x8001007F;
    NOT_CONNECTED = 0x80010080;
    FILE_QUOTA_EXCEEDED = 0x80010084;
    STALE_NETWORK_HANDLE = 0x80010085;
    STD_NOT_SUPPORTED = 0x80010086;
    NO_MEDIUM = 0x80010087;
    ADDR_OUT_MAIN_MEMORY = 0x8001B001;
    INVALID_NUM_UNIT = 0x8001B002;
    INVALID_FILE_SIZE = 0x8001B003;
    STD_INVALID_FLAG = 0x8001B004;
    NO_CACHE = 0x8001B005;
    WRONG_MEDIUM = 0x8001B006;
);

// Kernel errors
__err_def!(
    KERNEL_ERROR  = 0x80020001;
    KERNEL_NOTIMP = 0x80020002;
    KERNEL_ILLEGAL_EXPCODE = 0x80020032;
    KERNEL_EXPHANDLER_NOUSE = 0x80020033;
    KERNEL_EXPHANDLER_USED = 0x80020034;
    KERNEL_SYCALLTABLE_NOUSED = 0x80020035;
    KERNEL_SYCALLTABLE_USED = 0x80020036;
    KERNEL_INVALID_SYSCALLTABLE = 0x80020037;
    KERNEL_INVALID_PRIMARY_SYSCALL_NUMBER = 0x80020038;
    KERNEL_PRIMARY_SYSCALL_NUMBER_INUSE = 0x80020039;
    KERNEL_INVALID_NMI_CODE = 0x8002003A;
    KERNEL_INVALID_CONTEXT = 0x80020064;
    KERNEL_INVALID_INTRCODE = 0x80020065;
    KERNEL_CPUDI = 0x80020066;
    KERNEL_FOUND_HANDLER = 0x80020067;
    KERNEL_NOTFOUND_HANDLER = 0x80020068;
    KERNEL_ILLEGAL_INTRLEVEL = 0x80020069;
    KERNEL_ILLEGAL_ADDRESS = 0x8002006a;
    KERNEL_INVALID_INTRPARAM = 0x8002006b;
    KERNEL_INVALID_STACK_ADDRESS = 0x8002006c;
    KERNEL_ALREADY_STACK_SET = 0x8002006d;
    KERNEL_NO_TIMER = 0x80020096;
    KERNEL_INVALID_TIMERID = 0x80020097;
    KERNEL_ILLEGAL_SOURCE = 0x80020098;
    KERNEL_INVALID_PRESCALE = 0x80020099;
    KERNEL_TIMER_BUSY = 0x8002009a;
    KERNEL_TIMER_NOT_SETUP = 0x8002009b;
    KERNEL_TIMER_NOT_INUSE = 0x8002009c;
    KERNEL_UNIT_USED = 0x800200a0;
    KERNEL_UNIT_NOUSE = 0x800200a1;
    KERNEL_UNIT_NO_ROMDIR = 0x800200a2;
    KERNEL_IDTYPE_EXIST = 0x800200c8;
    KERNEL_IDTYPE_NOT_EXIST = 0x800200c9;
    KERNEL_IDTYPE_NOT_EMPTY = 0x800200ca;
    KERNEL_UNKNOWN_UID = 0x800200cb;
    KERNEL_UNMATCH_UID_TYPE = 0x800200cc;
    KERNEL_ID_NOT_EXIST = 0x800200cd;
    KERNEL_NOT_FOUND_UIDFUNC = 0x800200ce;
    KERNEL_UID_ALREADY_HOLDER = 0x800200cf;
    KERNEL_UID_NOT_HOLDER = 0x800200d0;
    KERNEL_INVALID_PERM = 0x800200d1;
    KERNEL_INVALID_ARGUMENT = 0x800200d2;
    KERNEL_ILLEGAL_ADDR = 0x800200d3;
    KERNEL_OUT_OF_RANGE = 0x800200d4;
    KERNEL_MEM_RANGE_OVERLAP = 0x800200d5;
    KERNEL_INVALID_PARTITION = 0x800200d6;
    KERNEL_PARTITION_INUSE = 0x800200d7;
    KERNEL_INVALID_MEMBLOCKTYPE = 0x800200d8;
    KERNEL_MEMBLOCK_ALLOC_FAILED = 0x800200d9;
    KERNEL_MEMBLOCK_RESIZE_LOCKED = 0x800200da;
    KERNEL_MEMBLOCK_RESIZE_FAILED = 0x800200db;
    KERNEL_HEAPBLOCK_ALLOC_FAILED = 0x800200dc;
    KERNEL_HEAP_ALLOC_FAILED = 0x800200dd;
    KERNEL_INVALID_CHUNK_ID = 0x800200de;
    KERNEL_NOCHUNK = 0x800200df;
    KERNEL_NO_FREECHUNK = 0x800200e0;
    KERNEL_MEMBLOCK_NOT_NEIGHBOR = 0x800200e1;
    KERNEL_MEMBLOCK_JOINT_FAIL = 0x800200e2;
    KERNEL_MEMBLOCK_SEPARATED_FAIL = 0x800200e3;
    KERNEL_INVALID_ALIGNMENT = 0x800200e4;
    KERNEL_INVALID_DEVKIT_VERSION = 0x800200e5;
    KERNEL_LINK = 0x8002012c;
    KERNEL_INVALID_OBJECT = 0x8002012d;
    KERNEL_UNKNOWN_MODULE = 0x8002012e;
    KERNEL_NOFILE = 0x8002012f;
    KERNEL_FILEERR = 0x80020130;
    KERNEL_MEMINUSE = 0x80020131;
    KERNEL_PARTITION_MISMATCH = 0x80020132;
    KERNEL_ALREADY_STARTED = 0x80020133;
    KERNEL_NOT_STARTED = 0x80020134;
    KERNEL_ALREADY_STOPPED = 0x80020135;
    KERNEL_CAN_NOT_STOP = 0x80020136;
    KERNEL_NOT_STOPPED = 0x80020137;
    KERNEL_NOT_REMOVABLE = 0x80020138;
    KERNEL_EXCLUSIVE_LOAD = 0x80020139;
    KERNEL_LIBRARY_NOT_YET_LINKED = 0x8002013a;
    KERNEL_LIBRARY_FOUND = 0x8002013b;
    KERNEL_LIBRARY_NOTFOUND = 0x8002013c;
    KERNEL_INVALID_LIBRARY = 0x8002013d;
    KERNEL_LIBRARY_INUSE = 0x8002013e;
    KERNEL_ALREADY_STOPPING = 0x8002013f;
    KERNEL_INVALID_OFFSET = 0x80020140;
    KERNEL_INVALID_POSITION = 0x80020141;
    KERNEL_INVALID_ACCESS = 0x80020142;
    KERNEL_MODULE_MGR_BUSY = 0x80020143;
    KERNEL_INVALID_FLAG = 0x80020144;
    KERNEL_CANNOT_GET_MODULELIST = 0x80020145;
    KERNEL_PROHIBIT_LOADMODULE_DEVICE = 0x80020146;
    KERNEL_PROHIBIT_DEVICE = 0x80020147;
    KERNEL_UNSUPPORTED_PRX_TYPE = 0x80020148;
    KERNEL_INVALID_PERM_CALL = 0x80020149;
    KERNEL_CANNOT_GET_MODULE_INFORMATION = 0x8002014a;
    KERNEL_INVALID_BUFFER = 0x8002014b;
    KERNEL_INVALID_FILENAME = 0x8002014c;
    KERNEL_NO_EXIT_CALLBACK = 0x8002014d;
    KERNEL_MEDIA_CHANGED = 0x8002014e;
    KERNEL_USE_BETA_VERSION_MODULE = 0x8002014f;
    KERNEL_BSOD = 0x80020150;
    KERNEL_REBOOT_AFTER_HIBERNATION = 0x80020151;
    KERNEL_LICENSE_EXPIRED = 0x80020152;
    KERNEL_NO_MEMORY = 0x80020190;
    KERNEL_INVALID_ATTR = 0x80020191;
    KERNEL_INVALID_ENTRY = 0x80020192;
    KERNEL_INVALID_PRIORITY = 0x80020193;
    KERNEL_INVALID_STACK_SIZE = 0x80020194;
    KERNEL_INVALID_MODE = 0x80020195;
    KERNEL_INVALID_MASK = 0x80020196;
    KERNEL_INVALID_THID = 0x80020197;
    KERNEL_UNKNOWN_THID = 0x80020198;
    KERNEL_UNKNOWN_SEMID = 0x80020199;
    KERNEL_UNKNOWN_EVFID = 0x8002019a;
    KERNEL_UNKNOWN_MBXID = 0x8002019b;
    KERNEL_UNKNOWN_VPLID = 0x8002019c;
    KERNEL_UNKNOWN_FPLID = 0x8002019d;
    KERNEL_UNKNOWN_MPPID = 0x8002019e;
    KERNEL_UNKNOWN_ALMID = 0x8002019f;
    KERNEL_UNKNOWN_TEID = 0x800201a0;
    KERNEL_UNKNOWN_CBID = 0x800201a1;
    KERNEL_DORMANT = 0x800201a2;
    KERNEL_SUSPEND = 0x800201a3;
    KERNEL_NOT_DORMANT = 0x800201a4;
    KERNEL_NOT_SUSPEND = 0x800201a5;
    KERNEL_NOT_WAIT = 0x800201a6;
    KERNEL_CAN_NOT_WAIT = 0x800201a7;
    KERNEL_WAIT_TIMEOUT = 0x800201a8;
    KERNEL_WAIT_CANCEL = 0x800201a9;
    KERNEL_RELEASE_WAIT = 0x800201aa;
    KERNEL_NOTIFY_CALLBACK = 0x800201ab;
    KERNEL_THREAD_TERMINATED = 0x800201ac;
    KERNEL_SEMA_ZERO = 0x800201ad;
    KERNEL_SEMA_OVF = 0x800201ae;
    KERNEL_EVF_COND = 0x800201af;
    KERNEL_EVF_MULTI = 0x800201b0;
    KERNEL_EVF_ILPAT = 0x800201b1;
    KERNEL_MBOX_NOMSG = 0x800201b2;
    KERNEL_MPP_FULL = 0x800201b3;
    KERNEL_MPP_EMPTY = 0x800201b4;
    KERNEL_WAIT_DELETE = 0x800201b5;
    KERNEL_INVALID_MEMBLOCK = 0x800201b6;
    KERNEL_INVALID_MEMSIZE = 0x800201b7;
    KERNEL_INVALID_SPADADDR = 0x800201b8;
    KERNEL_SPAD_INUSE = 0x800201b9;
    KERNEL_SPAD_NOT_INUSE = 0x800201ba;
    KERNEL_INVALID_TYPE = 0x800201bb;
    KERNEL_INVALID_SIZE = 0x800201bc;
    KERNEL_INVALID_COUNT = 0x800201bd;
    KERNEL_UNKNOWN_VTID = 0x800201be;
    KERNEL_INVALID_VTID = 0x800201bf;
    KERNEL_INVALID_KTLSID = 0x800201c0;
    KERNEL_KTLS_FULL = 0x800201c1;
    KERNEL_KTLS_BUSY = 0x800201c2;
    KERNEL_MUTEX_UNKNOWN_ID = 0x800201c3;
    KERNEL_MUTEX_OWN = 0x800201c4;
    KERNEL_MUTEX_NOT_OWNED = 0x800201c5;
    KERNEL_MUTEX_LOCK_OVERFLOW = 0x800201c6;
    KERNEL_MUTEX_UNLOCK_UNDERFLOW = 0x800201c7;
    KERNEL_MUTEX_NOT_REENTRANT = 0x800201c8;
    KERNEL_MSG_BOX_LOOP = 0x800201c9;
    KERNEL_LWMUTEX_UNKNOWN_ID = 0x800201ca;
    KERNEL_LWMUTEX_OWN = 0x800201cb;
    KERNEL_LWMUTEX_NOT_OWNED = 0x800201cc;
    KERNEL_LWMUTEX_LOCK_OVERFLOW = 0x800201cd;
    KERNEL_LWMUTEX_UNLOCK_UNDERFLOW = 0x800201ce;
    KERNEL_LWMUTEX_NOT_REENTRANT = 0x800201cf;
    KERNEL_TLS_POOL_UNKNWON_ID = 0x800201d0;
    KERNEL_UTLS_IS_FULL = 0x800201d1;
    KERNEL_UTLS_IS_BUSY = 0x800201d2;
    KERNEL_PM_INVALID_PRIORITY = 0x80020258;
    KERNEL_PM_INVALID_DEVNAME = 0x80020259;
    KERNEL_PM_UNKNOWN_DEVNAME = 0x8002025a;
    KERNEL_PM_PMINFO_REGISTERED = 0x8002025b;
    KERNEL_PM_PMINFO_UNREGISTERED = 0x8002025c;
    KERNEL_PM_INVALID_MAJOR_STATE = 0x8002025d;
    KERNEL_PM_INVALID_REQUEST = 0x8002025e;
    KERNEL_PM_UNKNOWN_REQUEST = 0x8002025f;
    KERNEL_PM_INVALID_UNIT = 0x80020260;
    KERNEL_PM_CANNOT_CANCEL = 0x80020261;
    KERNEL_PM_INVALID_PMINFO = 0x80020262;
    KERNEL_PM_INVALID_ARGUMENT = 0x80020263;
    KERNEL_PM_ALREADY_TARGET_PWRSTATE = 0x80020264;
    KERNEL_PM_CHANGE_PWRSTATE_FAILED = 0x80020265;
    KERNEL_PM_CANNOT_CHANGE_DEVPWR_STATE = 0x80020266;
    KERNEL_PM_NO_SUPPORT_DEVPWR_STATE = 0x80020267;
    KERNEL_DMAC_REQUEST_FAILED = 0x800202bc;
    KERNEL_DMAC_REQUEST_DENIED = 0x800202bd;
    KERNEL_DMAC_OP_QUEUED = 0x800202be;
    KERNEL_DMAC_OP_NOT_QUEUED = 0x800202bf;
    KERNEL_DMAC_OP_RUNNING = 0x800202c0;
    KERNEL_DMAC_OP_NOT_ASSIGNED = 0x800202c1;
    KERNEL_DMAC_OP_TIMEOUT = 0x800202c2;
    KERNEL_DMAC_OP_FREED = 0x800202c3;
    KERNEL_DMAC_OP_USED = 0x800202c4;
    KERNEL_DMAC_OP_EMPTY = 0x800202c5;
    KERNEL_DMAC_OP_ABORTED = 0x800202c6;
    KERNEL_DMAC_OP_ERROR = 0x800202c7;
    KERNEL_DMAC_CHANNEL_RESERVED = 0x800202c8;
    KERNEL_DMAC_CHANNEL_EXCLUDED = 0x800202c9;
    KERNEL_DMAC_PRIVILEGE_ADDRESS = 0x800202ca;
    KERNEL_DMAC_NO_ENOUGHSPACE = 0x800202cb;
    KERNEL_DMAC_CHANNEL_NOT_ASSIGNED = 0x800202cc;
    KERNEL_DMAC_CHILD_OPERATION = 0x800202cd;
    KERNEL_DMAC_TOO_MUCH_SIZE = 0x800202ce;
    KERNEL_DMAC_INVALID_ARGUMENT = 0x800202cf;
    KERNEL_MFILE = 0x80020320;
    KERNEL_NODEV = 0x80020321;
    KERNEL_XDEV = 0x80020322;
    KERNEL_BADF = 0x80020323;
    KERNEL_INVAL = 0x80020324;
    KERNEL_UNSUP = 0x80020325;
    KERNEL_ALIAS_USED = 0x80020326;
    KERNEL_CANNOT_MOUNT = 0x80020327;
    KERNEL_DRIVER_DELETED = 0x80020328;
    KERNEL_ASYNC_BUSY = 0x80020329;
    KERNEL_NOASYNC = 0x8002032a;
    KERNEL_REGDEV = 0x8002032b;
    KERNEL_NOCWD = 0x8002032c;
    KERNEL_NAMETOOLONG = 0x8002032d;
    KERNEL_STDIO_NOT_OPEN = 0x80020384;
    KERNEL_DECI2P_UNKNOWN_SOCKET_ID = 0x800203e8;
    KERNEL_DECI2P_REGISTERED_PROTOCOL = 0x800203e9;
    KERNEL_DECI2P_TOO_MANY_PROTOCOL = 0x800203ea;
    KERNEL_DECI2P_SMALL_BUFF = 0x800203eb;
    KERNEL_DECI2P_INVALID_PROTOCOL = 0x800203ec;
    KERNEL_DECI2P_INVALID_HEADER = 0x800203ed;
    KERNEL_DECI2P_NO_SPACE = 0x800203ef;
    KERNEL_DECI2P_NO_HOST = 0x800203f0;
    KERNEL_CACHE_ALIGNMENT = 0x8002044c;
    KERNEL_ERRORMAX = 0x8002044d;
);

// 1.50 Error codes
__err_def!(
    ERROR150_ENAMETOOLONG = 0x80010024;
    ERROR150_EADDRINUSE = 0x80010062;
    ERROR150_ECONNABORTED = 0x80010067;
    ERROR150_ETIMEDOUT = 0x8001006E;
    ERROR150_ENOMEDIUM = 0x8001007B;
    ERROR150_EMEDIUMTYPE = 0x8001007C;
    ERROR150_ENOTSUP = 0x8001B000;
);

// Atrac errors
__err_def!(
    ATRAC_PARAM_FAIL = 0x80630001;
    ATRAC_API_FAIL = 0x80630002;
    ATRAC_NO_ATRACID = 0x80630003;
    ATRAC_BAD_CODECTYPE = 0x80630004;
    ATRAC_BAD_ATRACID = 0x80630005;
    ATRAC_UNKNOWN_FORMAT = 0x80630006;
    ATRAC_UNMATCH_FORMAT = 0x80630007;
    ATRAC_BAD_DATA = 0x80630008;
    ATRAC_ALLDATA_IS_ONMEMORY = 0x80630009;
    ATRAC_UNSET_DATA = 0x80630010;

    ATRAC_READSIZE_IS_TOO_SMALL = 0x80630011;
    ATRAC_NEED_SECOND_BUFFER = 0x80630012;
    ATRAC_READSIZE_OVER_BUFFER = 0x80630013;
    ATRAC_NOT_4BYTE_ALIGNMENT = 0x80630014;
    ATRAC_BAD_SAMPLE = 0x80630015;
    ATRAC_WRITEBYTE_FIRST_BUFFER = 0x80630016;
    ATRAC_WRITEBYTE_SECOND_BUFFER = 0x80630017;
    ATRAC_ADD_DATA_IS_TOO_BIG = 0x80630018;

    ATRAC_UNSET_PARAM = 0x80630021;
    ATRAC_NONEED_SECOND_BUFFER = 0x80630022;
    ATRAC_NODATA_IN_BUFFER = 0x80630023;
    ATRAC_ALLDATA_WAS_DECODED = 0x80630024;
);

// Audio errors
__err_def!(
    AUDIO_NOT_INITIALIZED = 0x80260001;
    AUDIO_OUTPUT_BUSY = 0x80260002;
    AUDIO_INVALID_CH = 0x80260003;
    AUDIO_PRIV_REQUIRED = 0x80260004;
    AUDIO_NOT_FOUND = 0x80260005;
    AUDIO_INVALID_SIZE = 0x80260006;
    AUDIO_INVALID_FORMAT = 0x80260007;
    AUDIO_NOT_RESERVED = 0x80260008;
    AUDIO_NOT_OUTPUT = 0x80260009;
    AUDIO_INVALID_FREQUENCY = 0x8026000A;
    AUDIO_INVALID_VOLUME = 0x8026000B;
    AUDIO_INPUT_BUSY = 0x80260010;
);

// Display errors
__err_def!(
    DISPLAY_POINTER = 0x80000103;
    DISPLAY_ARGUMENT = 0x80000107;
);

// JPEG errors
__err_def!(
    JPEG_BAD_MARKER_LENGTH = 0x80650004;
    JPEG_INVALID_POINTER = 0x80650010;
    JPEG_UNSUPPORT_COLORSPACE = 0x80650013;
    JPEG_UNSUPPORT_SAMPLING = 0x80650016;
    JPEG_UNSUPPORT_IMAGE_SIZE = 0x80650020;
    JPEG_UNKNOWN_MARKER = 0x80650035;
);

// Chnnlsv errors
__err_def!(
    CHNNLSV_ILLEGAL_SIZE = 0xFFFFFBFE;
    CHNNLSV_ILLEGAL_ALIGNMENT_SIZE = 0xFFFFFBFF;
    CHNNLSV_KIRK_14_ERROR = 0xFFFFFEFB;
    CHNNLSV_SEMA_ERROR = 0xFFFFFEFC;
    CHNNLSV_ILLEGAL_ADDR = 0xFFFFFEFD;
    CHNNLSV_KIRK_IV_FUSE_ERROR = 0xFFFFFEFE;
    CHNNLSV_KIRK_IV_ERROR = 0xFFFFFEFF;
);


impl core::error::Error for SceError {}

impl core::fmt::Debug for SceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("SceError");

        d.field("facility", &self.facility())
            .field("code", &format_args!("{:#010X}", self.as_inner()));

        let msg = self.error_msg();

        if !msg.is_empty() {
            d.field("message", &msg);
        }

        d.finish()
    }
}

impl core::fmt::Display for SceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let err = self.error_msg();

        if err.is_empty() {
            write!(f, "{} error: {:#010X}", self.facility(), self.as_inner())
        } else {
            f.pad(err)
        }
    }
}

impl SceError {
    // FIXME: Add more errors
    pub(crate) const fn error_msg(&self) -> &'static str {
        match *self {
            SceError::NOT_INIT => "not initialized",
            SceError::WRONG_VERSION => "version does not match",
            SceError::NOT_IMPL => "not implemented",
            SceError::NOT_SUPPORTED => "operation not supported",
            SceError::ALREADY => "operation already in process",
            SceError::BUSY => "currently not possible to be processed",
            SceError::OUT_OF_MEMORY => "insufficient memory",
            SceError::PRIV_REQUIRED => "superior privilege required",
            SceError::TIMEOUT => "timeout reached",
            SceError::NOT_FOUND => "not found",
            SceError::ILLEGAL_CONTEXT => "illegal call done during interrupt handler",
            SceError::CPUDI => "process can not be executed in interrupt-disabled state",
            SceError::THREAD => "thread processing is not possible",
            SceError::SEMAPHORE => "semaphore processing is not possible",
            SceError::EVENT_FLAG => "event-flag processing is not possible",
            SceError::TIMER => "timer processing is not possible",
            SceError::ALARM => "alarm processing is not possible",
            SceError::INVALID_ID => "invalid ID argument",
            SceError::INVALID_NAME => "invalid name argument",
            SceError::INVALID_INDEX => "invalid index argument",
            SceError::INVALID_POINTER => "invalid pointer argument",
            SceError::INVALID_SIZE => "invalid size argument",
            SceError::INVALID_FLAG => "invalid flag argument",
            SceError::INVALID_COMMAND => "invalid command argument",
            SceError::INVALID_MODE => "invalid mode argument",
            SceError::INVALID_FORMAT => "invalid format argument",
            SceError::INVALID_VALUE => "invalid value argument",
            SceError::INVALID_ARGUMENT => "invalid argument",
            SceError::NO_ENTRY => "FAT driver: file or directory entry not found",
            SceError::BAD_FILE => "FAT driver: invalid file ID",
            SceError::ACCESS_ERROR => "FAT driver: permission denied",
            SceError::FILE_EXIST => {
                "FAT driver: file or directory with the same name already exists"
            },
            SceError::INVAL => "FAT driver: invalid argument",
            SceError::MFILE => "FAT driver: too many opened files",
            SceError::NO_SPACE => "FAT driver: device with insufficient space",
            SceError::DRIVER_FUNC => "FAT driver: internal driver error",

            SceError::OPERATION_NOT_PERMITTED => "operation not permitted",
            SceError::FILE_NOT_FOUND => "file or directory does not exist",
            SceError::SEARCH => "process does not exist",
            SceError::INTERRUPTED => "function was interrupted",
            SceError::IO => "I/O error",
            SceError::RR_DEVICE_IO => "device or device address was not found",
            SceError::ARG_LIST_TOO_LONG => "argument list is too long",
            SceError::NOT_EXEC => "not an executable file",
            SceError::INVALID_FILE_DESCRIPTOR => "invalid file descriptor",
            SceError::CHILD_PROC_NOT_FOUND => "child process not found",
            SceError::RESOURCE_UNAVAILABLE => "resource is temporarily unavailable",
            SceError::NO_MEMORY => "not enough memory",
            SceError::NO_PERM => "not enough file permission",
            SceError::FILE_INVALID_ADDR => "invalid file address",
            SceError::NOT_BLOCK_DEV => "not a block device",
            SceError::DEVICE_BUSY => "device or mount point is busy",
            SceError::FILE_ALREADY_EXISTS => "file already exists",
            SceError::CROSS_DEV_LINK => "cross-device link",
            SceError::DEVICE_NOT_FOUND => "device or mount point not found",
            SceError::NOT_A_DIRECTORY => "not a directory",
            SceError::IS_DIRECTORY => "is a directory",
            SceError::STD_INVALID_ARGUMENT => "invalid I/O argument",
            SceError::TOO_MANY_OPEN_SYSTEM_FILES => "too many open file on system",
            SceError::NOT_TTY => "not a TTY",
            SceError::TEXT_FILE_BUSY => "text file is busy",
            SceError::FILE_IS_TOO_BIG => "file is too big",
            SceError::DEVICE_NO_FREE_SPACE => "not enough space on device or mount point",
            SceError::NOT_SEEKABLE => "illegal seek operation or not seekable",
            SceError::READ_ONLY => "read-only filesystem",
            SceError::TOO_MANY_LINKS => "too many links",
            SceError::BROKEN_PIPE => "broken pipe",
            SceError::MATH_DOMAIN => "numeric value outside of domain",
            SceError::MATH_RANGE => "numeric value outside of expected range",
            SceError::NO_MESSAGE => "requested message not found",
            SceError::ID_DELETED => "deleted ID",
            SceError::CHANNEL_RANGE => "channel number out of range",
            SceError::ASYNC => "process is asynchronous",
            SceError::HALTED => "process has halted",
            SceError::RESETED => "process was reseted",
            SceError::LINK_RANGE => "link value out of range",
            SceError::NOT_ATTACHED => "driver not attached",
            SceError::DEADLOCK => "deadlock",
            SceError::NOT_LOCKED => "not locked or not possible to lock",
            SceError::INVALID_FILE_FMT => "invalid file format",
            SceError::NOT_SUPPORTED_OP => "file operation not supported on device or mount point",
            SceError::BAD_EXCHANGE => "bad or invalid exchange",
            SceError::INVALID_SLOT => "invalid slot value",
            SceError::FILE_DEADLOCK => "file lock deadlock",
            SceError::INVALID_FONT => "invalid font format",
            SceError::NOT_STREAM_DEV => "not a stream device",
            SceError::NO_DATA_FOUND => "no data was found",
            SceError::FILE_PROTOCOL => "file protocol error",
            SceError::DIRECTORY_IS_NOT_EMPTY => "directory is not empty",
            SceError::NAME_TOO_LONG => "file name or path is too long",
            SceError::TOO_MANY_SYMBOLIC_LINKS => "too many symbolic links",
            SceError::CONNECTION_RESET => "connection was reset",
            SceError::NO_FREE_BUF_SPACE => "not enough space on buffer",
            SceError::SHUTDOWN => "socket shutdown",
            SceError::CONNECTION_REFUSED => "connection was refused",
            SceError::ADDR_IN_USE => "address already in use",
            SceError::CONNECTION_ABORTED => "connection was aborted",
            SceError::NETWORK_UNREACHABLE => "network is unreachable",
            SceError::NETWORK_DOWN => "network is down",
            SceError::TIMEDOUT => "operation timed out",
            SceError::HOST_DOWN => "host is down",
            SceError::HOST_UNREACHABLE => "host is unreachable",
            SceError::IN_PROGRESS => "operation in progress",
            SceError::STD_ALREADY => "operation is already in progress",
            SceError::INVALID_PROTOCOL => "protocol not supported",
            SceError::INVALID_SOCKET_TYPE => "invalid socket type",
            SceError::ADDR_NOT_AVAILABLE => "address not available or allocated",
            SceError::IS_ALREADY_CONNECTED => "socket already connected",
            SceError::NOT_CONNECTED => "socket not connected",
            SceError::FILE_QUOTA_EXCEEDED => "quota exceeded",
            SceError::STALE_NETWORK_HANDLE => "stale network filesystem file handle",
            SceError::STD_NOT_SUPPORTED => "operation not supported",
            SceError::NO_MEDIUM => "no UMD media found",
            SceError::ADDR_OUT_MAIN_MEMORY => "address out of main userland memory",
            SceError::INVALID_NUM_UNIT => "invalid unit number",
            SceError::INVALID_FILE_SIZE => "invalid file size",
            SceError::STD_INVALID_FLAG => "invalid file flag",
            SceError::NO_CACHE => "no cache",
            SceError::WRONG_MEDIUM => "wrong UMD media kind",

            SceError::KERNEL_ERROR => "kernel error: generic error",
            SceError::KERNEL_NOTIMP => "kernel error: not implemented",
            SceError::KERNEL_ILLEGAL_EXPCODE => "kernel error: illegal exception code",
            SceError::KERNEL_EXPHANDLER_NOUSE => "kernel error: exception handler not used",
            SceError::KERNEL_EXPHANDLER_USED => "kernel error:exception handler already used",
            SceError::KERNEL_SYCALLTABLE_NOUSED => "kernel error: syscall table not used",
            SceError::KERNEL_SYCALLTABLE_USED => "kernel error: syscall table already used",
            SceError::KERNEL_INVALID_SYSCALLTABLE => "kernel error: invalid syscall table",
            SceError::KERNEL_INVALID_PRIMARY_SYSCALL_NUMBER => {
                "kernel error: invalid syscall number"
            },
            SceError::KERNEL_PRIMARY_SYSCALL_NUMBER_INUSE => {
                "kernel error: syscall number already in use"
            },
            SceError::KERNEL_INVALID_NMI_CODE => "kernel error: invalid NMI code",
            SceError::KERNEL_INVALID_CONTEXT => {
                "kernel error: invalid call from interrupt handler or thread"
            },
            SceError::KERNEL_INVALID_INTRCODE => "kernel error: invalid instruction code",
            SceError::KERNEL_CPUDI => "kernel error: CPU interrupt already disabled",
            SceError::KERNEL_FOUND_HANDLER => "kernel error: handler already exists",
            SceError::KERNEL_NOTFOUND_HANDLER => "kernel error: handler not found",
            SceError::KERNEL_ILLEGAL_INTRLEVEL => "kernel error: illegal interrupt level",
            SceError::KERNEL_ILLEGAL_ADDRESS => "kernel error: illegal address",
            SceError::KERNEL_INVALID_INTRPARAM => "kernel error: invalid IntrHandlerOptions::size",
            SceError::KERNEL_INVALID_STACK_ADDRESS => "kernel error: invalid stack address",
            SceError::KERNEL_ALREADY_STACK_SET => "kernel error: stack address already set",
            SceError::KERNEL_NO_TIMER => "kernel error: no available timer found",
            SceError::KERNEL_INVALID_TIMERID => "kernel error: invalid timer ID",
            SceError::KERNEL_ILLEGAL_SOURCE => "kernel error: illegal source",
            SceError::KERNEL_INVALID_PRESCALE => "kernel error: invalid prescale",
            SceError::KERNEL_TIMER_BUSY => "kernel error: timer is busy",
            SceError::KERNEL_TIMER_NOT_SETUP => "kernel error: timer was not setup",
            SceError::KERNEL_TIMER_NOT_INUSE => "kernel error: timer is not in use",
            SceError::KERNEL_UNIT_USED => "kernel error: unit used",
            SceError::KERNEL_UNIT_NOUSE => "kernel error: unit not used",
            SceError::KERNEL_UNIT_NO_ROMDIR => "kernel error: unit no ROM directory",
            SceError::KERNEL_IDTYPE_EXIST => "kernel error: UID type already exists",
            SceError::KERNEL_IDTYPE_NOT_EXIST => "kernel error: UID type not found",
            SceError::KERNEL_IDTYPE_NOT_EMPTY => "kernel error: UID type not empty",
            SceError::KERNEL_UNKNOWN_UID => "kernel error: unknown UID",
            SceError::KERNEL_UNMATCH_UID_TYPE => "kernel error: unmatched UID type",
            SceError::KERNEL_ID_NOT_EXIST => "kernel error: UID does not exist",
            SceError::KERNEL_NOT_FOUND_UIDFUNC => "kernel error: UID function not found",
            SceError::KERNEL_UID_ALREADY_HOLDER => "kernel error: UID already holder",
            SceError::KERNEL_UID_NOT_HOLDER => "kernel error: UID is not holder",
            SceError::KERNEL_INVALID_PERM => "kernel error: invalid permission",
            SceError::KERNEL_INVALID_ARGUMENT => "kernel error: invalid argument",
            SceError::KERNEL_ILLEGAL_ADDR => "kernel error: illegal address",
            SceError::KERNEL_OUT_OF_RANGE => "kernel error: memory area out of range",
            SceError::KERNEL_MEM_RANGE_OVERLAP => "kernel error: memory area overlaps",
            SceError::KERNEL_INVALID_PARTITION => "kernel error: invalid partition ID",
            SceError::KERNEL_PARTITION_INUSE => "kernel error: partition currently in use",
            SceError::KERNEL_INVALID_MEMBLOCKTYPE => "kernel error: invalid memory block kind",
            SceError::KERNEL_MEMBLOCK_ALLOC_FAILED => {
                "kernel error: failed to allocate a memory block"
            },
            SceError::KERNEL_MEMBLOCK_RESIZE_LOCKED => {
                "kernel error: memory block resize is inhibited"
            },
            SceError::KERNEL_MEMBLOCK_RESIZE_FAILED => {
                "kernel error: failed to resize a memory block"
            },
            SceError::KERNEL_HEAPBLOCK_ALLOC_FAILED => {
                "kernel error: failed to allocate a heap block"
            },
            SceError::KERNEL_HEAP_ALLOC_FAILED => "kernel error: failed to allocate a heap",
            SceError::KERNEL_INVALID_CHUNK_ID => "kernel error: invalid chunk ID",
            SceError::KERNEL_NOCHUNK => "kernel error: chunk not found",
            SceError::KERNEL_NO_FREECHUNK => "kernel error: no available chunk",
            SceError::KERNEL_MEMBLOCK_NOT_NEIGHBOR => {
                "kernel error: memory blocks are not neighbors"
            },
            SceError::KERNEL_MEMBLOCK_JOINT_FAIL => "kernel error: failed to join memory blocks",
            SceError::KERNEL_MEMBLOCK_SEPARATED_FAIL => {
                "kernel error: failed to separate memory blocks"
            },
            SceError::KERNEL_INVALID_ALIGNMENT => "kernel error: invalid alignment",
            SceError::KERNEL_INVALID_DEVKIT_VERSION => "kernel error: invalid DevKit version",
            SceError::KERNEL_LINK => "kernel error: failed to link module",
            SceError::KERNEL_INVALID_OBJECT => "kernel error: invalid object format for module",
            SceError::KERNEL_UNKNOWN_MODULE => "kernel error: module not found",
            SceError::KERNEL_NOFILE => "kernel error: module file not found",
            SceError::KERNEL_FILEERR => "kernel error: failed to read module file",
            SceError::KERNEL_MEMINUSE => "kernel error: memory currently in use",
            SceError::KERNEL_PARTITION_MISMATCH => "kernel error: partition mismatch",
            SceError::KERNEL_ALREADY_STARTED => "kernel error: module already started",
            SceError::KERNEL_NOT_STARTED => "kernel error: module not started yet",
            SceError::KERNEL_ALREADY_STOPPED => "kernel error: module already stopped",
            SceError::KERNEL_CAN_NOT_STOP => "kernel error: module can not be stopped",
            SceError::KERNEL_NOT_STOPPED => "kernel error: module not stopped yet",
            SceError::KERNEL_NOT_REMOVABLE => "kernel error: non removable module",
            SceError::KERNEL_EXCLUSIVE_LOAD => {
                "kernel error: module is exclusive load (only one version can be loaded)"
            },
            SceError::KERNEL_LIBRARY_NOT_YET_LINKED => "kernel error: library not linked yet",
            SceError::KERNEL_LIBRARY_FOUND => "kernel error: library already exist",
            SceError::KERNEL_LIBRARY_NOTFOUND => "kernel error: library not found",
            SceError::KERNEL_INVALID_LIBRARY => "kernel error: invalid library header",
            SceError::KERNEL_LIBRARY_INUSE => "kernel error: library is currently in use",
            SceError::KERNEL_ALREADY_STOPPING => "kernel error: library is already stopping",
            SceError::KERNEL_INVALID_OFFSET => "kernel error: invalid module offset value",
            SceError::KERNEL_INVALID_POSITION => "kernel error: invalid load-module position value",
            SceError::KERNEL_INVALID_ACCESS => "kernel error: invalid load-module access value",
            SceError::KERNEL_MODULE_MGR_BUSY => "kernel error: module manager is busy",
            SceError::KERNEL_INVALID_FLAG => "kernel error: invalid module flags",
            SceError::KERNEL_CANNOT_GET_MODULELIST => "kernel error: failed to get module list",
            SceError::KERNEL_PROHIBIT_LOADMODULE_DEVICE => {
                "kernel error: prohibit load module device"
            },
            SceError::KERNEL_PROHIBIT_DEVICE => "kernel error: prohibit load exec device",
            SceError::KERNEL_UNSUPPORTED_PRX_TYPE => "kernel error: unsupported PRX type",
            SceError::KERNEL_INVALID_PERM_CALL => "kernel error: invalid permission call",
            SceError::KERNEL_CANNOT_GET_MODULE_INFORMATION => {
                "kernel error: failed to get module information"
            },
            SceError::KERNEL_INVALID_BUFFER => "kernel error: invalid load-exec buffer",
            SceError::KERNEL_INVALID_FILENAME => "kernel error: invalid load-exec filename",
            SceError::KERNEL_NO_EXIT_CALLBACK => "kernel error: no exit callback",
            SceError::KERNEL_MEDIA_CHANGED => "kernel error: media has changed",
            SceError::KERNEL_USE_BETA_VERSION_MODULE => "kernel error: be version module not used",
            SceError::KERNEL_BSOD => "kernel error: blue screen of death occurred",
            SceError::KERNEL_REBOOT_AFTER_HIBERNATION => {
                "kernel error: system rebooted after hibernation"
            },
            SceError::KERNEL_LICENSE_EXPIRED => "kernel error: license has expired",
            SceError::KERNEL_NO_MEMORY => "kernel error: no memory",
            SceError::KERNEL_INVALID_ATTR => "kernel error: invalid attribute parameter",
            SceError::KERNEL_INVALID_ENTRY => "kernel error: invalid entry function",
            SceError::KERNEL_INVALID_PRIORITY => "kernel error: invalid priority value",
            SceError::KERNEL_INVALID_STACK_SIZE => "kernel error: invalid stack size",
            SceError::KERNEL_INVALID_MODE => "kernel error: invalid mode",
            SceError::KERNEL_INVALID_MASK => "kernel error: invalid mask",
            SceError::KERNEL_INVALID_THID => "kernel error: invalid thread ID",
            SceError::KERNEL_UNKNOWN_THID => "kernel error: unknown thread ID",
            SceError::KERNEL_UNKNOWN_SEMID => "kernel error: unknown semaphore ID",
            SceError::KERNEL_UNKNOWN_EVFID => "kernel error: unknown event-flag ID",
            SceError::KERNEL_UNKNOWN_MBXID => "kernel error: unknown message-box ID",
            SceError::KERNEL_UNKNOWN_VPLID => "kernel error: unknown variable-poll ID",
            SceError::KERNEL_UNKNOWN_FPLID => "kernel error: unknown fixed-poll ID",
            SceError::KERNEL_UNKNOWN_MPPID => "kernel error: unknown message-pipe IS",
            SceError::KERNEL_UNKNOWN_ALMID => "kernel error: unknown alarm ID",
            SceError::KERNEL_UNKNOWN_TEID => "kernel error: unknown thread event-handle ID",
            SceError::KERNEL_UNKNOWN_CBID => "kernel error: unknown callback ID",
            SceError::KERNEL_DORMANT => "kernel error: thread already dormant",
            SceError::KERNEL_SUSPEND => "kernel error: thread already suspended",
            SceError::KERNEL_NOT_DORMANT => "kernel error: thread not dormant",
            SceError::KERNEL_NOT_SUSPEND => "kernel error: thread not suspended",
            SceError::KERNEL_NOT_WAIT => "kernel error: thread not in wait state",
            SceError::KERNEL_CAN_NOT_WAIT => {
                "kernel error: thread can not enter wait state or thread dispatch is suspended"
            },
            SceError::KERNEL_WAIT_TIMEOUT => "kernel error: thread wait reached timeout",
            SceError::KERNEL_WAIT_CANCEL => "kernel error: thread wait was canceled",
            SceError::KERNEL_RELEASE_WAIT => "kernel error: thread wait was released",
            SceError::KERNEL_NOTIFY_CALLBACK => {
                "kernel error: thread wait was released with callback"
            },
            SceError::KERNEL_THREAD_TERMINATED => "kernel error: thread is terminated",
            SceError::KERNEL_SEMA_ZERO => "kernel error: semaphore counter is zero",
            SceError::KERNEL_SEMA_OVF => "kernel error: semaphore counter overflow",
            SceError::KERNEL_EVF_COND => "kernel error: event-flag condition did not happened",
            SceError::KERNEL_EVF_MULTI => {
                "kernel error: event-flag multiple threads waits disabled"
            },
            SceError::KERNEL_EVF_ILPAT => "kernel error: invalid event-flag wait pattern",
            SceError::KERNEL_MBOX_NOMSG => "kernel error: message-box without message",
            SceError::KERNEL_MPP_FULL => "kernel error: message-pipe is full",
            SceError::KERNEL_MPP_EMPTY => "kernel error: message-pipe is empty",
            SceError::KERNEL_WAIT_DELETE => "kernel error: wait object was deleted",
            SceError::KERNEL_INVALID_MEMBLOCK => "kernel error: invalid memory block",
            SceError::KERNEL_INVALID_MEMSIZE => "kernel error: invalid memory size",
            SceError::KERNEL_INVALID_SPADADDR => "kernel error: invalid scratchpad address",
            SceError::KERNEL_SPAD_INUSE => "kernel error: scratchpad is currently in use",
            SceError::KERNEL_SPAD_NOT_INUSE => "kernel error: scratchpad is not in use",
            SceError::KERNEL_INVALID_TYPE => "kernel error: invalid type",
            SceError::KERNEL_INVALID_SIZE => "kernel error: invalid size",
            SceError::KERNEL_INVALID_COUNT => "kernel error: invalid count",
            SceError::KERNEL_UNKNOWN_VTID => "kernel error: unknown vtimer ID",
            SceError::KERNEL_INVALID_VTID => "kernel error: invalid vtimer ID",
            SceError::KERNEL_INVALID_KTLSID => "kernel error: invalid KTLS ID",
            SceError::KERNEL_KTLS_FULL => "kernel error: KTLS is full",
            SceError::KERNEL_KTLS_BUSY => "kernel error: KTLS is busy",
            SceError::KERNEL_MUTEX_UNKNOWN_ID => "kernel error: unknown mutex ID",
            SceError::KERNEL_MUTEX_OWN => "kernel error: failed to own a mutex",
            SceError::KERNEL_MUTEX_NOT_OWNED => {
                "kernel error: mutex is not owned and can not be released"
            },
            SceError::KERNEL_MUTEX_LOCK_OVERFLOW => "kernel error: mutex lock counter overflow",
            SceError::KERNEL_MUTEX_UNLOCK_UNDERFLOW => "kernel error: mutex lock counter underflow",
            SceError::KERNEL_MUTEX_NOT_REENTRANT => "kernel error: mutex is not reentrant",
            SceError::KERNEL_MSG_BOX_LOOP => {
                "kernel error: infinite loop detected in a message box"
            },
            SceError::KERNEL_LWMUTEX_UNKNOWN_ID => "kernel error: unknown lightweight mutex ID",
            SceError::KERNEL_LWMUTEX_OWN => "kernel error: failed to own a lightweight mutex",
            SceError::KERNEL_LWMUTEX_NOT_OWNED => {
                "kernel error: lightweight mutex is not owned and can not be released"
            },
            SceError::KERNEL_LWMUTEX_LOCK_OVERFLOW => {
                "kernel error: lightweight mutex lock counter overflow"
            },
            SceError::KERNEL_LWMUTEX_UNLOCK_UNDERFLOW => {
                "kernel error: lightweight mutex lock counter underflow"
            },
            SceError::KERNEL_LWMUTEX_NOT_REENTRANT => {
                "kernel error: lightweight mutex is not reentrant"
            },
            SceError::KERNEL_TLS_POOL_UNKNWON_ID => "kernel error: unknown TLS pool ID",
            SceError::KERNEL_UTLS_IS_FULL => "kernel error: UTLS is full",
            SceError::KERNEL_UTLS_IS_BUSY => "kernel error: UTLS is busy",
            SceError::KERNEL_PM_INVALID_PRIORITY => "kernel error: power manager invalid priority",
            SceError::KERNEL_PM_INVALID_DEVNAME => {
                "kernel error: power manager invalid device name"
            },
            SceError::KERNEL_PM_UNKNOWN_DEVNAME => {
                "kernel error: power manager unknown device name"
            },
            SceError::KERNEL_PM_PMINFO_REGISTERED => {
                "kernel error: power manager information is already registered"
            },
            SceError::KERNEL_PM_PMINFO_UNREGISTERED => {
                "kernel error: power manager information is unregistered"
            },
            SceError::KERNEL_PM_INVALID_MAJOR_STATE => {
                "kernel error: power manager invalid major state"
            },
            SceError::KERNEL_PM_INVALID_REQUEST => "kernel error: power manager invalid request",
            SceError::KERNEL_PM_UNKNOWN_REQUEST => "kernel error: power manager unknown request",
            SceError::KERNEL_PM_INVALID_UNIT => "kernel error: power manager invalid unit",
            SceError::KERNEL_PM_CANNOT_CANCEL => {
                "kernel error: power manager operation can not cancel"
            },
            SceError::KERNEL_PM_INVALID_PMINFO => "kernel error: invalid power manager information",
            SceError::KERNEL_PM_INVALID_ARGUMENT => "kernel error: power manager invalid argument",
            SceError::KERNEL_PM_ALREADY_TARGET_PWRSTATE => {
                "kernel error: power manager power state already targeted"
            },
            SceError::KERNEL_PM_CHANGE_PWRSTATE_FAILED => {
                "kernel error: power manager failed to change power state"
            },
            SceError::KERNEL_PM_CANNOT_CHANGE_DEVPWR_STATE => {
                "kernel error: power manager can not change device power state"
            },
            SceError::KERNEL_PM_NO_SUPPORT_DEVPWR_STATE => {
                "kernel error: power manager device does not support a power state"
            },
            SceError::KERNEL_DMAC_REQUEST_FAILED => "kernel error: DMAC request failed",
            SceError::KERNEL_DMAC_REQUEST_DENIED => "kernel error: DMAC request was denied",
            SceError::KERNEL_DMAC_OP_QUEUED => "kernel error: DMAC operation is already queued",
            SceError::KERNEL_DMAC_OP_NOT_QUEUED => "kernel error: DMAC operation was not queued",
            SceError::KERNEL_DMAC_OP_RUNNING => "kernel error: DMAC operation is running",
            SceError::KERNEL_DMAC_OP_NOT_ASSIGNED => {
                "kernel error: DMAC operation was not assigned"
            },
            SceError::KERNEL_DMAC_OP_TIMEOUT => "kernel error: DMAC operation timed out",
            SceError::KERNEL_DMAC_OP_FREED => "kernel error: DMAC operation was already released",
            SceError::KERNEL_DMAC_OP_USED => "kernel error: DMAC operation is in use",
            SceError::KERNEL_DMAC_OP_EMPTY => "kernel error: DMAC operation is empty",
            SceError::KERNEL_DMAC_OP_ABORTED => "kernel error: DMAC operation was aborted",
            SceError::KERNEL_DMAC_OP_ERROR => "kernel error: DMAC operation is a error",
            SceError::KERNEL_DMAC_CHANNEL_RESERVED => {
                "kernel error: DMAC physical channel is already reserved"
            },
            SceError::KERNEL_DMAC_CHANNEL_EXCLUDED => {
                "kernel error: DMAC physical channel is not managed by DMAC manager"
            },
            SceError::KERNEL_DMAC_PRIVILEGE_ADDRESS => {
                "kernel error: DMAC has a privileged address in the link list"
            },
            SceError::KERNEL_DMAC_NO_ENOUGHSPACE => {
                "kernel error: DMAC link list buffer is not big enough"
            },
            SceError::KERNEL_DMAC_CHANNEL_NOT_ASSIGNED => {
                "kernel error: DMAC operation is not assigned to a physical channel"
            },
            SceError::KERNEL_DMAC_CHILD_OPERATION => {
                "kernel error: DMAC operation is a child operation"
            },
            SceError::KERNEL_DMAC_TOO_MUCH_SIZE => {
                "kernel error: DMAC has more data than transferable data size"
            },
            SceError::KERNEL_DMAC_INVALID_ARGUMENT => "kernel error: DMAC invalid argument",
            SceError::KERNEL_MFILE => "kernel error: too many open files",
            SceError::KERNEL_NODEV => "kernel error: no such device",
            SceError::KERNEL_XDEV => "kernel error: cross-device link",
            SceError::KERNEL_BADF => "kernel error: bad file descriptor",
            SceError::KERNEL_INVAL => "kernel error: invalid argument",
            SceError::KERNEL_UNSUP => "kernel error: operation is unsupported",
            SceError::KERNEL_ALIAS_USED => "kernel error: alias is already used",
            SceError::KERNEL_CANNOT_MOUNT => "kernel error: not possible to mount",
            SceError::KERNEL_DRIVER_DELETED => "kernel error: driver was deleted",
            SceError::KERNEL_ASYNC_BUSY => "kernel error: async operation is busy",
            SceError::KERNEL_NOASYNC => "kernel error: no async operation",
            SceError::KERNEL_REGDEV => "kernel error: device already registered",
            SceError::KERNEL_NOCWD => "kernel error: no current working directory",
            SceError::KERNEL_NAMETOOLONG => "kernel error: file name or path is too long",
            SceError::KERNEL_STDIO_NOT_OPEN => "kernel error: STDIO was not opened",
            SceError::KERNEL_DECI2P_UNKNOWN_SOCKET_ID => "kernel error: unknown DECI2 socket ID",
            SceError::KERNEL_DECI2P_REGISTERED_PROTOCOL => {
                "kernel error: DECI2p protocol already registered"
            },
            SceError::KERNEL_DECI2P_TOO_MANY_PROTOCOL => {
                "kernel error: too many open DECI2p protocols"
            },
            SceError::KERNEL_DECI2P_SMALL_BUFF => {
                "kernel error: DECI2p receive buffer is too small"
            },
            SceError::KERNEL_DECI2P_INVALID_PROTOCOL => "kernel error: invalid DECI2p protocol",
            SceError::KERNEL_DECI2P_INVALID_HEADER => "kernel error: invalid DECI2p header",
            SceError::KERNEL_DECI2P_NO_SPACE => "kernel error: no space left on DECI2p manager",
            SceError::KERNEL_DECI2P_NO_HOST => "kernel error: no DECI2p host interface",
            SceError::KERNEL_CACHE_ALIGNMENT => "kernel error: argument is not cache aligned",

            _ => "",
        }
    }
}

impl core::fmt::Display for ErrorFacility {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorFacility::Null => f.pad("generic"),
            ErrorFacility::Errno => f.pad("errno"),
            ErrorFacility::Kernel => f.pad("kernel"),
            ErrorFacility::Registry => f.pad("registry"),
            ErrorFacility::Vsh => f.pad("visual shell"),
            ErrorFacility::Utility => f.pad("utility"),
            ErrorFacility::SysFile => f.pad("system file"),
            ErrorFacility::MsApp => f.pad("ms app"),
            ErrorFacility::Umd => f.pad("UMD"),
            ErrorFacility::MemoryStick => f.pad("memory stick"),
            ErrorFacility::Flash => f.pad("flash"),
            ErrorFacility::Usb => f.pad("USB"),
            ErrorFacility::Syscon => f.pad("syscon"),
            ErrorFacility::Audio => f.pad("audio"),
            ErrorFacility::Lflash => f.pad("lflash"),
            ErrorFacility::Lfatfs => f.pad("lfatfs"),
            ErrorFacility::Sircs => f.pad("SIRCS"),
            ErrorFacility::Irda => f.pad("IRDA"),
            ErrorFacility::Power => f.pad("power"),
            ErrorFacility::AudioRouting => f.pad("audio routing"),
            ErrorFacility::MediaSync => f.pad("media sync"),
            ErrorFacility::Periph => f.pad("periph"),
            ErrorFacility::Network => f.pad("generic"),
            ErrorFacility::Sas => f.pad("network"),
            ErrorFacility::Http => f.pad("HTTP"),
            ErrorFacility::Wave => f.pad("wave"),
            ErrorFacility::Snd => f.pad("SND"),
            ErrorFacility::Font => f.pad("font"),
            ErrorFacility::P3DA => f.pad("P3PA"),
            ErrorFacility::Magicgate => f.pad("magicgate"),
            ErrorFacility::Cphio => f.pad("CphIO"),
            ErrorFacility::OpenPsId => f.pad("Open PS ID"),
            ErrorFacility::Dnas => f.pad("DNAS"),
            ErrorFacility::Mtp => f.pad("MTP"),
            ErrorFacility::Np => f.pad("NP"),
            ErrorFacility::GameUpdate => f.pad("game update"),
            ErrorFacility::Fmac => f.pad("FMAC"),
            ErrorFacility::Face => f.pad("face"),
            ErrorFacility::Library => f.pad("library"),
            ErrorFacility::Mpeg => f.pad("MPEG"),
            ErrorFacility::Avc => f.pad("AVC"),
            ErrorFacility::Atrac => f.pad("Atrac"),
            ErrorFacility::Asf => f.pad("ASF"),
            ErrorFacility::Jpeg => f.pad("JPEG"),
            ErrorFacility::Avi => f.pad("AVI"),
            ErrorFacility::MP3 => f.pad("MP3"),
            ErrorFacility::G729 => f.pad("G729"),
            ErrorFacility::Aac => f.pad("AAC"),
            ErrorFacility::Codec => f.pad("codec"),
            ErrorFacility::Other => write!(f, "{:#05X}", *self as u8),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::mem;

    use super::*;

    #[test]
    fn test_niche_type_attributes() {
        // Size of Result<(), SceError> and Option<SceError> must be 4 bytes
        assert_eq!(size_of::<Result<(), SceError>>(), size_of::<i32>());
        assert_eq!(size_of::<Option<SceError>>(), size_of::<i32>());

        // Result<(), SceError> ok value must be zero
        let result: Result<(), SceError> = Ok(());
        let as_int: u32 = unsafe { mem::transmute(result) };
        assert_eq!(as_int, 0);

        // Option<SceError> None value must be zero
        let option: Option<SceError> = None;
        let as_int: u32 = unsafe { mem::transmute(option) };
        assert_eq!(as_int, 0);
    }
}
