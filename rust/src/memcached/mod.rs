pub mod protocol_extension;
mod types;

use libc::*;
use libevent_sys::{event, event_base};

use protocol_extension::{types::ENGINE_ERROR_CODE, *};
use types::*;

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct MemcachedConn {
    pub sfd: c_int,
    pub nevents: c_short,
    pub sasl_conn: *mut c_void,
    #[cfg(feature = "sasl-dev")]
    pub sasl_started: bool,
    #[cfg(feature = "sasl-dev")]
    pub authenticated: bool,
    pub state: STATE_FUNC,
    pub substate: BinSubstates,
    pub event: event,
    pub ev_flags: c_short,
    pub which: c_short,
    pub rbuf: *mut i8,
    pub rcurr: *mut i8,
    pub rsize: c_int,
    pub rbytes: c_int,
    pub wbuf: *mut i8,
    pub wcurr: *mut i8,
    pub wsize: c_int,
    pub wbytes: c_int,
    pub write_and_go: STATE_FUNC,
    pub write_and_free: *mut c_void,
    pub rtype: c_int,
    pub rindex: c_int,
    pub ritem: *mut i8,
    pub rlbytes: u32,
    pub rltotal: u32,
    pub membk: *mut MblckNode,
    pub memblist: MblckList, //ok
    pub hinfo: ItemInfo,
    pub einfo: EitemInfo,
    pub coll_eitem: *mut c_void,
    pub coll_resps: *mut i8,
    pub coll_ecount: c_int,
    pub coll_op: c_int,
    pub coll_key: *mut i8,
    pub coll_nkey: c_int,
    pub coll_index: c_int,
    pub coll_attr_space: ItemAttr, // ok
    pub coll_attrp: *mut ItemAttr,
    pub coll_getrim: bool,
    pub coll_delete: bool,
    pub coll_drop: bool,
    #[cfg(feature = "jhpark-old-smget-interface")]
    pub coll_smgmode: c_int,
    #[cfg(not(feature = "jhpark-old-smget-interface"))]
    pub coll_unique: bool,
    pub coll_bkrange: BkeyRange, // not ok
    pub coll_efilter: EflagFilter,
    pub coll_eupdate: EflagUpdate,
    pub coll_roffset: u32,
    pub coll_rcount: u32,
    pub coll_numkeys: u32,
    pub coll_lenkeys: u32,
    pub coll_strkeys: *mut c_void,
    pub coll_field: Field,
    pub item: *mut c_void,
    pub store_op: EngineStoreOperation,
    pub sbytes: c_int,
    pub iov: *mut iovec,
    pub iovsize: c_int,
    pub iovused: c_int,
    pub msglist: *mut msghdr,
    pub msgsize: c_int,
    pub msgused: c_int,
    pub msgcurr: c_int,
    pub msgbytes: c_int,
    pub ilist: *mut *mut Item,
    pub isize: c_int,
    pub icurr: *mut *mut Item,
    pub ileft: c_int,
    #[cfg(feature = "scan-command")]
    pub pcurr: *mut *mut Item,
    #[cfg(feature = "scan-command")]
    pub pleft: c_int,
    pub suffixlist: *mut *mut i8,
    pub suffixsize: c_int,
    pub suffixcurr: *mut *mut i8,
    pub suffixleft: c_int,
    #[cfg(feature = "detect-long-query")]
    pub lq_result: *mut Field,
    pub protocol: Protocol,
    pub transport: NetworkTransport,
    pub request_id: c_int,
    pub request_addr: sockaddr,
    pub request_addr_size: socklen_t,
    pub hdrbuf: [u8; UDP_HEADER_SIZE], //not ok
    pub pipe_state: c_int,
    pub pipe_count: c_int,
    pub pipe_errlen: c_int,
    pub pipe_reslen: c_int,
    pub pipe_resbuf: [i8; PIPE_RES_MAX_SIZE],
    pub client_ip: [i8; 16],
    pub noreply: bool,
    pub dynamic_buffer: DynamicBuffer,
    pub engine_storage: *mut c_void,
    pub ascii_cmd: *mut EXTENSION_ASCII_PROTOCOL_DESCRIPTOR,
    pub binary_header: ProtocolBinaryRequest,
    pub cas: u64,
    pub cmd: c_short,
    pub opaque: c_int,
    pub keylen: c_int,
    pub next: *mut MemcachedConn,
    pub thread: *mut LibeventThread,
    pub conn_prev: *mut MemcachedConn,
    pub conn_next: *mut MemcachedConn,
    pub aiostat: ENGINE_ERROR_CODE,
    pub ewouldblock: bool,
    #[cfg(feature = "multi-notify-io-complete")]
    pub io_blocked: bool,
    #[cfg(feature = "multi-notify-io-complete")]
    pub current_io_wait: c_uint,
    #[cfg(feature = "multi-notify-io-complete")]
    pub premature_io_complete: c_uint,
    #[cfg(not(feature = "multi-notify-io-complete"))]
    pub io_blocked: bool,
    #[cfg(not(feature = "multi-notify-io-complete"))]
    pub premature_io_complete: bool,
}

#[allow(non_camel_case_types)]
pub type STATE_FUNC = Option<unsafe extern "C" fn(*mut MemcachedConn) -> bool>;

/* incomplete struct */
#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct LibeventThread {
    pub thread_id: pthread_t,
    pub base: *mut event_base,
    pub notify_event: event,
    pub notify_receive_fd: c_int,
    pub notify_send_fd: c_int,
    pub new_conn_queue: *mut c_void,
    pub suffix_cache: *mut c_void,
    pub mutex: pthread_mutex_t,
    pub is_locked: bool,
    pub pending_io: *mut MemcachedConn,
    pub conn_list: *mut MemcachedConn,
    pub index: c_int
}
