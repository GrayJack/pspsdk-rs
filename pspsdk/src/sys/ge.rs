//! Graphic Engine operation and management.
use core::{ffi::c_void, fmt};

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceIntoOkValue, SceResult, SceResultOk, SceSize, SceUid};

/// A Valid Display list ID.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayListId(SceUid);

/// A Graphic Engine callback ID.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeCallbackId(SceUid);

/// Storage for a Graphic Engine context.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[doc(alias("SceGeContext", "PspGeContext"))]
pub struct GeContext {
    buffer: [u32; 512],
}

/// The list of arguments when enqueueing a list.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[doc(alias("SceGeListArgs", "PspGeListArgs"))]
pub struct GeListArgs {
    /// The size of this structure.
    pub size: SceSize,
    /// A pointer to the context.
    pub ctx: *mut GeContext,
    /// The number of stacks to use.
    pub num_stacks: u32,
    /// A pointer to the stack list (unused).
    pub stacks: *mut GeStack,
}

/// Storage for a stack (for CALL/RET).
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
#[doc(alias("SceGeStack"))]
pub struct GeStack {
    /// The stack buffer.
    pub stack: [u32; 8],
}


/// The possible width values of the EDRAM address translation.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TranslationMemWidth {
    /// Linear mode.
    LinearMode = 0,
    /// Width of 512 bytes.
    Bytes512 = 512,
    /// Width of 1024 bytes.
    #[default]
    Bytes1024 = 1024,
    /// Width of 2048 bytes.
    Bytes2048 = 2048,
    /// Width of 4096 bytes.
    Bytes4096 = 4096,
}

/// The available GE commands.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum GeCmd {
    #[doc(alias("SCE_GE_CMD_NOP"))]
    Nop    = 0x00,
    #[doc(alias("SCE_GE_CMD_VADR"))]
    Vaddr  = 0x01,
    #[doc(alias("SCE_GE_CMD_IADR"))]
    Iaddr  = 0x02,
    #[doc(alias("SCE_GE_CMD_PRIM"))]
    Prim   = 0x04,
    #[doc(alias("SCE_GE_CMD_BEZIER"))]
    Bezier = 0x05,
    #[doc(alias("SCE_GE_CMD_SPLINE"))]
    Spline = 0x06,
    #[doc(alias("SCE_GE_CMD_BBOX"))]
    BoundingBox = 0x07,
    #[doc(alias("SCE_GE_CMD_JUMP"))]
    Jump   = 0x08,
    #[doc(alias("SCE_GE_CMD_BJUMP"))]
    BJump  = 0x09,
    #[doc(alias("SCE_GE_CMD_CALL"))]
    Call   = 0x0A,
    #[doc(alias("SCE_GE_CMD_RET"))]
    Ret    = 0x0B,
    #[doc(alias("SCE_GE_CMD_END"))]
    End    = 0x0C,
    #[doc(alias("SCE_GE_CMD_SIGNAL"))]
    Signal = 0x0E,
    #[doc(alias("SCE_GE_CMD_FINISH"))]
    Finish = 0x0F,
    #[doc(alias("SCE_GE_CMD_BASE"))]
    Base   = 0x10,
    #[doc(alias("SCE_GE_CMD_VTYPE"))]
    VertexType = 0x12,
    #[doc(alias("SCE_GE_CMD_OFFSET"))]
    OffsetAddr = 0x13,
    #[doc(alias("SCE_GE_CMD_ORIGIN"))]
    Origin = 0x14,
    #[doc(alias("SCE_GE_CMD_REGION1"))]
    Region1 = 0x15,
    #[doc(alias("SCE_GE_CMD_REGION2"))]
    Region2 = 0x16,
    #[doc(alias("SCE_GE_CMD_LTE"))]
    LightingEnable = 0x17,
    #[doc(alias("SCE_GE_CMD_LE0"))]
    LightEnable0 = 0x18,
    #[doc(alias("SCE_GE_CMD_LE1"))]
    LightEnable1 = 0x19,
    #[doc(alias("SCE_GE_CMD_LE2"))]
    LightEnable2 = 0x1A,
    #[doc(alias("SCE_GE_CMD_LE3"))]
    LightEnable3 = 0x1B,
    #[doc(alias("SCE_GE_CMD_CLE"))]
    DepthClampEnable = 0x1C,
    #[doc(alias("SCE_GE_CMD_BCE"))]
    CullFaceEnable = 0x1D,
    #[doc(alias("SCE_GE_CMD_TME"))]
    TextureMapEnable = 0x1E,
    #[doc(alias("SCE_GE_CMD_FGE"))]
    FogEnable = 0x1F,
    #[doc(alias("SCE_GE_CMD_DTE"))]
    DitherEnable = 0x20,
    #[doc(alias("SCE_GE_CMD_ABE"))]
    AlphaBlendEnable = 0x21,
    #[doc(alias("SCE_GE_CMD_ATE"))]
    AlphaTestEnable = 0x22,
    #[doc(alias("SCE_GE_CMD_ZTE"))]
    ZTestEnable = 0x23,
    #[doc(alias("SCE_GE_CMD_STE"))]
    StencilTestEnable = 0x24,
    #[doc(alias("SCE_GE_CMD_AAE"))]
    AntiAliasEnable = 0x25,
    #[doc(alias("SCE_GE_CMD_PCE"))]
    PatchCullEnable = 0x26,
    #[doc(alias("SCE_GE_CMD_CTE"))]
    ColorTestEnable = 0x27,
    #[doc(alias("SCE_GE_CMD_LOE"))]
    LogicOpEnable = 0x28,
    #[doc(alias("SCE_GE_CMD_BONEN"))]
    BoneMatrixNumber = 0x2A,
    #[doc(alias("SCE_GE_CMD_BONED"))]
    BoneMatrixData = 0x2B,
    #[doc(alias("SCE_GE_CMD_WEIGHT0"))]
    MorphWeight0 = 0x2C,
    #[doc(alias("SCE_GE_CMD_WEIGHT1"))]
    MorphWeight1 = 0x2D,
    #[doc(alias("SCE_GE_CMD_WEIGHT2"))]
    MorphWeight2 = 0x2E,
    #[doc(alias("SCE_GE_CMD_WEIGHT3"))]
    MorphWeight3 = 0x2F,
    #[doc(alias("SCE_GE_CMD_WEIGHT4"))]
    MorphWeight4 = 0x30,
    #[doc(alias("SCE_GE_CMD_WEIGHT5"))]
    MorphWeight5 = 0x31,
    #[doc(alias("SCE_GE_CMD_WEIGHT6"))]
    MorphWeight6 = 0x32,
    #[doc(alias("SCE_GE_CMD_WEIGHT7"))]
    MorphWeight7 = 0x33,
    #[doc(alias("SCE_GE_CMD_DIVIDE"))]
    PatchDivision = 0x36,
    #[doc(alias("SCE_GE_CMD_PPM"))]
    PatchPrimitive = 0x37,
    #[doc(alias("SCE_GE_CMD_PFACE"))]
    PatchFacing = 0x38,
    #[doc(alias("SCE_GE_CMD_WORLDN"))]
    WorldMatrixNumber = 0x3A,
    #[doc(alias("SCE_GE_CMD_WORLDD"))]
    WorldMatrixData = 0x3B,
    #[doc(alias("SCE_GE_CMD_VIEWN"))]
    ViewMatrixNumber = 0x3C,
    #[doc(alias("SCE_GE_CMD_VIEWD"))]
    ViewMatrixData = 0x3D,
    #[doc(alias("SCE_GE_CMD_PROJN"))]
    ProjMatrixNumber = 0x3E,
    #[doc(alias("SCE_GE_CMD_PROJD"))]
    ProjMatrixData = 0x3F,
    #[doc(alias("SCE_GE_CMD_TGENN"))]
    TGenMatrixNumber = 0x40,
    #[doc(alias("SCE_GE_CMD_TGEND"))]
    TGenMatrixData = 0x41,
    #[doc(alias("SCE_GE_CMD_SX"))]
    ViewportXScale = 0x42,
    #[doc(alias("SCE_GE_CMD_SY"))]
    ViewportYScale = 0x43,
    #[doc(alias("SCE_GE_CMD_SZ"))]
    ViewportZScale = 0x44,
    #[doc(alias("SCE_GE_CMD_TX"))]
    ViewportXCenter = 0x45,
    #[doc(alias("SCE_GE_CMD_TY"))]
    ViewportYCenter = 0x46,
    #[doc(alias("SCE_GE_CMD_TZ"))]
    ViewportZCenter = 0x47,
    #[doc(alias("SCE_GE_CMD_SU"))]
    TexScaleU = 0x48,
    #[doc(alias("SCE_GE_CMD_SV"))]
    TexScaleV = 0x49,
    #[doc(alias("SCE_GE_CMD_TU"))]
    TexOffsetU = 0x4A,
    #[doc(alias("SCE_GE_CMD_TV"))]
    TexOffsetV = 0x4B,
    #[doc(alias("SCE_GE_CMD_OFFSETX"))]
    OffsetX = 0x4C,
    #[doc(alias("SCE_GE_CMD_OFFSETY"))]
    OffsetY = 0x4D,
    #[doc(alias("SCE_GE_CMD_SHADE"))]
    ShadeMode = 0x50,
    #[doc(alias("SCE_GE_CMD_NREV"))]
    ReverseNormal = 0x51,
    #[doc(alias("SCE_GE_CMD_MATERIAL"))]
    MaterialUpdate = 0x53,
    #[doc(alias("SCE_GE_CMD_MEC"))]
    MaterialEmissive = 0x54,
    #[doc(alias("SCE_GE_CMD_MAC"))]
    MaterialAmbient = 0x55,
    #[doc(alias("SCE_GE_CMD_MDC"))]
    MaterialDiffuse = 0x56,
    #[doc(alias("SCE_GE_CMD_MSC"))]
    MaterialSpecular = 0x57,
    #[doc(alias("SCE_GE_CMD_MAA"))]
    MaterialAlpha = 0x58,
    #[doc(alias("SCE_GE_CMD_MK"))]
    MaterialSpecularCoef = 0x5B,
    #[doc(alias("SCE_GE_CMD_AC"))]
    AmbientColor = 0x5C,
    #[doc(alias("SCE_GE_CMD_AA"))]
    AmbientAlpha = 0x5D,
    #[doc(alias("SCE_GE_CMD_LMODE"))]
    LightMode = 0x5E,
    #[doc(alias("SCE_GE_CMD_LTYPE0"))]
    LightType0 = 0x5F,
    #[doc(alias("SCE_GE_CMD_LTYPE1"))]
    LightType1 = 0x60,
    #[doc(alias("SCE_GE_CMD_LTYPE2"))]
    LightType2 = 0x61,
    #[doc(alias("SCE_GE_CMD_LTYPE3"))]
    LightType3 = 0x62,
    #[doc(alias("SCE_GE_CMD_LX0"))]
    Light0X = 0x63,
    #[doc(alias("SCE_GE_CMD_LY0"))]
    Light0Y = 0x64,
    #[doc(alias("SCE_GE_CMD_LZ0"))]
    Light0Z = 0x65,
    #[doc(alias("SCE_GE_CMD_LX1"))]
    Light1X = 0x66,
    #[doc(alias("SCE_GE_CMD_LY1"))]
    Light1Y = 0x67,
    #[doc(alias("SCE_GE_CMD_LZ1"))]
    Light1Z = 0x68,
    #[doc(alias("SCE_GE_CMD_LX2"))]
    Light2X = 0x69,
    #[doc(alias("SCE_GE_CMD_LY2"))]
    Light2Y = 0x6A,
    #[doc(alias("SCE_GE_CMD_LZ2"))]
    Light2Z = 0x6B,
    #[doc(alias("SCE_GE_CMD_LX3"))]
    Light3X = 0x6C,
    #[doc(alias("SCE_GE_CMD_LY3"))]
    Light3Y = 0x6D,
    #[doc(alias("SCE_GE_CMD_LZ3"))]
    Light3Z = 0x6E,
    #[doc(alias("SCE_GE_CMD_LDX0"))]
    Light0DirectionX = 0x6F,
    #[doc(alias("SCE_GE_CMD_LDY0"))]
    Light0DirectionY = 0x70,
    #[doc(alias("SCE_GE_CMD_LDZ0"))]
    Light0DirectionZ = 0x71,
    #[doc(alias("SCE_GE_CMD_LDX1"))]
    Light1DirectionX = 0x72,
    #[doc(alias("SCE_GE_CMD_LDY1"))]
    Light1DirectionY = 0x73,
    #[doc(alias("SCE_GE_CMD_LDZ1"))]
    Light1DirectionZ = 0x74,
    #[doc(alias("SCE_GE_CMD_LDX2"))]
    Light2DirectionX = 0x75,
    #[doc(alias("SCE_GE_CMD_LDY2"))]
    Light2DirectionY = 0x76,
    #[doc(alias("SCE_GE_CMD_LDZ2"))]
    Light2DirectionZ = 0x77,
    #[doc(alias("SCE_GE_CMD_LDX3"))]
    Light3DirectionX = 0x78,
    #[doc(alias("SCE_GE_CMD_LDY3"))]
    Light3DirectionY = 0x79,
    #[doc(alias("SCE_GE_CMD_LDZ3"))]
    Light3DirectionZ = 0x7A,
    #[doc(alias("SCE_GE_CMD_LKA0"))]
    Light0ConstantAtten = 0x7B,
    #[doc(alias("SCE_GE_CMD_LKB0"))]
    Light0LinearAtten = 0x7C,
    #[doc(alias("SCE_GE_CMD_LKC0"))]
    Light0QuadtraticAtten = 0x7D,
    #[doc(alias("SCE_GE_CMD_LKA1"))]
    Light1ConstantAtten = 0x7E,
    #[doc(alias("SCE_GE_CMD_LKB1"))]
    Light1LinearAtten = 0x7F,
    #[doc(alias("SCE_GE_CMD_LKC1"))]
    Light1QuadtraticAtten = 0x80,
    #[doc(alias("SCE_GE_CMD_LKA2"))]
    Light2ConstantAtten = 0x81,
    #[doc(alias("SCE_GE_CMD_LKB2"))]
    Light2LinearAtten = 0x82,
    #[doc(alias("SCE_GE_CMD_LKC2"))]
    Light2QuadtraticAtten = 0x83,
    #[doc(alias("SCE_GE_CMD_LKA3"))]
    Light3ConstantAtten = 0x84,
    #[doc(alias("SCE_GE_CMD_LKB3"))]
    Light3LinearAtten = 0x85,
    #[doc(alias("SCE_GE_CMD_LKC3"))]
    Light3QuadtraticAtten = 0x86,
    #[doc(alias("SCE_GE_CMD_LKS0"))]
    Light0ExponentAtten = 0x87,
    #[doc(alias("SCE_GE_CMD_LKS1"))]
    Light1ExponentAtten = 0x88,
    #[doc(alias("SCE_GE_CMD_LKS2"))]
    Light2ExponentAtten = 0x89,
    #[doc(alias("SCE_GE_CMD_LKS3"))]
    Light3ExponentAtten = 0x8A,
    #[doc(alias("SCE_GE_CMD_LKO0"))]
    Light0CutoffAtten = 0x8B,
    #[doc(alias("SCE_GE_CMD_LKO1"))]
    Light1CutoffAtten = 0x8C,
    #[doc(alias("SCE_GE_CMD_LKO2"))]
    Light2CutoffAtten = 0x8D,
    #[doc(alias("SCE_GE_CMD_LKO3"))]
    Light3CutoffAtten = 0x8E,
    #[doc(alias("SCE_GE_CMD_LAC0"))]
    Light0Ambient = 0x8F,
    #[doc(alias("SCE_GE_CMD_LDC0"))]
    Light0Diffuse = 0x90,
    #[doc(alias("SCE_GE_CMD_LSC0"))]
    Light0Specular = 0x91,
    #[doc(alias("SCE_GE_CMD_LAC1"))]
    Light1Ambient = 0x92,
    #[doc(alias("SCE_GE_CMD_LDC1"))]
    Light1Diffuse = 0x93,
    #[doc(alias("SCE_GE_CMD_LSC1"))]
    Light1Specular = 0x94,
    #[doc(alias("SCE_GE_CMD_LAC2"))]
    Light2Ambient = 0x95,
    #[doc(alias("SCE_GE_CMD_LDC2"))]
    Light2Diffuse = 0x96,
    #[doc(alias("SCE_GE_CMD_LSC2"))]
    Light2Specular = 0x97,
    #[doc(alias("SCE_GE_CMD_LAC3"))]
    Light3Ambient = 0x98,
    #[doc(alias("SCE_GE_CMD_LDC3"))]
    Light3Diffuse = 0x99,
    #[doc(alias("SCE_GE_CMD_LSC3"))]
    Light3Specular = 0x9A,
    #[doc(alias("SCE_GE_CMD_CULL"))]
    Cull   = 0x9B,
    #[doc(alias("SCE_GE_CMD_FBP"))]
    FrameBufPtr = 0x9C,
    #[doc(alias("SCE_GE_CMD_FBW"))]
    FrameBufWidth = 0x9D,
    #[doc(alias("SCE_GE_CMD_ZBP"))]
    ZBufPtr = 0x9E,
    #[doc(alias("SCE_GE_CMD_ZBW"))]
    ZBufWidth = 0x9F,
    #[doc(alias("SCE_GE_CMD_TBP0"))]
    TexAddr0 = 0xA0,
    #[doc(alias("SCE_GE_CMD_TBP1"))]
    TexAddr1 = 0xA1,
    #[doc(alias("SCE_GE_CMD_TBP2"))]
    TexAddr2 = 0xA2,
    #[doc(alias("SCE_GE_CMD_TBP3"))]
    TexAddr3 = 0xA3,
    #[doc(alias("SCE_GE_CMD_TBP4"))]
    TexAddr4 = 0xA4,
    #[doc(alias("SCE_GE_CMD_TBP5"))]
    TexAddr5 = 0xA5,
    #[doc(alias("SCE_GE_CMD_TBP6"))]
    TexAddr6 = 0xA6,
    #[doc(alias("SCE_GE_CMD_TBP7"))]
    TexAddr7 = 0xA7,
    #[doc(alias("SCE_GE_CMD_TBW0"))]
    TexBufWidth0 = 0xA8,
    #[doc(alias("SCE_GE_CMD_TBW1"))]
    TexBufWidth1 = 0xA9,
    #[doc(alias("SCE_GE_CMD_TBW2"))]
    TexBufWidth2 = 0xAA,
    #[doc(alias("SCE_GE_CMD_TBW3"))]
    TexBufWidth3 = 0xAB,
    #[doc(alias("SCE_GE_CMD_TBW4"))]
    TexBufWidth4 = 0xAC,
    #[doc(alias("SCE_GE_CMD_TBW5"))]
    TexBufWidth5 = 0xAD,
    #[doc(alias("SCE_GE_CMD_TBW6"))]
    TexBufWidth6 = 0xAE,
    #[doc(alias("SCE_GE_CMD_TBW7"))]
    TexBufWidth7 = 0xAF,
    #[doc(alias("SCE_GE_CMD_CBP"))]
    ClutAddr = 0xB0,
    #[doc(alias("SCE_GE_CMD_CBW"))]
    ClutAddrUpper = 0xB1,
    #[doc(alias("SCE_GE_CMD_XBP1"))]
    TransferSrc = 0xB2,
    #[doc(alias("SCE_GE_CMD_XBW1"))]
    TransferSrcWide = 0xB3,
    #[doc(alias("SCE_GE_CMD_XBP2"))]
    TransferDst = 0xB4,
    #[doc(alias("SCE_GE_CMD_XBW2"))]
    TransferDstWide = 0xB5,
    #[doc(alias("SCE_GE_CMD_TSIZE0"))]
    TexSize0 = 0xB8,
    #[doc(alias("SCE_GE_CMD_TSIZE1"))]
    TexSize1 = 0xB9,
    #[doc(alias("SCE_GE_CMD_TSIZE2"))]
    TexSize2 = 0xBA,
    #[doc(alias("SCE_GE_CMD_TSIZE3"))]
    TexSize3 = 0xBB,
    #[doc(alias("SCE_GE_CMD_TSIZE4"))]
    TexSize4 = 0xBC,
    #[doc(alias("SCE_GE_CMD_TSIZE5"))]
    TexSize5 = 0xBD,
    #[doc(alias("SCE_GE_CMD_TSIZE6"))]
    TexSize6 = 0xBE,
    #[doc(alias("SCE_GE_CMD_TSIZE7"))]
    TexSize7 = 0xBF,
    #[doc(alias("SCE_GE_CMD_TMAP"))]
    TexMapMode = 0xC0,
    #[doc(alias("SCE_GE_CMD_TSHADE"))]
    TexShadeLs = 0xC1,
    #[doc(alias("SCE_GE_CMD_TMODE"))]
    TexMode = 0xC2,
    #[doc(alias("SCE_GE_CMD_TPF"))]
    TexFormat = 0xC3,
    #[doc(alias("SCE_GE_CMD_CLOAD"))]
    LoadClut = 0xC4,
    #[doc(alias("SCE_GE_CMD_CLUT"))]
    ClutFormat = 0xC5,
    #[doc(alias("SCE_GE_CMD_TFILTER"))]
    TexFilter = 0xC6,
    #[doc(alias("SCE_GE_CMD_TWRAP"))]
    TexWrap = 0xC7,
    #[doc(alias("SCE_GE_CMD_TLEVEL"))]
    TexLevel = 0xC8,
    #[doc(alias("SCE_GE_CMD_TFUNC"))]
    TexFunc = 0xC9,
    #[doc(alias("SCE_GE_CMD_TEC"))]
    TexEnvColor = 0xCA,
    #[doc(alias("SCE_GE_CMD_TFLUSH"))]
    TexFlush = 0xCB,
    #[doc(alias("SCE_GE_CMD_TSYNC"))]
    TexSync = 0xCC,
    #[doc(alias("SCE_GE_CMD_FOG1"))]
    Fog1   = 0xCD,
    #[doc(alias("SCE_GE_CMD_FOG2"))]
    Fog2   = 0xCE,
    #[doc(alias("SCE_GE_CMD_FC"))]
    FogColor = 0xCF,
    #[doc(alias("SCE_GE_CMD_TSLOPE"))]
    TexLodSlope = 0xD0,
    #[doc(alias("SCE_GE_CMD_FPF"))]
    FramebufPixFormat = 0xD2,
    #[doc(alias("SCE_GE_CMD_CMODE"))]
    ClearMode = 0xD3,
    #[doc(alias("SCE_GE_CMD_SCISSOR1"))]
    Scissor1 = 0xD4,
    #[doc(alias("SCE_GE_CMD_SCISSOR2"))]
    Scissor2 = 0xD5,
    #[doc(alias("SCE_GE_CMD_MINZ"))]
    MinZ   = 0xD6,
    #[doc(alias("SCE_GE_CMD_MAXZ"))]
    MaxZ   = 0xD7,
    #[doc(alias("SCE_GE_CMD_CTEST"))]
    ColorTest = 0xD8,
    #[doc(alias("SCE_GE_CMD_CREF"))]
    ColorRef = 0xD9,
    #[doc(alias("SCE_GE_CMD_CMSK"))]
    ColorTestmask = 0xDA,
    #[doc(alias("SCE_GE_CMD_ATEST"))]
    AlphaTest = 0xDB,
    #[doc(alias("SCE_GE_CMD_STEST"))]
    StencilTest = 0xDC,
    #[doc(alias("SCE_GE_CMD_SOP"))]
    StencilOp = 0xDD,
    #[doc(alias("SCE_GE_CMD_ZTEST"))]
    ZTest  = 0xDE,
    #[doc(alias("SCE_GE_CMD_BLEND"))]
    BlendMode = 0xDF,
    #[doc(alias("SCE_GE_CMD_FIXA"))]
    BlendFixedA = 0xE0,
    #[doc(alias("SCE_GE_CMD_FIXB"))]
    BlendFixedB = 0xE1,
    #[doc(alias("SCE_GE_CMD_DITH1"))]
    Dith0  = 0xE2,
    #[doc(alias("SCE_GE_CMD_DITH2"))]
    Dith1  = 0xE3,
    #[doc(alias("SCE_GE_CMD_DITH3"))]
    Dith2  = 0xE4,
    #[doc(alias("SCE_GE_CMD_DITH4"))]
    Dith3  = 0xE5,
    #[doc(alias("SCE_GE_CMD_LOP"))]
    LogicOp = 0xE6,
    #[doc(alias("SCE_GE_CMD_ZMSK"))]
    ZWriteDisable = 0xE7,
    #[doc(alias("SCE_GE_CMD_PMSK1"))]
    MaskRgb = 0xE8,
    #[doc(alias("SCE_GE_CMD_PMSK2"))]
    MaskAlpha = 0xE9,
    #[doc(alias("SCE_GE_CMD_XSTART"))]
    TransferStart = 0xEA,
    #[doc(alias("SCE_GE_CMD_XPOS1"))]
    TransferSrcPos = 0xEB,
    #[doc(alias("SCE_GE_CMD_XPOS2"))]
    TransferDstPos = 0xEC,
    #[doc(alias("SCE_GE_CMD_XSIZE"))]
    TransferSize = 0xEE,
    /// Vertex Screen/Texture/Color
    #[doc(alias("SCE_GE_CMD_X2"))]
    VscX   = 0xF0,
    /// Vertex Screen/Texture/Color
    #[doc(alias("SCE_GE_CMD_Y2"))]
    VscY   = 0xF1,
    /// Vertex Screen/Texture/Color
    #[doc(alias("SCE_GE_CMD_Z2"))]
    VscZ   = 0xF2,
    /// Vertex Screen/Texture/Color
    #[doc(alias("SCE_GE_CMD_S2"))]
    VtcS   = 0xF3,
    /// Vertex Screen/Texture/Color
    #[doc(alias("SCE_GE_CMD_T2"))]
    VtcT   = 0xF4,
    /// Vertex Screen/Texture/Color
    #[doc(alias("SCE_GE_CMD_Q2"))]
    VtcQ   = 0xF5,
    #[doc(alias("SCE_GE_CMD_RGB2"))]
    Vcv    = 0xF6,
    #[doc(alias("SCE_GE_CMD_AP2"))]
    Vap    = 0xF7,
    #[doc(alias("SCE_GE_CMD_F2"))]
    Vfc    = 0xF8,
    #[doc(alias("SCE_GE_CMD_I2"))]
    Vscv   = 0xF9,
}

/// The registers of the Graphic Engine matrices.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MatrixRegister {
    /// Bone matrix 0.
    #[doc(alias("SCE_GE_MTX_BONEA"))]
    Bone0  = 0,
    /// Bone matrix 1.
    #[doc(alias("SCE_GE_MTX_BONEB"))]
    Bone1  = 1,
    /// Bone matrix 2.
    #[doc(alias("SCE_GE_MTX_BONEC"))]
    Bone2  = 2,
    /// Bone matrix 3.
    #[doc(alias("SCE_GE_MTX_BONED"))]
    Bone3  = 3,
    /// Bone matrix 4.
    #[doc(alias("SCE_GE_MTX_BONEE"))]
    Bone4  = 4,
    /// Bone matrix 5.
    #[doc(alias("SCE_GE_MTX_BONEF"))]
    Bone5  = 5,
    /// Bone matrix 6.
    #[doc(alias("SCE_GE_MTX_BONEG"))]
    Bone6  = 6,
    /// Bone matrix 7.
    #[doc(alias("SCE_GE_MTX_BONEH"))]
    Bone7  = 7,
    /// World matrix
    #[doc(alias("SCE_GE_MTX_WORLD"))]
    World  = 8,
    /// View matrix
    #[doc(alias("SCE_GE_MTX_VIEW"))]
    View   = 9,
    /// Projection Matrix
    #[doc(alias("SCE_GE_MTX_PROJ"))]
    Projection = 10,
    /// TexGen Matrix
    #[doc(alias("SCE_GE_MTX_TGEN"))]
    TexGen = 11,
}

/// Graphics Engine display list synchronization mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum DisplayListSyncMode {
    /// Sync waits until the display list is completed.
    Wait  = 0,
    /// Sync checks the display list completion state and return immediately.
    Check = 1,
}

/// The state of a display list.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[doc(alias("SceGeListState", "PspGeListState"))]
pub enum DisplayListState {
    /// The list has been completed.
    #[doc(alias("PSP_GE_LIST_DONE", "SCE_GE_LIST_COMPLETED"))]
    Completed = 0,
    /// The list is queued but not executed yet.
    #[doc(alias("PSP_GE_LIST_QUEUED", "SCE_GE_LIST_QUEUED"))]
    Queued = 1,
    /// The list is currently being executed.
    #[doc(alias("PSP_GE_LIST_DRAWING_DONE", "SCE_GE_LIST_DRAWING"))]
    Drawing = 2,
    /// The list was stopped because it encountered stall address.
    #[doc(alias("PSP_GE_LIST_STALL_REACHED", "SCE_GE_LIST_STALLING"))]
    Stalling = 3,
    /// The list is paused because of a signal.
    #[doc(alias("PSP_GE_LIST_CANCEL_DONE", "SCE_GE_LIST_PAUSED"))]
    Paused = 4,
}

/// The break mode to interrupt display list operation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum BreakMode {
    Current = 0,
    AllQueue = 1,
}

/// Break unused parameter.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[doc(alias("SceGeBreakParam", "PspGeBreakParam"))]
pub struct BreakParam {
    pub buffer: [u32; 4],
}

/// Represents the function pointer of [`GeCallbackOptions`].
///
/// # Parameters
///
/// - `intr_code`: The instruction code.
/// - `arg` **[[InOut parameter]]**: The argument value on related [`GeCallbackOptions`] fields.
/// - `display_list_pos`: A pointer of the current display list position.
#[doc(alias("PspGeCallbackData", "SceGeCallbackData"))]
pub type GeCallbackFn =
    unsafe extern "C" fn(intr_code: u32, arg: *mut c_void, display_list_pos: *const c_void);

/// The Graphics Engine callback options.
#[doc(alias("SceGeCallbackData", "PspGeCallbackData"))]
pub struct GeCallbackOptions {
    /// The callback for the signal interrupt.
    pub signal_func: Option<GeCallbackFn>,
    /// The callback argument for signal interrupt.
    pub signal_arg: *mut c_void,
    /// The callback for the finish interrupt.
    pub finish_func: Option<GeCallbackFn>,
    /// The callback argument for finish interrupt.
    pub finish_arg: *mut c_void,
}

/// Size mode for the EDRAM.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum EdramSizeMode {
    /// The 01g model maximum size: `0x200000`.
    #[default]
    FatSize = 0x200000,
    /// The later models maximum size: `0x400000`.
    SlimSize = 0x400000,
}

/// A Graphics Engine breakpoint.
#[repr(C)]
#[doc(alias("SceGeBreakpoint"))]
pub struct GeBreakpoint {
    /// The address of the command where to break.
    pub addr: SceSize,
    /// The number of times to break there.
    pub count: u32,
}

#[psp_stub(libname = "sceGe_user", flags = 0x4009, use_crate)]
unsafe extern "C" {
    /// Gets the size of eDRAM.
    ///
    /// # Return Value
    ///
    /// Returns the size of the eDRAM in bytes.
    #[nid(0x1F6752AD)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeEdramGetSize() -> SceSize;

    /// Gets the physical eDRAM address.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the start of the eDRAM.
    #[nid(0xE47E40E4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeEdramGetAddr() -> *mut u8;

    /// Sets the EDRAM address translation.
    ///
    /// # Parameters
    ///
    /// - `width`: The memory width.
    ///
    /// # Return Value
    ///
    /// Returns the previous memory width on success, error value otherwise.
    #[nid(0xB77905EA)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeEdramSetAddrTranslation(
        width: TranslationMemWidth,
    ) -> SceResult<TranslationMemWidth>;

    /// Get the value of a command.
    ///
    /// In other words: Gets the last command ran for that operation, with its arguments.
    ///
    /// # Parameters
    ///
    /// - `cmd`: The drawing command register kind.
    ///
    /// # Return Value
    ///
    /// Returns the last set value in the specified GE drawing command register on success, error
    /// value otherwise.
    #[nid(0xDC93CFEF)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeGetCmd(cmd: GeCmd) -> SceResult<u32>;

    /// Get a matrix from a matrix register.
    ///
    /// # Parameters
    ///
    /// - `reg`: The matrix register to get the matrix.
    /// - `matrix` **[[Out parameter]]**: A pointer to a buffer to receive the matrix value.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `matrix` buffer must have at least [`MatrixRegister::matrix_size`] units allocated.
    #[nid(0x57C8945B)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceGeGetMtx(reg: MatrixRegister, matrix: *mut i32) -> SceResult<()>;

    /// Saves the Graphics Engine internal state context of the current display list.
    ///
    /// # Parameters
    ///
    /// - `ctx` **[[Out parameter]]**: A reference to save the GE context.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x438A385A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeSaveContext(ctx: &mut GeContext) -> SceResult<()>;

    /// Restores the Graphics Engine internal state context.
    ///
    /// # Parameters
    ///
    /// - `ctx` **[[In parameter]]**: A reference to restore the GE context.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x0BF608FB)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeRestoreContext(ctx: &GeContext) -> SceResult<()>;

    /// Enqueues a display list at the end of the queue.
    ///
    /// # Parameters
    ///
    /// - `list` **[[InOut parameter]]**: A pointer to the head of the list to queue.
    /// - `stall` **[[InOut parameter]]**: A pointer to the stall. If `null` then no stall address
    ///   is set and the list is transferred immediately.
    /// - `callback_id`: The GE callback ID of the callback to be used.
    /// - `args` **[[In parameter]]**: A optional reference to the list arguments.
    ///
    /// # Return Value
    ///
    /// Returns the display list ID on success, error value otherwise.
    #[nid(0xAB49E76A)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeListEnQueue(
        list: *mut c_void, stall: *mut c_void, callback_id: GeCallbackId, args: Option<&GeListArgs>,
    ) -> SceResult<DisplayListId>;

    /// Enqueues a display list at the start of the queue.
    ///
    /// It will be the next display list that will be executed.
    ///
    /// # Parameters
    ///
    /// - `list` **[[InOut parameter]]**: A pointer to the head of the list to queue.
    /// - `stall` **[[InOut parameter]]**: The stall address. If `null` then no stall address is set
    ///   and the list is transferred immediately.
    /// - `callback_id`: The GE callback ID of the callback to be used.
    /// - `args` **[[In parameter]]**: A optional reference to the list arguments.
    ///
    /// # Return Value
    ///
    /// Returns the display list ID on success, error value otherwise.
    #[nid(0x1C0D95A6)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeListEnQueueHead(
        list: *mut c_void, stall: *mut c_void, callback_id: GeCallbackId, args: Option<&GeListArgs>,
    ) -> SceResult<DisplayListId>;

    /// Dequeues a display list.
    ///
    /// # Parameters
    ///
    /// - `id`: The display list ID to dequeue.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5FB86AB0)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeListDeQueue(id: DisplayListId) -> SceResult<()>;


    /// Updates the stall address of a display list.
    ///
    /// # Parameters
    ///
    /// - `id`: The display list ID.
    /// - `stall` **[[InOut parameter]]**: The new stall address.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE0D68148)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeListUpdateStallAddr(id: DisplayListId, stall: *mut c_void) -> SceResult<()>;

    /// Synchronizes a display list.
    ///
    /// # Parameters
    ///
    /// - `id`: The display list ID.
    /// - `mode`: The synchronization mode to use.
    ///
    /// # Return Value
    ///
    /// Returns the state of the display list on success, error value otherwise.
    ///
    /// If `mode` is [`DisplayListSyncMode::Wait`], it will always return
    /// [`DisplayListState::Completed`] on success.
    #[nid(0x03444EB4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeListSync(
        id: DisplayListId, mode: DisplayListSyncMode,
    ) -> SceResult<DisplayListState>;

    /// Synchronizes a display list drawing.
    ///
    /// # Parameters
    ///
    /// - `mode`: The synchronization mode to use.
    ///
    /// # Return Value
    ///
    /// Returns the state of the display list on success, error value otherwise.
    ///
    /// If `mode` is [`DisplayListSyncMode::Wait`], it will always return
    /// [`DisplayListState::Completed`] on success.
    #[nid(0xB287BD61)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeDrawSync(mode: DisplayListSyncMode) -> SceResult<DisplayListState>;

    /// Interrupts the Graphic Engine drawing.
    ///
    /// # Parameters
    ///
    /// - `mode`: The interruption mode to use.
    /// - `param` **[[Out parameter]]**: Unused parameter.
    ///
    /// # Return Value
    ///
    /// Returns the display list ID of the interrupted display list.
    #[nid(0xB448EC0D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceGeBreak(mode: BreakMode, param: *mut BreakParam) -> SceResult<DisplayListId>;

    /// Continues the queue execution after a [`sceGeBreak`].
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x4C06E472)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeContinue() -> SceResult<()>;

    /// Sets Graphics Engine finish/signal callbacks.
    ///
    /// # Parameters
    ///
    /// - `options` **[[In parameter]]**: A reference to the options to configure callback.
    ///
    /// # Return Value
    ///
    /// Returns the GE callback ID on success, error value otherwise.
    #[nid(0xA4FC06A4)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeSetCallback(options: &GeCallbackOptions) -> SceResult<GeCallbackId>;

    /// Unset a Graphics Engine finish/signal callbacks.
    ///
    /// # Parameters
    ///
    /// - `id`: The GE callback ID to unset.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x05DB22CE)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceGeUnsetCallback(id: GeCallbackId) -> SceResult<()>;
}

// FIXME: Add missing functions:
// - `sceGeEdramInit`
// - `sceGeEdramSetRefreshParam`
// - `sceGeEnd`
// - `sceGeGetList`
// - `sceGeGetReg`
// - `sceGeInit`
// - `sceGeRegisterLogHandler`
// - `sceGeSetCmd`
// - `sceGeSetGeometryClock`
// - `sceGeSetReg`
#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceGe_driver", flags = 0x0009, use_crate)]
unsafe extern "C" {
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
    pub safe fn sceGeEdramGetSize() -> SceSize;

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
    pub safe fn sceGeEdramGetAddr() -> *mut u8;

    /// Sets the EDRAM address translation.
    ///
    /// # Parameters
    ///
    /// - `width`: The memory width.
    ///
    /// # Return Value
    ///
    /// Returns the previous memory width on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x2DFAAF99 }
        else if cfg!(feature = "psp_630") { 0xD8E03A2B }
        else if cfg!(feature = "psp_600") { 0x8F1E3AF7 }
        else if cfg!(feature = "psp_570") { 0x658A68F7 }
        else if cfg!(feature = "psp_500") { 0xB97713A6 }
        else if cfg!(feature = "psp_420") { 0xFF6E98FA }
        else if cfg!(feature = "psp_395") { 0xAE983AF9 }
        else if cfg!(feature = "psp_380") { 0xD6CCE983 }
        else if cfg!(feature = "psp_370") { 0x60D5FA09 }
        else { 0xB77905EA }
    )]
    pub safe fn sceGeEdramSetAddrTranslation(
        width: TranslationMemWidth,
    ) -> SceResult<TranslationMemWidth>;

    /// Get the value of a command.
    ///
    /// In other words: Gets the last command ran for that operation, with its arguments.
    ///
    /// # Parameters
    ///
    /// - `cmd`: The drawing command register kind.
    ///
    /// # Return Value
    ///
    /// Returns the last set value in the specified GE drawing command register on success, error
    /// value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE6EE2394 }
        else if cfg!(feature = "psp_630") { 0xED34CF92 }
        else if cfg!(feature = "psp_600") { 0x2B37561E }
        else if cfg!(feature = "psp_570") { 0xC9C4D13C }
        else if cfg!(feature = "psp_500") { 0xEA35519F }
        else if cfg!(feature = "psp_420") { 0xA984777C }
        else if cfg!(feature = "psp_395") { 0x69C787E1 }
        else if cfg!(feature = "psp_380") { 0x5D9A4FD0 }
        else if cfg!(feature = "psp_370") { 0xAF86B4AE }
        else { 0xDC93CFEF }
    )]
    pub safe fn sceGeGetCmd(cmd: GeCmd) -> SceResult<u32>;

    /// Get a matrix from a matrix register.
    ///
    /// # Parameters
    ///
    /// - `reg`: The matrix register to get the matrix.
    /// - `matrix` **[[Out parameter]]**: A pointer to a buffer to receive the matrix value.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `matrix` buffer must have at least [`MatrixRegister::matrix_size`] units allocated.
    #[nid(if cfg!(feature = "psp_660") { 0x95D0CDC2 }
        else if cfg!(feature = "psp_630") { 0x7FDFE7D4 }
        else if cfg!(feature = "psp_600") { 0xA23C2482 }
        else if cfg!(feature = "psp_570") { 0xB875A664 }
        else if cfg!(feature = "psp_500") { 0xA0FC93DA }
        else if cfg!(feature = "psp_420") { 0x95BBBE85 }
        else if cfg!(feature = "psp_395") { 0xD7268C51 }
        else if cfg!(feature = "psp_380") { 0x06F13897 }
        else if cfg!(feature = "psp_370") { 0xC993F3D5 }
        else { 0x57C8945B }
    )]
    pub unsafe fn sceGeGetMtx(reg: MatrixRegister, matrix: *mut i32) -> SceResult<()>;

    /// Sets a matrix of a matrix register.
    ///
    /// # Parameters
    ///
    /// - `reg`: The matrix register.
    /// - `matrix` **[[InOut parameter]]**: A pointer to a buffer storing the matrix.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Safety
    ///
    /// The `matrix` buffer must have at least [`MatrixRegister::matrix_size`] units allocated.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.00.
    #[nid(if cfg!(feature = "psp_660") { 0x5E7DE870 }
        else if cfg!(feature = "psp_630") { 0x8083DAFC }
        else if cfg!(feature = "psp_600") { 0x56A5110E }
        else if cfg!(feature = "psp_570") { 0x3F792435 }
        else if cfg!(feature = "psp_500") { 0x3F10632E }
        else if cfg!(feature = "psp_420") { 0xA465179C }
        else if cfg!(feature = "psp_395") { 0xB6D2B704 }
        else if cfg!(feature = "psp_380") { 0x3F736D2A }
        else if cfg!(feature = "psp_370") { 0xB048AC9D }
        else { 0x5A0103E6 }
    )]
    pub unsafe fn sceGeSetMtx(reg: MatrixRegister, matrix: *mut i32) -> SceResult<()>;

    /// Saves the Graphics Engine internal state context of the current display list.
    ///
    /// # Parameters
    ///
    /// - `ctx` **[[Out parameter]]**: A reference to save the GE context.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x0FFB3AFD }
        else if cfg!(feature = "psp_630") { 0x04B53579 }
        else if cfg!(feature = "psp_600") { 0x36585054 }
        else if cfg!(feature = "psp_570") { 0xB866B7F4 }
        else if cfg!(feature = "psp_500") { 0x882C0F2A }
        else if cfg!(feature = "psp_420") { 0x791D06F8 }
        else if cfg!(feature = "psp_395") { 0xA7254F4F }
        else if cfg!(feature = "psp_380") { 0x0A8DDCC0 }
        else if cfg!(feature = "psp_370") { 0x19D5EC6E }
        else { 0x438A385A }
    )]
    pub safe fn sceGeSaveContext(ctx: &mut GeContext) -> SceResult<()>;

    /// Restores the Graphics Engine internal state context.
    ///
    /// # Parameters
    ///
    /// - `ctx` **[[In parameter]]**: A reference to restore the GE context.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x9B893EFC }
        else if cfg!(feature = "psp_630") { 0xB4E22A53 }
        else if cfg!(feature = "psp_600") { 0x87A22519 }
        else if cfg!(feature = "psp_570") { 0x76C850F0 }
        else if cfg!(feature = "psp_500") { 0x5289FB40 }
        else if cfg!(feature = "psp_420") { 0x4129CE0E }
        else if cfg!(feature = "psp_395") { 0x98FD5CA9 }
        else if cfg!(feature = "psp_380") { 0xB988F615 }
        else if cfg!(feature = "psp_370") { 0xA946F63B }
        else { 0x0BF608FB }
    )]
    pub safe fn sceGeRestoreContext(ctx: &GeContext) -> SceResult<()>;

    /// Enqueues a display list at the end of the queue.
    ///
    /// # Parameters
    ///
    /// - `list` **[[InOut parameter]]**: A pointer to the head of the list to queue.
    /// - `stall` **[[InOut parameter]]**: A pointer to the stall. If `null` then no stall address
    ///   is set and the list is transferred immediately.
    /// - `callback_id`: The GE callback ID of the callback to be used.
    /// - `args` **[[In parameter]]**: A optional reference to the list arguments.
    ///
    /// # Return Value
    ///
    /// Returns the display list ID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xAFE6C9EF }
        else if cfg!(feature = "psp_630") { 0xD8A53104 }
        else if cfg!(feature = "psp_600") { 0x7E2381D2 }
        else if cfg!(feature = "psp_570") { 0x917EBCB4 }
        else if cfg!(feature = "psp_500") { 0x0C9C3686 }
        else if cfg!(feature = "psp_420") { 0x10F91FD0 }
        else if cfg!(feature = "psp_395") { 0xDC09D302 }
        else if cfg!(feature = "psp_380") { 0x4659CC7C }
        else if cfg!(feature = "psp_370") { 0x1F9C7FE1 }
        else { 0xAB49E76A }
    )]
    pub safe fn sceGeListEnQueue(
        list: *mut c_void, stall: *mut c_void, callback_id: GeCallbackId, args: Option<&GeListArgs>,
    ) -> SceResult<DisplayListId>;

    /// Enqueues a display list at the start of the queue.
    ///
    /// It will be the next display list that will be executed.
    ///
    /// # Parameters
    ///
    /// - `list` **[[InOut parameter]]**: A pointer to the head of the list to queue.
    /// - `stall` **[[InOut parameter]]**: A pointer to the stall. If `null` then no stall address
    ///   is set and the list is transferred immediately.
    /// - `callback_id`: The GE callback ID of the callback to be used.
    /// - `args` **[[In parameter]]**: A optional reference to the list arguments.
    ///
    /// # Return Value
    ///
    /// Returns the display list ID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x762FA6D0 }
        else if cfg!(feature = "psp_630") { 0xBB302D0B }
        else if cfg!(feature = "psp_600") { 0x362B73C5 }
        else if cfg!(feature = "psp_570") { 0x5511452E }
        else if cfg!(feature = "psp_500") { 0xEB355BCE }
        else if cfg!(feature = "psp_420") { 0x910228B9 }
        else if cfg!(feature = "psp_395") { 0xD2BF1F07 }
        else if cfg!(feature = "psp_380") { 0xC49D0BA4 }
        else if cfg!(feature = "psp_370") { 0x9CBD5F11 }
        else { 0x1C0D95A6 }
    )]
    pub safe fn sceGeListEnQueueHead(
        list: *mut c_void, stall: *mut c_void, callback_id: GeCallbackId, args: Option<&GeListArgs>,
    ) -> SceResult<DisplayListId>;

    /// Dequeues a display list.
    ///
    /// # Parameters
    ///
    /// - `id`: The display list ID to dequeue.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xA741076A }
        else if cfg!(feature = "psp_630") { 0x0BC8BCED }
        else if cfg!(feature = "psp_600") { 0x7845ABFC }
        else if cfg!(feature = "psp_570") { 0x70959D25 }
        else if cfg!(feature = "psp_500") { 0xB7B12709 }
        else if cfg!(feature = "psp_420") { 0x33875173 }
        else if cfg!(feature = "psp_395") { 0xF5A03D9B }
        else if cfg!(feature = "psp_380") { 0xED59B402 }
        else if cfg!(feature = "psp_370") { 0xCA1CAC6F }
        else { 0x5FB86AB0 }
    )]
    pub safe fn sceGeListDeQueue(id: DisplayListId) -> SceResult<()>;

    /// Updates the stall address of a display list.
    ///
    /// # Parameters
    ///
    /// - `id`: The display list ID.
    /// - `stall` **[[InOut parameter]]**: The new stall address.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x7CFD2733 }
        else if cfg!(feature = "psp_630") { 0xEF1B48C6 }
        else if cfg!(feature = "psp_600") { 0x862248FD }
        else if cfg!(feature = "psp_570") { 0x56509208 }
        else if cfg!(feature = "psp_500") { 0x65FA79F8 }
        else if cfg!(feature = "psp_420") { 0xB808FF1F }
        else if cfg!(feature = "psp_395") { 0xC545B81E }
        else if cfg!(feature = "psp_380") { 0xB267CB88 }
        else if cfg!(feature = "psp_370") { 0x2FA58122 }
        else { 0xE0D68148 }
    )]
    pub unsafe fn sceGeListUpdateStallAddr(id: DisplayListId, stall: *mut c_void) -> SceResult<()>;

    /// Synchronizes a display list.
    ///
    /// # Parameters
    ///
    /// - `id`: The display list ID.
    /// - `mode`: The synchronization mode to use.
    ///
    /// # Return Value
    ///
    /// Returns the state of the display list on success, error value otherwise.
    ///
    /// If `mode` is [`DisplayListSyncMode::Wait`], it will always return
    /// [`DisplayListState::Completed`] on success.
    #[nid(if cfg!(feature = "psp_660") { 0x4B9554EB }
        else if cfg!(feature = "psp_630") { 0xFEEC36F7 }
        else if cfg!(feature = "psp_600") { 0x23E9F5FA }
        else if cfg!(feature = "psp_570") { 0x0086946E }
        else if cfg!(feature = "psp_500") { 0x6849A0B0 }
        else if cfg!(feature = "psp_420") { 0xCACD154E }
        else if cfg!(feature = "psp_395") { 0xA402A161 }
        else if cfg!(feature = "psp_380") { 0x7008C80C }
        else if cfg!(feature = "psp_370") { 0x264E8BED }
        else { 0x03444EB4 }
    )]
    pub safe fn sceGeListSync(
        id: DisplayListId, mode: DisplayListSyncMode,
    ) -> SceResult<DisplayListState>;

    /// Synchronizes a display list drawing.
    ///
    /// # Parameters
    ///
    /// - `mode`: The synchronization mode to use.
    ///
    /// # Return Value
    ///
    /// Returns the state of the display list on success, error value otherwise.
    ///
    /// If `mode` is [`DisplayListSyncMode::Wait`], it will always return
    /// [`DisplayListState::Completed`] on success.
    #[nid(if cfg!(feature = "psp_660") { 0x1F6B3C34 }
        else if cfg!(feature = "psp_630") { 0xF490E8A0 }
        else if cfg!(feature = "psp_600") { 0xAC348640 }
        else if cfg!(feature = "psp_570") { 0xFF66B09A }
        else if cfg!(feature = "psp_500") { 0x28F51074 }
        else if cfg!(feature = "psp_420") { 0xD194EB8C }
        else if cfg!(feature = "psp_395") { 0x58624E06 }
        else if cfg!(feature = "psp_380") { 0xBAB91657 }
        else if cfg!(feature = "psp_370") { 0xDDB5D956 }
        else { 0xB287BD61 }
    )]
    pub safe fn sceGeDrawSync(mode: DisplayListSyncMode) -> SceResult<DisplayListState>;

    /// Interrupts the Graphic Engine drawing.
    ///
    /// # Parameters
    ///
    /// - `mode`: The interruption mode to use.
    /// - `param` **[[Out parameter]]**: Unused parameter.
    ///
    /// # Return Value
    ///
    /// Returns the display list ID of the interrupted display list.
    #[nid(if cfg!(feature = "psp_660") { 0x71F8DB0E }
        else if cfg!(feature = "psp_630") { 0x34A061D2 }
        else if cfg!(feature = "psp_600") { 0x3351EE04 }
        else if cfg!(feature = "psp_570") { 0x200678C5 }
        else if cfg!(feature = "psp_500") { 0x37A4A9BB }
        else if cfg!(feature = "psp_420") { 0xD4CAFB7C }
        else if cfg!(feature = "psp_395") { 0x2BEAF4F0 }
        else if cfg!(feature = "psp_380") { 0x0385F69C }
        else if cfg!(feature = "psp_370") { 0x417E61FD }
        else { 0xB448EC0D }
    )]
    pub unsafe fn sceGeBreak(mode: BreakMode, param: *mut BreakParam) -> SceResult<DisplayListId>;

    /// Continues the queue execution after a [`sceGeBreak`].
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x816D56EE }
        else if cfg!(feature = "psp_630") { 0xA462A747 }
        else if cfg!(feature = "psp_600") { 0x32A76BF7 }
        else if cfg!(feature = "psp_570") { 0x2148218F }
        else if cfg!(feature = "psp_500") { 0xD31AE2FE }
        else if cfg!(feature = "psp_420") { 0xFA48620D }
        else if cfg!(feature = "psp_395") { 0xCDE4BE78 }
        else if cfg!(feature = "psp_380") { 0x76587173 }
        else if cfg!(feature = "psp_370") { 0x45A5CA45 }
        else { 0x4C06E472 }
    )]
    pub safe fn sceGeContinue() -> SceResult<()>;

    /// Sets Graphics Engine finish/signal callbacks.
    ///
    /// # Parameters
    ///
    /// - `options` **[[In parameter]]**: A reference to the options to configure callback.
    ///
    /// # Return Value
    ///
    /// Returns the GE callback ID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x6A600875 }
        else if cfg!(feature = "psp_630") { 0x1709686F }
        else if cfg!(feature = "psp_600") { 0xD57B9873 }
        else if cfg!(feature = "psp_570") { 0xC9E6DD6F }
        else if cfg!(feature = "psp_500") { 0x9E6022CE }
        else if cfg!(feature = "psp_420") { 0xB6744FA6 }
        else if cfg!(feature = "psp_395") { 0x8EB2762D }
        else if cfg!(feature = "psp_380") { 0xEB9A3134 }
        else if cfg!(feature = "psp_370") { 0x983F796E }
        else { 0xA4FC06A4 }
    )]
    pub safe fn sceGeSetCallback(options: &GeCallbackOptions) -> SceResult<GeCallbackId>;

    /// Unset a Graphics Engine finish/signal callbacks.
    ///
    /// # Parameters
    ///
    /// - `id`: The GE callback ID to unset.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x0B986A8D }
        else if cfg!(feature = "psp_630") { 0x2994C7F0 }
        else if cfg!(feature = "psp_600") { 0xFFD33EAC }
        else if cfg!(feature = "psp_570") { 0xCA4AB6DE }
        else if cfg!(feature = "psp_500") { 0xBD23B64C }
        else if cfg!(feature = "psp_420") { 0x50A00971 }
        else if cfg!(feature = "psp_395") { 0x854B1235 }
        else if cfg!(feature = "psp_380") { 0x47E21E2F }
        else if cfg!(feature = "psp_370") { 0x1F992647 }
        else { 0x05DB22CE }
    )]
    pub safe fn sceGeUnsetCallback(id: GeCallbackId) -> SceResult<()>;

    /// Sets the EDRAM size to be enabled.
    ///
    /// # Parameters
    ///
    /// - `size_mode`: the size mode to enable.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// If `size_mode` is specified as [`EdramSizeMode::SlimSize`] on a PSP Fat model, this function
    /// will result in a error.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.80.
    #[nid(if cfg!(feature = "psp_660") { 0xD8633888 }
        else if cfg!(feature = "psp_630") { 0x2444EC4D }
        else if cfg!(feature = "psp_600") { 0xD4D665C9 }
        else if cfg!(feature = "psp_570") { 0x2A933C1D }
        else if cfg!(feature = "psp_500") { 0x1F226BA9 }
        else if cfg!(feature = "psp_420") { 0x993B1EFC }
        else if cfg!(feature = "psp_395") { 0x1DDA69CE }
        else if cfg!(feature = "psp_380") { 0x0C15B44A }
        else if cfg!(feature = "psp_370") { 0x58C59880 }
        else { 0x5BAA5439 }
    )]
    pub safe fn sceGeEdramSetSize(size_mode: EdramSizeMode) -> SceResult<()>;

    /// Gets the EDRAM physical size.
    ///
    /// # Return Value
    ///
    /// Returns the EDRAM physical size.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.80.
    #[nid(if cfg!(feature = "psp_660") { 0x547EC5F0 }
        else if cfg!(feature = "psp_630") { 0x35AF4E6C }
        else if cfg!(feature = "psp_600") { 0xC774B373 }
        else if cfg!(feature = "psp_570") { 0x0EDEC8A2 }
        else if cfg!(feature = "psp_500") { 0x244A3948 }
        else if cfg!(feature = "psp_420") { 0x5F966E31 }
        else if cfg!(feature = "psp_395") { 0xB5891BE0 }
        else if cfg!(feature = "psp_380") { 0xD6DF36A0 }
        else if cfg!(feature = "psp_370") { 0x847675FB }
        else { 0xC576E897 }
    )]
    pub safe fn sceGeEdramGetHwSize() -> SceSize;

    /// Gets a list of the IDs of the display lists currently being in the queue.
    ///
    /// # Parameters
    ///
    /// - `id_list` **[[Out parameter]]**: A buffer to store the the display lists IDs.
    /// - `id_list_size`: The size of the `id_list` buffer.
    /// - `total_count`: A reference to receive total number of display lists.
    ///
    /// # Return Value
    ///
    /// Returns the number of stored list IDs on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.50.
    #[nid(if cfg!(feature = "psp_660") { 0x82F1049F }
        else if cfg!(feature = "psp_630") { 0x20EF3AC2 }
        else if cfg!(feature = "psp_600") { 0x002E0226 }
        else if cfg!(feature = "psp_570") { 0xD76BD1D7 }
        else if cfg!(feature = "psp_500") { 0x2B0A156D }
        else if cfg!(feature = "psp_420") { 0x96AB2F8B }
        else if cfg!(feature = "psp_395") { 0x7E8DFAC9 }
        else if cfg!(feature = "psp_380") { 0x4DE7F12C }
        else if cfg!(feature = "psp_370") { 0x4894708B }
        else { 0x9ACFF59D }
    )]
    pub unsafe fn sceGeGetListIdList(
        id_list: *mut DisplayListId, id_list_size: SceSize, total_count: &mut SceSize,
    ) -> SceResult<SceSize>;

    /// Put breakpoints in the display list execution.
    ///
    /// # Parameters
    ///
    /// - `bps`: A list of breakpoints to set.
    /// - `bps_size`: The size of `bps` list.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.50.
    #[nid(if cfg!(feature = "psp_660") { 0x05238809 }
        else if cfg!(feature = "psp_630") { 0x39F15186 }
        else if cfg!(feature = "psp_600") { 0x3832B8F7 }
        else if cfg!(feature = "psp_570") { 0xB126F8F0 }
        else if cfg!(feature = "psp_500") { 0xEFF2BDAF }
        else if cfg!(feature = "psp_420") { 0x7896A960 }
        else if cfg!(feature = "psp_395") { 0x25D52F49 }
        else if cfg!(feature = "psp_380") { 0xB0ED9AF1 }
        else if cfg!(feature = "psp_370") { 0x13B01FA7 }
        else { 0xAEC21518 }
    )]
    pub unsafe fn sceGePutBreakpoint(bps: *const GeBreakpoint, bps_size: SceSize) -> SceResult<()>;

    /// Gets set breakpoints.
    ///
    /// # Parameters
    ///
    /// - `bps`: A buffer to receive the breakpoints.
    /// - `bps_size`: The size of `bps` buffer.
    /// - `total_count`: A reference to receive total number of breakpoints.
    ///
    /// # Return Value
    ///
    /// Returns the number of stored breakpoints on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x6E9C829D }
        else if cfg!(feature = "psp_630") { 0x4508BF79 }
        else if cfg!(feature = "psp_600") { 0xDABC7000 }
        else if cfg!(feature = "psp_570") { 0x1DA9DCE2 }
        else if cfg!(feature = "psp_500") { 0x6CC7CC78 }
        else if cfg!(feature = "psp_420") { 0x9C8ED4BC }
        else if cfg!(feature = "psp_395") { 0x0832D7DA }
        else if cfg!(feature = "psp_380") { 0x913D81CD }
        else if cfg!(feature = "psp_370") { 0x71F71E3C }
        else { 0x7B481502 }
    )]
    pub unsafe fn sceGeGetBreakpoint(
        bps: *mut GeBreakpoint, bps_size: SceSize, total_count: &mut SceSize,
    ) -> SceResult<SceSize>;
}

impl crate::private::Sealed for TranslationMemWidth {}
unsafe impl SceResultOk for TranslationMemWidth {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, super::SceError> {
        match ok_value {
            0 => Ok(Self::LinearMode),
            512 => Ok(Self::Bytes512),
            1024 => Ok(Self::Bytes1024),
            2048 => Ok(Self::Bytes2048),
            4096 => Ok(Self::Bytes4096),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for TranslationMemWidth {
    fn into_ok_value(self) -> u32 {
        match self {
            TranslationMemWidth::LinearMode => 0,
            TranslationMemWidth::Bytes512 => 512,
            TranslationMemWidth::Bytes1024 => 1024,
            TranslationMemWidth::Bytes2048 => 2048,
            TranslationMemWidth::Bytes4096 => 4096,
        }
    }
}

impl MatrixRegister {
    /// Get the Graphics Engine matrix size of the this kind of matrix register.
    ///
    /// This is the value that the GE matrix size should have to use [`sceGeGetMtx`] and
    /// [`sceGeSetMtx`] safely.
    pub const fn matrix_size(&self) -> SceSize {
        match self {
            Self::Projection => 0x10,
            _ => 0x0C,
        }
    }
}

impl GeContext {
    /// Creates a empty `GeContext` to be filled with [`sceGeSaveContext`].
    #[inline]
    pub const fn empty() -> Self {
        Self { buffer: [0; 512] }
    }
}

impl Default for GeContext {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl DisplayListId {
    /// Create a new display list ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new display list ID structure from a raw value without checking value range.
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
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for DisplayListId {}
unsafe impl SceResultOk for DisplayListId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for DisplayListId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl fmt::Debug for DisplayListId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DisplayListId").field(&self.to_inner()).finish()
    }
}

impl Default for GeListArgs {
    #[inline]
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            ctx: Default::default(),
            num_stacks: Default::default(),
            stacks: Default::default(),
        }
    }
}

impl GeCallbackId {
    /// Create a new GE callback ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceUid`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new GE callback ID structure from a raw value without checking value range.
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
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for GeCallbackId {}
unsafe impl SceResultOk for GeCallbackId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for GeCallbackId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl fmt::Debug for GeCallbackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GeCallbackId").field(&self.to_inner()).finish()
    }
}

impl crate::private::Sealed for DisplayListState {}
unsafe impl SceResultOk for DisplayListState {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0 => Ok(Self::Completed),
            1 => Ok(Self::Queued),
            2 => Ok(Self::Drawing),
            3 => Ok(Self::Stalling),
            4 => Ok(Self::Paused),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}
unsafe impl SceIntoOkValue for DisplayListState {
    fn into_ok_value(self) -> u32 {
        match self {
            DisplayListState::Completed => 0,
            DisplayListState::Queued => 1,
            DisplayListState::Drawing => 2,
            DisplayListState::Stalling => 3,
            DisplayListState::Paused => 4,
        }
    }
}
