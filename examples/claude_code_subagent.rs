//! Claude Code 3.7 Subagent Integration — `.claude/agents/code-quality.md`
//!
//! This example demonstrates the R5 agent-ecosystem integration pattern:
//! generating a Claude Code *subagent* manifest scoped to pmat-driven code
//! quality reviews. Subagents isolate context (sparse worktrees + tool
//! whitelists) so the parent session can delegate audits without pulling in
//! the full repo or losing token budget to transcript churn.
//!
//! Reference: `.bench-results/claude-code-integration.md` recommendation #1.
//!
//! Run with: `cargo run --example claude_code_subagent`
//!
//! # What this example does
//!
//! 1. Builds a complete `.claude/agents/code-quality.md` (frontmatter + body).
//! 2. Shows a `worktree.sparsePaths` allowlist limited to `server/src/**` so
//!    the subagent only sees production code.
//! 3. Demonstrates the tools whitelist: `Bash(pmat *)`, `pmat_query_code`,
//!    `pmat_get_function` — all other tools (including network, shell
//!    escapes) are denied by default.
//!
//! No new dependencies — pure `std` string construction.
//!
//! # Related reading
//!
//! - Claude Code 3.7 subagent spec: sparse worktrees, `@mention` typeahead,
//!   `initialPrompt`, model pinning.
//! - pmat MCP tools: `pmat_query_code`, `pmat_get_function`,
//!   `pmat_find_similar`, `pmat_index_stats`.

fn main() {
    println!("=== Claude Code 3.7 Subagent Generator: code-quality ===\n");

    println!("# Emitted .claude/agents/code-quality.md\n");
    println!("{}", "-".repeat(60));
    println!("{}", build_code_quality_subagent());
    println!("{}", "-".repeat(60));

    println!("\nWhy this shape?");
    for note in rationale() {
        println!("  - {note}");
    }

    println!("\nTo install:");
    println!("  mkdir -p .claude/agents");
    println!("  cargo run --example claude_code_subagent > .claude/agents/code-quality.md");
    println!("  # (trim the trailing 'Why this shape?' section before saving)");
}

/// Build a complete subagent manifest scoped to pmat-driven code review.
fn build_code_quality_subagent() -> String {
    let frontmatter = r#"---
name: code-quality
description: Audit code for defects, coverage gaps, and quality violations using pmat. Delegates reviews without flooding parent context.
model: claude-haiku-4
initialPrompt: |
  You are a code-quality auditor. Use pmat_query_code and `pmat query` to
  surface fault patterns, coverage gaps, and complexity hotspots. Report
  findings as a short bulleted list with file:line refs — never dump full
  function bodies unless explicitly asked.
worktree:
  sparsePaths:
    - server/src/**
    - Cargo.toml
    - .pmat/context.db
  fetch: lazy
tools:
  allow:
    - Bash(pmat *)
    - Bash(cargo test *)
    - pmat_query_code
    - pmat_get_function
    - pmat_find_similar
    - pmat_index_stats
  deny:
    - Bash(git push *)
    - Bash(cargo publish *)
    - Bash(rm *)
    - WebFetch
    - WebSearch
env:
  PMAT_MIN_GRADE: B
  PMAT_COVERAGE_MIN: "90"
---"#;

    let body = r#"
# code-quality subagent

Context-isolated auditor for pmat-managed Rust repos. Use this subagent to
delegate review tasks so the parent session keeps a small context window.

## Canonical workflows

### 1. Fault-pattern sweep

```bash
pmat query --faults --exclude-tests --limit 30
```

Returns functions with `unwrap()`, `panic!`, `unsafe`, or other fault
markers. Use as a mutation-testing or hardening backlog.

### 2. Coverage-gap triage

```bash
pmat query --coverage-gaps --rank-by impact --limit 20
```

Sorts uncovered functions by ROI (missed_lines * pagerank / complexity).
Top entries are the highest-leverage test targets.

### 3. Hot-spot detection

```bash
pmat query "error handling" --churn --duplicates --min-grade C
```

Finds volatile + duplicated + low-grade code — prime refactor candidates.

## Output contract

- Always cite `file:line` for every finding.
- Never dump function bodies larger than 40 lines; link to
  `pmat query "<name>" --include-source --limit 1` instead.
- If a batch returns 0 results, say so explicitly — silence is ambiguous.

## Escalation

If a finding blocks `make lint`, `cargo test`, or `pmat comply check`, raise
it to the parent session via `@mention` rather than attempting autonomous
fixes. Hard write operations (commits, pushes, publishes) are denied by
policy — the parent session owns state mutations.
"#;

    format!("{frontmatter}\n{body}", body = body.trim_start())
}

/// Short rationale bullets explaining the trade-offs of each frontmatter
/// field — useful for reviewers unfamiliar with Claude Code 3.7 internals.
fn rationale() -> Vec<&'static str> {
    vec![
        "model: claude-haiku-4 — audits rarely need deep reasoning; Haiku is ~5x cheaper.",
        "worktree.sparsePaths limits what the subagent checks out to server/src/** + Cargo.toml.",
        "fetch: lazy defers file content reads until a tool actually references them.",
        "tools.allow restricts the subagent to pmat + cargo-test + read-only MCP tools.",
        "tools.deny explicitly blocks git push / cargo publish / rm so reviews cannot mutate.",
        "env injects PMAT_MIN_GRADE and PMAT_COVERAGE_MIN so thresholds are discoverable.",
        "initialPrompt sets output-contract expectations before the first user message.",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_has_frontmatter() {
        let md = build_code_quality_subagent();
        assert!(md.starts_with("---\n"));
        assert!(md.contains("\n---\n"));
    }

    #[test]
    fn manifest_whitelists_pmat_tools() {
        let md = build_code_quality_subagent();
        assert!(md.contains("Bash(pmat *)"));
        assert!(md.contains("pmat_query_code"));
        assert!(md.contains("pmat_get_function"));
    }

    #[test]
    fn manifest_restricts_worktree() {
        let md = build_code_quality_subagent();
        assert!(md.contains("sparsePaths"));
        assert!(md.contains("server/src/**"));
    }

    #[test]
    fn manifest_denies_destructive_ops() {
        let md = build_code_quality_subagent();
        assert!(md.contains("Bash(git push *)"));
        assert!(md.contains("Bash(cargo publish *)"));
    }

    #[test]
    fn rationale_is_non_empty() {
        assert!(!rationale().is_empty());
    }
}
