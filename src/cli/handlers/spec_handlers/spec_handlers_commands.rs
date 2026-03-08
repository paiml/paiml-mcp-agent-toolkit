/// Handle spec score command (S-001)
/// Validates specification with 100-point Popperian score
pub async fn handle_spec_score(
    spec_path: &Path,
    format: SpecOutputFormat,
    output: Option<&Path>,
    verbose: bool,
) -> anyhow::Result<()> {
    let parser = SpecParser::new();
    let spec = parser.parse_file(spec_path)?;

    // Simple score calculation (will be enhanced with full validation)
    let score = calculate_spec_score(&spec);

    let output_text = match format {
        SpecOutputFormat::Text => format_spec_score_text(&spec, score, verbose),
        SpecOutputFormat::Json => format_spec_score_json(&spec, score)?,
        SpecOutputFormat::Markdown => format_spec_score_markdown(&spec, score),
    };

    if let Some(output_path) = output {
        use crate::cli::colors as c;
        fs::write(output_path, &output_text)?;
        println!("{}", c::pass(&format!("Spec score written to {}", c::path(&output_path.display().to_string()))));
    } else {
        println!("{}", output_text);
    }

    // Fail if below 95 threshold (S-002)
    if score < 95.0 {
        use crate::cli::colors as c;
        println!(
            "\n{}",
            c::warn(&format!(
                "Spec score {:.1} is below 95 threshold. Run `pmat spec comply` to fix.",
                score
            ))
        );
        std::process::exit(1);
    }

    Ok(())
}

/// Handle spec comply command (S-003)
/// Auto-fixes spec issues to meet 95-point threshold
pub async fn handle_spec_comply(
    spec_path: &Path,
    dry_run: bool,
    _format: SpecOutputFormat,
) -> anyhow::Result<()> {
    let parser = SpecParser::new();
    let spec = parser.parse_file(spec_path)?;

    let mut fixes: Vec<String> = Vec::new();

    // Check minimum requirements and suggest fixes
    if spec.issue_refs.is_empty() {
        fixes.push(
            "- Add issue_refs in YAML frontmatter (e.g., issue_refs: [\"#123\"])".to_string(),
        );
    }

    if spec.code_examples.len() < 5 {
        fixes.push(format!(
            "- Add {} more code examples (minimum 5 required)",
            5 - spec.code_examples.len()
        ));
    }

    if spec.acceptance_criteria.len() < 10 {
        fixes.push(format!(
            "- Add {} more acceptance criteria (minimum 10 required)",
            10 - spec.acceptance_criteria.len()
        ));
    }

    // Count unique citations [1], [2], etc. from full spec content
    let citation_count = {
        let mut seen = std::collections::HashSet::new();
        let re = regex::Regex::new(r"\[(\d+)\]").expect("internal error");
        for caps in re.captures_iter(&spec.raw_content) {
            if let Some(m) = caps.get(1) {
                seen.insert(m.as_str().to_string());
            }
        }
        seen.len()
    };
    if citation_count < 5 {
        fixes.push(format!(
            "- Add {} more peer-reviewed citations (found {}, minimum 5 required)",
            5 - citation_count,
            citation_count
        ));
    }

    if fixes.is_empty() {
        use crate::cli::colors as c;
        println!("{}", c::pass("Spec already meets all requirements!"));
        return Ok(());
    }

    {
        use crate::cli::colors as c;
        println!("{}\n", c::header("Spec Compliance Issues Found:"));
        for fix in &fixes {
            println!("{}", fix);
        }

        if dry_run {
            println!("\n{}", c::dim("(Dry run - no changes made)"));
        } else {
            println!("\n{}", c::warn("Auto-fix not yet implemented. Please apply fixes manually."));
        }
    }

    Ok(())
}

/// Handle spec create command
pub async fn handle_spec_create(
    name: &str,
    issue: Option<&str>,
    epic: Option<&str>,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    let slug = name.to_lowercase().replace(' ', "-");
    let output_dir = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new("docs/specifications").to_path_buf());

    let file_path = output_dir.join(format!("{}.md", slug));

    let issue_ref = issue.unwrap_or("#TODO");
    let epic_name = epic.unwrap_or("PMAT-TODO");
    let date = chrono::Local::now().format("%Y-%m-%d");

    let template = format!(
        r#"---
title: "{name}"
version: "1.0.0"
status: "Draft"
created: "{date}"
updated: "{date}"
issue_refs: ["{issue_ref}"]
epic: "{epic_name}"
---

# {name}

## Executive Summary

[Brief description of what this specification defines]

## Scientific Foundation

[Cite minimum 5 peer-reviewed sources]

1. [Author et al., Year. Title. Journal/Conference.]
2. [...]

## Requirements

### Functional Requirements

- [ ] FR-001: [Requirement description]
- [ ] FR-002: [Requirement description]

### Non-Functional Requirements

- [ ] NFR-001: [Performance requirement]
- [ ] NFR-002: [Security requirement]

## Acceptance Criteria

### Category 1 (AC-001 to AC-005)

- [ ] AC-001: [Testable criterion]
- [ ] AC-002: [Testable criterion]
- [ ] AC-003: [Testable criterion]
- [ ] AC-004: [Testable criterion]
- [ ] AC-005: [Testable criterion]

### Category 2 (AC-006 to AC-010)

- [ ] AC-006: [Testable criterion]
- [ ] AC-007: [Testable criterion]
- [ ] AC-008: [Testable criterion]
- [ ] AC-009: [Testable criterion]
- [ ] AC-010: [Testable criterion]

## Code Examples

### Example 1: Basic Usage

```rust
// Example code here
```

### Example 2: Advanced Usage

```rust
// Example code here
```

### Example 3: Error Handling

```rust
// Example code here
```

### Example 4: Integration

```rust
// Example code here
```

### Example 5: Performance

```rust
// Example code here
```

## Testing Strategy

- Unit tests: [Coverage target]
- Integration tests: [Scope]
- Property tests: [Properties to verify]

## References

[1] [Citation]
[2] [Citation]
[3] [Citation]
[4] [Citation]
[5] [Citation]
"#
    );

    // Create directory if needed
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&file_path, template)?;
    {
        use crate::cli::colors as c;
        println!("{}", c::pass(&format!("Created specification: {}", c::path(&file_path.display().to_string()))));
        println!("\n{}", c::label("Next steps:"));
        println!("  1. Edit the specification with your requirements");
        println!(
            "  2. Run `{}` to validate",
            c::label(&format!("pmat spec score {}", file_path.display()))
        );
        println!(
            "  3. Run `{}` to fix issues",
            c::label(&format!("pmat spec comply {}", file_path.display()))
        );
    }

    Ok(())
}

/// Handle spec list command
pub async fn handle_spec_list(
    path: &Path,
    min_score: Option<u8>,
    failing_only: bool,
    format: SpecOutputFormat,
) -> anyhow::Result<()> {
    let parser = SpecParser::new();
    let specs = parser.find_specs(path)?;

    let mut results = Vec::new();

    for spec_path in specs {
        if let Ok(spec) = parser.parse_file(&spec_path) {
            let score = calculate_spec_score(&spec);
            let passing = score >= 95.0;

            if let Some(min) = min_score {
                if score < f64::from(min) {
                    continue;
                }
            }

            if failing_only && passing {
                continue;
            }

            results.push((spec_path, spec.title.clone(), score, passing));
        }
    }

    print_spec_list(&results, path, format)?;

    Ok(())
}

fn spec_display_name<'a>(path: &'a Path, title: &'a str) -> &'a str {
    if title.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
    } else {
        title
    }
}

fn print_spec_list(
    results: &[(std::path::PathBuf, String, f64, bool)],
    dir: &Path,
    format: SpecOutputFormat,
) -> anyhow::Result<()> {
    match format {
        SpecOutputFormat::Text => {
            use crate::cli::colors as c;
            println!("{}\n", c::header(&format!("Specifications in {}", c::path(&dir.display().to_string()))));
            println!(
                "{}{:<50}{} {:>8} {:>8}",
                c::BOLD, "SPECIFICATION", c::RESET, "SCORE", "STATUS"
            );
            println!("{}", c::separator());
            for (path, title, score, passing) in results {
                let status = if *passing {
                    c::pass("PASS")
                } else {
                    c::fail("FAIL")
                };
                let score_str = c::pct(*score, 95.0, 80.0);
                println!("{:<50} {:>15} {}", spec_display_name(path, title), score_str, status);
            }
            println!(
                "\n{} {} specs, {} passing",
                c::label("Total:"),
                c::number(&results.len().to_string()),
                c::number(&results.iter().filter(|(_, _, _, p)| *p).count().to_string())
            );
        }
        SpecOutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|(path, title, score, passing)| {
                    serde_json::json!({
                        "path": path.display().to_string(),
                        "title": title,
                        "score": score,
                        "passing": passing,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
        SpecOutputFormat::Markdown => {
            println!("# Specification Status Report\n");
            println!("| Specification | Score | Status |");
            println!("|---------------|-------|--------|");
            for (path, title, score, passing) in results {
                let status = if *passing { "✅ PASS" } else { "❌ FAIL" };
                println!("| {} | {:.1} | {} |", spec_display_name(path, title), score, status);
            }
        }
    }
    Ok(())
}
