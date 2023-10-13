use async_graphql::SimpleObject;

use crate::device_manager;

#[derive(SimpleObject)]
pub struct Device {
    pub name: String,
}

#[derive(Clone)]
pub struct Api {
    pub device_manager: device_manager::Actor,
}
