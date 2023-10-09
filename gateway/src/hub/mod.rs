mod worker;

use std::sync::Arc;
use std::{error::Error, time::Duration};

use async_trait::async_trait;
use ractor::{call, ActorProcessingErr, ActorRef, SupervisionEvent};
use tokio::{spawn, sync::oneshot};
use tracing::{error, info};

use crate::{
    service::Service,
    service_finder::{self, ServiceFinder},
};

use self::worker::Worker;

pub type Actor = ActorRef<Msg>;
pub type FoundResponse = Result<Arc<Service>, Box<dyn Error + Send + Sync>>;

const HUB_SERVICE: &'static str = "_smultron_hub._tcp.local";

#[derive(Debug)]
pub struct Arguments;

#[derive(Debug)]
pub struct Hub;

impl Hub {
    async fn pre_start(actor: Actor) -> Result<State, ActorProcessingErr> {
        info!("Starting hub");

        Ok(State {
            state: Self::find(actor).await,
        })
    }

    async fn find(actor: Actor) -> InnerState {
        info!("Starting finder for hub");

        start_finder(actor).await;

        info!("Started finder for hub");

        InnerState::Disconnected
    }

    async fn handle(actor: Actor, state: &mut State, msg: Msg) -> Result<(), ActorProcessingErr> {
        info!("Handling message {:?}", msg);

        match msg {
            Msg::FoundResponse(response) => Self::found_response(actor, state, response).await,
            Msg::Publish(subject, payload) => Self::publish(actor, state, subject, payload).await,
        }

        info!("Handled message");

        Ok(())
    }

    async fn handle_supervisor_evt(
        actor: Actor,
        state: &mut State,
        event: SupervisionEvent,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling supervisor event {:?}", event);

        use SupervisionEvent::*;
        match event {
            ActorTerminated(_, _, _) => {
                state.state = Self::find(actor).await;
            }
            ActorPanicked(_, _) => {
                state.state = Self::find(actor).await;
            }
            _ => {}
        }

        info!("Handled supervisor event");
        Ok(())
    }

    async fn publish(_actor: Actor, state: &mut State, subject: String, payload: Vec<u8>) {
        let InnerState::Connected(worker) = &state.state else {
            return;
        };

        call!(worker, worker::Msg::Publish, subject, payload)
            .unwrap()
            .unwrap();
    }

    async fn found_response(actor: Actor, state: &mut State, response: FoundResponse) {
        let InnerState::Disconnected = state.state else {
            return;
        };

        let Ok(service) = response else {
            start_finder(actor).await;
            return;
        };

        let (worker, _) = ractor::Actor::spawn_linked(
            Some("hub_worker".to_owned()),
            Worker,
            worker::Arguments { service },
            actor.into(),
        )
        .await
        .unwrap();

        state.state = InnerState::Connected(worker);
    }
}

pub enum InnerState {
    Disconnected,
    Connected(worker::Actor),
}

pub struct State {
    state: InnerState,
}

#[derive(Debug)]
pub enum Msg {
    FoundResponse(FoundResponse),
    Publish(String, Vec<u8>),
}

async fn start_finder(actor: Actor) {
    let (tx, rx) = oneshot::channel();
    let arguments = service_finder::Arguments {
        name: HUB_SERVICE.to_owned(),
        port: Some(service_finder::Port::Reply(tx.into())),
        interval: Some(Duration::from_secs(12)),
        timeout: Some(Duration::from_secs(30)),
    };
    ractor::Actor::spawn(
        Some("hub_service_finder".to_owned()),
        ServiceFinder,
        arguments,
    )
    .await
    .unwrap();

    spawn(async move {
        info!("Waiting for service finder response in hub");

        let response = rx.await.map_err(|e| e.into());

        info!("Received response from service finder in hub");

        if let Err(e) = actor.send_message(Msg::FoundResponse(response)) {
            error!("Failed to send found response {:?}", e);
        }
    });
}

#[async_trait]
impl ractor::Actor for Hub {
    type Msg = Msg;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        actor: Actor,
        _: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Hub::pre_start(actor).await
    }

    async fn handle(
        &self,
        actor: Actor,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Hub::handle(actor, state, msg).await
    }

    async fn handle_supervisor_evt(
        &self,
        actor: Actor,
        event: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Hub::handle_supervisor_evt(actor, state, event).await
    }
}
