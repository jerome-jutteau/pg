use error::Error;
use packetgraph_sys::{pg_brick,
                      pg_brick_link,
                      pg_brick_unlink_edge,
                      pg_brick_unlink,
                      pg_brick_poll};
use nop::Nop;
use firewall::Firewall;
use tap::Tap;

pub enum Brick {
    Nop(Nop),
    Firewall(Firewall),
    Tap(Tap),
}

impl<'a> Brick {
    pub fn link(&mut self, east: &mut Brick) -> Result<(), Error> {
        let mut error = Error::new();
        let west = self.get_brick();
        let east = east.get_brick();
        unsafe {
            pg_brick_link(west, east, &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
                 false => Ok(()),
        }
    }

    pub fn unlink_from(&mut self, east: &mut Brick) -> Result<(), Error> {
        let mut error = Error::new();
        let west = self.get_brick();
        let east = east.get_brick();
        unsafe {
            pg_brick_unlink_edge(west, east, &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
                 false => Ok(()),
        }
    }

    pub fn unlink(&mut self) {
        let mut error = Error::new();
        let brick = self.get_brick();
        unsafe {
            pg_brick_unlink(brick, &mut error.ptr);
        }
        assert!(!error.is_set());
    }

    pub fn poll(&mut self) -> Result<usize, Error> {
        let mut error = Error::new();
        if !self.pollable() {
            error.set("Brick is not pollable");
            return Err(error);
        }

        let mut n: u16 = 0;
        let brick = self.get_brick();
        unsafe {
            pg_brick_poll(brick, &mut n, &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(n as usize),
        }
    }

    pub fn pollable(&self) -> bool {
        match *self {
            Brick::Firewall(_) => false,
            Brick::Nop(_) => false,
            Brick::Tap(_) => true,
        }
    }


    pub fn name(&self) -> String {
        match *self {
            Brick::Firewall(ref b) => b.name.clone(),
            Brick::Nop(ref b) => b.name.clone(),
            Brick::Tap(ref b) => b.name.clone(),
        }
    }

    fn get_brick(&mut self) -> *mut pg_brick {
        match *self {
            Brick::Firewall(ref b) => b.brick,
            Brick::Nop(ref b) => b.brick,
            Brick::Tap(ref b) => b.brick,
        }
    }

    pub fn firewall(&mut self) -> Option<&mut Firewall> {
        match *self {
            Brick::Firewall(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub fn nop(&mut self) -> Option<&mut Nop> {
        match *self {
            Brick::Nop(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub fn tap(&mut self) -> Option<&mut Tap> {
        match *self {
            Brick::Tap(ref mut b) => Some(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::init;
    use nop::Nop;
	use firewall::Firewall;
    use tap::Tap;

    #[test]
    fn link_unlink() {
        init();
        let mut tap1 = Brick::Tap(Tap::new("tap1"));
        let mut nop1 = Brick::Nop(Nop::new("nop1"));
        let mut nop2 = Brick::Nop(Nop::new("nop2"));
        let mut tap2 = Brick::Tap(Tap::new("tap2"));
        tap1.link(&mut nop1).unwrap();
        nop1.link(&mut nop2).unwrap();
        nop2.link(&mut tap2).unwrap();
        assert!(nop2.link(&mut tap2).is_err());
        assert!(nop1.unlink_from(&mut tap2).is_err());
        assert!(nop2.unlink_from(&mut nop1).is_err());
        nop1.unlink_from(&mut nop2).unwrap();
        assert!(nop1.unlink_from(&mut nop2).is_err());
        tap2.unlink();
    }

    #[test]
    fn poll() {
        init();
        let mut tap1 = Brick::Tap(Tap::new("tap1"));
        let mut nop1 = Brick::Nop(Nop::new("nop1"));
        let mut nop2 = Brick::Nop(Nop::new("nop2"));
        let mut tap2 = Brick::Tap(Tap::new("tap2"));

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

    #[test]
    fn name() {
        init();
        let nop = Nop::new("noppy");
        assert_eq!(nop.name, String::from("noppy"));
    }

	#[test]
	fn specialized() {
		init();
		let mut b = Brick::Firewall(Firewall::new("fw"));
		b.firewall().unwrap().flush();
	}
}
