use brick::{Brick, self};
use error::Error;
use std::ffi::CString;
use packetgraph_sys::{pg_brick,
					  pg_brick_destroy,
                      pg_nop_new};

pub struct Nop {
    brick: *mut pg_brick,
}

impl Brick for Nop {
    fn get_brick(&self) -> *mut pg_brick {
        self.brick
    }

	fn get_type(&self) -> brick::Type {
		brick::Type::NOP
	}
}

impl Nop {
    pub fn new(name: &str) -> Nop {
        let name = CString::new(name).unwrap();
        let mut error = Error::new();
        unsafe {
            Nop {
                brick: pg_nop_new(name.as_ptr(), &mut error.ptr),
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