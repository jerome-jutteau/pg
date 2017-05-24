use brick::{Brick, self};
use error::Error;
use std::ffi::CString;
use std::ptr;
use packetgraph_sys::{pg_brick,
					  pg_brick_destroy,
                      pg_tap_new};

pub struct Tap {
    brick: *mut pg_brick,
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

    fn link<B: Brick>(&mut self, east: &mut B) -> Result<(), Error> {
        brick::link(&mut self, &mut east)
    }

    fn unlink<B: Brick>(&mut self, east: &mut B) -> Result<(), Error> {
        brick::unlink(&mut self, &mut east)
    }

    fn unlink_all(&mut self) {
        brick::unlink_all(&mut self)
    }

    fn poll(&mut self)  -> Result<usize, Error> {
        brick::poll(&mut self)
    }
}

impl Tap {
    pub fn new(name: &str) -> Tap {
        let name = CString::new(name).unwrap();
        let mut error = Error::new();
        unsafe {
            Tap {
                brick: pg_tap_new(name.as_ptr(),
                                  ptr::null(),
                                  &mut error.ptr),
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
