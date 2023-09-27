use std::time::Duration;

use futures_util::{pin_mut, StreamExt};
use mdns::discover;
use tokio::{sync::RwLock, time::interval};
use tracing::info;

use crate::AppResult;

const ONE_MINUTE: Duration = Duration::from_secs(10);
const HUB_SERVICE: &'static str = "_smultron_hub._tcp.local";

pub struct Hub {
    client: RwLock<Option<async_nats::Client>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            client: Default::default(),
        }
    }

    pub async fn run(self) -> AppResult {
        self.discover().await
    }

    async fn discover(&self) -> AppResult {
        info!("starting service discovery for {:?}", HUB_SERVICE);

        let discovery_stream = discover::all(HUB_SERVICE, ONE_MINUTE)?.listen();
        pin_mut!(discovery_stream);

        let client = loop {
            if let Some(Ok(service)) = discovery_stream.next().await {
                if let Some(connection_string) = self.on_discovered(service).await {
                    info!("found a hub: {}", connection_string);
                    break async_nats::connect(connection_string).await;
                }
            }
        }?;

        self.client.write().await.replace(client);

        self.publish().await
    }

    async fn publish(&self) -> AppResult {
        let mut publish_ticker = interval(Duration::from_secs(10));

        loop {
            publish_ticker.tick().await;
        }
    }

    async fn on_discovered(&self, discovered: mdns::Response) -> Option<String> {
        info!("hub discovery: {:?}", discovered);
        discovered.records().find_map(|record| match &record.kind {
            mdns::RecordKind::SRV { port, target, .. } => Some(format!("{}:{}", target, port)),
            _ => None,
        })
    }
}
