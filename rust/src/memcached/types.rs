use libc::*;

pub type RealTime = u32;
pub type Item = c_void;
#[allow(dead_code)]
pub type Eitem = c_void;

pub const MAX_BKEY_LENG: usize = 31;
pub const MAX_EFLAG_LENG: usize = 31;
pub const MAX_EFLAG_COMPARE_COUNT: usize = 100;
pub const UDP_HEADER_SIZE: usize = 8;

#[allow(dead_code)]
const PIPE_CMD_MAX_COUNT: usize = 500;
#[allow(dead_code)]
const PIPE_RES_DATA_SIZE: usize = 40;
#[allow(dead_code)]
const PIPE_RES_HEAD_SIZE: usize = 20;
#[allow(dead_code)]
const PIPE_RES_TAIL_SIZE: usize = 40;
pub const PIPE_RES_MAX_SIZE: usize = (PIPE_CMD_MAX_COUNT*PIPE_RES_DATA_SIZE)
                                     +PIPE_RES_HEAD_SIZE+PIPE_RES_TAIL_SIZE;

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct DynamicBuffer {
    pub buffer: *mut i8,
    pub size: usize,
    pub offset: usize,
}

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum BinSubstates {
    BinNoState,
    BinReadingSetHeader,
    BinReadingCasHeader,
    BinReadSetValue,
    BinReadingGetKey,
    BinReadingStat,
    BinReadingDelHeader,
    BinReadingIncrHeader,
    BinReadFlushExptime,
    BinReadFlushPrefixExptime,
    BinReadingSaslAuth,
    BinReadingSaslAuthData,
    BinReadingGetattr,
    BinReadingSetattr,
    BinReadingLopCreate,
    BinReadingLopPrepareNread,
    BinReadingLopNreadComplete,
    BinReadingLopDelete,
    BinReadingLopGet,
    BinReadingSopCreate,
    BinReadingSopPrepareNread,
    BinReadingSopNreadComplete,
    BinReadingSopGet,
    BinReadingBopCreate,
    BinReadingBopPrepareNread,
    BinReadingBopNreadComplete,
    BinReadingBopUpdatePrepareNread,
    BinReadingBopUpdateNreadComplete,
    BinReadingBopDelete,
    BinReadingBopGet,
    BinReadingBopCount,
    BinReadingBopPosition,
    BinReadingBopPwg,
    BinReadingBopGbp,
    #[cfg(not(feature = "old"))]
    BinReadingBopPrepareNreadKeys,
    #[cfg(not(feature = "old"))]
    BinReadingBopNreadKeysComplete,
    BinReadingPacket,
}

#[repr(C)]
pub struct MblckNode {
    pub next: *mut MblckNode,
    pub data: [c_char; 1],
}

#[repr(C)]
pub struct Bkey {
    pub val: [c_uchar; MAX_BKEY_LENG],
    len: u8,
}

#[repr(C)]
pub struct MblckList {
    pub pool: *mut c_void,
    pub head: *mut MblckNode,
    pub tail: *mut MblckNode,
    pub blck_cnt: u32,
    pub body_len: u32,
    pub item_cnt: u32,
    pub item_len: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct MblckPool {
    pub head: *mut MblckNode,
    pub tail: *mut MblckNode,
    pub blck_len: u32,
    pub body_len: u32,
    pub used_cnt: u32,
    pub free_cnt: u32,
}

#[repr(C)]
pub struct ValueItem {
    pub len: u32,
    pub ptr: [c_char; 1],
}

#[repr(C)]
pub struct ItemInfo {
    pub cas: u64,
    pub flags: u32,
    pub exptime: RealTime,
    pub item_type: u8,
    pub clsid: u8,
    pub nkey: u16,
    pub nbytes: u32,
    pub nvalue: u32,
    pub naddnl: u32,
    pub key: *const c_void,
    pub value: *const c_void,
    pub addnl: *mut *mut ValueItem,
}

#[repr(C)]
pub struct EitemInfo {
    pub nbytes: u32,
    pub nvalue: u32,
    pub naddnl: u32,
    pub nscore: u16,
    pub neflag: u16,
    pub value: *const c_char,
    pub addnl: *mut *mut ValueItem,
    pub score: *const c_uchar,
    pub eflag: *const c_uchar,
}

#[repr(C)]
pub struct ItemAttr {
    pub flags: c_uint,
    pub exptime: RealTime,
    pub count: c_int,
    pub maxcount: c_int,
    pub maxbkeyrange: Bkey,
    pub minbkey: Bkey,
    pub maxbkey: Bkey,
    pub item_type: c_uchar,
    pub ovflaction: c_uchar,
    pub readable: c_uchar,
    pub trimmed: c_uchar,
}


#[repr(C)]
#[allow(dead_code)]
pub enum Protocol {
    AsciiProt = 3,
    BinaryProt,
    NegotiatingProt,
}

#[repr(C)]
#[allow(dead_code)]
pub enum NetworkTransport {
    LocalTransport = 0,
    TcpTransport = 1,
    UdpTransport = 2,
}

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum EngineStoreOperation{
    OPERATION_ADD = 1,
    OPERATION_SET,
    OPERATION_REPLACE,
    OPERATION_APPEND,
    OPERATION_PREPEND,
    OPERATION_CAS
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ProtocolBinaryRequest {
    pub magic: c_uchar,
    pub opcode: c_uchar,
    pub keylen: c_ushort,
    pub extlen: c_uchar,
    pub datatype: c_uchar,
    pub vbucket: c_ushort,
    pub bodylen: c_uint,
    pub opaque: c_uint,
    pub cas: c_ulonglong,
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub union ProtocolBinaryRequestHeader {
    pub request: ProtocolBinaryRequest,
    pub bytes: [c_uchar; 24],
}

#[repr(C)]
pub struct BkeyRange {
    from_bkey: [c_uchar; MAX_BKEY_LENG],
    to_bkey: [c_uchar; MAX_BKEY_LENG],
    from_nbkey: u8,
    to_nbkey: u8,
}

#[repr(C)]
pub struct EflagFilter {
    bitwval: [c_uchar; MAX_EFLAG_LENG],
    compval: [c_uchar; MAX_EFLAG_LENG * MAX_EFLAG_COMPARE_COUNT],
    nbitwval: u8,
    ncompval: u8,
    compvcnt: u8,
    offset: u8,
    bitwop: u8,
    compop: u8,
}

#[repr(C)]
pub struct EflagUpdate {
    eflag: [c_uchar; MAX_EFLAG_LENG],
    neflag: u8,
    offset: u8,
    bitwop: u8,
    reserved: [c_uchar; 6],
}

#[repr(C)]
pub struct Field {
    value: *mut c_char,
    length: usize,
}