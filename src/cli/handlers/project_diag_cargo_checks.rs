// ============================================================================
// Cargo Config Checks (6)
// ============================================================================

fn check_edition_2021(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("edition = \"2021\"") {
        (
            HealthStatus::Green,
            5.0,
            "Edition 2021 configured".to_string(),
        )
    } else if content.contains("edition = \"2024\"") {
        (
            HealthStatus::Green,
            5.0,
            "Edition 2024 configured".to_string(),
        )
    } else if content.contains("edition") {
        (
            HealthStatus::Yellow,
            2.0,
            "Older edition configured - consider upgrading to 2021+".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No edition specified - defaults to 2015".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Edition 2021+".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_resolver_v2(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("resolver = \"2\"") {
        (
            HealthStatus::Green,
            5.0,
            "Resolver v2 explicitly configured".to_string(),
        )
    } else if content.contains("edition = \"2021\"") || content.contains("edition = \"2024\"") {
        // Edition 2021+ implies resolver v2
        (
            HealthStatus::Green,
            5.0,
            "Resolver v2 via edition 2021+".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            2.0,
            "Using legacy resolver - add resolver = \"2\"".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Resolver v2".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_dependency_count(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    // Count dependencies
    let mut count = 0;
    let mut in_deps = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]"
            || trimmed == "[dev-dependencies]"
            || trimmed == "[build-dependencies]"
        {
            in_deps = true;
            continue;
        }
        if in_deps && trimmed.starts_with('[') {
            in_deps = false;
        }
        if in_deps && !trimmed.starts_with('#') && trimmed.contains('=') {
            count += 1;
        }
    }

    let (status, score, message) = if count <= 20 {
        (
            HealthStatus::Green,
            5.0,
            format!("{} dependencies (excellent)", count),
        )
    } else if count <= 50 {
        (
            HealthStatus::Yellow,
            3.0,
            format!("{} dependencies (consider reducing)", count),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            format!("{} dependencies (too many)", count),
        )
    };

    DiagnosticCheck {
        name: "Dependencies <= 50".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_lto_enabled(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_lto = content.contains("lto = true")
        || content.contains("lto = \"thin\"")
        || content.contains("lto = \"fat\"");

    let (status, score, message) = if has_lto {
        (
            HealthStatus::Green,
            5.0,
            "LTO enabled for release builds".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "LTO not configured - add lto = true to [profile.release]".to_string(),
        )
    };

    DiagnosticCheck {
        name: "LTO Enabled".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_workspace_lints(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_workspace_lints = content.contains("[workspace.lints");

    let (status, score, message) = if has_workspace_lints {
        (
            HealthStatus::Green,
            5.0,
            "Workspace-level lints configured".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No workspace lints - add [workspace.lints.rust] section".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Workspace Lints".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_workspace_deps(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_workspace_deps = content.contains("[workspace.dependencies]");

    let (status, score, message) = if has_workspace_deps {
        (
            HealthStatus::Green,
            5.0,
            "Workspace dependencies configured".to_string(),
        )
    } else if content.contains("[workspace]") {
        (
            HealthStatus::Yellow,
            2.0,
            "Workspace exists but no shared dependencies".to_string(),
        )
    } else {
        (
            HealthStatus::Skip,
            0.0,
            "Single-crate project (N/A)".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Workspace Deps".to_string(),
        category: "Cargo Config".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Dependencies Checks (3)
// ============================================================================

fn check_target_dir_size(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let target_path = project_path.join("target");

    if !target_path.exists() {
        return DiagnosticCheck {
            name: "Target Dir <= 10GB".to_string(),
            category: "Dependencies".to_string(),
            status: HealthStatus::Green,
            message: "No target directory (clean state)".to_string(),
            score: 5.0,
            max_score: 5.0,
        };
    }

    // Calculate size (best effort)
    let size_bytes = dir_size(&target_path).unwrap_or(0);
    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let (status, score, message) = if size_gb <= 2.0 {
        (
            HealthStatus::Green,
            5.0,
            format!("{:.1} GB (excellent)", size_gb),
        )
    } else if size_gb <= 5.0 {
        (
            HealthStatus::Green,
            4.0,
            format!("{:.1} GB (good)", size_gb),
        )
    } else if size_gb <= 10.0 {
        (
            HealthStatus::Yellow,
            2.0,
            format!("{:.1} GB (consider cargo clean)", size_gb),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            format!("{:.1} GB (run cargo clean)", size_gb),
        )
    };

    DiagnosticCheck {
        name: "Target Dir <= 10GB".to_string(),
        category: "Dependencies".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_cargo_lock(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_lock = project_path.join("Cargo.lock");

    let (status, score, message) = if cargo_lock.exists() {
        (
            HealthStatus::Green,
            5.0,
            "Cargo.lock present (reproducible builds)".to_string(),
        )
    } else {
        (
            HealthStatus::Red,
            0.0,
            "No Cargo.lock - run cargo build to generate".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Cargo.lock Present".to_string(),
        category: "Dependencies".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_audit_config(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let deny_toml = project_path.join("deny.toml");
    let audit_toml = project_path.join(".cargo").join("audit.toml");

    let (status, score, message) = if deny_toml.exists() {
        (
            HealthStatus::Green,
            5.0,
            "cargo-deny configured (deny.toml)".to_string(),
        )
    } else if audit_toml.exists() {
        (
            HealthStatus::Green,
            4.0,
            "cargo-audit configured".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No audit config - add deny.toml for security scanning".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Audit Config".to_string(),
        category: "Dependencies".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

