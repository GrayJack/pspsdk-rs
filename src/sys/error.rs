/// Type representing return errors from the PSP APIs, official or otherwise.
///
/// It's valid range is `(0x80000001, 0xFFFFFFFF]`.
#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(0x80000001)]
#[rustc_layout_scalar_valid_range_end(0xFFFFFFFF)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceError(u32);

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
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0x80000001..=0xFFFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
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
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { Self(raw) }
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }

    /// Get the facility of the error.
    pub const fn facility(self) -> ErrorFacility {
        match (self.0 >> 16) & 0xFF {
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
    NOT_IMPL= 0x80000003;
    NOT_SUPPORTED = 0x80000004;
    ALREADY = 0x80000020;
    BUSY = 0x80000021;
    OUT_OF_MEMORY = 0x80000022;
    PRIV_REQUIRED = 0x80000023;
    NOT_FOUND = 0x80000025;
    ILLEGAL_CONTEXT = 0x80000030;
    CPUDI = 0x80000031;
    SEMAPHORE = 0x80000041;
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
    NOSPC = 0x8000021C;
    DFUNC = 0x800002FF;
);

// Standard errors
__err_def!(
    OPERATION_NOT_PERMITTED = 0x80010001;
    FILE_NOT_FOUND = 0x80010002;
    FILE_OPEN = 0x80010003;
    IO = 0x80010005;
    RR_DEVICE_IO = 0x80010006;
    ARG_LIST_TOO_LONG = 0x80010007;
    INVALID_FILE_DESCRIPTOR = 0x80010009;
    RESOURCE_UNAVAILABLE = 0x8001000B;
    NO_MEMORY = 0x8001000C;
    NO_PERM = 0x8001000D;
    FILE_INVALID_ADDR = 0x8001000E;
    DEVICE_BUSY = 0x80010010;
    FILE_ALREADY_EXISTS = 0x80010011;
    CROSS_DEV_LINK = 0x80010012;
    DEVICE_NOT_FOUND = 0x80010013;
    NOT_A_DIRECTORY = 0x80010014;
    IS_DIRECTORY = 0x80010015;
    STD_INVALID_ARGUMENT = 0x80010016;
    TOO_MANY_OPEN_SYSTEM_FILES = 0x80010018;
    FILE_IS_TOO_BIG = 0x8001001B;
    DEVICE_NO_FREE_SPACE = 0x8001001C;
    READ_ONLY = 0x8001001E;
    CLOSED = 0x80010020;
    IDRM = 0x80010024;
    FILE_PROTOCOL = 0x80010047;
    DIRECTORY_IS_NOT_EMPTY = 0x8001005A;
    NAME_TOO_LONG = 0x8001005B;
    TOO_MANY_SYMBOLIC_LINKS = 0x8001005C;
    CONNECTION_RESET = 0x80010068;
    NO_FREE_BUF_SPACE = 0x80010069;
    SHUTDOWN = 0x8001006E;
    ADDR_IN_USE = 0x80010070 ;
    CONNECTION_ABORTED = 0x80010071 ;
    TIMEDOUT = 0x80010074 ;
    IN_PROGRESS = 0x80010077;
    STD_ALREADY = 0x80010078;
    INVALID_PROTOCOL = 0x8001007B ;
    INVALID_SOCKET_TYPE = 0x8001007C ;
    ADDR_NOT_AVAILABLE = 0x8001007D;
    IS_ALREADY_CONNECTED = 0x8001007F;
    NOT_CONNECTED = 0x80010080;
    FILE_QUOTA_EXCEEDED = 0x80010084;
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
    KERNEL_ILLEGAL_SYSCALLTABLE = 0x80020037;
    KERNEL_ILLEGAL_PRIMARY_SYSCALL_NUMBER = 0x80020038;
    KERNEL_PRIMARY_SYSCALL_NUMBER_INUSE = 0x80020039;
    KERNEL_ILLEGAL_CONTEXT = 0x80020064;
    KERNEL_ILLEGAL_INTRCODE = 0x80020065;
    KERNEL_CPUDI = 0x80020066;
    KERNEL_FOUND_HANDLER = 0x80020067;
    KERNEL_NOTFOUND_HANDLER = 0x80020068;
    KERNEL_ILLEGAL_INTRLEVEL = 0x80020069;
    KERNEL_ILLEGAL_ADDRESS = 0x8002006a;
    KERNEL_ILLEGAL_INTRPARAM = 0x8002006b;
    KERNEL_ILLEGAL_STACK_ADDRESS = 0x8002006c;
    KERNEL_ALREADY_STACK_SET = 0x8002006d;
    KERNEL_NO_TIMER = 0x80020096;
    KERNEL_ILLEGAL_TIMERID = 0x80020097;
    KERNEL_ILLEGAL_SOURCE = 0x80020098;
    KERNEL_ILLEGAL_PRESCALE = 0x80020099;
    KERNEL_TIMER_BUSY = 0x8002009a;
    KERNEL_TIMER_NOT_SETUP = 0x8002009b;
    KERNEL_TIMER_NOT_INUSE = 0x8002009c;
    KERNEL_UNIT_USED = 0x800200a0;
    KERNEL_UNIT_NOUSE = 0x800200a1;
    KERNEL_NO_ROMDIR = 0x800200a2;
    KERNEL_IDTYPE_EXIST = 0x800200c8;
    KERNEL_IDTYPE_NOT_EXIST = 0x800200c9;
    KERNEL_IDTYPE_NOT_EMPTY = 0x800200ca;
    KERNEL_UNKNOWN_UID = 0x800200cb;
    KERNEL_UNMATCH_UID_TYPE = 0x800200cc;
    KERNEL_ID_NOT_EXIST = 0x800200cd;
    KERNEL_NOT_FOUND_UIDFUNC = 0x800200ce;
    KERNEL_UID_ALREADY_HOLDER = 0x800200cf;
    KERNEL_UID_NOT_HOLDER = 0x800200d0;
    KERNEL_ILLEGAL_PERM = 0x800200d1;
    KERNEL_ILLEGAL_ARGUMENT = 0x800200d2;
    KERNEL_ILLEGAL_ADDR = 0x800200d3;
    KERNEL_OUT_OF_RANGE = 0x800200d4;
    KERNEL_MEM_RANGE_OVERLAP = 0x800200d5;
    KERNEL_ILLEGAL_PARTITION = 0x800200d6;
    KERNEL_PARTITION_INUSE = 0x800200d7;
    KERNEL_ILLEGAL_MEMBLOCKTYPE = 0x800200d8;
    KERNEL_MEMBLOCK_ALLOC_FAILED = 0x800200d9;
    KERNEL_MEMBLOCK_RESIZE_LOCKED = 0x800200da;
    KERNEL_MEMBLOCK_RESIZE_FAILED = 0x800200db;
    KERNEL_HEAPBLOCK_ALLOC_FAILED = 0x800200dc;
    KERNEL_HEAP_ALLOC_FAILED = 0x800200dd;
    KERNEL_ILLEGAL_CHUNK_ID = 0x800200de;
    KERNEL_NOCHUNK = 0x800200df;
    KERNEL_NO_FREECHUNK = 0x800200e0;
    KERNEL_LINKERNEL = 0x8002012c;
    KERNEL_ILLEGAL_OBJECT = 0x8002012d;
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
    KERNEL_ILLEGAL_LIBRARY = 0x8002013d;
    KERNEL_LIBRARY_INUSE = 0x8002013e;
    KERNEL_ALREADY_STOPPING = 0x8002013f;
    KERNEL_ILLEGAL_OFFSET = 0x80020140;
    KERNEL_ILLEGAL_POSITION = 0x80020141;
    KERNEL_ILLEGAL_ACCESS = 0x80020142;
    KERNEL_MODULE_MGR_BUSY = 0x80020143;
    KERNEL_ILLEGAL_FLAG = 0x80020144;
    KERNEL_CANNOT_GET_MODULELIST = 0x80020145;
    KERNEL_PROHIBIT_LOADMODULE_DEVICE = 0x80020146;
    KERNEL_PROHIBIT_DEVICE = 0x80020147;
    KERNEL_UNSUPPORTED_PRX_TYPE = 0x80020148;
    KERNEL_ILLEGAL_PERM_CALL = 0x80020149;
    KERNEL_CANNOT_GET_MODULE_INFORMATION = 0x8002014a;
    KERNEL_ILLEGAL_BUFFER = 0x8002014b;
    KERNEL_ILLEGAL_FILENAME = 0x8002014c;
    KERNEL_NO_EXIT_CALLBACK = 0x8002014d;
    KERNEL_NO_MEMORY = 0x80020190;
    KERNEL_ILLEGAL_ATTR = 0x80020191;
    KERNEL_ILLEGAL_ENTRY = 0x80020192;
    KERNEL_ILLEGAL_PRIORITY = 0x80020193;
    KERNEL_ILLEGAL_STACK_SIZE = 0x80020194;
    KERNEL_ILLEGAL_MODE = 0x80020195;
    KERNEL_ILLEGAL_MASK = 0x80020196;
    KERNEL_ILLEGAL_THID = 0x80020197;
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
    KERNEL_ILLEGAL_MEMBLOCK = 0x800201b6;
    KERNEL_ILLEGAL_MEMSIZE = 0x800201b7;
    KERNEL_ILLEGAL_SPADADDR = 0x800201b8;
    KERNEL_SPAD_INUSE = 0x800201b9;
    KERNEL_SPAD_NOT_INUSE = 0x800201ba;
    KERNEL_ILLEGAL_TYPE = 0x800201bb;
    KERNEL_ILLEGAL_SIZE = 0x800201bc;
    KERNEL_ILLEGAL_COUNT = 0x800201bd;
    KERNEL_UNKNOWN_VTID = 0x800201be;
    KERNEL_ILLEGAL_VTID = 0x800201bf;
    KERNEL_ILLEGAL_KTLSID = 0x800201c0;
    KERNEL_KTLS_FULL = 0x800201c1;
    KERNEL_KTLS_BUSY = 0x800201c2;
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
    KERNEL_NXIO = 0x800203e8;
    KERNEL_IO = 0x800203e9;
    KERNEL_NOMEM = 0x800203ea;
    KERNEL_STDIO_NOT_OPENED = 0x800203eb;
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
