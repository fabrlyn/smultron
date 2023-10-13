use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

use crate::gql::{model::Device, Handler};

pub struct GqlPlug {}

#[async_trait]
impl Handler for GqlPlug {
    async fn get_devices(&self) -> Vec<Device> {
        todo!()
    }

    async fn subscribe_to_devices(&self) -> Receiver<String> {
        todo!()
    }
}
