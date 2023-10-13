use async_graphql::{Context, Object};

use super::{model::Device, Handler};

pub struct Root;

#[Object(name = "Query")]
impl Root {
    async fn echo(&self, input: String) -> String {
        format!("{} {}", input, input)
    }

    async fn devices(&self, ctx: &Context<'_>) -> Vec<Device> {
        ctx.data_unchecked::<Box<dyn Handler>>().get_devices().await
    }
}
