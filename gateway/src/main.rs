mod device;
mod gateway;

use std::error::Error;

use tracing::info;

use crate::gateway::Gateway;

type AppResult = Result<(), Box<dyn Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> AppResult {
    tracing_subscriber::fmt().try_init()?;

    info!("Starting gateway");

    Gateway::new().run().await?;

    Ok(())
}
