//! Claude Code 3.7 Skill Integration — `.claude/skills/pmat-coverage/SKILL.md`
//!
//! This example demonstrates the R5 agent-ecosystem integration pattern:
//! generating a Claude Code *skill* file that wraps `pmat query --coverage-gaps`
//! as a single-keystroke workflow. Claude Code skills live under
//! `.claude/skills/<name>/SKILL.md` and combine YAML frontmatter with a markdown
//! body the model consults when the skill name is invoked (`/pmat-coverage`).
//!
//! Reference: `.bench-results/claude-code-integration.md` recommendation #3.
//!
//! Run with: `cargo run --example claude_code_skill`
//!
//! # What this example does
//!
//! 1. Programmatically builds a complete SKILL.md payload (frontmatter + body)
//!    and prints it to stdout — copy-paste into `.claude/skills/pmat-coverage/SKILL.md`.
//! 2. Parses a sample in-memory SKILL.md and extracts the YAML frontmatter,
//!    demonstrating the round-trip shape Claude Code expects.
//!
//! No new dependencies: uses `std` + a hand-rolled minimal YAML emitter (the
//! frontmatter surface is small and fixed).
//!
//! # Related CLI
//!
//! ```bash
//! pmat query --coverage-gaps --rank-by impact --limit 20
//! pmat query --coverage-gaps --limit 30 --exclude-tests
//! ```

use std::collections::BTreeMap;

fn main() {
    println!("=== Claude Code 3.7 Skill Generator: pmat-coverage ===\n");

    // Example 1: Generate SKILL.md from scratch
    println!("# Example 1: Emitted .claude/skills/pmat-coverage/SKILL.md\n");
    println!("{}", "-".repeat(60));
    let skill_md = build_coverage_skill();
    println!("{skill_md}");
    println!("{}", "-".repeat(60));

    // Example 2: Parse a sample SKILL.md and dump its frontmatter keys
    println!("\n# Example 2: Parse existing SKILL.md frontmatter\n");
    println!("{}", "-".repeat(60));
    let sample = sample_existing_skill();
    let frontmatter = parse_frontmatter(&sample);
    for (k, v) in &frontmatter {
        println!("  {k}: {v}");
    }
    println!("{}", "-".repeat(60));

    println!("\nTo install:");
    println!("  mkdir -p .claude/skills/pmat-coverage");
    println!("  cargo run --example claude_code_skill > .claude/skills/pmat-coverage/SKILL.md");
    println!("  # (strip the Example 2 section before saving)");
}

/// Build a complete SKILL.md for the pmat-coverage skill.
fn build_coverage_skill() -> String {
    let mut fm: BTreeMap<&str, String> = BTreeMap::new();
    fm.insert("name", "pmat-coverage".to_string());
    fm.insert(
        "description",
        "Rank uncovered functions by ROI impact using pmat query --coverage-gaps".to_string(),
    );
    fm.insert("disable-model-invocation", "true".to_string());
    fm.insert("effort", "low".to_string());
    fm.insert("context", "fork".to_string());
    fm.insert(
        "allowed-tools",
        "[Bash(pmat query *), pmat_query_code]".to_string(),
    );
    fm.insert("paths", "[server/src/**]".to_string());

    let body = r#"
# pmat-coverage

Find uncovered functions in the current workspace ranked by impact score
(missed_lines * pagerank / complexity). Returns the top 20 highest-ROI
targets so the next test-writing loop spends effort where it matters most.

## Workflow

1. Invoke `pmat query --coverage-gaps --rank-by impact --limit 20` to
   materialise the gap list.
2. For each candidate, re-run `pmat query "<function>" --include-source --limit 1`
   to pull the full body (never use `cat`/`grep` — quality context is lost).
3. Write tests targeting uncovered lines, re-run the skill to verify deltas.

## Live Command

!`pmat query --coverage-gaps --rank-by impact --limit 20`

## Notes

- `disable-model-invocation: true` keeps this deterministic — the shell
  command is executed by the Claude Code harness, not paraphrased.
- Requires an up-to-date coverage profile (`make coverage`) and
  `.pmat/context.db` (auto-built on first `pmat query`).
- To narrow scope, restrict via `--exclude-tests` or `pmat query
  "<concept>" --coverage --uncovered-only`.
"#;

    render_skill(&fm, body.trim_start())
}

/// Render a SKILL.md as `---\n<yaml>\n---\n\n<body>`.
fn render_skill(frontmatter: &BTreeMap<&str, String>, body: &str) -> String {
    let mut out = String::from("---\n");
    for (k, v) in frontmatter {
        // Keep the emitter intentionally tiny — all values are short
        // scalars or inline flow sequences. No multi-line scalars used.
        out.push_str(k);
        out.push_str(": ");
        out.push_str(v);
        out.push('\n');
    }
    out.push_str("---\n\n");
    out.push_str(body);
    out
}

/// A representative SKILL.md as the harness would emit it, for Example 2.
fn sample_existing_skill() -> String {
    String::from(
        "---\n\
         name: pmat-faults\n\
         description: Surface unwrap/panic/unsafe fault patterns via pmat query --faults\n\
         disable-model-invocation: true\n\
         allowed-tools: [Bash(pmat query *)]\n\
         ---\n\
         \n\
         # pmat-faults\n\
         \n\
         Runs `pmat query --faults --exclude-tests --limit 30` to surface\n\
         mutation-testing targets and boundary conditions.\n",
    )
}

/// Extract YAML frontmatter from a SKILL.md. Returns `key -> value` pairs
/// preserving insertion order (BTreeMap = alphabetical for display clarity).
fn parse_frontmatter(md: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let trimmed = md.trim_start();
    if !trimmed.starts_with("---") {
        return out;
    }
    let after_first = &trimmed[3..].trim_start_matches('\n');
    let Some(end) = after_first.find("\n---") else {
        return out;
    };
    let yaml = &after_first[..end];
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            out.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_has_frontmatter_markers() {
        let md = build_coverage_skill();
        assert!(md.starts_with("---\n"));
        assert!(md.contains("\n---\n\n"));
        assert!(md.contains("pmat query --coverage-gaps"));
        assert!(md.contains("disable-model-invocation: true"));
    }

    #[test]
    fn round_trip_parse() {
        let md = build_coverage_skill();
        let fm = parse_frontmatter(&md);
        assert_eq!(fm.get("name").map(String::as_str), Some("pmat-coverage"));
        assert!(fm.contains_key("allowed-tools"));
    }

    #[test]
    fn sample_parse_recovers_keys() {
        let fm = parse_frontmatter(&sample_existing_skill());
        assert_eq!(fm.get("name").map(String::as_str), Some("pmat-faults"));
    }
}
