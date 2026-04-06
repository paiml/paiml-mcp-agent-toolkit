pub struct QualityGateSupervisor {
    analyzer: Addr<AnalyzerActor>,
    _transformer: Addr<TransformerActor>,
    validator: Addr<ValidatorActor>,
}

impl QualityGateSupervisor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(true, "contract: restarting");
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
