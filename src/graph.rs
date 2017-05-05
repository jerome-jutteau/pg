use std::ffi::CString;
use std::collections::HashMap;
use brick::Brick;
use error::Error;
use std::ptr;
use packetgraph_sys::{pg_graph,
					  pg_graph_new,
					  pg_graph_destroy,
					  pg_graph_poll,
					  pg_graph_push,
                      pg_graph_unsafe_pop};

pub struct Graph<'a>{
    pub name: String,
    graph: *mut pg_graph,
	bricks: HashMap<String, Box<Brick + 'a>>,
}

impl<'a> Drop for Graph<'a> {
    fn drop(&mut self) {
		// TODO: use pg_graph_empty (once available)
		for (name, _) in self.bricks.iter() {
            let name = CString::new(name.as_str()).unwrap();
			unsafe { pg_graph_unsafe_pop(self.graph, name.as_ptr()); }
		}
        unsafe { pg_graph_destroy(self.graph) }
    }
}

impl<'a> Graph<'a> {
    pub fn new<S: Into<String>>(name: S) -> Graph<'a> {
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

	pub fn add<B: Brick + 'a>(&mut self, brick: B) -> Result<(), Error> {
        let mut error = Error::new();
        match unsafe {pg_graph_push(self.graph, brick.brick(), &mut error.ptr)} {
            0 => {},
            _ => return Err(error),
        };

		let b = Box::new(brick);
		self.bricks.insert(b.name(), b);
		Ok(())
	}

    pub fn get<S: Into<String>>(&mut self, brick_name: S) -> Option<&mut Box<Brick + 'a>> {
        self.bricks.get_mut(&brick_name.into())
    }

    pub fn pop<S: Into<String>>(&mut self, brick_name: S) -> Option<Box<Brick + 'a>> {
        let name = brick_name.into();
        let r = self.bricks.remove(&name);
        if r.is_some() {
            let name = CString::new(name.as_str()).unwrap();
            unsafe { pg_graph_unsafe_pop(self.graph, name.as_ptr()); }
        }
        return r;
    }

    pub fn len(&self) -> usize {
        self.bricks.len()
    }
}

#[cfg(test)]
mod tests {
	use super::Graph;
	use super::super::init;
    use super::super::Side;
    use super::super::brick;
    use super::super::nop::Nop;
    use super::super::tap::Tap;
    use super::super::firewall::Firewall;

    #[test]
    fn add_poll() {
		init();
		let mut g = Graph::new("graph");
        let mut tap1 = Tap::new("tap1");
        let mut nop = Nop::new("nop");
        let mut tap2 = Tap::new("tap2");
        brick::link(&mut tap1, &mut nop).unwrap();
        brick::link(&mut nop, &mut tap2).unwrap();
        g.add(tap1).unwrap();
        g.add(nop).unwrap();
        g.add(tap2).unwrap();
        g.poll().unwrap();
    }

    #[test]
    fn get() {
		init();
		let mut g = Graph::new("graph");
        let mut tap1 = Tap::new("tap1");
        let mut nop = Nop::new("nop");
        let mut tap2 = Tap::new("tap2");
        brick::link(&mut tap1, &mut nop).unwrap();
        brick::link(&mut nop, &mut tap2).unwrap();
        g.add(tap1).unwrap();
        g.add(nop).unwrap();
        g.add(tap2).unwrap();
        g.get("tap1").unwrap().poll().unwrap();
        g.get("tap1").unwrap().poll().unwrap();
        g.get("tap2").unwrap().poll().unwrap();
        assert_eq!(g.get("nop").unwrap().get_type(), brick::Type::NOP);
        assert_eq!(g.len(), 3);
        g.poll().unwrap();
    }


    #[test]
    fn pop() {
		init();
        let mut tap1 = Tap::new("tap1");
        let mut nop = Nop::new("nop");
        let mut tap2 = Tap::new("tap2");
        brick::link(&mut tap1, &mut nop).unwrap();
        brick::link(&mut nop, &mut tap2).unwrap();
		let mut g = Graph::new("graph");
        g.add(tap1).unwrap();
        g.add(nop).unwrap();
        g.add(tap2).unwrap();
        g.poll().unwrap();
        assert_eq!(g.len(), 3);
        let mut tap = g.pop("tap1").unwrap();
        assert_eq!(g.len(), 2);
        g.poll().unwrap();
        tap.poll().unwrap();
    }
    
    #[test]
    fn firewall_in_graph() {
        init();
        let mut tap1 = Tap::new("tap1");
        let mut fw = Firewall::new("fw");
        let mut tap2 = Tap::new("tap2");
        brick::link(&mut tap1, &mut fw).unwrap();
        brick::link(&mut fw, &mut tap2).unwrap();
		let mut g = Graph::new("graph");
        g.add(tap1).unwrap();
        g.add(fw).unwrap();
        g.add(tap2).unwrap();
        g.get("fw").unwrap().fw_rule_add("src host 10::1", Side::WEST).unwrap();
        g.get("fw").unwrap().fw_reload().unwrap();
        g.poll().unwrap();
    }
}
