use super::{port::BroadcastPort, Event, ReplyPort};
use crate::{
    mdns_service::{self, MdnsService},
    service_finder::task,
};
use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, RpcReplyPort};
use std::{fmt, sync::Arc, time::Duration};
use tokio::{spawn, sync::oneshot, time::sleep};
use tracing::{error, info, warn};

pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(30);

pub type Actor = ActorRef<Msg>;

pub type GetTx = RpcReplyPort<Vec<Arc<MdnsService>>>;

#[derive(Debug)]
pub struct Arguments {
    pub name: mdns_service::Name,
    pub port: Option<super::Port>,
    pub interval: Option<Duration>,
    pub timeout: Option<Duration>,
}

#[derive(Debug)]
pub enum Msg {
    Found(MdnsService),
    Get(RpcReplyPort<Vec<Arc<MdnsService>>>),
    TaskStopping,
}

enum Port {
    Broadcast(BroadcastPort),
    Reply(Option<ReplyPort>),
}

#[derive(Debug)]
pub struct State {
    port: Option<Port>,
    stop_tx: Option<task::StopTx>,

    pub services: Vec<Arc<MdnsService>>,
}

impl State {
    fn find_by_service(&self, subject: &MdnsService) -> Option<&Arc<MdnsService>> {
        self.services
            .iter()
            .find(|service| service.as_ref() == subject)
    }

    fn put_service(&mut self, service: MdnsService) -> Option<Arc<MdnsService>> {
        if self.service_exists(&service) {
            return None;
        }

        let service = Arc::new(service);
        self.services.push(service.clone());

        Some(service)
    }

    fn service_exists(&self, service: &MdnsService) -> bool {
        self.find_by_service(service).is_some()
    }
}

pub struct Worker;

impl Worker {
    fn found(actor: Actor, state: &mut State, service: MdnsService) {
        let Some(service) = state.put_service(service.clone()) else {
            return;
        };

        info!("Service found {:?}", service);
        send(actor, state, service);
    }

    fn get(state: &mut State, get_tx: GetTx) {
        if let Err(e) = get_tx.send(state.services.clone()) {
            warn!("Failed to send on get tx: {:?}", e);
        }
    }

    fn handle(actor: Actor, state: &mut State, msg: Msg) -> Result<(), ActorProcessingErr> {
        info!("Handle message {:?}", msg);

        use Msg::*;
        match msg {
            Found(service) => Self::found(actor, state, service),
            TaskStopping => Self::stop(actor, state),
            Get(tx) => Self::get(state, tx),
        }

        info!("Handled message");
        Ok(())
    }

    fn pre_start(actor: Actor, arguments: Arguments) -> Result<State, ActorProcessingErr> {
        info!("Starting worker with arguments {:?}", arguments);

        let (stop_tx, stop_rx) = oneshot::channel();
        spawn(task::start(
            actor.clone(),
            stop_rx,
            arguments.name,
            arguments.interval.unwrap_or(DEFAULT_INTERVAL),
        ));

        if let Some(timeout) = arguments.timeout {
            spawn(async move {
                sleep(timeout).await;
                actor.stop(None);
            });
        }

        Ok(State {
            stop_tx: Some(stop_tx),
            port: arguments.port.map(Into::into),
            services: vec![],
        })
    }

    fn stop(actor: Actor, state: &mut State) {
        info!("Stopping worker");

        stop_task(&mut state.stop_tx);

        actor.stop(None);

        info!("Stopped worker");
    }
}

fn broadcast(port: &BroadcastPort, event: Event) {
    info!("Broadcasting event {:?}", event);
    port.send(event);
}

fn reply(actor: Actor, state: &mut State, service: Arc<MdnsService>) {
    info!("Replying with found service {:?}", service);

    let Some(Port::Reply(Some(port))) = state.port.take() else {
        error!(
            "Failed to reply, expected reply port but got {:?}",
            state.port
        );
        return;
    };

    if let Err(e) = port.send(service) {
        warn!("Failed to reply with found service: {:?}", e);
    } else {
        info!("Replied to port");
    }

    Worker::stop(actor, state);
}

fn send(actor: Actor, state: &mut State, service: Arc<MdnsService>) {
    match &state.port {
        Some(Port::Reply(_)) => reply(actor, state, service),
        Some(Port::Broadcast(port)) => broadcast(port, Event::Found(service)),
        None => return,
    }
}

fn stop_task(stop_tx: &mut Option<task::StopTx>) {
    info!("Stopping task");

    let Some(stop_tx) = stop_tx.take() else {
        info!("Task already stopped");
        return;
    };

    if let Err(e) = stop_tx.send(()) {
        info!("Failed to stop task {:?}", e);
        return;
    }

    info!("Stopped task");
}

#[async_trait]
impl ractor::Actor for Worker {
    type Arguments = Arguments;
    type Msg = Msg;
    type State = State;

    async fn handle(
        &self,
        actor: Actor,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Worker::handle(actor, state, msg)
    }

    async fn pre_start(
        &self,
        actor: Actor,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Worker::pre_start(actor, arguments)
    }
}

impl From<super::Port> for Port {
    fn from(value: super::Port) -> Self {
        match value {
            super::Port::Broadcast(port) => Self::Broadcast(port),
            super::Port::Reply(port) => Self::Reply(Some(port)),
        }
    }
}

impl fmt::Debug for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Port::Broadcast(_) => write!(f, "Broadcast"),
            Port::Reply(Some(_)) => write!(f, "Reply(Some)"),
            Port::Reply(None) => write!(f, "Reply(None)"),
        }
    }
}
