use super::messages::AnalyzeMessage;
use super::{AgentError, AgentResponse};
use crate::modules::analyzer::{AnalyzerImpl, AnalyzerModule, Metrics};
use actix::prelude::*;
use std::collections::HashMap;

pub struct AnalyzerActor {
    analyzer: AnalyzerImpl,
    complexity_cache: HashMap<String, Metrics>,
    max_cache_size: usize,
}

impl Default for AnalyzerActor {
    fn default() -> Self {
        Self {
            analyzer: AnalyzerImpl::new(),
            complexity_cache: HashMap::new(),
            max_cache_size: 100,
        }
    }
}

impl Actor for AnalyzerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.set_mailbox_capacity(1024);
    }
}

impl Handler<AnalyzeMessage> for AnalyzerActor {
    type Result = ResponseActFuture<Self, Result<AgentResponse, AgentError>>;

    fn handle(&mut self, msg: AnalyzeMessage, _ctx: &mut Context<Self>) -> Self::Result {
        // Check cache first
        if let Some(cached) = self.complexity_cache.get(&msg.code) {
            return Box::pin(fut::ready(Ok(AgentResponse::Analyzed(cached.clone()))));
        }

        let analyzer = self.analyzer.clone();
        let code = msg.code.clone();

        Box::pin(
            async move {
                let metrics = analyzer
                    .analyze(&code)
                    .await
                    .map_err(|e| AgentError::ProcessingFailed(e.to_string()))?;
                Ok(metrics)
            }
            .into_actor(self)
            .map(move |result, actor, _ctx| {
                match result {
                    Ok(metrics) => {
                        // Cache the result
                        if actor.complexity_cache.len() >= actor.max_cache_size {
                            actor.complexity_cache.clear();
                        }
                        actor
                            .complexity_cache
                            .insert(msg.code.clone(), metrics.clone());
                        Ok(AgentResponse::Analyzed(metrics))
                    }
                    Err(e) => Err(e),
                }
            }),
        )
    }
}
