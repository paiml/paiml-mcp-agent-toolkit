// Message trait implementations and Handler implementations
// for OrchestratorActor request routing.

// Make request types implement Message trait
impl Message for AnalyzeRequest {
    type Result = Result<AgentResponse, AgentError>;
}

impl Message for TransformRequest {
    type Result = Result<AgentResponse, AgentError>;
}

impl Message for ValidateRequest {
    type Result = Result<AgentResponse, AgentError>;
}

impl Handler<AnalyzeRequest> for OrchestratorActor {
    type Result = Result<AgentResponse, AgentError>;

    fn handle(&mut self, _msg: AnalyzeRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // Forward to analyzer actor
        Err(AgentError::ProcessingFailed("Not implemented".to_string()))
    }
}

impl Handler<TransformRequest> for OrchestratorActor {
    type Result = Result<AgentResponse, AgentError>;

    fn handle(&mut self, _msg: TransformRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // Forward to transformer actor
        Err(AgentError::ProcessingFailed("Not implemented".to_string()))
    }
}

impl Handler<ValidateRequest> for OrchestratorActor {
    type Result = Result<AgentResponse, AgentError>;

    fn handle(&mut self, _msg: ValidateRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // Forward to validator actor
        Err(AgentError::ProcessingFailed("Not implemented".to_string()))
    }
}

// We don't need to implement PmatAgent for OrchestratorActor for now
// since the trait requires AgentMessage which our request types don't implement
