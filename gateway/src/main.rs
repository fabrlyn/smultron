mod debugger;
mod device;
mod gateway;
mod hub;
mod ipso;
mod service;
mod service_finder;
mod timeout;

use std::error::Error;

use ractor::{Actor, OutputPort};
use tokio::select;
use tracing::info;
use web_linking::links;

use crate::{
    debugger::Debugger,
    gateway::Gateway,
    hub::Hub,
    service_finder::{Port, ServiceFinder},
};

type AppResult = Result<(), Box<dyn Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> AppResult {
    tracing_subscriber::fmt().try_init()?;
    test_service_finder().await
}

async fn real_main() -> AppResult {
    let all_links = links("</3/0/5852>;ct=42").unwrap();
    let first_link = all_links.get(0).unwrap();
    println!("all links:  {:?}", all_links);
    println!(
        "resource instance: {:?}",
        ipso::ResourceInstance::from_str(first_link.value.value)
    );

    info!("Starting gateway");

    let (_, hub_handle) = Actor::spawn(Some("hub".to_string()), Hub, ()).await?;

    select! {
        result = Gateway::new().run() => {
            result
        }
        result = hub_handle => {
            result.map_err(|e| e.into())
        }
    }
}

async fn test_service_finder() -> AppResult {
    let port: service_finder::BroadcastPort = Default::default();

    let (service_finder, _) = Actor::spawn(
        Some("service_finder".to_owned()),
        ServiceFinder,
        service_finder::Arguments {
            port: Some(Port::Broadcast(port.clone())),
            name: "_companion-link._tcp.local".to_owned(),
        },
    )
    .await?;

    let (debugger, debugger_handle) =
        Actor::spawn(Some("debugger".to_owned()), Debugger, service_finder).await?;

    port.subscribe(debugger, |msg| Some(format!("{:?}", msg)));

    select! {
        result = debugger_handle => {
            Ok(result?)
        }
    }
}
