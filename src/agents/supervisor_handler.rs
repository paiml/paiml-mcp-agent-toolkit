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
