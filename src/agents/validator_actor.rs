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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_actor_default() {
        let actor = ValidatorActor::default();
        // Verify the validator is initialized (existence test)
        let _ = actor.validator;
    }

    #[actix_rt::test]
    async fn test_validator_actor_starts() {
        let actor = ValidatorActor::default();
        let addr = actor.start();
        assert!(addr.connected());
    }
}
