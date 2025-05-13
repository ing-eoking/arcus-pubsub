use std::ffi::{c_char, c_void, c_int, c_long};
use super::types::{ENGINE_ERROR_CODE, rel_time_t, auth_data_t};
use super::config_parser::config_item;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct SERVER_CORE_API {
    pub get_current_time: unsafe extern "C" fn() -> rel_time_t,
    pub realtime: unsafe extern "C" fn(exptime: c_long) -> rel_time_t,
    pub server_version: unsafe extern "C" fn() -> *const c_char,
    pub hash: unsafe extern "C" fn(data: *const c_void, size: usize, seed: u32) -> u32,
    pub parse_config: unsafe extern "C" fn(str: *const c_char, items: *mut config_item, error: *mut c_void) -> c_int,
    pub get_auth_data: unsafe extern "C" fn(cookie: *const c_void, data: *mut auth_data_t),
    pub store_engine_specific: unsafe extern "C" fn(cookie: *const c_void, engine_data: *mut c_void),
    pub get_engine_specific: unsafe extern "C" fn(cookie: *const c_void) -> *mut c_void,
    pub get_socket_fd: unsafe extern "C" fn(cookie: *const c_void) -> c_int,
    pub get_client_ip: unsafe extern "C" fn(cookie: *const c_void) -> *const c_char,
    pub get_thread_index: unsafe extern "C" fn(cookie: *const c_void) -> c_int,
    pub get_noreply: unsafe extern "C" fn(cookie: *const c_void) -> bool,
    pub notify_io_complete: unsafe extern "C" fn(cookie: *const c_void, status: ENGINE_ERROR_CODE),
    pub shutdown: unsafe extern "C" fn(),
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct SERVER_STAT_API {
    pub new_stats: unsafe extern "C" fn() -> *mut c_void,
    pub release_stats: unsafe extern "C" fn(*mut c_void),
    pub evicting: unsafe extern "C" fn(*const c_void, *const c_void, c_int),
}
