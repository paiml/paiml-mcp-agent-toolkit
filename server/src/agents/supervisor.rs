use super::analyzer_actor::AnalyzerActor;
use super::messages::{AnalyzeMessage, ValidateMessage};
use super::transformer_actor::TransformerActor;
use super::validator_actor::ValidatorActor;
use super::{AgentError, AgentResponse};
use actix::prelude::*;

pub struct QualityGateSupervisor {
    analyzer: Addr<AnalyzerActor>,
    _transformer: Addr<TransformerActor>,
    validator: Addr<ValidatorActor>,
}

impl QualityGateSupervisor {
    pub fn new(
        analyzer: Addr<AnalyzerActor>,
        transformer: Addr<TransformerActor>,
        validator: Addr<ValidatorActor>,
    ) -> Self {
        Self {
            analyzer,
            _transformer: transformer,
            validator,
        }
    }
}

impl Actor for QualityGateSupervisor {
    type Context = Context<Self>;
}

impl Supervised for QualityGateSupervisor {
    fn restarting(&mut self, _ctx: &mut Context<Self>) {
        tracing::info!("QualityGateSupervisor restarting");
    }
}

#[derive(Message)]
#[rtype(result = "Result<ValidationResult, AgentError>")]
pub struct ValidateCode {
    pub code: String,
    pub thresholds: crate::modules::validator::Thresholds,
}

pub struct ValidationResult {
    pub passed: bool,
    pub metrics: crate::modules::analyzer::Metrics,
    pub validation: crate::modules::validator::ValidationResult,
}

impl Handler<ValidateCode> for QualityGateSupervisor {
    type Result = ResponseFuture<Result<ValidationResult, AgentError>>;

    fn handle(&mut self, msg: ValidateCode, _ctx: &mut Context<Self>) -> Self::Result {
        let analyzer = self.analyzer.clone();
        let validator = self.validator.clone();

        Box::pin(async move {
            // Step 1: Analyze code
            let analyze_msg = AnalyzeMessage {
                code: msg.code,
                priority: super::Priority::Normal,
            };

            let analyze_result = analyzer
                .send(analyze_msg)
                .await
                .map_err(|e| AgentError::CommunicationFailed(e.to_string()))?;

            let metrics = match analyze_result? {
                AgentResponse::Analyzed(m) => m,
                _ => {
                    return Err(AgentError::ProcessingFailed(
                        "Unexpected response".to_string(),
                    ))
                }
            };

            // Step 2: Validate metrics
            let validate_msg = ValidateMessage {
                metrics: metrics.clone(),
                thresholds: msg.thresholds,
                priority: super::Priority::Normal,
            };

            let validate_result = validator
                .send(validate_msg)
                .await
                .map_err(|e| AgentError::CommunicationFailed(e.to_string()))?;

            let validation = match validate_result? {
                AgentResponse::Validated(v) => v,
                _ => {
                    return Err(AgentError::ProcessingFailed(
                        "Unexpected response".to_string(),
                    ))
                }
            };

            Ok(ValidationResult {
                passed: validation.passed,
                metrics,
                validation,
            })
        })
    }
}
