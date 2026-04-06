/// CLI Adapter implementation
pub struct CliAdapter;

#[async_trait]
impl ProtocolAdapter for CliAdapter {
    type Request = CliRequest;
    type Response = CliResponse;

    fn decode(&self, raw: &[u8]) -> Result<UnifiedRequest, ProtocolError> {
        debug_assert!(!raw.is_empty(), "raw must not be empty");
        let cli_request: CliRequest = serde_json::from_slice(raw)?;

        let command = cli_request.command.clone();
        let args = cli_request.args.clone();

        // Map CLI command to Operation
        let operation = match command.as_str() {
            "analyze" => match cli_request.subcommand.as_deref() {
                Some("complexity") => {
                    Operation::AnalyzeComplexity(serde_json::from_value(args.clone())?)
                }
                Some("satd") => Operation::AnalyzeSatd(serde_json::from_value(args.clone())?),
                Some("dead-code") => {
                    Operation::AnalyzeDeadCode(serde_json::from_value(args.clone())?)
                }
                _ => return Err(ProtocolError::UnknownMethod(command)),
            },
            "quality-gate" => Operation::QualityGate(serde_json::from_value(args.clone())?),
            "refactor" => match cli_request.subcommand.as_deref() {
                Some("start") => Operation::RefactorStart(serde_json::from_value(args.clone())?),
                Some("next") => Operation::RefactorNext(serde_json::from_value(args.clone())?),
                Some("stop") => Operation::RefactorStop(serde_json::from_value(args.clone())?),
                _ => return Err(ProtocolError::UnknownMethod(command)),
            },
            _ => return Err(ProtocolError::UnknownMethod(command)),
        };

        Ok(UnifiedRequest {
            operation,
            params: args,
            context: RequestContext::new("cli"),
        })
    }

    fn encode(&self, response: UnifiedResponse) -> Result<Vec<u8>, ProtocolError> {
        let cli_response = CliResponse {
            success: response.error.is_none(),
            result: response.result,
            error: response.error.map(|e| e.message),
        };

        Ok(serde_json::to_vec(&cli_response)?)
    }

    async fn handle(&self, _request: Self::Request) -> Self::Response {
        // This would be implemented to process the request
        // For now, return a placeholder response
        CliResponse {
            success: true,
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        }
    }
}
