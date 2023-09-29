use std::{
    fmt,
    future::ready,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use futures_util::StreamExt;
use mdns::discover;
use ractor::{ActorProcessingErr, ActorRef};
use tokio::sync::RwLock;
use tracing::info;

use crate::service::{self, Service};

use super::{event::EventPort, Event};

type Actor = ActorRef<()>;

pub struct Worker;

pub struct Arguments {
    pub event_port: Option<EventPort>,
    pub name: service::Name,
}

pub struct State {
    event_port: Option<EventPort>,
    services: Vec<Arc<Service>>,
    name: service::Name,
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

async fn discovered(state: Arc<RwLock<&mut State>>, response: mdns::Response) {
    let mut state = state.write().await;

    response
        .records()
        .filter_map(into_service)
        .for_each(|service| found(&mut state, service))
}

fn found(state: &mut State, service: Service) {
    if let Some(service) = state.put_service(service.clone()) {
        info!("Service found {:?}", service);
        state.emit_event(Event::Found(service))
    }
}

fn into_service(record: &mdns::Record) -> Option<Service> {
    match &record.kind {
        mdns::RecordKind::SRV { port, target, .. } => Some(Service {
            found_at: Instant::now(),
            name: record.name.clone(),
            port: *port,
            target: target.clone(),
        }),
        _ => None,
    }
}

async fn start(state: &mut State) -> Result<(), ActorProcessingErr> {
    let name = state.name.clone();

    let state = Arc::new(RwLock::new(state));

    discover::all(name, Duration::from_secs(15))?
        .listen()
        .filter_map(|result| ready(result.ok()))
        .for_each(|response| discovered(state.clone(), response))
        .await;

    Ok(())
}

#[async_trait]
impl ractor::Actor for Worker {
    type Arguments = Arguments;
    type Msg = ();
    type State = State;

    async fn pre_start(
        &self,
        _: Actor,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        info!("Starting worker with arguments {:?}", arguments);
        Ok(Self::State {
            event_port: arguments.event_port,
            services: vec![],
            name: arguments.name,
        })
    }

    async fn post_start(
        &self,
        _: Actor,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Starting discovery");
        start(state).await
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
