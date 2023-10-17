use async_graphql::{Context, Object};

use crate::{application::port::Port, domain::get_devices::get_devices};

use super::model::Device;

pub struct Root;

#[Object(name = "Query")]
impl Root {
    async fn echo(&self, input: String) -> String {
        format!("{} {}", input, input)
    }

    async fn devices(&self, ctx: &Context<'_>) -> Vec<Device> {
        get_devices(ctx.data_unchecked::<Box<dyn Port>>().as_ref())
            .await
            .into_iter()
            .map(Into::into)
            .collect()
    }
}
