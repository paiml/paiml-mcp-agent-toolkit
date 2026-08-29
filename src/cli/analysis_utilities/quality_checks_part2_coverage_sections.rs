// Coverage checking and documentation section validation
// Included by quality_checks_part2.rs

async fn check_coverage(project_path: &Path, min_coverage: f64) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Read actual coverage from cache files (#228: was hardcoded 75.0)
    let current_coverage = read_coverage_from_cache(project_path);

    if let Some(coverage) = current_coverage {
        if coverage < min_coverage {
            violations.push(QualityViolation {
                check_type: "coverage".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Code coverage {coverage:.1}% is below minimum {min_coverage:.1}%"
                ),
                file: "project".to_string(),
                line: None,
                details: None,
            });
        }
    }
    // If no coverage data available, skip check (no violation)

    Ok(violations)
}

/// Read coverage percentage from cache files (#228).
///
/// Priority: `.pmat/coverage-cache.json` > `.pmat-metrics/coverage.json`
/// Computes line coverage from per-file hit data in coverage-cache.json.
fn read_coverage_from_cache(project_path: &Path) -> Option<f64> {
    read_coverage_from_detail_cache(project_path)
        .or_else(|| read_coverage_from_metrics(project_path))
}

/// Read line coverage from `.pmat/coverage-cache.json` per-file hit data.
fn read_coverage_from_detail_cache(project_path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(project_path.join(".pmat/coverage-cache.json")).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&content).ok()?;
    let files = cache.get("files")?.as_object()?;
    let (total, covered) = compute_line_coverage(files);
    (total > 0).then(|| covered as f64 / total as f64 * 100.0)
}

/// Compute total and covered lines from per-file hit maps.
fn compute_line_coverage(files: &serde_json::Map<String, serde_json::Value>) -> (u64, u64) {
    let mut total = 0u64;
    let mut covered = 0u64;
    for line_hits in files.values() {
        let Some(hits) = line_hits.as_object() else {
            continue;
        };
        for count in hits.values() {
            total += 1;
            if count.as_u64().unwrap_or(0) > 0 {
                covered += 1;
            }
        }
    }
    (total, covered)
}

/// Read aggregate coverage from `.pmat-metrics/coverage.json`.
fn read_coverage_from_metrics(project_path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(project_path.join(".pmat-metrics/coverage.json")).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&content).ok()?;
    cache.get("coverage")?.as_f64()
}

/// The README the sections check reads, or `None` when there is none to read.
///
/// ONE answer to "is there anything here for the sections check to measure".
/// `check_sections` returns an empty violation list for a project with no
/// README, which is the same value it returns for a README carrying all four
/// required sections — so a caller reporting what the gate covered asks this
/// rather than inferring "clean" from a zero, exactly as `read_coverage_from_cache`
/// is asked instead of reading `check_coverage`'s empty list as a pass.
fn sections_source(project_path: &Path) -> Option<PathBuf> {
    let readme = project_path.join("README.md");
    readme.is_file().then_some(readme)
}

/// The text of an ATX heading, with the `#` run and any leading decoration
/// removed; `None` when the line is not a heading.
///
/// Decoration is *only* the leading run of non-alphanumeric characters — an
/// emoji, a badge, `1.`, `*`. Once the text has begun it is returned verbatim,
/// so `## Q&A` stays `Q&A` and `## C++ Usage` stays `C++ Usage`.
///
/// The whitespace requirement after the `#` run is CommonMark's: `##Contributing`
/// is a paragraph that starts with hashes, not a heading, and must not be able
/// to satisfy a required section.
fn heading_text(line: &str) -> Option<&str> {
    let after_hashes = line.trim_start().strip_prefix('#')?.trim_start_matches('#');
    if !after_hashes.is_empty() && !after_hashes.starts_with([' ', '\t']) {
        return None;
    }
    let text = after_hashes
        .trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim_start();
    let text = strip_leading_ordinal(text);
    (!text.is_empty()).then_some(text)
}

/// Drop a leading `1.` / `2)` / `3:` numbering from heading text.
///
/// Digits are alphanumeric, so the non-alphanumeric trim above cannot see this
/// one, and `## 1. Installation` is the same false positive as `## 🚀 Installation`.
///
/// Narrow on purpose: BOTH a separator and the whitespace after it are
/// required, so `## 2024 Roadmap` keeps its number and `## 3.5 Release` keeps
/// its version. A rule that dropped any leading digit run would rename those.
fn strip_leading_ordinal(text: &str) -> &str {
    let digits = text.len() - text.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits == 0 {
        return text;
    }
    let Some(after_sep) = text[digits..].strip_prefix(['.', ')', ':']) else {
        return text;
    };
    if !after_sep.starts_with([' ', '\t']) {
        return text;
    }
    after_sep.trim_start()
}

/// Does `readme` provide `section`?
///
/// THE DEFECT THIS EXISTS FOR.
///
/// The check used to ask `readme.contains("## Contributing")` against the whole
/// file. Measured on `paiml/interactive.paiml.com`, whose README carries
///
/// ```markdown
/// ## 🤝 Contributing
/// ## 📄 License
/// ```
///
/// `pmat quality-gate` answered `Missing required section: Contributing` and
/// `Missing required section: License`. Both sections are there, in the right
/// place, at the right level. Two of that repository's 60 blocking violations
/// were an emoji: a substring test has no notion of a heading, so the only
/// headings it can recognise are those whose text begins immediately after the
/// `# ` — and a README that decorates its headings, which is most of them, is
/// told to add sections it already has.
///
/// That is the shape of defect this gate exists to catch in other people's
/// code: a check reporting a result it did not measure. It measured "does this
/// byte sequence occur", and reported "is this section present".
///
/// THE FIX IS ADDITIVE, ON PURPOSE. The original substring test is still
/// consulted first, so no README that satisfied this check before can begin
/// failing it — every change below can only remove a false "missing", never
/// add one. That matters because this check is already enforced in repositories
/// nobody here can see; a matcher that got *stricter* would turn green gates
/// red on a pmat upgrade, for a reason the owner did not choose.
///
/// The second path is heading-aware: it compares the required section against
/// `heading_text` for each line, case-insensitively, and requires a word
/// boundary after it — `Licenses` is a different section from `License`, and
/// only the boundary check stops the decoration-stripping path from accepting
/// it. (The substring path has always accepted `Licensed under MIT`, which the
/// boundary does not undo; widening THAT is a behaviour change, and this commit
/// deliberately makes none.)
fn readme_provides_section(readme: &str, section: &str) -> bool {
    if readme.contains(&format!("# {section}")) || readme.contains(&format!("## {section}")) {
        return true;
    }
    let wanted = section.to_lowercase();
    readme.lines().any(|line| {
        heading_text(line).is_some_and(|text| {
            let lowered = text.to_lowercase();
            lowered.starts_with(&wanted)
                && lowered[wanted.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric())
        })
    })
}

async fn check_sections(project_path: &Path) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Check for required documentation sections
    let Some(readme_path) = sections_source(project_path) else {
        return Ok(violations);
    };
    if let Ok(readme) = tokio::fs::read_to_string(readme_path).await {
        let required_sections = ["Installation", "Usage", "Contributing", "License"];
        for section in required_sections {
            if !readme_provides_section(&readme, section) {
                violations.push(QualityViolation {
                    check_type: "sections".to_string(),
                    severity: "warning".to_string(),
                    message: format!("Missing required section: {section}"),
                    file: "README.md".to_string(),
                    line: None,
                    details: None,
                });
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod coverage_sections_tests {
    //! Covers quality_checks_part2_coverage_sections.rs pure-compute helpers
    //! (73 uncov on broad, 0% cov).
    use super::*;

    // ── compute_line_coverage: sum across per-file hit maps ──

    #[test]
    fn test_compute_line_coverage_empty_files_map_is_zero_zero() {
        let files = serde_json::Map::new();
        let (total, covered) = compute_line_coverage(&files);
        assert_eq!(total, 0);
        assert_eq!(covered, 0);
    }

    #[test]
    fn test_compute_line_coverage_counts_only_object_values() {
        let json = serde_json::json!({
            "src/a.rs": {"1": 5, "2": 0, "3": 12},
            "src/b.rs": "not-an-object",  // skipped by as_object() branch
            "src/c.rs": {"1": 0}
        });
        let files = json.as_object().unwrap().clone();
        let (total, covered) = compute_line_coverage(&files);
        assert_eq!(total, 4, "3 from a.rs + 1 from c.rs = 4");
        assert_eq!(covered, 2, "a.rs:1=5 and a.rs:3=12 → 2 covered");
    }

    #[test]
    fn test_compute_line_coverage_non_numeric_hits_count_as_uncovered() {
        // count.as_u64() returns None for strings → unwrap_or(0) → not > 0 → uncovered.
        let json = serde_json::json!({
            "f.rs": {"1": "notanumber", "2": 3}
        });
        let files = json.as_object().unwrap().clone();
        let (total, covered) = compute_line_coverage(&files);
        assert_eq!(total, 2);
        assert_eq!(covered, 1);
    }

    // ── read_coverage_from_detail_cache ──

    #[test]
    fn test_read_coverage_from_detail_cache_missing_file_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_malformed_json_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        std::fs::write(tmp.path().join(".pmat/coverage-cache.json"), "not json").unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_no_files_key_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        std::fs::write(tmp.path().join(".pmat/coverage-cache.json"), "{}").unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_total_zero_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        // Empty files object → total=0 → None.
        std::fs::write(
            tmp.path().join(".pmat/coverage-cache.json"),
            "{\"files\":{}}",
        )
        .unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_valid_returns_percentage() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        // 4 lines, 3 covered → 75%.
        std::fs::write(
            tmp.path().join(".pmat/coverage-cache.json"),
            "{\"files\":{\"a.rs\":{\"1\":1,\"2\":2,\"3\":0,\"4\":5}}}",
        )
        .unwrap();
        let pct = read_coverage_from_detail_cache(tmp.path()).unwrap();
        assert!((pct - 75.0).abs() < 1e-6);
    }

    // ── read_coverage_from_metrics ──

    #[test]
    fn test_read_coverage_from_metrics_missing_file_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_coverage_from_metrics(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_metrics_valid_returns_value() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 92.5}",
        )
        .unwrap();
        let pct = read_coverage_from_metrics(tmp.path()).unwrap();
        assert!((pct - 92.5).abs() < 1e-6);
    }

    #[test]
    fn test_read_coverage_from_metrics_missing_coverage_key_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"other\": 1}",
        )
        .unwrap();
        assert!(read_coverage_from_metrics(tmp.path()).is_none());
    }

    // ── read_coverage_from_cache: falls back from detail to metrics ──

    #[test]
    fn test_read_coverage_from_cache_detail_preferred_over_metrics() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        // Detail cache present + metrics also present → detail wins.
        std::fs::write(
            tmp.path().join(".pmat/coverage-cache.json"),
            "{\"files\":{\"a.rs\":{\"1\":1,\"2\":0}}}",
        )
        .unwrap(); // 50%
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 92.5}",
        )
        .unwrap();
        let pct = read_coverage_from_cache(tmp.path()).unwrap();
        assert!((pct - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_read_coverage_from_cache_falls_back_to_metrics() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        // No detail cache → fallback to metrics.
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 80.0}",
        )
        .unwrap();
        let pct = read_coverage_from_cache(tmp.path()).unwrap();
        assert!((pct - 80.0).abs() < 1e-6);
    }

    #[test]
    fn test_read_coverage_from_cache_no_files_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_coverage_from_cache(tmp.path()).is_none());
    }

    // ── check_coverage async ──

    #[tokio::test]
    async fn test_check_coverage_no_data_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        let v = check_coverage(tmp.path(), 95.0).await.unwrap();
        assert!(v.is_empty(), "no data → skip, no violations");
    }

    #[tokio::test]
    async fn test_check_coverage_below_threshold_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 60.0}",
        )
        .unwrap();
        let v = check_coverage(tmp.path(), 95.0).await.unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].check_type, "coverage");
        assert_eq!(v[0].severity, "error");
    }

    #[tokio::test]
    async fn test_check_coverage_above_threshold_no_violation() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 97.0}",
        )
        .unwrap();
        let v = check_coverage(tmp.path(), 95.0).await.unwrap();
        assert!(v.is_empty());
    }

    // ── check_sections async ──

    #[tokio::test]
    async fn test_check_sections_no_readme_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_check_sections_complete_readme_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n## Installation\n## Usage\n## Contributing\n## License\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_check_sections_missing_sections_all_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("README.md"), "# Project only\n").unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert_eq!(v.len(), 4, "all 4 required sections missing");
        for x in &v {
            assert_eq!(x.check_type, "sections");
            assert_eq!(x.severity, "warning");
        }
    }

    // ── check_sections: a heading the matcher could not see past ──
    //
    // Every test below fails against the substring matcher these replace, or
    // passes against it for a reason that must survive the fix. The four
    // counter-tests are marked; they are green before AND after, which is what
    // makes them evidence of no over-reach rather than passengers.

    /// THE DEFECT, verbatim. paiml/interactive.paiml.com's README carries
    /// `## 🤝 Contributing` and `## 📄 License`, and the gate reported both as
    /// missing — two of that repository's 60 blocking violations were an emoji.
    #[tokio::test]
    async fn test_check_sections_emoji_prefixed_headings_are_found() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Python Command Line Tools: Interactive Edition\n\
             ## 🚀 Overview\n\
             ## Installation\n\
             ## Usage\n\
             ## 🤝 Contributing\n\
             ## 📄 License\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert!(
            v.is_empty(),
            "an emoji before the heading text is decoration, not a missing section; got {:?}",
            v.iter().map(|x| &x.message).collect::<Vec<_>>()
        );
    }

    /// Decoration is not only emoji. A numbered or bulleted heading is the same
    /// shape of false positive.
    #[tokio::test]
    async fn test_check_sections_numbered_and_bulleted_headings_are_found() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n\
             ## 1. Installation\n\
             ## 2. Usage\n\
             ## ✦ Contributing\n\
             ### 📄 License\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert!(
            v.is_empty(),
            "leading decoration of any kind is not a missing section; got {:?}",
            v.iter().map(|x| &x.message).collect::<Vec<_>>()
        );
    }

    /// COUNTER-TEST. A README that genuinely lacks a section must still be
    /// flagged — the fix must remove false positives, not the check.
    #[tokio::test]
    async fn test_check_sections_emoji_readme_missing_one_is_still_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n## 🚀 Installation\n## 📖 Usage\n## 🤝 Contributing\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert_eq!(v.len(), 1, "only License is absent; got {v:?}");
        assert_eq!(v[0].message, "Missing required section: License");
    }

    /// COUNTER-TEST. The section name in body text is not a section. The check
    /// reads headings; a mention in a paragraph must not satisfy it.
    #[tokio::test]
    async fn test_check_sections_name_in_prose_is_not_a_heading() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n\
             ## Installation\n\
             ## Usage\n\
             🤝 Contributing guidelines live in CONTRIBUTING.md.\n\
             📄 License information is in LICENSE.\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert_eq!(v.len(), 2, "prose is not a heading; got {v:?}");
    }

    /// COUNTER-TEST. CommonMark requires whitespace after the `#` run, so
    /// `##Contributing` is a paragraph beginning with hashes, not a heading.
    #[tokio::test]
    async fn test_check_sections_hashes_without_space_are_not_a_heading() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n## Installation\n## Usage\n##Contributing\n##📄License\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert_eq!(v.len(), 2, "`#` with no space is not a heading; got {v:?}");
    }

    /// COUNTER-TEST for the word boundary. `Licenses` starts with `License`,
    /// and only the boundary check stops the decoration-stripping path from
    /// accepting it. The old substring matcher never sees this line at all
    /// (the emoji separates `##` from the word), so this discriminates the new
    /// path on its own.
    #[tokio::test]
    async fn test_check_sections_longer_word_does_not_satisfy_the_section() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n## Installation\n## Usage\n## 🤝 Contributing\n### 📄 Licenses\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert_eq!(v.len(), 1, "`Licenses` is a different word; got {v:?}");
        assert_eq!(v[0].message, "Missing required section: License");
    }

    // ── heading_text: the pure part ──

    #[test]
    fn test_heading_text_strips_hashes_and_leading_decoration() {
        assert_eq!(heading_text("## 🤝 Contributing"), Some("Contributing"));
        assert_eq!(heading_text("# License"), Some("License"));
        assert_eq!(heading_text("###   1. Installation"), Some("Installation"));
        assert_eq!(
            heading_text("## ⚠️ CRITICAL: Testing"),
            Some("CRITICAL: Testing")
        );
    }

    #[test]
    fn test_heading_text_keeps_punctuation_once_the_text_has_begun() {
        // Only the LEADING run is decoration. A heading is not re-punctuated.
        assert_eq!(heading_text("## Q&A"), Some("Q&A"));
        assert_eq!(heading_text("## C++ Usage"), Some("C++ Usage"));
    }

    /// COUNTER-TEST for `strip_leading_ordinal`'s narrowness. A digit run that
    /// is not a numbering must survive: dropping it would rename the heading.
    #[test]
    fn test_heading_text_keeps_a_leading_number_that_is_not_a_numbering() {
        assert_eq!(heading_text("## 2024 Roadmap"), Some("2024 Roadmap"));
        assert_eq!(heading_text("## 3.5 Release"), Some("3.5 Release"));
        assert_eq!(heading_text("## 10x Faster"), Some("10x Faster"));
    }

    #[test]
    fn test_heading_text_rejects_non_headings() {
        assert_eq!(heading_text("Contributing"), None);
        assert_eq!(heading_text("##Contributing"), None);
        assert_eq!(heading_text("  not a heading"), None);
        assert_eq!(heading_text(""), None);
    }

    #[test]
    fn test_readme_provides_section_is_case_insensitive_on_the_decorated_path() {
        assert!(readme_provides_section(
            "## 🤝 contributing\n",
            "Contributing"
        ));
        assert!(!readme_provides_section(
            "## 🤝 contributors\n",
            "Contributing"
        ));
    }
}
