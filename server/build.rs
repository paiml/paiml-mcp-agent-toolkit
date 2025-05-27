fn main() {
    println!("cargo:rerun-if-changed=src/installer/mod.rs");
    println!("cargo:rerun-if-changed=../scripts/install.sh");

    // Verify critical dependencies at build time
    verify_dependency_versions();
}

fn verify_dependency_versions() {
    // In a workspace, Cargo.lock is in the parent directory
    let lock_path = if std::path::Path::new("../Cargo.lock").exists() {
        "../Cargo.lock"
    } else {
        "Cargo.lock"
    };
    let lock_content = std::fs::read_to_string(lock_path).expect("Cargo.lock must exist");

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
