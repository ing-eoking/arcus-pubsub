mod protocol_extension;

use std::ptr;
use std::ffi::*;
use std::os::fd::{RawFd};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::sync::{LazyLock, Mutex};
use std::time::{Instant, Duration};

// use libevent_sys::*;

use protocol_extension::*;

struct IEKData {
    pub lease_time: Instant,
    pub owner: Option<RawFd>,
    pub waiters: HashSet<RawFd>
}

static mut SERVER_API: *mut SERVER_HANDLE_V1 = ptr::null_mut();

static mut IEK_LOCK_DESCRIPTOR: EXTENSION_ASCII_PROTOCOL_DESCRIPTOR =
    EXTENSION_ASCII_PROTOCOL_DESCRIPTOR {
        get_name: get_name,
        accept: accept_command,
        execute: execute_command,
        abort: abort_command,
        cookie: &raw const IEK_LOCK_DESCRIPTOR as *const _ as *const c_void,
        next: std::ptr::null_mut()
    };

static mut IEK_UNLOCK_DESCRIPTOR: EXTENSION_ASCII_PROTOCOL_DESCRIPTOR =
    EXTENSION_ASCII_PROTOCOL_DESCRIPTOR {
        get_name: get_name,
        accept: accept_command,
        execute: execute_command,
        abort: abort_command,
        cookie:  &raw const IEK_UNLOCK_DESCRIPTOR as *const _ as *const c_void,
        next: std::ptr::null_mut()
    };

static IEK: LazyLock<Mutex<HashMap<String, IEKData>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

static CONN: LazyLock<Mutex<HashMap<RawFd, HashSet<String>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

const SUB_CMD: usize = 1;

fn process_lock_command(cookie: *const c_void, iekey: String, lease_time: f64) -> String {
    let mut result = "OK\r\n";
    let sfd: RawFd = unsafe { ((*((*SERVER_API).core)).get_socket_fd)(cookie) };

    {
        let cur_time = Instant::now();
        let exp_time = cur_time + Duration::from_millis((lease_time * 1000.0) as u64);
        let mut iek = IEK.lock().unwrap();
        match iek.entry(iekey.clone()) {
            Entry::Vacant(e) => {
                e.insert(IEKData {
                    lease_time: exp_time,
                    owner: Some(sfd),
                    waiters: HashSet::new()
                });
            },
            Entry::Occupied(mut e) => {
                let data = e.get_mut();
                if data.owner.is_none() || data.lease_time < cur_time {
                    data.owner = Some(sfd);
                    data.lease_time = exp_time;
                    if data.waiters.contains(&sfd) {
                        data.waiters.remove(&sfd);
                    }
                } else if data.owner.unwrap() == sfd {
                    result = "OWNED\r\n";
                } else {
                    data.waiters.insert(sfd);
                    result = "LOCKED\r\n";
                }
            }
        }
    }

    {
        let mut conn = CONN.lock().unwrap();
        let s = match conn.entry(sfd) {
            Entry::Vacant(e) => e.insert(HashSet::new()),
            Entry::Occupied(e) => e.into_mut()
        };

        if !s.contains(&iekey) {
            s.insert(iekey);
        }
    }

    return result.to_string();
}

fn process_unlock_command(cookie: *const c_void, iekey: String) -> String {
    let mut result = "SUCCESS\r\n";

    return result.to_string();
}

#[allow(unused_variables)]
extern "C" fn get_name(cmd_cookie: *const c_void) -> *const c_char {
    return "ingeoking\0".as_ptr() as *const c_char;
}

#[allow(unused_variables)]
extern "C" fn accept_command(cmd_cookie: *const c_void, cookie: *mut c_void,
                             argc: c_int, argv: *mut token_t, ndata: *mut usize,
                             ptr: *mut *mut c_char) -> bool {
    let op = unsafe { CStr::from_ptr((*argv).value)};
    if cmd_cookie == &raw const IEK_LOCK_DESCRIPTOR as *const _ as *const c_void {
        if argc == 3 && op.to_str().unwrap_or("") == "lock" {
            return true;
        }
    } else if cmd_cookie == &raw const IEK_UNLOCK_DESCRIPTOR as *const _ as *const c_void {
        if argc == 2 && op.to_str().unwrap_or("") == "unlock" {
            return true;
        }
    }
    return false;
}

#[allow(unused_variables)]
extern "C" fn execute_command(cmd_cookie: *const c_void, cookie: *const c_void,
    argc: c_int, argv: *mut token_t,
    response_handler: ResponseHandler) -> bool {
    let iekey = unsafe { CStr::from_ptr((*argv.add(SUB_CMD)).value) };
    let mut result  = "ERROR unknown command\r\n".to_string();

    if cmd_cookie == &raw const IEK_LOCK_DESCRIPTOR as *const _ as *const c_void {
        let str_time = unsafe { CStr::from_ptr((*argv.add(SUB_CMD + 1)).value) };
        let lease_time: Result<f64, _> = str_time.to_str().unwrap_or("").parse();
        match lease_time {
            Ok(num) =>
                result = process_lock_command(cookie, iekey.to_string_lossy().into_owned(), num),
            Err(e) =>
                result = "CLIENT_ERROR bad command line format\r\n".to_string(),
        }
    } else if cmd_cookie == &raw const IEK_UNLOCK_DESCRIPTOR as *const _ as *const c_void {

    }
    return unsafe { response_handler(cookie, result.len() as i32, result.as_ptr() as *const c_char) };
}

#[allow(unused_variables)]
extern "C" fn abort_command(cmd_cookie: *const c_void, cookie: *const c_void) {

}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern "C" fn memcached_extensions_initialize(
        config: *const c_char,
        get_server_api: GET_SERVER_API
    ) -> EXTENSION_ERROR_CODE {
    unsafe { SERVER_API =  get_server_api() };

    if unsafe { SERVER_API.is_null() } {
        return EXTENSION_ERROR_CODE::EXTENSION_FATAL;
    }

    let extension = unsafe { (*SERVER_API).extension };
    if unsafe { !((*extension).register_extension)(
                    extension_type_t::EXTENSION_ASCII_PROTOCOL,
                    &raw mut IEK_LOCK_DESCRIPTOR as *mut c_void) }
    {
        return EXTENSION_ERROR_CODE::EXTENSION_FATAL;
    }

    if unsafe { !((*extension).register_extension)(
                    extension_type_t::EXTENSION_ASCII_PROTOCOL,
                    &raw mut IEK_UNLOCK_DESCRIPTOR as *mut c_void) }
    {
        return EXTENSION_ERROR_CODE::EXTENSION_FATAL;
    }

    return EXTENSION_ERROR_CODE::EXTENSION_SUCCESS;

}