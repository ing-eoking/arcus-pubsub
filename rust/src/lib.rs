mod memcached;

use std::ptr;
use std::fs::File;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::net::TcpStream;
use std::mem::ManuallyDrop;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::sync::{LazyLock, Mutex};
use std::time::{Instant, Duration};
use std::ffi::CStr;

use libevent_sys::*;
use libc::{c_void, c_char, c_int, c_short};

use memcached::MemcachedConn;
use memcached::protocol_extension::*;

#[derive(PartialEq)]
#[allow(dead_code)]
enum IEKType {
    PubSub = 0,
    Lock = 1
}

#[derive(PartialEq)]
#[allow(dead_code)]
enum CMDType {
    Publish,
    Subscribe,
    Lock,
    Unlock,
    Unknown
}

struct IEKData {
    pub iek_type: IEKType,
    pub sub_key: Option<i32>,
    pub lease_time: Instant,
    pub owner: Option<usize>,
    pub waiters: HashMap<usize, HashSet<Option<i32>>>
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

static mut IEK_PUB_DESCRIPTOR: EXTENSION_ASCII_PROTOCOL_DESCRIPTOR =
    EXTENSION_ASCII_PROTOCOL_DESCRIPTOR {
        get_name: get_name,
        accept: accept_command,
        execute: execute_command,
        abort: abort_command,
        cookie: &raw const IEK_PUB_DESCRIPTOR as *const _ as *const c_void,
        next: std::ptr::null_mut()
    };

static mut IEK_SUB_DESCRIPTOR: EXTENSION_ASCII_PROTOCOL_DESCRIPTOR =
    EXTENSION_ASCII_PROTOCOL_DESCRIPTOR {
        get_name: get_name,
        accept: accept_command,
        execute: execute_command,
        abort: abort_command,
        cookie: &raw const IEK_SUB_DESCRIPTOR as *const _ as *const c_void,
        next: std::ptr::null_mut()
    };

static IEK: LazyLock<Mutex<HashMap<String, IEKData>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

static CONN: LazyLock<Mutex<HashMap<usize, HashSet<String>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

struct EventMessage {
    ev: *mut event,
    message: String
}

#[allow(unused_variables)]
unsafe extern "C" fn event_resp_cb(fd: i32, _events: i16, arg: *mut c_void) {
    let mut stream =  unsafe {
        ManuallyDrop::new(TcpStream::from_raw_fd(fd))
    };
    let ev_msg = unsafe{ &mut *(arg as *mut EventMessage) };

    if let Err(_e) = stream.write(ev_msg.message.as_bytes()) {
        /* log: "Write failure" */
    }

    unsafe { event_free(ev_msg.ev) };
}

#[allow(unused_variables)]
extern "C" fn unsubscribe_all(cookie: *const c_void,
                              _type: callback::ENGINE_EVENT_TYPE,
                              event_data: *const c_void, cb_data: *const c_void)
{
    let ptr = cookie as usize;
    let s: HashSet<String>;
    {
        let mut conn = CONN.lock().unwrap();
        s = match conn.entry(ptr) {
            Entry::Vacant(e) => HashSet::new(),
            Entry::Occupied(e) => e.get().clone()
        };
    }

    {
        let mut iek = IEK.lock().unwrap();
        for iekey in s {
            match iek.get_mut(&iekey) {
                Some(iekdata) => {
                    iekdata.waiters.remove(&ptr);
                    if iekdata.owner.is_none() ||
                       (!iekdata.owner.is_none() && iekdata.owner.unwrap() == ptr) {
                        iekdata.owner = None;
                        if iekdata.waiters.is_empty() {
                            iek.remove(&iekey);
                        } else if iekdata.iek_type == IEKType::Lock {
                            do_publish(iekdata, format!("UNLOCKED {}", iekey));
                        }
                    }
                }
                None => ()
            }
        }
    }
}

fn add_event_msg(c: usize, msg: String) -> i32 {
    unsafe {
        let mconn = &*(c as *const MemcachedConn);
        let base = mconn.event.ev_base;
        let write_ev = event_new(base, mconn.sfd, EV_WRITE as c_short,
                    Some(event_resp_cb), std::ptr::null_mut());
        let arg = Box::new(EventMessage {
            ev: write_ev,
            message: msg
        });
        event_assign(write_ev, base, mconn.sfd,
                     EV_WRITE as c_short, Some(event_resp_cb),
                     Box::into_raw(arg) as *mut c_void);

        event_add(write_ev, std::ptr::null_mut());

        return (*mconn.thread).notify_send_fd;
    }
}

fn do_publish(iekdata: &IEKData, msg: String) {
    let mut notify_thread: HashSet<i32> = HashSet::new();

    for (conn, s) in &iekdata.waiters {
        for waiter in s {
            let mut new_msg = msg.clone();
            if !waiter.is_none() {
                new_msg += &format!(" [sub_key={}]", waiter.unwrap());
            }
            new_msg += "\r\n";
            notify_thread.insert(add_event_msg(*conn, new_msg));
        }
    }

    for fd in notify_thread {
        unsafe {
            let mut wfd = ManuallyDrop::new(File::from_raw_fd(fd));
            if let Err(_e) = write!(wfd, "{}", '\0') {
                /* log: "Write failure" */
            }
        }
    }
}

#[allow(unused_variables)]
fn process_publish_command(cookie: *const c_void, iekey: String, msg: String) -> String {
    let mut result = "PUBLISHED\r\n".to_string();

    {
        let mut iek = IEK.lock().unwrap();
        match iek.get_mut(&iekey) {
            Some(iekdata) => {
                if iekdata.iek_type != IEKType::PubSub {
                    return "TYPE_MISMATCH\r\n".to_string();
                }
                do_publish(iekdata, format!("CHANNEL {} {}", iekey, msg));
            }
            None => result = "NOT_FOUND\r\n".to_string()
        }
    }
    return result;
}

fn process_subscribe_command(cookie: *const c_void, iekey: String) -> String {
    let ptr = cookie as usize;
    let result = format!("{} SUCCESS\r\n", iekey);

    {
        let mut iek = IEK.lock().unwrap();
        match iek.entry(iekey.clone()) {
            Entry::Vacant(e) => {
                e.insert(IEKData {
                    iek_type: IEKType::PubSub,
                    sub_key: None,
                    lease_time: Instant::now(),
                    owner: None,
                    waiters: [(ptr, None)].into_iter()
                                          .map(|(k, v)| (k, HashSet::from([v])))
                                          .collect()

                });
            },
            Entry::Occupied(mut e) => {
                let data = e.get_mut();
                if data.iek_type != IEKType::PubSub {
                    return format!("{} TYPE_MISMATCH\r\n", iekey);
                }
                data.waiters.entry(ptr)
                            .or_insert_with(HashSet::new)
                            .insert(None);
            }
        }
    }

    {
        let mut conn = CONN.lock().unwrap();
        let s = match conn.entry(ptr) {
            Entry::Vacant(e) => e.insert(HashSet::new()),
            Entry::Occupied(e) => e.into_mut()
        };

        if !s.contains(&iekey) {
            s.insert(iekey);
        }
    }

    return result;
}

fn process_lock_command(cookie: *const c_void, iekey: String, sub_key: Option<i32>, lease_time: f64) -> String {
    let ptr = cookie as usize;
    let mut result = String::new();
    {
        let cur_time = Instant::now();
        let exp_time = cur_time + Duration::from_millis((lease_time * 1000.0) as u64);
        let mut iek = IEK.lock().unwrap();
        match iek.entry(iekey.clone()) {
            Entry::Vacant(e) => {
                e.insert(IEKData {
                    iek_type: IEKType::Lock,
                    sub_key: sub_key,
                    lease_time: exp_time,
                    owner: Some(cookie as usize),
                    waiters: HashMap::new()
                });
                result = "OK\r\n".to_string();
            },
            Entry::Occupied(mut e) => {
                let data = e.get_mut();
                if data.iek_type != IEKType::Lock {
                    return "TYPE_MISMATCH\r\n".to_string();
                }
                if data.owner.is_none() || data.lease_time < cur_time {
                    data.owner = Some(ptr);
                    data.sub_key = sub_key;
                    data.lease_time = exp_time;
                    if let Some(h) = data.waiters.get_mut(&ptr) {
                        if h.contains(&sub_key) {
                            h.remove(&sub_key);
                        }
                    }
                    result = "OK\r\n".to_string();
                } else if data.owner.unwrap() == ptr && data.sub_key == sub_key {
                    data.lease_time = exp_time;
                    result = "OWNED\r\n".to_string();
                } else {
                    data.waiters.entry(ptr)
                                .or_insert_with(HashSet::new)
                                .insert(sub_key);
                    let remaining: f64 = (data.lease_time - cur_time).as_millis() as f64 / 1000.0;
                    result += &format!("RETRY_LATER {:.3}\n", remaining);
                }
            }
        }
    }

    {
        let mut conn = CONN.lock().unwrap();
        let s = match conn.entry(ptr) {
            Entry::Vacant(e) => e.insert(HashSet::new()),
            Entry::Occupied(e) => e.into_mut()
        };

        if !s.contains(&iekey) {
            s.insert(iekey);
        }
    }

    return result;
}

#[allow(dead_code)]
fn process_unlock_command(cookie: *const c_void, iekey: String, sub_key: Option<i32>) -> String {
    let result: String;
    let ptr = cookie as usize;

    {
        let mut iek = IEK.lock().unwrap();
        match iek.get_mut(&iekey) {
            Some(iekdata) => {
                if iekdata.iek_type != IEKType::Lock {
                    return "TYPE_MISMATCH\r\n".to_string();
                }
                if !iekdata.owner.is_none() && iekdata.owner.unwrap() == ptr && iekdata.sub_key == sub_key {
                    iekdata.owner = None;
                    if iekdata.waiters.is_empty() {
                        iek.remove(&iekey);
                    } else {
                        do_publish(iekdata, format!("UNLOCKED {}", iekey));
                    }
                    result = "SUCCESS\r\n".to_string();
                } else {
                    result = "NOT_OWNED\r\n".to_string();
                }
            }
            None => result = "NOT_FOUND\r\n".to_string()
        }
    }

    {
        let mut conn = CONN.lock().unwrap();
        let s = match conn.entry(ptr) {
            Entry::Vacant(e) => e.insert(HashSet::new()),
            Entry::Occupied(e) => e.into_mut()
        };

        if  s.contains(&iekey) {
            s.remove(&iekey);
        }
    }

    return result;
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
        if argc >= 3 &&  argc <= 4 && op.to_str().unwrap_or("") == "lock" {
            return true;
        }
    } else if cmd_cookie == &raw const IEK_UNLOCK_DESCRIPTOR as *const _ as *const c_void {
        if argc >= 2 && argc <= 3 && op.to_str().unwrap_or("") == "unlock" {
            return true;
        }
    } else if cmd_cookie == &raw const IEK_PUB_DESCRIPTOR as *const _ as *const c_void {
        if argc == 3 && op.to_str().unwrap_or("") == "publish" {
            return true;
        }
    } else if cmd_cookie == &raw const IEK_SUB_DESCRIPTOR as *const _ as *const c_void {
        if argc >= 2 && op.to_str().unwrap_or("") == "subscribe" {
            return true;
        }
    }
    return false;
}

#[allow(unused_variables)]
extern "C" fn execute_command(cmd_cookie: *const c_void, cookie: *const c_void,
    argc: c_int, argv: *mut token_t,
    response_handler: ResponseHandler) -> bool {
    let mut cur_token: usize = 1;
    let mut iekey = unsafe { CStr::from_ptr((*argv.add(cur_token)).value) }.to_string_lossy().into_owned();
    let mut result  = "ERROR unknown command\r\n".to_string();
    cur_token += 1;

    let mut cmd_type = CMDType::Unknown;
    if cmd_cookie == &raw const IEK_LOCK_DESCRIPTOR as *const _ as *const c_void {
        cmd_type = CMDType::Lock;
    } else if cmd_cookie == &raw const IEK_UNLOCK_DESCRIPTOR as *const _ as *const c_void {
        cmd_type = CMDType::Unlock;
    } else if cmd_cookie == &raw const IEK_PUB_DESCRIPTOR as *const _ as *const c_void {
        cmd_type = CMDType::Publish;
    } else if cmd_cookie == &raw const IEK_SUB_DESCRIPTOR as *const _ as *const c_void {
        cmd_type = CMDType::Subscribe;
    }


    let mut sub_key: Option<i32> = None;
    let mut is_success = true;
    if (cmd_type == CMDType::Lock && (argc - cur_token as c_int) == 2) ||
       (cmd_type == CMDType::Unlock && (argc - cur_token as c_int) == 1) {
        let str_sub_key = unsafe { CStr::from_ptr((*argv.add(cur_token)).value) };
        let cvt_sub_key: Result<i32, _> = str_sub_key.to_str().unwrap_or("").parse();
        cur_token += 1;
        match cvt_sub_key {
            Ok(num) => sub_key = Some(num),
            Err(e) => {
                is_success = false;
                result = "CLIENT_ERROR bad command line format\r\n".to_string();
            }
        }
    }

    if is_success {
        match cmd_type {
            CMDType::Lock => {
                let str_time = unsafe { CStr::from_ptr((*argv.add(cur_token)).value) };
                let lease_time: Result<f64, _> = str_time.to_str().unwrap_or("").parse();
                // cur_token += 1;
                match lease_time {
                    Ok(num) =>
                        result = process_lock_command(cookie, iekey, sub_key, num),
                    Err(e) =>
                        result = "CLIENT_ERROR bad command line format\r\n".to_string(),
                }
            },
            CMDType::Unlock => {
                result = process_unlock_command(cookie, iekey, sub_key);
            },
            CMDType::Publish => {
                let msg = unsafe { CStr::from_ptr((*argv.add(cur_token)).value) }.to_string_lossy().into_owned();
                // cur_token += 1;
                result = process_publish_command(cookie, iekey, msg);
            },
            CMDType::Subscribe => {
                result = format!("SUBSCRIBE {}\r\n", argc - 1);
                loop {
                    result += &process_subscribe_command(cookie, iekey.clone());
                    if cur_token < argc as usize {
                        iekey = unsafe { CStr::from_ptr((*argv.add(cur_token)).value) }.to_string_lossy().into_owned();
                        cur_token += 1;
                    } else {
                        break;
                    }
                }
                result += "END\r\n";
            },
            _ => ()

        }
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

    if unsafe { !((*extension).register_extension)(
                    extension_type_t::EXTENSION_ASCII_PROTOCOL,
                    &raw mut IEK_PUB_DESCRIPTOR as *mut c_void) }
    {
        return EXTENSION_ERROR_CODE::EXTENSION_FATAL;
    }

    if unsafe { !((*extension).register_extension)(
                    extension_type_t::EXTENSION_ASCII_PROTOCOL,
                    &raw mut IEK_SUB_DESCRIPTOR as *mut c_void) }
    {
        return EXTENSION_ERROR_CODE::EXTENSION_FATAL;
    }

    let cb = unsafe { (*SERVER_API).callback };
    unsafe { ((*cb).register_callback)(ptr::null_mut(), callback::ENGINE_EVENT_TYPE::ON_DISCONNECT,
                                       unsubscribe_all, ptr::null()) };

    return EXTENSION_ERROR_CODE::EXTENSION_SUCCESS;

}