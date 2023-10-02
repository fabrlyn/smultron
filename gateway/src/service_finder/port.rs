use std::{fmt, sync::Arc};

use ractor::RpcReplyPort;

use crate::service::Service;

pub type BroadcastPort = Arc<ractor::OutputPort<Event>>;
pub type ReplyPort = RpcReplyPort<Arc<Service>>;

#[derive(Clone, Debug)]
pub enum Event {
    Found(Arc<Service>),
}

pub enum Port {
    Broadcast(BroadcastPort),
    Reply(Option<ReplyPort>),
}

impl fmt::Debug for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Port::Broadcast(_) => write!(f, "Broadcast"),
            Port::Reply(Some(_)) => write!(f, "Reply(Some)"),
            Port::Reply(None) => write!(f, "Reply(None)"),
        }
    }
}
