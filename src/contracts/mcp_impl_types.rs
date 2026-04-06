/// MCP operation result for consistent error handling (TICKET-PMAT-6022)
#[derive(Debug, Serialize, Deserialize)]
pub struct McpOperationResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<Vec<String>>,
}

impl McpOperationResult {
    /// Create a success result
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            error_details: None,
        }
    }

    /// Create an error result
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn error(message: String, details: Option<Vec<String>>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            error_details: details,
        }
    }

    /// Create an error result from an anyhow error
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_error(err: anyhow::Error) -> Self {
        let error_chain: Vec<String> = err
            .chain()
            .map(|e| e.to_string())
            .collect();

        Self {
            success: false,
            data: None,
            error: Some(err.to_string()),
            error_details: if error_chain.len() > 1 {
                Some(error_chain)
            } else {
                None
            },
        }
    }
}
