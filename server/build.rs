fn main() {
    println!("cargo:rerun-if-changed=src/installer/mod.rs");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    if cfg!(feature = "installer-gen") {
        generate_installer();
    }
}

#[cfg(feature = "installer-gen")]
fn generate_installer() {
    // Set deterministic environment
    std::env::set_var("SOURCE_DATE_EPOCH", "1234567890");
    std::env::set_var("DETERMINISTIC_BUILD", "1");

    // Build with installer feature to generate the shell script
    println!("cargo:warning=Generating installer shell script...");

    // The installer will be generated at compile time by the procedural macro
    // and will be available as INSTALL_PAIML_MCP_AGENT_TOOLKIT_SHELL constant
}

#[cfg(not(feature = "installer-gen"))]
fn generate_installer() {
    // No-op when feature is not enabled
}
