// Helper structures for protocol adapters

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub command: String,
    pub subcommand: Option<String>,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}
