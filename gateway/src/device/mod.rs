use std::{sync::Arc, time::Duration};

/*

TODO:

- connect to device
- publish connected to device
- discover resources
- publish discovered resources
- ping every 10 seconds
- publish success full ping
- publish unsuccessful ping
- publish connection lost to device

*/
use async_trait::async_trait;
use coapium::{
    asynchronous::{default_parameters, default_reliability, Client},
    codec::{message::GetOptions, url::Endpoint},
    protocol::{get::Get, new_request::NewRequest, ping::Ping},
};
use ractor::{ActorProcessingErr, ActorRef, OutputPort};
use web_linking::links;

use crate::{
    ipso::{self, ResourceInstance},
    service::Service,
};

pub type Actor = ActorRef<Msg>;

pub type EventPort = Arc<OutputPort<Event>>;

#[derive(Clone, Debug)]
pub enum Event {
    Connected,
    Discovered,
    PollingResource,
    ResourcePolled,
    Pinging,
    Pinged,
    PingFailed,
}

pub struct Device;

impl Device {
    async fn connect(service: Arc<Service>, event_port: EventPort) -> Client {
        let endpoint = format!("coap://{}", service.socket_address());
        let client = Client::new(Endpoint::from_str(&endpoint).unwrap()).await;

        event_port.send(Event::Connected);

        client
    }

    async fn ping(state: &mut State) {
        state.event_port.send(Event::Pinging);

        if let Err(e) = //ping("coap://192.168.1.217:5683".try_into().unwrap())
            state
                .client
                .ping(Ping {
                    confirmable_parameters: default_parameters(),
                })
                .await
        {
            state.event_port.send(Event::PingFailed);

            panic!("Failed to ping device: {:?}", e);
        }

        state.event_port.send(Event::Pinged);
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
            .map(ipso::ResourceInstance::from_str)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        state.resources = Arc::new(resource_instances);
        state.event_port.send(Event::Discovered);
    }

    async fn pre_start(actor: Actor, arguments: Arguments) -> Result<State, ActorProcessingErr> {
        let client = Self::connect(arguments.service, arguments.event_port.clone()).await;

        let mut state = State {
            client,
            event_port: arguments.event_port,
            resources: Arc::new(vec![]),
        };

        Self::ping(&mut state).await;

        Self::discover(&mut state).await;

        actor.send_interval(Duration::from_secs(10), || Msg::Ping);

        actor.send_interval(Duration::from_secs(30), || Msg::Poll);

        Ok(state)
    }

    async fn poll(state: &mut State) {
        for resource in state.resources.iter() {
            Self::poll_resource(&mut state.client, &state.event_port, resource).await;
        }
    }

    async fn poll_resource(
        client: &mut Client,
        event_port: &EventPort,
        resource: &ResourceInstance,
    ) {
        event_port.send(Event::PollingResource);

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

        event_port.send(Event::ResourcePolled);
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
    pub service: Arc<Service>,
}

pub enum Msg {
    Ping,
    Poll,
}

pub struct State {
    client: Client,
    event_port: EventPort,
    resources: Arc<Vec<ResourceInstance>>,
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
