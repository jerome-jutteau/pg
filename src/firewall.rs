use brick::{Brick, self};
use error::Error;
use std::ffi::CString;
use packetgraph_sys::{pg_brick,
                      pg_brick_destroy,
                      pg_firewall_new,
                      PG_NONE};

pub struct Firewall {
    brick: *mut pg_brick,
}

impl Brick for Firewall {
    fn get_brick(&self) -> *mut pg_brick {
        self.brick
    }

	fn get_type(&self) -> brick::Type {
		brick::Type::FIREWALL
	}
}

impl Firewall {
    pub fn new(name: &str) -> Firewall {
        let name = CString::new(name).unwrap();
        let mut error = Error::new();
        unsafe {
            Firewall {
                brick: pg_firewall_new(name.as_ptr(), PG_NONE as u64, &mut error.ptr),
            }
        }
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        unsafe {
			pg_brick_destroy(self.brick);
		}
    }
}