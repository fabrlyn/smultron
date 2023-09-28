use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, OutputPort};
use tokio::{select, spawn, sync::oneshot};
use tracing::warn;

use crate::service::{self, Service};

pub type Actor = ActorRef<Msg>;
pub type EventPort = Arc<OutputPort<Event>>;

#[derive(Clone)]
pub enum Event {
    Found(Service),
}

pub struct ServiceFinder;

pub struct Arguments {
    event_port: Option<EventPort>,
    name: service::Name,
    stop_after: Option<Duration>,
}

pub enum Msg {
    Found(Service),
    Stop,
}

pub struct State {
    event_port: Option<EventPort>,
    services: Vec<Service>,
    stop_sender: oneshot::Sender<()>,
}

impl State {
    fn find_by_service(&self, subject: &Service) -> Option<&Service> {
        self.services.iter().find(|service| *service == subject)
    }

    fn put_service(&mut self, service: Service) -> bool {
        if self.service_exists(&service) {
            return false;
        }

        self.services.push(service);
        true
    }

    fn service_exists(&self, service: &Service) -> bool {
        self.find_by_service(service).is_some()
    }

    fn emit_event(&self, event: Event) {
        if let Some(event_port) = self.event_port {
            event_port.send(event);
        }
    }
}

fn found(state: &mut State, service: Service) {
    if state.put_service(service.clone()) {
        state.emit_event(Event::Found(service))
    }
}

async fn start(service_finder: ActorRef<Msg>) -> oneshot::Sender<()> {
    let (tx, rx) = oneshot::channel();
    spawn(async {
        select! {
            result = rx => {
            }
        }
    });
    tx
}

fn stop(actor: Actor, state: &mut State) {
    // TODO: Fix this with a shared boolean instead?
    if let Err(e) = state.stop_sender.send(()) {
        warn!("Failed to send stop for finder task {:?}", e);
    }
    actor.stop(None);
}

#[async_trait]
impl ractor::Actor for ServiceFinder {
    type Arguments = Arguments;
    type Msg = Msg;
    type State = State;

    async fn pre_start(
        &self,
        actor: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            event_port: arguments.event_port,
            services: vec![],
            stop_sender: start(actor).await,
        })
    }

    async fn handle(
        &self,
        actor: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            Msg::Found(service) => found(state, service),
            Msg::Stop => stop(actor, state),
        }

        Ok(())
    }
}

impl From<service::Name> for Arguments {
    fn from(name: service::Name) -> Self {
        Self {
            event_port: None,
            stop_after: None,
            name,
        }
    }
}
