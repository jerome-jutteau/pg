use brick::{Brick, self};
use error::Error;
use std::ffi::CString;
use packetgraph_sys::{pg_brick,
					  pg_brick_destroy,
                      pg_nop_new};

pub struct Nop {
    brick: *mut pg_brick,
    name: String,
}

impl Brick for Nop {
    fn brick(&self) -> *mut pg_brick {
        self.brick
    }

	fn get_type(&self) -> brick::Type {
		brick::Type::NOP
	}

    fn pollable(&self) -> bool {
        false
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Nop {
    pub fn new<S: Into<String>>(name: S) -> Nop {
        let name = name.into();
        let cname = CString::new(name.as_str()).unwrap();
        let mut error = Error::new();
        unsafe {
            Nop {
                brick: pg_nop_new(cname.as_ptr(), &mut error.ptr),
                name: name,
            }
        }
    }
}

impl Drop for Nop {
    fn drop(&mut self) {
        unsafe {
			pg_brick_destroy(self.brick);
		}
    }
}
