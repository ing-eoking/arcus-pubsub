use std::ffi::c_char;

#[allow(non_camel_case_types)]
pub type rel_time_t = u32;

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum ENGINE_ERROR_CODE {
    ENGINE_SUCCESS     = 0x00,
    ENGINE_KEY_ENOENT  = 0x01,
    ENGINE_KEY_EEXISTS = 0x02,
    ENGINE_ENOMEM      = 0x03,
    ENGINE_NOT_STORED  = 0x04,
    ENGINE_EINVAL      = 0x05,
    ENGINE_ENOTSUP     = 0x06,
    ENGINE_EWOULDBLOCK = 0x07,
    ENGINE_E2BIG       = 0x08,
    ENGINE_WANT_MORE   = 0x09,
    ENGINE_DISCONNECT  = 0x0a,
    ENGINE_EACCESS     = 0x0b,
    ENGINE_NOT_MY_VBUCKET = 0x0c,
    ENGINE_EDUPLICATE  = 0x0d,
    ENGINE_EBADTYPE     = 0x32,
    ENGINE_EOVERFLOW    = 0x33,
    ENGINE_EBADVALUE    = 0x34,
    ENGINE_EINDEXOOR    = 0x35,
    ENGINE_EBKEYOOR     = 0x36,
    ENGINE_ELEM_ENOENT  = 0x37,
    ENGINE_ELEM_EEXISTS = 0x38,
    ENGINE_EBADATTR     = 0x39,
    ENGINE_EBADBKEY     = 0x3a,
    ENGINE_EBADEFLAG    = 0x3b,
    ENGINE_UNREADABLE   = 0x3c,
    ENGINE_PREFIX_ENAME  = 0x51,
    ENGINE_PREFIX_ENOENT = 0x52,
    ENGINE_FAILED      = 0xff
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct auth_data_t {
    username: *const c_char,
    config: *const c_char
}
