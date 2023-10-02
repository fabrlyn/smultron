mod port;
mod task;
mod worker;

pub use port::BroadcastPort;
pub use port::Event;
pub use port::Port;
pub use port::ReplyPort;

use std::sync::Arc;

use async_trait::async_trait;
use ractor::{call, ActorProcessingErr, ActorRef, SupervisionEvent};
use tracing::info;

use crate::{
    service::{self, Service},
    service_finder::worker::Worker,
};

pub type Actor = ActorRef<Msg>;

pub struct ServiceFinder;

#[derive(Debug)]
pub struct Arguments {
    pub port: Option<Port>,
    pub name: service::Name,
}

#[derive(Debug)]
pub enum Msg {
    Stop,
}

pub struct State {
    worker: worker::Actor,
    services: Vec<Arc<Service>>,
}

impl State {
    fn stop(&self) {
        info!("Stopping worker");

        self.worker.stop(None);

        info!("Stopping worker");
    }
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

        let (worker, _) = ractor::Actor::spawn_linked(
            None,
            Worker,
            worker::Arguments {
                port: arguments.port,
                name: arguments.name,
            },
            actor.into(),
        )
        .await?;

        Ok(Self::State {
            worker,
            services: vec![],
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        _: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling message {:?}", message);
        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        actor: ActorRef<Self::Msg>,
        event: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling supervision event {:?}", event);

        match event {
            SupervisionEvent::ActorTerminated(_, worker_state, _) => {
                let result = match worker_state {
                    Some(mut worker_state) => {
                        let services = worker_state.take::<worker::State>().unwrap().services;
                        state.services = services;
                        Ok(())
                    }
                    None => Ok(()),
                };

                actor.stop(None);

                result
            }
            SupervisionEvent::ActorPanicked(_, e) => panic!("{:?}", e),
            _ => Ok(()),
        }
    }

    async fn post_stop(
        &self,
        _: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Stopping service finder");

        if state.services.is_empty() {
            info!("Getting found services from worker");
            if let Ok(services) = call!(state.worker, worker::Msg::Get) {
                state.services = services;
            }
        }

        state.stop();

        Ok(())
    }
}
