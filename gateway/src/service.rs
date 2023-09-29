use std::time::Instant;

pub type Name = String;
pub type Port = u16;
pub type SocketAddress = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Service {
    pub found_at: Instant,
    pub name: String,
    pub port: Port,
    pub target: String,
}

impl Service {
    pub fn socket_address(&self) -> SocketAddress {
        format!("{}:{}", self.target, self.port)
    }
}
