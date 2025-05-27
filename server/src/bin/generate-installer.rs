//! Binary to output the generated installer shell script

#[cfg(feature = "installer-gen")]
use paiml_mcp_agent_toolkit::installer::INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL;

fn main() {
    #[cfg(feature = "installer-gen")]
    {
        println!("{}", INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL);
    }

    #[cfg(not(feature = "installer-gen"))]
    {
        eprintln!("Error: This binary requires the 'installer-gen' feature to be enabled.");
        eprintln!("Run with: cargo run --features installer-gen --bin generate-installer");
        std::process::exit(1);
    }
}
