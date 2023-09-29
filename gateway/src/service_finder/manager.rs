use std::{fmt, sync::Arc, time::Duration};

use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, OutputPort, SupervisionEvent};
use tracing::info;

use crate::{service, service_finder::worker::Worker};

use super::{worker, Event};

pub type EventPort = Arc<OutputPort<Event>>;

pub struct Manager;

pub struct Arguments {
    pub event_port: Option<EventPort>,
    pub name: service::Name,
}

pub struct State {
    worker: ActorRef<()>,
}

impl State {
    async fn stop(&self) {
        info!("Killing worker");
        self.worker.kill_and_wait(None).await.unwrap();
        info!("Killed worker");
    }
}

#[async_trait]
impl ractor::Actor for Manager {
    type Arguments = Arguments;
    type Msg = ();
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
                event_port: arguments.event_port,
                name: arguments.name,
            },
            actor.into(),
        )
        .await?;

        Ok(Self::State { worker })
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
        _: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        _: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling supervision event {:?}", message);
        Ok(())
    }

    async fn post_stop(
        &self,
        _: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Stopping service finder");

        state.stop().await;

        Ok(())
    }
}

impl From<service::Name> for Arguments {
    fn from(name: service::Name) -> Self {
        Self {
            event_port: None,
            name,
        }
    }
}

impl fmt::Debug for Arguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arguments")
            .field("event_port", &self.event_port.is_some())
            .field("name", &self.name)
            .finish()
    }
}
