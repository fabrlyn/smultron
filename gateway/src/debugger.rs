use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef};
use tracing::info;

pub struct Debugger;

#[async_trait]
impl ractor::Actor for Debugger {
    type Arguments = ActorRef<()>;
    type Msg = String;
    type State = ActorRef<()>;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(arguments)
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("__-- Debugger --__\n{}", msg);

        state.stop(None);

        Ok(())
    }
}
