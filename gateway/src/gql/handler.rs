use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

use super::model::Device;

#[async_trait]
pub trait Handler: Send + Sync {
    async fn get_devices(&self) -> Vec<Device>;

    async fn subscribe_to_devices(&self) -> Receiver<String>;
}
