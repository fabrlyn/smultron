use super::{resource_instance::ResourceInstance, service_record::ServiceRecord};

#[derive(Clone)]
pub struct Device {
    pub service_record: ServiceRecord,
    pub resource_instances: Vec<ResourceInstance>,
}

#[derive(Clone)]
pub enum Event {
    Ping,
}

#[derive(Clone)]
pub struct DeviceEvent {
    pub device: Device,
    pub event: Event,
}
