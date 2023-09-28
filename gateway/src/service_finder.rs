use std::{fmt, sync::Arc, time::Duration};

use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, OutputPort};
use tokio::{select, spawn, sync::oneshot};
use tracing::{info, warn};

use crate::service::{self, Service};

pub type Actor = ActorRef<Msg>;
pub type EventPort = Arc<OutputPort<Event>>;
pub type StopSender = Option<oneshot::Sender<()>>;

#[derive(Clone, Debug)]
pub enum Event {
    Found(Arc<Service>),
}

pub struct ServiceFinder;

pub struct Arguments {
    event_port: Option<EventPort>,
    name: service::Name,
    stop_after: Option<Duration>,
}

#[derive(Debug)]
pub enum Msg {
    Found(Service),
    Stop,
}

pub struct State {
    event_port: Option<EventPort>,
    services: Vec<Arc<Service>>,
    stop_sender: StopSender,
}

impl State {
    fn emit_event(&self, event: Event) {
        if let Some(event_port) = &self.event_port {
            info!("Emitting event {:?}", event);
            event_port.send(event);
        }
    }

    fn find_by_service(&self, subject: &Service) -> Option<&Arc<Service>> {
        self.services
            .iter()
            .find(|service| service.as_ref() == subject)
    }

    fn put_service(&mut self, service: Service) -> Option<Arc<Service>> {
        if self.service_exists(&service) {
            return None;
        }

        let service = Arc::new(service);
        self.services.push(service.clone());

        Some(service)
    }

    fn service_exists(&self, service: &Service) -> bool {
        self.find_by_service(service).is_some()
    }
}

fn found(state: &mut State, service: Service) {
    if let Some(service) = state.put_service(service.clone()) {
        info!("Service found {:?}", service);
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
    info!("Stopping service finder");

    stop_discovery(state);

    actor.stop(None);
}

fn stop_discovery(state: &mut State) {
    info!("Stopping discovery");
    let Some(stop_sender) = state.stop_sender.take() else {
        info!("Discovery already stopped");
        return;
    };

    if let Err(e) = stop_sender.send(()) {
        warn!("Failed to send stop for discovery {:?}", e);
    }
    info!("Discovery stopped");
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
        info!("Starting service finder with arguments {:?}", arguments);

        Ok(Self::State {
            event_port: arguments.event_port,
            services: vec![],
            stop_sender: Some(start(actor).await),
        })
    }

    async fn handle(
        &self,
        actor: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling message {:?}", message);

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

impl fmt::Debug for Arguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arguments")
            .field("event_port", &self.event_port.is_some())
            .field("name", &self.name)
            .field("stop_after", &self.stop_after)
            .finish()
    }
}
