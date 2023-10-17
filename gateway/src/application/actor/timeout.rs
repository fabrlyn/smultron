use std::time::Duration;

use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef};
use tokio::time::sleep;
use tracing::info;

pub struct Timeout;

#[async_trait]
impl ractor::Actor for Timeout {
    type Arguments = ();
    type Msg = ();
    type State = ();

    async fn pre_start(
        &self,
        a: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        a.send_message(())?;
        Ok(arguments)
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        _: Self::Msg,
        _: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("sleeping");
        sleep(Duration::from_secs(10)).await;
        info!("done sleeping");

        Ok(())
    }
}
