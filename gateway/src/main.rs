mod device;
mod gateway;
mod hub;
mod ipso;
mod service_finder;
mod service;

use std::error::Error;

use ractor::Actor;
use tokio::select;
use tracing::info;
use web_linking::links;

use crate::{gateway::Gateway, hub::Hub};

type AppResult = Result<(), Box<dyn Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> AppResult {
    tracing_subscriber::fmt().try_init()?;

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
