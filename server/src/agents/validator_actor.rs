use actix::prelude::*;
use super::messages::ValidateMessage;
use super::{AgentResponse, AgentError};
use crate::modules::validator::{ValidatorModule, ValidatorImpl};

pub struct ValidatorActor {
    validator: ValidatorImpl,
}

impl Default for ValidatorActor {
    fn default() -> Self {
        Self {
            validator: ValidatorImpl::new(),
        }
    }
}

impl Actor for ValidatorActor {
    type Context = Context<Self>;
}

impl Handler<ValidateMessage> for ValidatorActor {
    type Result = ResponseActFuture<Self, Result<AgentResponse, AgentError>>;

    fn handle(&mut self, msg: ValidateMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let validator = self.validator.clone();
        let metrics = msg.metrics.clone();
        let thresholds = msg.thresholds.clone();

        Box::pin(
            async move {
                let result = validator.validate(&metrics, &thresholds).await;
                Ok(AgentResponse::Validated(result))
            }
            .into_actor(self)
        )
    }
}