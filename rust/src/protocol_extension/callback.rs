use std::ffi::c_void;
use super::engine_common::ENGINE_HANDLE;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum ENGINE_EVENT_TYPE {
    ON_CONNECT     = 0,
    ON_DISCONNECT  = 1,
    ON_AUTH        = 2,
    ON_SWITCH_CONN = 3,
    ON_LOG_LEVEL   = 4
}

// const MAX_ENGINE_EVENT_TYPE: i32 = 5;

#[allow(non_camel_case_types)]
pub type EVENT_CALLBACK = unsafe extern "C" fn(
    cookie: *const c_void,
    event_type: ENGINE_EVENT_TYPE,
    event_data: *const c_void,
    cb_data: *const c_void,
);

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct SERVER_CALLBACK_API {
    pub register_callback:
        unsafe extern "C" fn(
            engine: *mut ENGINE_HANDLE,
            event_type: ENGINE_EVENT_TYPE,
            callback: EVENT_CALLBACK,
            cb_data: *const c_void,
        )
    ,
    pub perform_callbacks:
        unsafe extern "C" fn(
            event_type: ENGINE_EVENT_TYPE,
            data: *const c_void,
            cookie: *const c_void,
        )
    ,
}
