mod port;
mod task;
mod worker;

pub use port::BroadcastPort;
pub use port::Event;
pub use port::Port;
pub use port::ReplyPort;
use ractor::actor::messages::BoxedState;
use ractor::ActorCell;

use crate::{
    mdns_service::{self, MdnsService},
    service_finder::worker::Worker,
};
use async_trait::async_trait;
use ractor::{call, ActorProcessingErr, ActorRef, SupervisionEvent};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub type Actor = ActorRef<Msg>;

#[derive(Debug)]
pub struct Arguments {
    pub name: mdns_service::Name,
    pub port: Option<Port>,
    pub interval: Option<Duration>,
    pub timeout: Option<Duration>,
}

#[derive(Debug)]
pub enum Msg {}

pub struct ServiceFinder;

impl ServiceFinder {
    fn handle_supervisor_evt(
        actor: Actor,
        state: &mut State,
        event: SupervisionEvent,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling {:?}", event);

        use SupervisionEvent::*;
        match event {
            ActorTerminated(link_actor, link_state, _) => {
                link_terminated(actor, state, link_actor, link_state)
            }
            ActorPanicked(link_actor, error) => link_panicked(link_actor, error),
            _ => {}
        }
        info!("Handled event");
        Ok(())
    }

    async fn post_stop(state: &mut State) -> Result<(), ActorProcessingErr> {
        info!("Stopping service finder");

        if state.services.is_empty() {
            info!("Getting services from worker");
            if let Ok(services) = call!(state.worker, worker::Msg::Get) {
                info!("Got services: {:?}", services);
                state.services = services;
            }
        }

        state.stop();

        Ok(())
    }

    async fn pre_start(actor: Actor, arguments: Arguments) -> Result<State, ActorProcessingErr> {
        info!("Starting service finder with {:?}", arguments);

        let (worker, _) = ractor::Actor::spawn_linked(
            None,
            Worker,
            worker::Arguments {
                port: arguments.port,
                name: arguments.name,
                interval: arguments.interval,
                timeout: arguments.timeout,
            },
            actor.into(),
        )
        .await?;

        Ok(State {
            worker,
            services: vec![],
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    worker: worker::Actor,

    pub services: Vec<Arc<MdnsService>>,
}

impl State {
    fn merge(&mut self, state: &mut BoxedState) {
        let Ok(state) = state.take::<worker::State>() else {
            return;
        };

        self.services = state.services;
    }

    fn stop(&self) {
        info!("Stopping worker");

        self.worker.stop(None);

        info!("Stopped worker");
    }
}

fn link_terminated(
    actor: Actor,
    state: &mut State,
    _link_actor: ActorCell,
    link_state: Option<BoxedState>,
) {
    info!("Linked actor terminated {:?}", actor);
    if let Some(mut link_state) = link_state {
        state.merge(&mut link_state);
    }

    actor.stop(None);
}

fn link_panicked(actor: ActorCell, error: Box<dyn Error>) {
    panic!("Linked actor {:?} panicked{:?}", actor, error);
}

#[async_trait]
impl ractor::Actor for ServiceFinder {
    type Arguments = Arguments;
    type Msg = Msg;
    type State = State;

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        _: Self::Msg,
        _: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        actor: ActorRef<Self::Msg>,
        event: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        ServiceFinder::handle_supervisor_evt(actor, state, event)
    }

    async fn post_stop(
        &self,
        _: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        ServiceFinder::post_stop(state).await
    }

    async fn pre_start(
        &self,
        actor: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        ServiceFinder::pre_start(actor, arguments).await
    }
}
