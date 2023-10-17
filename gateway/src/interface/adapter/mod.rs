use async_trait::async_trait;
use tokio::sync::broadcast::{channel, Receiver};

use crate::{
    application::port::{GetDevices, ObserveDevices},
    domain::{
        device::DeviceEvent, resource_instance::ResourceInstance, service_record::ServiceRecord,
        Device,
    },
};

pub struct InMemoryAdapter {}

#[async_trait]
impl GetDevices for InMemoryAdapter {
    async fn get_devices(&self) -> Vec<Device> {
        vec![Device {
            service_record: ServiceRecord {
                target: "alpha_ab-bc-cd-de-ef".to_owned().into(),
                port: 5683.into(),
            },
            resource_instances: vec![ResourceInstance {
                object_id: 5700,
                instance_id: 0,
                resource_id: 3303,
            }],
        }]
    }
}

#[async_trait]
impl ObserveDevices for InMemoryAdapter {
    async fn observe_devices(&self) -> Receiver<DeviceEvent> {
        let (tx, rx) = channel(100);
        rx
    }
}
