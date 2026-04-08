// Helper structures for protocol adapters

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Response from http operation.
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Request for cli operation.
pub struct CliRequest {
    pub command: String,
    pub subcommand: Option<String>,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Response from cli operation.
pub struct CliResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}
