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
    fn brick(&self) -> *mut pg_brick {
        self.brick
    }

	fn get_type(&self) -> brick::Type {
		brick::Type::NOP
	}

    fn pollable(&self) -> bool {
        false
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
