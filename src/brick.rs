use error::Error;
use super::Side;
use std::ffi::CString;
use packetgraph_sys::{pg_brick,
                      pg_brick_link,
                      pg_brick_unlink_edge,
                      pg_brick_unlink,
                      pg_brick_poll};
use packetgraph_sys::{pg_firewall_rule_add,
                      pg_firewall_rule_flush,
                      pg_firewall_reload};

#[derive(PartialEq, Debug)]
pub enum Type {
	NOP,
	FIREWALL,
	TAP,
}

pub trait Brick {
    fn brick(&self) -> *mut pg_brick;
	fn get_type(&self) -> Type;
    fn pollable(&self) -> bool;
    // TODO: maybe return a Cow instead ?
    fn name(&self) -> String;

    fn poll(&mut self) -> Result<usize, Error> {
        let mut error = Error::new();
        if !self.pollable() {
            error.set("Brick is not pollable");
            return Err(error);
        }
    
        let mut n: u16 = 0;
        unsafe {
            pg_brick_poll(self.brick(), &mut n, &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(n as usize),
        }
    }

    fn unlink(&mut self) {
        let mut error = Error::new();
        unsafe {
            pg_brick_unlink(self.brick(), &mut error.ptr);
        }
        assert!(!error.is_set());
    }

    // TODO if we can downcast Brick trait to a specific brick, we can avoid this

	fn fw_rule_add(&mut self, rule: &str, side: Side) -> Result<(), Error> {
		let mut error = Error::new();
		match self.get_type() {
			Type::FIREWALL => {},
			_ => {
				error.set(format!("Brick {} is not a firewall", self.name().as_str()));
				return Err(error);
			},
		};

		let filter = CString::new(rule).unwrap();
		unsafe {
			pg_firewall_rule_add(self.brick(), filter.as_ptr(), side.into(), 1, &mut error.ptr);
		}

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
	}

	fn fw_flush(&mut self) -> Result<(), Error> {
		let mut error = Error::new();
		match self.get_type() {
			Type::FIREWALL => {},
			_ => {
				error.set(format!("Brick {} is not a firewall", self.name().as_str()));
				return Err(error);
			},
		};

		unsafe {
			pg_firewall_rule_flush(self.brick());
		}
        Ok(())
	}

	fn fw_reload(&mut self) ->  Result<(), Error> {
		let mut error = Error::new();
		match self.get_type() {
			Type::FIREWALL => {},
			_ => {
				error.set(format!("Brick {} is not a firewall", self.name().as_str()));
				return Err(error);
			},
		};

		unsafe {
			pg_firewall_reload(self.brick(), &mut error.ptr);
		}

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
	}
}

pub fn link<W: Brick, E: Brick>(west: &mut W, east: &mut E) -> Result<(), Error> {
    let mut error = Error::new();
    unsafe {
        pg_brick_link(west.brick(), east.brick(), &mut error.ptr);
    }

    match error.is_set() {
        true => Err(error),
             false => Ok(()),
    }
}

pub fn unlink<W: Brick, E: Brick>(west: &mut W, east: &mut E) -> Result<(), Error> {
    let mut error = Error::new();
    unsafe {
        pg_brick_unlink_edge(west.brick(), east.brick(), &mut error.ptr);
    }

    match error.is_set() {
        true => Err(error),
             false => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::init;
	use nop::Nop;
	use tap::Tap;

    #[test]
    fn link_unlink() {
        init();
		let mut tap1 = Tap::new("tap1");
		let mut nop1 = Nop::new("nop1");
		let mut nop2 = Nop::new("nop2");
		let mut tap2 = Tap::new("tap2");
        link(&mut tap1, &mut nop1).unwrap();
        link(&mut nop1, &mut nop2).unwrap();
        link(&mut nop2, &mut tap2).unwrap();
        assert!(link(&mut nop2, &mut tap2).is_err());
        assert!(unlink(&mut nop1, &mut tap2).is_err());
        assert!(unlink(&mut nop2, &mut nop1).is_err());
        unlink(&mut nop1, &mut nop2).unwrap();
        assert!(unlink(&mut nop1, &mut nop2).is_err());
        tap2.unlink();
    }

    #[test]
    fn poll() {
        init();
		let mut tap1 = Tap::new("tap1");
		let mut nop1 = Nop::new("nop1");
		let mut nop2 = Nop::new("nop2");
		let mut tap2 = Tap::new("tap2");
        link(&mut tap1, &mut nop1).unwrap();
        link(&mut nop1, &mut nop2).unwrap();
        link(&mut nop2, &mut tap2).unwrap();
        assert!(tap1.pollable());
        assert!(tap2.pollable());
        assert!(!nop1.pollable());
        assert!(!nop2.pollable());
        tap1.poll().unwrap();
        tap2.poll().unwrap();
    }

    #[test]
    fn name() {
        init();
		let nop = Nop::new("noppy");
        assert_eq!(nop.name(), String::from("noppy"));
    }
}

