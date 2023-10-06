use std::{sync::Arc, time::Duration};

use async_nats::client::Client;
use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, RpcReplyPort};
use tracing::{error, info, warn};

use crate::service::Service;

pub type Actor = ActorRef<Msg>;
pub type ReplyPort = RpcReplyPort<Result<(), ()>>;

#[derive(Debug)]
pub struct Arguments {
    pub service: Arc<Service>,
}

#[derive(Debug)]
pub struct Worker;

impl Worker {
    async fn connect(service: Arc<Service>) -> Result<State, ActorProcessingErr> {
        info!("Connecting to {}", service.socket_address());

        let client = async_nats::ConnectOptions::new()
            .ping_interval(Duration::from_secs(10))
            .event_callback(|event| async move {
                info!("NATS event: {:?}", event);
            })
            .connect(service.socket_address())
            .await
            .unwrap();

        info!("Connected to {}", service.socket_address());

        Ok(State { client })
    }

    async fn publish(
        actor: Actor,
        state: &mut State,
        subject: String,
        payload: Vec<u8>,
        reply_port: ReplyPort,
    ) -> Result<(), ActorProcessingErr> {
        info!("Publishing to subject {}", subject);

        if let Err(e) = state.client.publish(subject.clone(), payload.into()).await {
            warn!("Failed to publish {:?}", e);

            if let Err(e) = reply_port.send(Err(())) {
                error!("Failed to reply error to port {:?}", e);
            }

            actor.stop(None);

            return Err(e.into());
        }

        info!("Published to subject {}", subject);
        if let Err(e) = reply_port.send(Ok(())) {
            error!("Failed to reply ok to port {:?}", e);
        }
        Ok(())
    }
}

pub struct State {
    client: Client,
}

#[derive(Debug)]
pub enum Msg {
    Publish(String, Vec<u8>, ReplyPort),
}

#[async_trait]
impl ractor::Actor for Worker {
    type Msg = Msg;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        _: Actor,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Worker::connect(arguments.service).await
    }

    async fn handle(
        &self,
        actor: Actor,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            Msg::Publish(subject, payload, reply_port) => {
                Worker::publish(actor, state, subject, payload, reply_port).await
            }
        }
    }
}
