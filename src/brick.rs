use error::Error;
use packetgraph_sys::{pg_brick,
                      pg_brick_link,
                      pg_brick_unlink_edge,
                      pg_brick_unlink,
                      pg_brick_poll};

pub enum Type {
	NOP,
	FIREWALL,
	TAP,
}

pub trait Brick {
    fn brick(&self) -> *mut pg_brick;
	fn get_type(&self) -> Type;
    fn pollable(&self) -> bool;

/*
    fn link<B: Brick>(&mut self, east: &mut B) -> Result<(), Error> {
        let mut error = Error::new();
        unsafe {
            pg_brick_link(self.brick(), east.brick(), &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
    }

    fn unlink<B: Brick>(&mut self, east: &mut B) -> Result<(), Error> {
        let mut error = Error::new();
        unsafe {
            pg_brick_unlink_edge(self.brick(), east.brick(), &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
    }

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

    fn unlink_all(&mut self) {
        let mut error = Error::new();
        unsafe {
            pg_brick_unlink(self.brick(), &mut error.ptr);
        }
        assert!(!error.is_set());
    }
*/

    fn link<B: Brick>(&mut self, east: &mut B) -> Result<(), Error>;
    fn unlink<B: Brick>(&mut self, east: &mut B) -> Result<(), Error>;
    fn unlink_all(&mut self);
    fn poll(&mut self)  -> Result<usize, Error>;
}


pub    fn link<B: Brick>(west: &mut B, east: &mut B) -> Result<(), Error> {
        let mut error = Error::new();
        unsafe {
            pg_brick_link(west.brick(), east.brick(), &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
    }

pub    fn unlink<B: Brick>(west: &mut B, east: &mut B) -> Result<(), Error> {
        let mut error = Error::new();
        unsafe {
            pg_brick_unlink_edge(self.brick(), east.brick(), &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
    }

pub    fn poll<B: Brick>(b: &mut B) -> Result<usize, Error> {
        let mut error = Error::new();
        if !b.pollable() {
            error.set("Brick is not pollable");
            return Err(error);
        }
    
        let mut n: u16 = 0;
        unsafe {
            pg_brick_poll(b.brick(), &mut n, &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(n as usize),
        }
    }

pub    fn unlink_all<B: Brick>(b: &mut B) {
        let mut error = Error::new();
        unsafe {
            pg_brick_unlink(b.brick(), &mut error.ptr);
        }
        assert!(!error.is_set());
    }

#[cfg(test)]
mod tests {
    use super::Brick;
    use super::super::init;
	use nop::Nop;
	use tap::Tap;
    use std::thread::sleep;
    use std::time;

    #[test]
    fn link_unlink() {
        init();
		let mut tap1 = Tap::new("tap1");
		let mut nop1 = Nop::new("nop1");
		let mut nop2 = Nop::new("nop2");
		let mut tap2 = Tap::new("tap2");
        tap1.link(&mut nop1).unwrap();
        nop1.link(&mut nop2).unwrap();
        nop2.link(&mut tap2).unwrap();
        assert!(nop2.link(&mut tap2).is_err());
        assert!(nop1.unlink(&mut tap2).is_err());
        assert!(nop2.unlink(&mut nop1).is_err());
        nop1.unlink(&mut nop2).unwrap();
        assert!(nop1.unlink(&mut nop2).is_err());
        tap2.unlink_all();
    }

    #[test]
    fn poll() {
        init();
		let mut tap1 = Tap::new("tap1");
		let mut nop1 = Nop::new("nop1");
		let mut nop2 = Nop::new("nop2");
		let mut tap2 = Tap::new("tap2");
        tap1.link(&mut nop1).unwrap();
        nop1.link(&mut nop2).unwrap();
        nop2.link(&mut tap2).unwrap();
        assert!(tap1.pollable());
        assert!(tap2.pollable());
        assert!(!nop1.pollable());
        assert!(!nop2.pollable());
        tap1.poll().unwrap();
        tap2.poll().unwrap();
    }
}

