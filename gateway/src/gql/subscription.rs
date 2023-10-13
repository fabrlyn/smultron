use futures_util::StreamExt;
use std::future::ready;

use async_graphql::{Context, Subscription};
use futures_util::Stream;
use tokio_stream::wrappers::BroadcastStream;

use super::Handler;

pub struct Root;

#[Subscription(name = "Subscription")]
impl Root {
    async fn device_events(&self, ctx: &Context<'_>) -> impl Stream<Item = String> {
        let subscription = ctx
            .data_unchecked::<Box<dyn Handler>>()
            .subscribe_to_devices()
            .await;

        BroadcastStream::new(subscription).filter_map(|event| {
            ready(match event {
                Ok(e) => Some(e),
                Err(_) => None,
            })
        })
    }
}
