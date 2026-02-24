#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTemplatesArgs {
    pub toolchain: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTemplateArgs {
    pub resource_uri: String,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaffoldProjectArgs {
    pub toolchain: String,
    pub templates: Vec<String>,
    pub parameters: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTemplatesArgs {
    pub query: String,
    pub toolchain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptGetParams {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
