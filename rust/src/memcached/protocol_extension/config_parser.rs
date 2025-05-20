use std::ffi::{c_char, c_float};

#[repr(C)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum config_datatype {
    DT_SIZE,
    DT_UINT32,
    DT_FLOAT,
    DT_BOOL,
    DT_STRING,
    DT_CONFIGFILE,
    DT_CHAR
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub union config_value {
    pub dt_size: *const usize,
    pub dt_uint32: *const u32,
    pub dt_float: *const c_float,
    pub dt_bool: *const bool,
    pub dt_string: *const *const c_char,
    pub dt_char: *const c_char
}


#[repr(C)]
#[allow(non_camel_case_types)]
pub struct config_item {
    pub key: *const c_char,
    pub datatype: config_datatype,
    pub value: config_value,
    pub found: bool
}
