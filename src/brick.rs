use packetgraph_sys::pg_brick;

pub enum Type {
	NOP,
	FIREWALL,
	TAP,
}

pub trait Brick {
    fn get_brick(&self) -> *mut pg_brick;
	fn get_type(&self) -> Type;
}
