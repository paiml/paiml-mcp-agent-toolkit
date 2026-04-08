#[derive(Debug, Serialize, Deserialize)]
/// Parameters for tool call.
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
/// Generate template args.
pub struct GenerateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
/// List templates args.
pub struct ListTemplatesArgs {
    pub toolchain: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Parameters for resource read.
pub struct ResourceReadParams {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Validate template args.
pub struct ValidateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Scaffold project args.
pub struct ScaffoldProjectArgs {
    pub toolchain: String,
    pub templates: Vec<String>,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Search templates args.
pub struct SearchTemplatesArgs {
    pub query: String,
    pub toolchain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Parameters for prompt get.
pub struct PromptGetParams {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Prompt.
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Prompt argument.
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
