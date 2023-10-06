use crate::service::Service;
use ractor::RpcReplyPort;
use std::{fmt, sync::Arc};

pub type BroadcastPort = Arc<ractor::OutputPort<Event>>;
pub type ReplyPort = RpcReplyPort<Arc<Service>>;

#[derive(Clone, Debug)]
pub enum Event {
    Found(Arc<Service>),
}

pub enum Port {
    Broadcast(BroadcastPort),
    Reply(ReplyPort),
}

impl fmt::Debug for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Port::Broadcast(_) => write!(f, "Broadcast"),
            Port::Reply(_) => write!(f, "Reply"),
        }
    }
}
