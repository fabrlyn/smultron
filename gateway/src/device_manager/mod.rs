/*
TODO:

- start discovery for devices
- handle discovered device
- scheduled discovery
- publish discovered device
- handle stopped device, publish events to hub
*/

use std::error::Error;
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use ractor::{ActorCell, ActorProcessingErr, ActorRef, OutputPort, SupervisionEvent};
use tracing::info;

use crate::{
    service::Service,
    service_finder::{self, ServiceFinder},
};

pub const DEVICE_SERVICE_NAME: &str = "_coap._udp.local";

pub type Actor = ActorRef<Msg>;

pub type EventPort = Arc<OutputPort<Event>>;

#[derive(Clone, Debug)]
pub enum Event {
    Found(Arc<Service>),
}

pub struct Arguments {
    event_port: Option<EventPort>,
}

pub enum Msg {
    Found(Arc<Service>),
    Search,
}

pub struct State {
    service_finder: Option<service_finder::Actor>,
    event_port: Option<EventPort>,
    devices: Vec<Arc<Service>>,
}

impl State {
    fn has_service(&self, service: &Service) -> bool {
        self.devices.iter().any(|s| s.name == service.name)
    }

    fn put_service(&mut self, service: Arc<Service>) -> Option<Arc<Service>> {
        if self.has_service(&service) {
            return None;
        }

        self.devices.push(service.clone());
        Some(service)
    }
}

pub struct DeviceManager;

impl DeviceManager {
    async fn pre_start(actor: Actor, arguments: Arguments) -> Result<State, ActorProcessingErr> {
        let mut state = State {
            service_finder: None,
            event_port: arguments.event_port,
            devices: vec![],
        };

        Self::search(actor, &mut state).await;

        Ok(state)
    }

    async fn found(state: &mut State, service: Arc<Service>) {
        let Some(service) = state.put_service(service) else {
            return;
        };

        let Some(event_port) = &state.event_port else {
            return;
        };

        event_port.send(Event::Found(service));
    }

    async fn handle(actor: Actor, state: &mut State, msg: Msg) -> Result<(), ActorProcessingErr> {
        match msg {
            Msg::Found(service) => Self::found(state, service).await,
            Msg::Search => Self::search(actor, state).await,
        }
        Ok(())
    }

    async fn search(actor: Actor, state: &mut State) {
        if let Some(_) = state.service_finder {
            return;
        }

        let port: service_finder::BroadcastPort = Default::default();

        port.subscribe(actor.clone(), |event| match event {
            service_finder::Event::Found(service) => Some(Msg::Found(service)),
        });

        let (service_finder, _) = ractor::Actor::spawn_linked(
            Some("device_manager_service_finder".to_owned()),
            ServiceFinder,
            service_finder::Arguments {
                name: DEVICE_SERVICE_NAME.to_owned(),
                port: Some(service_finder::Port::Broadcast(port)),
                interval: Some(Duration::from_secs(12)),
                timeout: Some(Duration::from_secs(30)),
            },
            actor.into(),
        )
        .await
        .unwrap();

        state.service_finder = Some(service_finder);
    }

    fn service_finder_panicked(actor: Actor, state: &mut State) {
        Self::schedule_service_finder(actor, state);
    }

    fn service_finder_terminated(actor: Actor, state: &mut State) {
        Self::schedule_service_finder(actor, state)
    }

    fn schedule_service_finder(actor: Actor, state: &mut State) {
        if let Some(service_finder) = &state.service_finder {
            service_finder.stop(None);
            state.service_finder = None;
        }

        actor.send_after(Duration::from_secs(60 * 5), || Msg::Search);
    }

    async fn link_panicked(
        actor: Actor,
        state: &mut State,
        link: ActorCell,
        _: Box<dyn Error + Send + Sync>,
    ) {
        if let Some(service_finder) = &state.service_finder {
            if service_finder.get_id() == link.get_id() {
                Self::service_finder_panicked(actor, state);
            }
        }
    }

    async fn handle_supervisor_evt(
        actor: Actor,
        state: &mut State,
        event: SupervisionEvent,
    ) -> Result<(), ActorProcessingErr> {
        use SupervisionEvent::*;
        match event {
            ActorTerminated(_, _, _) => Self::service_finder_terminated(actor, state),
            ActorPanicked(link, error) => Self::link_panicked(actor, state, link, error).await,
            event => {
                info!("Other supervisor event: {:?}", event);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ractor::Actor for DeviceManager {
    type Msg = Msg;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        actor: Actor,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Self::pre_start(actor, arguments).await
    }

    async fn handle(
        &self,
        actor: Actor,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Self::handle(actor, state, msg).await
    }

    async fn handle_supervisor_evt(
        &self,
        actor: Actor,
        event: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Self::handle_supervisor_evt(actor, state, event).await
    }
}
