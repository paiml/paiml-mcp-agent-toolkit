// Build Performance Checks (4)
// ============================================================================

fn check_cargo_config(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let config_toml = project_path.join(".cargo").join("config.toml");
    let config_legacy = project_path.join(".cargo").join("config");

    let (status, score, message) = if config_toml.exists() {
        (
            HealthStatus::Green,
            5.0,
            ".cargo/config.toml present".to_string(),
        )
    } else if config_legacy.exists() {
        (
            HealthStatus::Yellow,
            3.0,
            ".cargo/config exists (rename to config.toml)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No .cargo/config.toml - consider adding build config".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Cargo Config".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_incremental_builds(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // Incremental is on by default for dev builds
    let config_toml = project_path.join(".cargo").join("config.toml");
    let cargo_toml = project_path.join("Cargo.toml");

    // Check if explicitly disabled
    let config_content = std::fs::read_to_string(&config_toml).unwrap_or_default();
    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if config_content.contains("incremental = false")
        || cargo_content.contains("incremental = false")
    {
        (
            HealthStatus::Red,
            0.0,
            "Incremental builds disabled".to_string(),
        )
    } else if config_content.contains("incremental = true") {
        (
            HealthStatus::Green,
            5.0,
            "Incremental builds explicitly enabled".to_string(),
        )
    } else {
        (
            HealthStatus::Green,
            4.0,
            "Incremental builds enabled (default)".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Incremental Builds".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_codegen_units(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("codegen-units = 1") {
        (
            HealthStatus::Green,
            5.0,
            "codegen-units = 1 (maximum optimization)".to_string(),
        )
    } else if content.contains("codegen-units") {
        (
            HealthStatus::Yellow,
            3.0,
            "codegen-units configured (not optimal)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No codegen-units - add codegen-units = 1 to [profile.release]".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Codegen Units".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_build_system(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let makefile = project_path.join("Makefile");
    let justfile = project_path.join("justfile");
    let justfile_cap = project_path.join("Justfile");
    let build_rs = project_path.join("build.rs");

    let mut found = Vec::new();
    if makefile.exists() {
        found.push("Makefile");
    }
    if justfile.exists() || justfile_cap.exists() {
        found.push("justfile");
    }
    if build_rs.exists() {
        found.push("build.rs");
    }

    let (status, score, message) = if !found.is_empty() {
        (
            HealthStatus::Green,
            5.0,
            format!("Build automation: {}", found.join(", ")),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No build automation - add Makefile or justfile".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Build System".to_string(),
        category: "Build Performance".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Code Quality Checks (4)
// ============================================================================

fn check_clippy_config(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let clippy_toml = project_path.join(".clippy.toml");
    let clippy_toml_alt = project_path.join("clippy.toml");
    let cargo_toml = project_path.join("Cargo.toml");
    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_workspace_clippy = cargo_content.contains("[workspace.lints.clippy]")
        || cargo_content.contains("[lints.clippy]");

    let (status, score, message) = if clippy_toml.exists() || clippy_toml_alt.exists() {
        (
            HealthStatus::Green,
            5.0,
            ".clippy.toml configured".to_string(),
        )
    } else if has_workspace_clippy {
        (
            HealthStatus::Green,
            5.0,
            "Clippy lints in Cargo.toml".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No Clippy config - add [lints.clippy] or .clippy.toml".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Clippy Config".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_rustfmt_config(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let rustfmt_toml = project_path.join("rustfmt.toml");
    let rustfmt_toml_alt = project_path.join(".rustfmt.toml");

    let (status, score, message) = if rustfmt_toml.exists() || rustfmt_toml_alt.exists() {
        (HealthStatus::Green, 5.0, "rustfmt configured".to_string())
    } else {
        (
            HealthStatus::Yellow,
            2.0,
            "No rustfmt.toml - using defaults".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Rustfmt Config".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_tests_present(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let tests_dir = project_path.join("tests");
    let src_dir = project_path.join("src");

    // Check for tests/ directory
    let has_integration_tests = tests_dir.exists() && tests_dir.is_dir();

    // Check for #[test] or #[cfg(test)] in src/
    let has_unit_tests = if src_dir.exists() {
        has_test_annotations(&src_dir)
    } else {
        false
    };

    let (status, score, message) = if has_integration_tests && has_unit_tests {
        (
            HealthStatus::Green,
            5.0,
            "Both unit and integration tests present".to_string(),
        )
    } else if has_integration_tests || has_unit_tests {
        (HealthStatus::Yellow, 3.0, "Some tests present".to_string())
    } else {
        (HealthStatus::Red, 0.0, "No tests found".to_string())
    };

    DiagnosticCheck {
        name: "Tests Present".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_readme(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let readme = project_path.join("README.md");
    let readme_alt = project_path.join("README");

    let (status, score, message) = if readme.exists() {
        let content = std::fs::read_to_string(&readme).unwrap_or_default();
        if content.len() > 500 {
            (
                HealthStatus::Green,
                5.0,
                "Comprehensive README.md present".to_string(),
            )
        } else {
            (
                HealthStatus::Yellow,
                3.0,
                "README.md exists but is short".to_string(),
            )
        }
    } else if readme_alt.exists() {
        (
            HealthStatus::Yellow,
            2.0,
            "README exists (consider README.md)".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No README - add README.md".to_string(),
        )
    };

    DiagnosticCheck {
        name: "README".to_string(),
        category: "Code Quality".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

