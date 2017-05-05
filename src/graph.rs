/*pub struct Graph{
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
        match unsafe {pg_graph_push(self.graph, brick.brick(), &mut error.ptr)} {
            0 => Ok(()),
            _ => Err(error),
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
			pg_firewall_rule_add(b.brick(), filter.as_ptr(), side.into(), 1, &mut error.ptr);
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
			pg_firewall_rule_flush(b.brick());
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
*/
