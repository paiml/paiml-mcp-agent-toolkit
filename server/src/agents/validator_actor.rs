use super::messages::ValidateMessage;
use super::{AgentError, AgentResponse};
use crate::modules::validator::{ValidatorImpl, ValidatorModule};
use actix::prelude::*;

#[derive(Default)]
pub struct ValidatorActor {
    validator: ValidatorImpl,
}

impl Actor for ValidatorActor {
    type Context = Context<Self>;
}

impl Handler<ValidateMessage> for ValidatorActor {
    type Result = ResponseActFuture<Self, Result<AgentResponse, AgentError>>;

    fn handle(&mut self, msg: ValidateMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let validator = self.validator.clone();
        let metrics = msg.metrics.clone();
        let thresholds = msg.thresholds;

        Box::pin(
            async move {
                let result = validator.validate(&metrics, &thresholds).await;
                Ok(AgentResponse::Validated(result))
            }
            .into_actor(self),
        )
    }
}
