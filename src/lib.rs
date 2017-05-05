extern crate packetgraph_sys;
#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod nop;
pub mod firewall;
pub mod tap;
pub mod brick;
pub mod graph;

use std::ffi::CString;
use brick::Brick;
use error::Error;
use std::ptr;
use std::sync::Mutex;
use std::boxed::Box;
use std::collections::HashMap;
use nop::Nop;
use firewall::Firewall;
use tap::Tap;
use std::convert::Into;
use packetgraph_sys::{pg_side,
					  pg_start_str,
				      pg_brick_link,
					  pg_brick_unlink_edge,
                      pg_graph,
                      pg_graph_new,
                      pg_graph_push,
                      pg_graph_poll,
					  pg_graph_unsafe_pop,
					  pg_firewall_rule_add,
					  pg_firewall_rule_flush,
					  pg_firewall_reload};

lazy_static! {
    static ref DPDK_OPTS: Mutex<String> = Mutex::new(String::from("-c1 -n1 --no-huge --no-shconf --lcores 0,1 -l 0,1"));
	static ref DPDK_OK: Mutex<bool> = Mutex::new(false);
}

pub fn set_dpdk_params<S: Into<String>>(params: S) {
	let mut s = DPDK_OPTS.lock().unwrap();
	*s = params.into();
}

fn init_pg() {
	let mut ok = DPDK_OK.lock().unwrap();
	if !*ok {
		let dpdk_opt = DPDK_OPTS.lock().unwrap();
		let params = CString::new(dpdk_opt.as_str()).unwrap();
		if unsafe { pg_start_str(params.as_ptr()) } < 0 {
			panic!("Cannot init packetgraph with dpdk parameters {}, adjust with set_dpdk_params()", params.into_string().unwrap());
		} else {
			*ok = true;
		}
	}
}

pub enum Side {
	WEST,
	EAST,
}

impl Into<pg_side> for Side {
	fn into(self) -> pg_side {
		match self {
			Side::WEST => pg_side::PG_WEST_SIDE,
			Side::EAST => pg_side::PG_EAST_SIDE,
		}
	}
}

pub struct Graph{
    pub name: String,
    graph: *mut pg_graph,
	bricks: HashMap<String, Box<Brick>>,
}

impl Drop for Graph {
    fn drop(&mut self) {
		// TODO: must empty graph before feeing pg_graph
		// We should use:
		// - pg_graph_empty (once available) followed by
        // - pg_graph_destroy
		for (n, _) in self.bricks.iter() {
			let cname = CString::new(n.as_str()).unwrap();
			unsafe { pg_graph_unsafe_pop(self.graph, cname.as_ptr()); }
		}
    }
}

impl Graph {
    pub fn new<S: Into<String>>(name: S) -> Graph {
		init_pg();
		let name = name.into();

        let cname = CString::new(name.as_str()).unwrap();
        let mut error = Error::new();
        Graph {
            name: name,
            graph: unsafe {pg_graph_new(cname.as_ptr(), ptr::null_mut(), &mut error.ptr)},
			bricks: HashMap::new(),
        }        
   }

    pub fn poll(&mut self) -> Result<(), Error> {
        let mut error = Error::new();
        match unsafe {pg_graph_poll(self.graph, &mut error.ptr)} {
            0 => Ok(()),
            _ => Err(error),
        }
    }

	fn pg_insert<B: Brick>(&mut self, brick: &Box<B>) -> Result<(), Error> {
        let mut error = Error::new();
        match unsafe {pg_graph_push(self.graph, brick.get_brick(), &mut error.ptr)} {
            0 => Ok(()),
            _ => Err(error),
        }
	}

	pub fn link<S: Into<String>>(&mut self, west: S, east: S) -> Result<(), Error> {
        let mut error = Error::new();
		let west = west.into();
		let east = east.into();
		let w = match self.bricks.get(west.as_str()) {
			None => {
				error.set(format!("West brick {} does not exists in graph", west.as_str()));
				return Err(error);
			},
			Some(b) => b,
		};

		let e = match self.bricks.get(east.as_str()) {
			None => {
				error.set(format!("East brick {} does not exists in graph", east.as_str()));
				return Err(error);
			},
			Some(b) => b,
		};

        unsafe {
            pg_brick_link(w.get_brick(), e.get_brick(), &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
	}

	pub fn unlink<S: Into<String>>(&mut self, west: S, east: S) -> Result<(), Error> {
        let mut error = Error::new();
		let west = west.into();
		let east = east.into();
		let w = match self.bricks.get(west.as_str()) {
			None => {
				error.set(format!("West brick {} does not exists in graph", west.as_str()));
				return Err(error);
			},
			Some(b) => b,
		};

		let e = match self.bricks.get(east.as_str()) {
			None => {
				error.set(format!("East brick {} does not exists in graph", east.as_str()));
				return Err(error);
			},
			Some(b) => b,
		};

        unsafe {
            pg_brick_unlink_edge(w.get_brick(), e.get_brick(), &mut error.ptr);
        }

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
	}

	pub fn nop_new<S: Into<String>>(&mut self, name: S) -> Result<(), Error> {
		let name = name.into();
		let b = Box::new(Nop::new(name.as_str()));
		self.pg_insert(&b)?;
		self.bricks.insert(name, b);
		Ok(())
	}

	pub fn fw_new<S: Into<String>>(&mut self, name: S) -> Result<(), Error> {
		let name = name.into();
		let b = Box::new(Firewall::new(name.as_str()));
		self.pg_insert(&b)?;
		self.bricks.insert(name, b);
		Ok(())
	}

	pub fn fw_rule_add<S: Into<String>>(&mut self, name: S, rule: S, side: Side) -> Result<(), Error> {
		let mut error = Error::new();
		let name = name.into();
		let b = match self.bricks.get(name.as_str()) {
			None => {
				error.set(format!("Brick {} does not exists in graph", name.as_str()));
				return Err(error)},
			Some(b) => b,
		};

		match b.get_type() {
			brick::Type::FIREWALL => {},
			_ => {
				error.set(format!("Brick {} is not a firewall", name.as_str()));
				return Err(error);
			},
		};

		let filter = CString::new(rule.into().as_str()).unwrap();
		unsafe {
			pg_firewall_rule_add(b.get_brick(), filter.as_ptr(), side.into(), 1, &mut error.ptr);
		}

        match error.is_set() {
            true => Err(error),
            false => Ok(()),
        }
	}

	pub fn fw_flush<S: Into<String>>(&mut self, name: S) -> Result<(), Error> {
		let mut error = Error::new();
		let name = name.into();
		let b = match self.bricks.get(name.as_str()) {
			None => {
				error.set(format!("Brick {} does not exists in graph", name.as_str()));
				return Err(error)},
			Some(b) => b,
		};

		match b.get_type() {
			brick::Type::FIREWALL => {},
			_ => {
				error.set(format!("Brick {} is not a firewall", name.as_str()));
				return Err(error);
			},
		};

		unsafe {
			pg_firewall_rule_flush(b.get_brick());
		}
		Ok(())
	}

	pub fn tap_new<S: Into<String>>(&mut self, name: S) -> Result<(), Error> {
		let name = name.into();
		let b = Box::new(Tap::new(name.as_str()));
		self.pg_insert(&b)?;
		self.bricks.insert(name, b);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::Graph;
	use super::Side;

    #[test]
    fn link_unlink() {
		let mut g = Graph::new("graph");
		g.nop_new("nop1").expect("nop1 creation ok");
		g.nop_new("nop2").expect("nop2 creation ok");
		assert!(g.nop_new("nop2").is_err());
		g.link("nop1", "nop2").expect("link [nop1]--[nop2] ok");
		assert!(g.link("?", "nop2").is_err());
		g.poll().expect("poll ok");
		g.unlink("nop1", "nop2").expect("unlink [nop1]--[nop2] ok");
		g.link("nop1", "nop2").expect("link [nop1]--[nop2] ok");
		assert!(g.unlink("nop2", "nop1").is_err());
		assert!(g.unlink("nop2", "nop2").is_err());
    }

    #[test]
    fn poll() {
		let mut g = Graph::new("graph");
		g.tap_new("tap1").unwrap();
		g.fw_new("fw").unwrap();
		g.fw_rule_add("fw", "src host 10::1", Side::WEST).unwrap();
		g.tap_new("tap2").unwrap();
		g.link("tap1", "fw").unwrap();
		g.link("fw", "tap2").unwrap();
		assert!(g.link("fw", "tap2").is_err());
		assert!(g.link("tap2", "fw").is_err());
		for _ in 0..100 {
			g.poll().expect("poll ok");
		}
    }

    #[test]
    fn firewall() {
		let mut g = Graph::new("graph");
		g.fw_new("fw").unwrap();
		g.nop_new("nop").unwrap();
		assert!(g.fw_rule_add("nop", "c host 10::1", Side::WEST).is_err());
		g.fw_rule_add("fw", "src host 10::1", Side::WEST).unwrap();
		g.fw_rule_add("fw", "src host 10::1", Side::WEST).unwrap();
		g.fw_rule_add("fw", "src host 10::2", Side::EAST).unwrap();
		assert!(g.fw_rule_add("invalid brick", "src host 10::2", Side::EAST).is_err());
		assert!(g.fw_rule_add("fw", "invalid rule", Side::WEST).is_err());
		g.fw_flush("fw").unwrap();
		g.fw_flush("fw").unwrap();
		assert!(g.fw_flush("nop").is_err());
    }
}

