use std::time::Duration;

use async_trait::async_trait;
use futures_util::{pin_mut, StreamExt};
use mdns::discover;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use tracing::{error, info};

use crate::service_finder::{self, ServiceFinder};

const ONE_MINUTE: Duration = Duration::from_secs(10);
const HUB_SERVICE: &'static str = "_smultron_hub._tcp.local";

#[derive(Debug)]
pub struct Hub;

pub struct WrappedState<S> {
    state: S,
}

pub enum HubState {
    Disconnected(service_finder::Actor),
    Connected(async_nats::Client),
}

async fn discover(hub: ActorRef<Message>) {
    let discovery_stream = discover::all(HUB_SERVICE, ONE_MINUTE).unwrap().listen();
    pin_mut!(discovery_stream);

    let url = loop {
        if let Some(Ok(service)) = discovery_stream.next().await {
            if let Some(url) = on_discovered(service).await {
                break url;
            }
        }
    };

    if let Err(e) = hub.send_message(Message::Connect(url.clone())) {
        error!("Failed to send url {} to Hub actor: {}", url, e);
    }
}

async fn on_discovered(discovered: mdns::Response) -> Option<String> {
    info!("hub discovery: {:?}", discovered);
    discovered.records().find_map(|record| match &record.kind {
        mdns::RecordKind::SRV { port, target, .. } => Some(format!("{}:{}", target, port)),
        _ => None,
    })
}

#[derive(Debug)]
pub enum Message {
    Connect(String),
}

#[async_trait]
impl Actor for Hub {
    type Msg = Message;
    type State = HubState;
    type Arguments = ();

    async fn pre_start(
        &self,
        actor: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        info!("Starting hub actor");

        let (service_finder, _) = Actor::spawn_linked(
            Some("hub_service_finder".to_owned()),
            ServiceFinder,
            service_finder::Arguments {
                port: None,
                name: HUB_SERVICE.to_owned(),
            },
            actor.into(),
        )
        .await?;

        Ok(HubState::Disconnected(service_finder))
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Message,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Hub actor received message: {:?}", message);

        Ok(())
    }
}

impl Hub {
    async fn on_connect(&self, state: &mut WrappedState<HubState>, Message::Connect(url): Message) {
        info!("Hub found, establishing connection to {}", url);

        let client = async_nats::connect(url.clone()).await.unwrap();
        state.state = HubState::Connected(client);

        info!("Hub connected to {}", url);
    }
}
