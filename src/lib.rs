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
use std::sync::Mutex;
use packetgraph_sys::{pg_start_str, pg_side};

lazy_static! {
    static ref DPDK_OPTS: Mutex<String> = Mutex::new(String::from("-c1 -n1 --no-huge --no-shconf --lcores 0,1 -l 0,1"));
	static ref DPDK_OK: Mutex<bool> = Mutex::new(false);
}

pub fn set_dpdk_params<S: Into<String>>(params: S) {
	let mut s = DPDK_OPTS.lock().unwrap();
	*s = params.into();
}

pub fn init() {
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

