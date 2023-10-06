mod debugger;
mod device;
mod gateway;
mod hub;
mod ipso;
mod service;
mod service_finder;
mod timeout;

use std::{
    error::Error,
    time::{Duration, Instant},
};

use async_nats::ServerAddr;
use futures_util::StreamExt;
use ractor::{call, call_t, Actor, OutputPort};
use tokio::{
    select, spawn,
    sync::oneshot,
    time::{interval, sleep},
};
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
    //test_service_finder().await
    test_hub().await
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

    let (_, hub_handle) = Actor::spawn(Some("hub".to_string()), Hub, hub::Arguments).await?;

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
    let (debugger, debugger_handle) =
        Actor::spawn(Some("debugger".to_owned()), Debugger, ()).await?;

    //let port: service_finder::BroadcastPort = Default::default();
    let (tx, rx) = oneshot::channel();
    let port = tx.into();

    let (service_finder, _) = Actor::spawn_linked(
        Some("service_finder".to_owned()),
        ServiceFinder,
        service_finder::Arguments {
            //port: Some(Port::Broadcast(port.clone())),
            port: Some(Port::Reply(port)),
            name: "_coap._udp.local".to_owned(),
        },
        debugger.clone().into(),
    )
    .await?;

    //port.subscribe(debugger, |msg| Some(format!("{:#?}", msg)));

    let msg = rx.await.unwrap();
    debugger.send_message(format!("{:#?}", msg)).unwrap();

    select! {
        result = debugger_handle => {
            Ok(result?)
        }
    }
}

async fn test_hub() -> AppResult {
    let (actor, _) = Actor::spawn(Some("hub".to_owned()), Hub, hub::Arguments)
        .await
        .unwrap();

    let now = Instant::now();

    let mut ticker = interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        let t = Instant::now() - now;
        let s = format!("{}", t.as_secs()).as_bytes().to_vec();
        actor
            .send_message(hub::Msg::Publish("test".to_owned(), s))
            .unwrap();
    }
}
