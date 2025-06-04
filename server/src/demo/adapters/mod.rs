pub mod cli;
pub mod http;
pub mod mcp;

pub use cli::{CliDemoAdapter, CliRequest, CliResponse};
pub use http::{HttpDemoAdapter, HttpRequest, HttpResponse};
pub use mcp::{McpDemoAdapter, McpRequest, McpResponse};
