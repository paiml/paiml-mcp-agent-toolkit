pub mod cli;
pub mod http;
pub mod mcp;

pub use cli::CliAdapter;
pub use http::HttpAdapter;
pub use mcp::McpAdapter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
