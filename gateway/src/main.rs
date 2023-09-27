mod device;
mod gateway;
mod ipso;
mod hub;

use std::error::Error;

use tracing::info;
use web_linking::links;

use crate::gateway::Gateway;

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

    Gateway::new().run().await?;

    Ok(())
}
