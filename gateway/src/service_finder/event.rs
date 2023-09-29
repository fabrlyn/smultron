use std::sync::Arc;

use ractor::OutputPort;

use crate::service::Service;

pub type EventPort = Arc<OutputPort<Event>>;

#[derive(Clone, Debug)]
pub enum Event {
    Found(Arc<Service>),
}
