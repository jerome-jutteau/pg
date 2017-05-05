use std::collections::HashMap;
use brick::Brick;
use error::Error;

pub struct Graph {
    pub name: String,
    pub bricks: HashMap<String, Brick>,
}

impl Graph {
    pub fn new<S: Into<String>>(name: S) -> Graph {
        Graph {
            name: name.into(),
            bricks: HashMap::new(),
        }
   }

    pub fn poll(&mut self) -> Vec<Result<usize, Error>> {
        self.bricks.values_mut()
            .filter(|b| b.pollable())
            .map(|b| b.poll())
            .collect::<Vec<Result<usize, Error>>>()
    }

    pub fn add(&mut self, brick: Brick) -> &mut Graph {
        self.bricks.insert(brick.name(), brick);
        return self
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;
    use super::super::init;
    use super::super::Side;
    use super::super::brick::Brick;
    use super::super::nop::Nop;
    use super::super::tap::Tap;
    use super::super::firewall::Firewall;

    #[test]
    fn add_poll() {
        init();
        let mut tap1 = Brick::Tap(Tap::new("tap1"));
        let mut nop = Brick::Nop(Nop::new("nop"));
        let mut tap2 = Brick::Tap(Tap::new("tap2"));
        tap1.link(&mut nop).unwrap();
        nop.link(&mut tap2).unwrap();
        let mut g = Graph::new("graph");
        g.add(tap1).add(nop).add(tap2);
        assert_eq!(g.bricks.len(), 3);
		assert_eq!(g.poll().len(), 2);
    }
    
	#[test]
    fn get_brick() {
        init();
        let mut tap1 = Brick::Tap(Tap::new("tap1"));
        let mut nop = Brick::Nop(Nop::new("nop"));
        let mut tap2 = Brick::Tap(Tap::new("tap2"));
        tap1.link(&mut nop).unwrap();
        nop.link(&mut tap2).unwrap();
        let mut g = Graph::new("graph");
        g.add(tap1).add(nop).add(tap2);
		g.bricks.get_mut("tap1").unwrap().poll().unwrap();
    }

	#[test]
    fn get_special_brick() {
        init();
		let mut g = Graph::new("graph");
        g.add(Brick::Firewall(Firewall::new("fw")));
		let firewall = g.bricks.get_mut("fw").unwrap().firewall().unwrap();
		firewall.rule_add("src host 10::2", Side::WEST).unwrap();
		firewall.reload().unwrap();
    }
}
