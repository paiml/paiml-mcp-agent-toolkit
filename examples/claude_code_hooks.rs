//! Claude Code 3.7 Hooks Integration — `.claude/settings.json` skeleton
//!
//! This example demonstrates the R5 agent-ecosystem integration pattern:
//! generating a Claude Code project `settings.json` that wires a `PreToolUse`
//! hook to invoke `pmat comply check` before any `git push` escapes the
//! session. Hooks execute in the Claude Code harness (not the model), so they
//! enforce quality gates deterministically even if the model would otherwise
//! skip them.
//!
//! Reference: `.bench-results/claude-code-integration.md` recommendation #5.
//!
//! Run with: `cargo run --example claude_code_hooks`
//!
//! # What this example does
//!
//! 1. Builds a JSON object that wires `PreToolUse` with matcher
//!    `Bash(git push *)` to `pmat comply check`.
//! 2. Pretty-prints the JSON so it can be written directly to
//!    `.claude/settings.json`.
//! 3. Adds permissions + env defaults that complement the hook.
//!
//! Dependencies: only `serde_json` (already in `[dependencies]`).
//!
//! # Related CLI
//!
//! ```bash
//! pmat comply check              # Run the same gate locally
//! pmat comply check --format json
//! ```

use serde_json::{json, Value};

fn main() {
    println!("=== Claude Code 3.7 settings.json Skeleton ===\n");

    let settings = build_settings();
    let pretty =
        serde_json::to_string_pretty(&settings).expect("settings construction is infallible");

    println!("# Emitted .claude/settings.json\n");
    println!("{}", "-".repeat(60));
    println!("{pretty}");
    println!("{}", "-".repeat(60));

    println!("\nHook behaviour:");
    println!("  - Fires before the harness runs any `git push *` bash command.");
    println!("  - Executes `pmat comply check` with a 30s timeout.");
    println!("  - Exit code != 0 blocks the push; stdout is surfaced to the model.");
    println!("  - Bypass for one-off emergencies: `git push --no-verify` is NOT a bypass");
    println!("    for harness hooks — use the Claude Code `--skip-hooks` runtime flag.");

    println!("\nTo install:");
    println!("  mkdir -p .claude");
    println!("  cargo run --example claude_code_hooks > .claude/settings.json");
    println!("  # (strip the explanatory output above before saving)");
}

/// Build a `.claude/settings.json` with a `PreToolUse` compliance gate.
fn build_settings() -> Value {
    json!({
        "version": 1,
        "permissions": {
            "allow": [
                "Bash(pmat *)",
                "Bash(cargo test *)",
                "Bash(cargo check *)",
                "pmat_query_code",
                "pmat_get_function",
                "pmat_find_similar"
            ],
            "deny": [
                "Bash(rm -rf *)",
                "Bash(git push --force *)"
            ]
        },
        "env": {
            "PMAT_MIN_GRADE": "B",
            "PMAT_COVERAGE_MIN": "90",
            "PMAT_FAIL_ON_CONTRADICTION": "1"
        },
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash(git push *)",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "pmat comply check",
                            "timeout": 30,
                            "blockOnFailure": true,
                            "description": "Block pushes when the repo is non-compliant (CB-16xx)."
                        }
                    ]
                },
                {
                    "matcher": "Bash(git commit *)",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "pmat query --faults --exclude-tests --limit 5",
                            "timeout": 10,
                            "blockOnFailure": false,
                            "description": "Advisory: surface fault patterns before each commit."
                        }
                    ]
                }
            ],
            "PostToolUse": [
                {
                    "matcher": "Bash(pmat work start *)",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "pmat comply check --format json",
                            "timeout": 15,
                            "blockOnFailure": false,
                            "description": "Record baseline compliance at the start of a pmat work ticket."
                        }
                    ]
                }
            ]
        },
        "allowManagedHooksOnly": true,
        "channelsEnabled": false,
        "disableSkillShellExecution": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_has_prepush_hook() {
        let settings = build_settings();
        let hooks = settings
            .pointer("/hooks/PreToolUse")
            .and_then(Value::as_array)
            .expect("PreToolUse must be an array");
        let pushes_blocked = hooks
            .iter()
            .any(|h| h.get("matcher").and_then(Value::as_str) == Some("Bash(git push *)"));
        assert!(pushes_blocked, "git push must be gated");
    }

    #[test]
    fn settings_invokes_pmat_comply() {
        let settings = build_settings();
        let text = serde_json::to_string(&settings).unwrap();
        assert!(text.contains("pmat comply check"));
    }

    #[test]
    fn settings_is_serialisable_pretty() {
        let settings = build_settings();
        let pretty = serde_json::to_string_pretty(&settings).unwrap();
        assert!(pretty.contains("\"version\": 1"));
        assert!(pretty.contains("PreToolUse"));
    }

    #[test]
    fn settings_allowlist_includes_pmat() {
        let settings = build_settings();
        let allow = settings
            .pointer("/permissions/allow")
            .and_then(Value::as_array)
            .expect("permissions.allow must be an array");
        assert!(allow.iter().any(|v| v.as_str() == Some("Bash(pmat *)")));
    }

    #[test]
    fn settings_denies_force_push() {
        let settings = build_settings();
        let deny = settings
            .pointer("/permissions/deny")
            .and_then(Value::as_array)
            .expect("permissions.deny must be an array");
        assert!(deny
            .iter()
            .any(|v| v.as_str() == Some("Bash(git push --force *)")));
    }
}
