use actix::prelude::*;
use super::messages::TransformMessage;
use super::{AgentResponse, AgentError};
use crate::modules::transformer::{TransformerModule, TransformerImpl};

pub struct TransformerActor {
    transformer: TransformerImpl,
}

impl Default for TransformerActor {
    fn default() -> Self {
        Self {
            transformer: TransformerImpl::new(),
        }
    }
}

impl Actor for TransformerActor {
    type Context = Context<Self>;
}

impl Handler<TransformMessage> for TransformerActor {
    type Result = ResponseActFuture<Self, Result<AgentResponse, AgentError>>;

    fn handle(&mut self, msg: TransformMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let transformer = self.transformer.clone();
        let code = msg.code.clone();

        Box::pin(
            async move {
                let result = transformer.transform(&code).await
                    .map_err(|e| AgentError::ProcessingFailed(e.to_string()))?;
                Ok(AgentResponse::Transformed(result))
            }
            .into_actor(self)
        )
    }
}