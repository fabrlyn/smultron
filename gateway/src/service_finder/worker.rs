use std::sync::Arc;

use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, RpcReplyPort};
use tokio::{spawn, sync::oneshot};
use tracing::{info, warn};

use crate::{
    service::{self, Service},
    service_finder::task,
};

use super::{
    port::{BroadcastPort, ReplyPort},
    Event, Port,
};

pub type Actor = ActorRef<Msg>;

pub struct Worker;

#[derive(Debug)]
pub struct Arguments {
    pub name: service::Name,
    pub port: Option<Port>,
}

#[derive(Debug)]
pub enum Msg {
    Found(Service),
    Get(RpcReplyPort<Vec<Arc<Service>>>),
    TaskStopping,
}

#[derive(Debug)]
pub struct State {
    name: service::Name,
    port: Option<Port>,
    pub services: Vec<Arc<Service>>,
    stop_tx: task::StopTx,
}

impl State {
    fn emit_event(&self, event: Event) {
        if let Some(Port::Broadcast(port)) = &self.port {
            info!("Emitting event {:?}", event);
            port.send(event);
        }
    }

    fn find_by_service(&self, subject: &Service) -> Option<&Arc<Service>> {
        self.services
            .iter()
            .find(|service| service.as_ref() == subject)
    }

    fn is_done(&self) -> bool {
        match self.port {
            Some(Port::Reply(None)) => true,
            _ => false,
        }
    }

    fn put_service(&mut self, service: Service) -> Option<Arc<Service>> {
        if self.service_exists(&service) {
            return None;
        }

        let service = Arc::new(service);
        self.services.push(service.clone());

        Some(service)
    }

    fn send(&mut self, actor: &Actor, service: Arc<Service>) {
        match &mut self.port {
            Some(Port::Reply(port)) => reply(actor, self, port, service),
            Some(Port::Broadcast(port)) => broadcast(port, Event::Found(service)),
            None => return,
        }
    }

    fn service_exists(&self, service: &Service) -> bool {
        self.find_by_service(service).is_some()
    }

    fn stop(&self, actor: &Actor) {
        info!("Stopping worker");

        self.stop_task();

        actor.stop(None);
        info!("Stopped worker");
    }

    fn stop_task(&self) {
        info!("Stopping task");
        if let Err(_) = self.stop_tx.send(()) {
            info!("Stopping task failed, could already be stopped");
        } else {
            info!("Stopped task");
        }
    }
}

fn broadcast(port: &BroadcastPort, event: Event) {
    info!("Broadcasting event {:?}", event);
    port.send(event);
}

fn found(actor: &Actor, state: &mut State, service: Service) {
    if let Some(service) = state.put_service(service.clone()) {
        info!("Service found {:?}", service);
        state.send(actor, service);
    };
}

fn reply(actor: &Actor, state: &mut State, port: &mut Option<ReplyPort>, service: Arc<Service>) {
    if let Some(port) = port.take() {
        info!("Replying to port");
        if let Err(e) = port.send(service) {
            warn!("Failed to reply with found service");
        } else {
            info!("Replied to port");
        }
        state.stop(actor);
    }
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
        info!("Handle message {:?}", msg);

        match msg {
            Msg::Found(service) => found(&actor, state, service),
            Msg::TaskStopping => state.stop(&actor),
        }

        info!("Handled message");
        Ok(())
    }

    async fn pre_start(
        &self,
        actor: Actor,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        info!("Starting worker with arguments {:?}", arguments);

        let (stop_tx, stop_rx) = oneshot::channel();

        spawn(async { task::start(actor, stop_rx, arguments.name) });

        Ok(Self::State {
            stop_tx,
            port: arguments.port,
            services: vec![],
            name: arguments.name,
        })
    }
}
