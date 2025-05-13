pub mod callback;
pub mod config_parser;
pub mod engine_common;
pub mod server_api;
pub mod types;

use std::ffi::{c_char, c_void, c_int};
use server_api::{SERVER_CORE_API, SERVER_STAT_API};
use engine_common::ENGINE_HANDLE;
use callback::SERVER_CALLBACK_API;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum EXTENSION_ERROR_CODE {
    EXTENSION_SUCCESS = 0x00,
    EXTENSION_FATAL = 0xfe,
    EXTENSION_FAILED = 0xff
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum EXTENSION_LOG_LEVEL {
    EXTENSION_LOG_DETAIL,
    EXTENSION_LOG_DEBUG,
    EXTENSION_LOG_INFO,
    EXTENSION_LOG_WARNING
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum extension_type_t {
    EXTENSION_DAEMON = 0x00,
    EXTENSION_LOGGER,
    EXTENSION_ASCII_PROTOCOL
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct token_t {
    pub value: *const c_char,
    pub length: usize,
}

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct EXTENSION_DAEMON_DESCRIPTOR {
    pub get_name: Option<unsafe extern "C" fn() -> *const c_char>,
    pub next: *mut EXTENSION_DAEMON_DESCRIPTOR,
}

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct EXTENSION_LOGGER_DESCRIPTOR {
    pub get_name: Option<unsafe extern "C" fn() -> *const c_char>,
    pub log: Option<unsafe extern "C" fn(severity: EXTENSION_LOG_LEVEL,
                                        client_cookie: *const c_void,
                                        fmt: *const c_char, ...)>,
}

pub type ResponseHandler = unsafe extern "C" fn(
    cookie: *const c_void,
    nbytes: c_int,
    dta: *const c_char,
) -> bool;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct EXTENSION_ASCII_PROTOCOL_DESCRIPTOR {
    pub get_name: unsafe extern "C" fn(cmd_cookie: *const c_void) -> *const c_char,
    pub accept:
        unsafe extern "C" fn(
            cmd_cookie: *const c_void,
            cookie: *mut c_void,
            argc: c_int,
            argv: *mut token_t,
            ndata: *mut usize,
            ptr: *mut *mut c_char,
        ) -> bool,
    pub execute:
        unsafe extern "C" fn(
            cmd_cookie: *const c_void,
            cookie: *const c_void,
            argc: c_int,
            argv: *mut token_t,
            response_handler: ResponseHandler,
        ) -> bool,
    pub abort: unsafe extern "C" fn(cmd_cookie: *const c_void, cookie: *const c_void),
    pub cookie: *const c_void,
    pub next: *mut EXTENSION_ASCII_PROTOCOL_DESCRIPTOR,
}

#[repr(C)]
pub struct SERVER_LOG_API {
    pub get_logger: Option<unsafe extern "C" fn() -> *mut EXTENSION_LOGGER_DESCRIPTOR>,
    pub get_level: Option<unsafe extern "C" fn() -> EXTENSION_LOG_LEVEL>,
    pub set_level: Option<unsafe extern "C" fn(severity: EXTENSION_LOG_LEVEL)>,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct SERVER_EXTENSION_API {
    pub register_extension:
        unsafe extern "C" fn(
            _type: extension_type_t,
            extension: *mut c_void
        ) -> bool,
    pub unregister_extension:
        unsafe extern "C" fn(
            _type: extension_type_t,
            extension: *mut c_void
        ),
    pub get_extension: unsafe extern "C" fn(_type: extension_type_t) -> *mut c_void,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct SERVER_HANDLE_V1 {
    pub interface: u64,
    pub core: *mut SERVER_CORE_API,
    pub stat: *mut SERVER_STAT_API,
    pub extension: *mut SERVER_EXTENSION_API,
    pub callback: *mut SERVER_CALLBACK_API,
    pub engine: *mut ENGINE_HANDLE,
    pub log: *mut SERVER_LOG_API,
}

#[allow(non_camel_case_types)]
pub type GET_SERVER_API = unsafe extern "C" fn() -> *mut SERVER_HANDLE_V1;
