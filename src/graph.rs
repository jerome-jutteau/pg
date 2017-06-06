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
        self.bricks
            .values_mut()
            .filter(|b| b.pollable())
            .map(|b| b.poll())
            .collect::<Vec<Result<usize, Error>>>()
    }

    pub fn add(&mut self, brick: Brick) -> &mut Graph {
        self.bricks.insert(brick.name(), brick);
        return self;
    }

    pub fn dot(&mut self) -> Result<String, Error> {
        match self.bricks.iter_mut().next() {
            Some((_, b)) => b.dot(),
            None => {
                let mut e = Error::new();
                e.set("no brick available in graph");
                return Err(e);
            }
        }
    }

    pub fn svg(&mut self) -> Result<String, Error> {
        match self.bricks.iter_mut().next() {
            Some((_, b)) => b.svg(),
            None => {
                let mut e = Error::new();
                e.set("no brick available in graph");
                return Err(e);
            }
        }
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
        firewall.rule_add("src host 10::2", Side::West).unwrap();
        firewall.reload().unwrap();
    }
}
