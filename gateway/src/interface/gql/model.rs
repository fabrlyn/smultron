use async_graphql::SimpleObject;

use crate::domain;

#[derive(SimpleObject)]
pub struct Device {
    pub name: String,
}

impl From<domain::Device> for Device {
    fn from(value: domain::Device) -> Self {
        Self {
            name: value.service_record.target.into(),
        }
    }
}
