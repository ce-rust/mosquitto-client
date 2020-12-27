
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::ffi::CStr;
use std::os::raw::{c_int, c_void};


pub const MOSQ_ERR_CONN_PENDING:i32 = -1;
pub const MOSQ_ERR_SUCCESS:i32 = 0;
pub const MOSQ_ERR_NOMEM:i32 = 1;
pub const MOSQ_ERR_PROTOCOL:i32 = 2;
pub const MOSQ_ERR_INVAL:i32 = 3;
pub const MOSQ_ERR_NO_CONN:i32 = 4;
pub const MOSQ_ERR_CONN_REFUSED:i32 = 5;
pub const MOSQ_ERR_NOT_FOUND:i32 = 6;
pub const MOSQ_ERR_CONN_LOST:i32 = 7;
pub const MOSQ_ERR_TLS:i32 = 8;
pub const MOSQ_ERR_PAYLOAD_SIZE:i32 = 9;
pub const MOSQ_ERR_NOT_SUPPORTED:i32 = 10;
pub const MOSQ_ERR_AUTH:i32 = 11;
pub const MOSQ_ERR_ACL_DENIED:i32 = 12;
pub const MOSQ_ERR_UNKNOWN:i32 = 13;
pub const MOSQ_ERR_ERRNO:i32 = 14;
pub const MOSQ_ERR_EAI:i32 = 15;
pub const MOSQ_ERR_PROXY:i32 = 16;

// extended error
pub const MOSQ_ERR_TIMEOUT:i32 = 16;

pub const MOSQ_CONNECT_ERR_OK:i32 = 0;
pub const MOSQ_CONNECT_ERR_PROTOCOL:i32 = 1;
pub const MOSQ_CONNECT_ERR_BADID:i32 = 2;
pub const MOSQ_CONNECT_ERR_NOBROKER:i32 = 3;

// extended error
pub const MOSQ_CONNECT_ERR_TIMEOUT:i32 = 256;

pub type Mosq = mosquitto;
pub type Data = c_void;

pub type Message = mosquitto_message;

pub fn mosq_strerror(rc: c_int) -> String {
    unsafe {
        if rc == MOSQ_ERR_TIMEOUT {
            return "timeout".into();
        }
        let errs = mosquitto_strerror(rc);
        CStr::from_ptr(errs).to_str().unwrap().to_string()
    }
}

pub fn connect_error(rc: i32) -> &'static str {
    match rc {
    MOSQ_CONNECT_ERR_OK => "connect: ok",
    MOSQ_CONNECT_ERR_PROTOCOL => "connect: bad protocol version",
    MOSQ_CONNECT_ERR_BADID => "connect: id rejected",
    MOSQ_CONNECT_ERR_NOBROKER => "connect: broker unavailable",
    MOSQ_CONNECT_ERR_TIMEOUT => "connect: timed out",
    _ => "connect: unknown"
    }
}


