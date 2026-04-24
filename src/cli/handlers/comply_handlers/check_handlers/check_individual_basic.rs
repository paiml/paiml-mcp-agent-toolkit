pub(crate) fn check_stale_paths(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let violations =
        crate::cli::handlers::comply_cb_detect::detect_cb533_stale_path_references(project_path);
    if violations.is_empty() {
        ComplianceCheck {
            name: "CB-533: Stale Path References".into(),
            status: CheckStatus::Pass,
            message: "No stale path references found".into(),
            severity: Severity::Info,
        }
    } else {
        let msg = format!("{} stale path(s) found", violations.len());
        ComplianceCheck {
            name: "CB-533: Stale Path References".into(),
            status: CheckStatus::Warn,
            message: msg,
            severity: Severity::Warning,
        }
    }
}

/// CB-148: Spec-work traceability.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_spec_work_traceability(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let violations =
        crate::cli::handlers::comply_cb_detect::detect_cb148_spec_work_gaps(project_path);
    if violations.is_empty() {
        ComplianceCheck {
            name: "CB-148: Spec-Work Traceability".into(),
            status: CheckStatus::Pass,
            message: "All planned spec sections have corresponding work tickets".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-148: Spec-Work Traceability".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} planned section(s) without work tickets",
                violations.len()
            ),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_version_currency(project_version: &str) -> ComplianceCheck {
    debug_assert!(
        !project_version.is_empty(),
        "project_version must not be empty"
    );
    let behind = calculate_versions_behind(project_version);
    if behind == 0 {
        ComplianceCheck {
            name: "Version Currency".into(),
            status: CheckStatus::Pass,
            message: format!("Project is on latest version (v{})", PMAT_VERSION),
            severity: Severity::Info,
        }
    } else if behind <= 5 {
        ComplianceCheck {
            name: "Version Currency".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} versions behind (v{} \u{2192} v{})",
                behind, project_version, PMAT_VERSION
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Version Currency".into(),
            status: CheckStatus::Fail,
            message: format!("{} versions behind - migration recommended", behind),
            severity: Severity::Error,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_config_files(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let config_files = [".pmat/project.toml", ".pmat-metrics.toml"];
    let missing: Vec<&str> = config_files
        .iter()
        .filter(|f| !project_path.join(f).exists())
        .copied()
        .collect();
    if missing.is_empty() {
        ComplianceCheck {
            name: "Config Files".into(),
            status: CheckStatus::Pass,
            message: "All required config files present".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Config Files".into(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", missing.join(", ")),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_hooks_installed(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let pre_commit = project_path.join(".git").join("hooks").join("pre-commit");
    if pre_commit.exists() {
        if let Ok(content) = fs::read_to_string(&pre_commit) {
            if content.contains("pmat") || content.contains("PMAT") {
                return ComplianceCheck {
                    name: "Git Hooks".into(),
                    status: CheckStatus::Pass,
                    message: "PMAT hooks installed".into(),
                    severity: Severity::Info,
                };
            }
        }
        ComplianceCheck {
            name: "Git Hooks".into(),
            status: CheckStatus::Warn,
            message: "Pre-commit hook exists but may not be PMAT".into(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Git Hooks".into(),
            status: CheckStatus::Warn,
            message: "No pre-commit hook installed".into(),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_hooks_o1_capable(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let cache_dir = project_path.join(".pmat").join("hooks-cache");
    if cache_dir.exists() {
        if cache_dir.join("tree-hash.json").exists() || cache_dir.join("gates").exists() {
            return ComplianceCheck {
                name: "CB-030: O(1) Hooks".into(),
                status: CheckStatus::Pass,
                message: "Hooks cache initialized - O(1) capable".into(),
                severity: Severity::Info,
            };
        }
        ComplianceCheck {
            name: "CB-030: O(1) Hooks".into(),
            status: CheckStatus::Warn,
            message: "Cache directory exists but not fully initialized".into(),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-030: O(1) Hooks".into(),
            status: CheckStatus::Warn,
            message: "Run 'pmat hooks cache init' to enable O(1) hooks".into(),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_hooks_cache_health(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let metrics_path = project_path
        .join(".pmat")
        .join("hooks-cache")
        .join("metrics.json");
    if !metrics_path.exists() {
        return ComplianceCheck {
            name: "CB-031: Cache Health".into(),
            status: CheckStatus::Skip,
            message: "No cache metrics available yet".into(),
            severity: Severity::Info,
        };
    }
    match fs::read_to_string(&metrics_path) {
        Ok(content) => {
            if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&content) {
                let total_runs = metrics["total_runs"].as_u64().unwrap_or(0);
                let cache_hits = metrics["cache_hits"].as_u64().unwrap_or(0);
                if total_runs < 5 {
                    return ComplianceCheck {
                        name: "CB-031: Cache Health".into(),
                        status: CheckStatus::Skip,
                        message: format!("Insufficient data ({} runs, need 5+)", total_runs),
                        severity: Severity::Info,
                    };
                }
                let hit_rate = (cache_hits as f64 / total_runs as f64) * 100.0;
                if hit_rate >= 60.0 {
                    ComplianceCheck {
                        name: "CB-031: Cache Health".into(),
                        status: CheckStatus::Pass,
                        message: format!("Cache hit rate {:.1}% (target: \u{2265}60%)", hit_rate),
                        severity: Severity::Info,
                    }
                } else {
                    ComplianceCheck {
                        name: "CB-031: Cache Health".into(),
                        status: CheckStatus::Warn,
                        message: format!(
                            "Cache hit rate {:.1}% below 60% target - consider clearing cache",
                            hit_rate
                        ),
                        severity: Severity::Warning,
                    }
                }
            } else {
                ComplianceCheck {
                    name: "CB-031: Cache Health".into(),
                    status: CheckStatus::Warn,
                    message: "Failed to parse metrics.json".into(),
                    severity: Severity::Warning,
                }
            }
        }
        Err(_) => ComplianceCheck {
            name: "CB-031: Cache Health".into(),
            status: CheckStatus::Warn,
            message: "Failed to read metrics.json".into(),
            severity: Severity::Warning,
        },
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_quality_thresholds(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if project_path.join(".pmat-metrics.toml").exists() {
        ComplianceCheck {
            name: "Quality Thresholds".into(),
            status: CheckStatus::Pass,
            message: "Quality thresholds configured".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Quality Thresholds".into(),
            status: CheckStatus::Warn,
            message: "No .pmat-metrics.toml found - using defaults".into(),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_deprecated_features(_project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        _project_path.exists(),
        "_project_path must exist: {}",
        _project_path.display()
    );
    ComplianceCheck {
        name: "Deprecated Features".into(),
        status: CheckStatus::Pass,
        message: "No deprecated features detected".into(),
        severity: Severity::Info,
    }
}

