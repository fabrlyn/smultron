use async_trait::async_trait;
use ractor::{ActorProcessingErr, ActorRef, SupervisionEvent};
use tracing::info;

use crate::service_finder;

pub struct Debugger;

#[async_trait]
impl ractor::Actor for Debugger {
    type Arguments = ();
    type Msg = String;
    type State = ();

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        _: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        msg: Self::Msg,
        _: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("\n__-- Debugger --__\n{}", msg);

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _: ActorRef<Self::Msg>,
        event: SupervisionEvent,
        _: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        info!("Handling {:?}", event);
        match event {
            SupervisionEvent::ActorStarted(_) => {}
            SupervisionEvent::ActorTerminated(actor, state, _) => {
                info!("actor: {:?}", actor);

                let Some(mut state) = state else {
                    return Ok(());
                };
                info!("state: {:?}", state.take::<service_finder::State>());
            }
            SupervisionEvent::ActorPanicked(_, _) => {}
            SupervisionEvent::ProcessGroupChanged(_) => {}
        }
        Ok(())
    }
}
