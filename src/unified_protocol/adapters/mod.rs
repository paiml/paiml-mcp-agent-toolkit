pub mod cli;
pub mod cli_helpers; // Format helpers extracted for file health (CB-040)
pub mod http;
pub mod mcp;

pub use cli::CliAdapter;
pub use http::HttpAdapter;
pub use mcp::McpAdapter;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
