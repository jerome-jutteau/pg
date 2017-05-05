use error::Error;
use std::ffi::CString;
use std::ptr;
use packetgraph_sys::{pg_brick,
                      pg_brick_destroy,
                      pg_tap_new};

pub struct Tap {
    pub brick: *mut pg_brick,
    pub name: String,
}

impl Tap {
    pub fn new<S: Into<String>>(name: S) -> Tap {
        let name = name.into();
        let cname = CString::new(name.as_str()).unwrap();
        let mut error = Error::new();
        unsafe {
            Tap {
                brick: pg_tap_new(cname.as_ptr(), ptr::null(), &mut error.ptr),
                name: name,
            }
        }
    }

    pub fn pollable(&self) -> bool {
        true
    }
}

impl Drop for Tap {
    fn drop(&mut self) {
        unsafe {
            pg_brick_destroy(self.brick);
        }
    }
}
