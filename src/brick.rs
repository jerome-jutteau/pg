/* Copyright 2017 Outscale SAS
 *
 * This file is part of Pg, a Rust Wrapper for packetgraph C library.
 *
 * Pg is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 3 as published
 * by the Free Software Foundation.
 *
 * Packetgraph is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Packetgraph.  If not, see <http://www.gnu.org/licenses/>.
 */

use error::Error;
use packetgraph_sys::{pg_brick, pg_brick_link, pg_brick_unlink_edge, pg_brick_unlink,
                      pg_brick_poll, pg_brick_dot_mem};
use nop::Nop;
use firewall::Firewall;
use tap::Tap;
use switch::Switch;
use nic::Nic;
use hub::Hub;
use vhost::Vhost;
use std::process::{Command, Stdio};
use std::error::Error as StdErr;
use std::io::prelude::*;

// Maybe use a better wrapper of pg_brick raw pointer
unsafe impl Send for Brick {}
unsafe impl Sync for Brick {}

pub enum Brick {
    Nop(Nop),
    Firewall(Firewall),
    Tap(Tap),
    Switch(Switch),
    Nic(Nic),
    Hub(Hub),
    Vhost(Vhost),
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
            Brick::Firewall(ref b) => b.pollable(),
            Brick::Nop(ref b) => b.pollable(),
            Brick::Tap(ref b) => b.pollable(),
            Brick::Switch(ref b) => b.pollable(),
            Brick::Nic(ref b) => b.pollable(),
            Brick::Hub(ref b) => b.pollable(),
            Brick::Vhost(ref b) => b.pollable(),
        }
    }

    pub fn name(&self) -> String {
        match *self {
            Brick::Firewall(ref b) => b.name.clone(),
            Brick::Nop(ref b) => b.name.clone(),
            Brick::Tap(ref b) => b.name.clone(),
            Brick::Switch(ref b) => b.name.clone(),
            Brick::Nic(ref b) => b.name.clone(),
            Brick::Hub(ref b) => b.name.clone(),
            Brick::Vhost(ref b) => b.name.clone(),
        }
    }

    pub fn type_str(&self) -> &'static str {
        match *self {
            Brick::Firewall(_) => "firewall",
            Brick::Nop(_) => "nop",
            Brick::Tap(_) => "tap",
            Brick::Switch(_) => "switch",
            Brick::Nic(_) => "nic",
            Brick::Hub(_) => "hub",
            Brick::Vhost(_) => "vhost",
        }
    }

    fn get_brick(&mut self) -> *mut pg_brick {
        match *self {
            Brick::Firewall(ref b) => b.brick,
            Brick::Nop(ref b) => b.brick,
            Brick::Tap(ref b) => b.brick,
            Brick::Switch(ref b) => b.brick,
            Brick::Nic(ref b) => b.brick,
            Brick::Hub(ref b) => b.brick,
            Brick::Vhost(ref b) => b.brick,
        }
    }

    pub fn dot(&mut self) -> Result<String, Error> {
        let mut error = Error::new();
        let mut v = vec![0u8; 1_000_000];
        unsafe {
            if pg_brick_dot_mem(self.get_brick(),
                                v.as_mut_ptr() as *mut ::std::os::raw::c_char,
                                1_000_000,
                                &mut error.ptr) < 0 {
                return Err(error);
            }
        }
        let mut s = String::new();
        for c in v.iter() {
            if *c == 0 {
                break;
            } else {
                s.push(*c as char);
            }
        }
        Ok(s)
    }

    pub fn svg(&mut self) -> Result<String, Error> {
        let dot = match self.dot() {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        let mut err = Error::new();
        let process = match Command::new("dot")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-Tsvg")
            .spawn() {
                Err(e) => {
                    err.set(format!("cannot spawn dot: {}", e.description()));
                    return Err(err);
                },
                Ok(p) => p,
            };

        match process.stdin.unwrap().write_all(dot.as_bytes()) {
            Err(e) => {
                err.set(format!("cannot write dot stdin: {}", e.description()));
                return Err(err);
            },
            Ok(_) => {},
        };

        let mut out = String::new();
        match process.stdout.unwrap().read_to_string(&mut out) {
             Err(e) => {
                err.set(format!("cannot read dot stdout: {}", e.description()));
                return Err(err);
            },
            Ok(_) => {},
        }

        return Ok(out);
    }

    // TODO: use macro ?
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

    pub fn switch(&mut self) -> Option<&mut Switch> {
        match *self {
            Brick::Switch(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub fn nic(&mut self) -> Option<&mut Nic> {
        match *self {
            Brick::Nic(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub fn hub(&mut self) -> Option<&mut Hub> {
        match *self {
            Brick::Hub(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub fn vhost(&mut self) -> Option<&mut Vhost> {
        match *self {
            Brick::Vhost(ref mut b) => Some(b),
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
        tap1.dot().unwrap();
        tap1.svg().unwrap();
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
