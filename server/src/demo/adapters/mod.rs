pub mod cli;
pub mod http;
pub mod mcp;
pub mod tui;

pub use cli::{CliDemoAdapter, CliRequest, CliResponse};
pub use http::{HttpDemoAdapter, HttpRequest, HttpResponse};
pub use mcp::{McpDemoAdapter, McpRequest, McpResponse};
pub use tui::{TuiDemoAdapter, TuiRequest, TuiResponse};

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
