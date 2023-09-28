use std::time::Instant;

pub type Name = String;
pub type Port = u16;
pub type SocketAddress = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Service {
    found_at: Instant,
    name: String,
    port: Port,
    target: String,
}

impl Service {
    pub fn socket_address(&self) -> SocketAddress {
        format!("{}:{}", self.target, self.port)
    }
}
