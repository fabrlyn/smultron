use std::time::Instant;
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use ractor::{ActorCell, ActorProcessingErr, ActorRef, OutputPort, SupervisionEvent};
use tracing::info;

use crate::device::{self};
use crate::{
    service::Service,
    service_finder::{self, ServiceFinder},
};

pub const DEVICE_SERVICE_NAME: &str = "_coap._udp.local";

pub type Actor = ActorRef<Msg>;

pub type EventPort = Arc<OutputPort<Event>>;

#[derive(Clone)]
pub struct Device {
    pub service: Arc<Service>,
    pub event_port: device::EventPort,
    pub actor: device::Actor,
}

#[derive(Clone, Debug)]
pub enum Event {
    Found(Arc<Service>),
    Device(device::Event),
}

pub struct Arguments {
    pub event_port: Option<EventPort>,
}

pub enum Msg {
    Published(device::Event),
    Found(Arc<Service>),
    Search,
}

pub struct State {
    service_finder: Option<service_finder::Actor>,
    event_port: Option<EventPort>,
    devices: Vec<Device>,
}

impl State {
    fn has_service(&self, service: &Service) -> bool {
        self.devices
            .iter()
            .any(|device| device.service.name == service.name)
    }

    fn add_device(&mut self, device: Device) {
        self.devices.push(device.clone());
    }

    fn is_service_finder(&self, other: &ActorCell) -> bool {
        if let Some(service_finder) = &self.service_finder {
            service_finder.get_id() == other.get_id()
        } else {
            return false;
        }
    }

    fn remove_by_actor(&mut self, actor: &ActorCell) -> Option<Device> {
        let device_position = self
            .devices
            .iter()
            .position(|device| device.actor.get_id() == actor.get_id())?;

        Some(self.devices.swap_remove(device_position))
    }

    fn send(&self, event: Event) {
        if let Some(event_port) = &self.event_port {
            event_port.send(event);
        }
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

    async fn found(actor: Actor, state: &mut State, service: Arc<Service>) {
        if state.has_service(&service) {
            return;
        }

        let event_port: device::EventPort = Default::default();
        event_port.subscribe(actor.clone(), move |event| Some(Msg::Published(event)));

        let device_name = Some(format!("device_{}", &service.target));
        let arguments = device::Arguments {
            event_port: event_port.clone(),
            service: service.clone(),
        };

        let (device_actor, _) =
            ractor::Actor::spawn_linked(device_name, device::Device, arguments, actor.into())
                .await
                .unwrap();

        state.add_device(Device {
            service: service.clone(),
            event_port,
            actor: device_actor,
        });

        state.send(Event::Found(service));
    }

    fn published(state: &mut State, event: device::Event) {
        state.send(Event::Device(event))
    }

    async fn handle(actor: Actor, state: &mut State, msg: Msg) -> Result<(), ActorProcessingErr> {
        match msg {
            Msg::Found(service) => Self::found(actor, state, service).await,
            Msg::Published(event) => Self::published(state, event),
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

    fn service_finder_stopped(actor: Actor, state: &mut State, stopped: &ActorCell) {
        if state.is_service_finder(stopped) {
            Self::schedule_service_finder(actor, state)
        }
    }

    fn schedule_service_finder(actor: Actor, state: &mut State) {
        if let Some(service_finder) = &state.service_finder {
            service_finder.stop(None);
            state.service_finder = None;
        }

        actor.send_after(Duration::from_secs(60 * 5), || Msg::Search);
    }

    async fn remove_device(state: &mut State, stopped: &ActorCell) {
        let Some(device) = state.remove_by_actor(&stopped) else {
            return;
        };

        if let Some(event_port) = &state.event_port {
            event_port.send(Event::Device(device::Event::Disconnected(
                device.service.clone(),
                Instant::now(),
            )))
        }
    }

    async fn actor_stopped(actor: Actor, state: &mut State, terminated: ActorCell) {
        Self::remove_device(state, &terminated).await;
        Self::service_finder_stopped(actor, state, &terminated);
    }

    async fn handle_supervisor_evt(
        actor: Actor,
        state: &mut State,
        event: SupervisionEvent,
    ) -> Result<(), ActorProcessingErr> {
        use SupervisionEvent::*;
        match event {
            ActorTerminated(terminated_actor, _, _) => {
                Self::actor_stopped(actor, state, terminated_actor).await
            }
            ActorPanicked(panicked_actor, _) => {
                Self::actor_stopped(actor, state, panicked_actor).await
            }
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
