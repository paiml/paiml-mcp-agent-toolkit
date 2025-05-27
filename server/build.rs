fn main() {
    println!("cargo:rerun-if-changed=src/installer/mod.rs");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    // Verify critical dependencies at build time
    verify_dependency_versions();

    if cfg!(feature = "installer-gen") {
        generate_installer();
    }
}

fn verify_dependency_versions() {
    let lock_content = std::fs::read_to_string("Cargo.lock").expect("Cargo.lock must exist");

    // Critical dependencies for your MCP server
    let critical_deps = [
        "tokio",      // Async runtime
        "serde",      // Serialization
        "handlebars", // Template engine
    ];

    for dep in &critical_deps {
        if !lock_content.contains(&format!("name = \"{}\"", dep)) {
            panic!("Critical dependency {} not found", dep);
        }
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
