use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

use crate::domain::device::{Device, DeviceEvent};

pub trait Base: Send + Sync {}

impl<T> Base for T where T: Send + Sync + ?Sized {}

pub trait Port: GetDevices + ObserveDevices {}

impl<T> Port for T where T: GetDevices + ObserveDevices {}

#[async_trait]
pub trait GetDevices: Base {
    async fn get_devices(&self) -> Vec<Device>;
}

#[async_trait]
pub trait ObserveDevices: Base {
    async fn observe_devices(&self) -> Receiver<DeviceEvent>;
}
