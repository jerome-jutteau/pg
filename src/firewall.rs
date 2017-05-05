use brick::{Brick, self};
use super::Side;
use error::Error;
use std::ffi::CString;
use packetgraph_sys::{pg_brick,
                      pg_brick_destroy,
                      pg_firewall_new,
                      pg_firewall_rule_add,
                      pg_firewall_rule_flush,
                      pg_firewall_reload,
                      PG_NONE};

pub struct Firewall {
    brick: *mut pg_brick,
    name: String,
}

impl Brick for Firewall {
    fn brick(&self) -> *mut pg_brick {
        self.brick
    }

	fn get_type(&self) -> brick::Type {
		brick::Type::FIREWALL
	}

    fn pollable(&self) -> bool {
        false
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Firewall {
    pub fn new<S: Into<String>>(name: S) -> Firewall {
        let name = name.into();
        let cname = CString::new(name.as_str()).unwrap();
        let mut error = Error::new();
        unsafe {
            Firewall {
                brick: pg_firewall_new(cname.as_ptr(), PG_NONE as u64, &mut error.ptr),
                name: name,
            }
        }
    }

	pub fn rule_add<S: Into<String>>(&mut self, rule: S, side: Side) -> Result<(), Error> {
		let mut error = Error::new();
		let filter = CString::new(rule.into().as_str()).unwrap();
		unsafe {
			pg_firewall_rule_add(self.brick, filter.as_ptr(), side.into(), 1, &mut error.ptr);
		}

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
	}

	pub fn flush(&mut self) {
		unsafe {
			pg_firewall_rule_flush(self.brick);
		}
	}

	pub fn reload(&mut self) ->  Result<(), Error> {
		let mut error = Error::new();
		unsafe {
			pg_firewall_reload(self.brick, &mut error.ptr);
		}

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Side;

    #[test]
    fn add_flush_reload() {
        let mut fw = Firewall::new("fw");
		fw.rule_add("src host 10::1", Side::WEST).unwrap();
		fw.rule_add("src host 10::1", Side::WEST).unwrap();
		fw.rule_add("src host 10::2", Side::EAST).unwrap();
		assert!(fw.rule_add("invalid rule", Side::WEST).is_err());
		fw.flush();
		fw.flush();
		fw.rule_add("src host 10::1", Side::WEST).unwrap();
		fw.rule_add("src host 10::2", Side::EAST).unwrap();
        fw.reload().unwrap();
        fw.reload().unwrap();
		fw.flush();
        fw.reload().unwrap();
    }
}
