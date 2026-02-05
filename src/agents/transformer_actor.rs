use super::messages::TransformMessage;
use super::{AgentError, AgentResponse};
use crate::modules::transformer::{TransformerImpl, TransformerModule};
use actix::prelude::*;

#[derive(Default)]
pub struct TransformerActor {
    transformer: TransformerImpl,
}

impl Actor for TransformerActor {
    type Context = Context<Self>;
}

impl Handler<TransformMessage> for TransformerActor {
    type Result = ResponseActFuture<Self, Result<AgentResponse, AgentError>>;

    fn handle(&mut self, msg: TransformMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let transformer = self.transformer.clone();
        let code = msg.code;

        Box::pin(
            async move {
                let result = transformer
                    .transform(&code)
                    .await
                    .map_err(|e| AgentError::ProcessingFailed(e.to_string()))?;
                Ok(AgentResponse::Transformed(result))
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
    fn test_transformer_actor_default() {
        let actor = TransformerActor::default();
        // Verify the transformer is initialized (existence test)
        let _ = actor.transformer;
    }

    #[actix_rt::test]
    async fn test_transformer_actor_starts() {
        let actor = TransformerActor::default();
        let addr = actor.start();
        assert!(addr.connected());
    }
}
