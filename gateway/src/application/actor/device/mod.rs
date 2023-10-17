use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use coapium::{
    asynchronous::{default_parameters, default_reliability, Client},
    codec::{message::GetOptions, url::Endpoint},
    protocol::{get::Get, new_request::NewRequest, ping::Ping},
};
use ractor::{ActorProcessingErr, ActorRef, OutputPort};
use web_linking::links;

use crate::{domain::resource_instance::ResourceInstance, mdns_service::MdnsService};

pub type Actor = ActorRef<Msg>;

pub type EventPort = Arc<OutputPort<Event>>;

#[derive(Clone, Debug)]
pub enum Event {
    Connected(Arc<MdnsService>, Instant),
    Discovered(Arc<MdnsService>, Instant, Arc<Vec<ResourceInstance>>),
    PollingResource(Arc<MdnsService>, Instant, ResourceInstance),
    ResourcePolled(Arc<MdnsService>, Instant, ResourceInstance, Vec<u8>),
    Pinging(Arc<MdnsService>, Instant),
    Pinged(Arc<MdnsService>, Instant),
    PingFailed(Arc<MdnsService>, Instant),
    Disconnected(Arc<MdnsService>, Instant),
}

pub struct Device;

impl Device {
    async fn connect(service: Arc<MdnsService>, event_port: EventPort) -> Client {
        let endpoint = format!("coap://{}", service.socket_address());
        let client = Client::new(Endpoint::from_str(&endpoint).unwrap()).await;

        event_port.send(Event::Connected(service.clone(), Instant::now()));

        client
    }

    async fn ping(state: &mut State) {
        state
            .event_port
            .send(Event::Pinging(state.service.clone(), Instant::now()));

        if let Err(e) = state
            .client
            .ping(Ping {
                confirmable_parameters: default_parameters(),
            })
            .await
        {
            state
                .event_port
                .send(Event::PingFailed(state.service.clone(), Instant::now()));

            panic!("Failed to ping device: {:?}", e);
        }

        state
            .event_port
            .send(Event::Pinged(state.service.clone(), Instant::now()));
    }

    async fn discover(state: &mut State) {
        let mut options = GetOptions::new();
        options.set_uri_path("/.well-known/core".try_into().unwrap());

        let result = state
            .client
            .execute(NewRequest::Get(Get {
                options,
                reliability: default_reliability(),
            }))
            .await;

        let response = match result {
            Ok(response) => response,
            Err(e) => panic!("{:?}", e),
        };

        let payload = String::from_utf8(response.payload.value().to_vec()).unwrap();

        let resource_instances = links(payload.as_str())
            .unwrap()
            .into_iter()
            .map(|l| l.value.value)
            .map(ResourceInstance::from_str)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        state.resources = Arc::new(resource_instances);
        state.event_port.send(Event::Discovered(
            state.service.clone(),
            Instant::now(),
            state.resources.clone(),
        ));
    }

    async fn pre_start(actor: Actor, arguments: Arguments) -> Result<State, ActorProcessingErr> {
        let client = Self::connect(arguments.service.clone(), arguments.event_port.clone()).await;

        let mut state = State {
            client,
            event_port: arguments.event_port,
            resources: Arc::new(vec![]),
            service: arguments.service,
        };

        Self::discover(&mut state).await;

        actor.send_interval(Duration::from_secs(10), || Msg::Ping);

        actor.send_interval(Duration::from_secs(30), || Msg::Poll);

        Ok(state)
    }

    async fn poll(state: &mut State) {
        for resource in state.resources.iter() {
            Self::poll_resource(
                state.service.clone(),
                &mut state.client,
                &state.event_port,
                resource,
            )
            .await;
        }
    }

    async fn poll_resource(
        service: Arc<MdnsService>,
        client: &mut Client,
        event_port: &EventPort,
        resource: &ResourceInstance,
    ) {
        event_port.send(Event::PollingResource(
            service.clone(),
            Instant::now(),
            *resource,
        ));

        let mut options = GetOptions::new();
        options.set_uri_path(resource.to_path().try_into().unwrap());

        let response = client
            .execute(NewRequest::Get(Get {
                options,
                reliability: default_reliability(),
            }))
            .await
            .unwrap();
        if !response.response_code.is_success() {
            panic!("response is not success: {:?}", response.response_code);
        }

        event_port.send(Event::ResourcePolled(
            service,
            Instant::now(),
            *resource,
            response.payload.value().to_vec(),
        ));
    }

    async fn handle(_: Actor, state: &mut State, msg: Msg) -> Result<(), ActorProcessingErr> {
        match msg {
            Msg::Ping => Self::ping(state).await,
            Msg::Poll => Self::poll(state).await,
        }
        Ok(())
    }
}

pub struct Arguments {
    pub event_port: EventPort,
    pub service: Arc<MdnsService>,
}

pub enum Msg {
    Ping,
    Poll,
}

pub struct State {
    client: Client,
    event_port: EventPort,
    resources: Arc<Vec<ResourceInstance>>,
    service: Arc<MdnsService>,
}

#[async_trait]
impl ractor::Actor for Device {
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
}
