use brick::{Brick, self};
use error::Error;
use std::ffi::CString;
use std::ptr;
use packetgraph_sys::{pg_brick,
					  pg_brick_destroy,
                      pg_tap_new};

pub struct Tap {
    brick: *mut pg_brick,
    name: String,
}

impl Brick for Tap {
    fn brick(&self) -> *mut pg_brick {
        self.brick
    }

	fn get_type(&self) -> brick::Type {
		brick::Type::TAP
	}

    fn pollable(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        self.name.clone()
    }
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
}

impl Drop for Tap {
    fn drop(&mut self) {
        unsafe {
			pg_brick_destroy(self.brick);
		}
    }
}
