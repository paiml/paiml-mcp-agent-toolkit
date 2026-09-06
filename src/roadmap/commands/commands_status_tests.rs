// Regression tests for `roadmap status --format`
// Included from mod.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod status_format_tests {
    //! `roadmap status` advertises nine `--format` values but implemented one:
    //! table, yaml, markdown, csv, summary, text, plain and junit all fell
    //! through a bare `_ =>` arm onto the human table and were byte-identical
    //! (8 of 9 shared a single md5). `stack status` already supported yaml, so
    //! the two commands contradicted each other about what `--format yaml` did.
    use super::*;
    use crate::roadmap::{Complexity, Priority};
    use chrono::TimeZone;

    fn task(id: &str, status: TaskStatus) -> Task {
        Task {
            id: id.to_string(),
            description: "Ship the thing".to_string(),
            status,
            complexity: Complexity::Medium,
            priority: Priority::P0,
            assignee: None,
            started_at: None,
            completed_at: None,
        }
    }

    fn sprint() -> Sprint {
        Sprint {
            version: "1.2.0".to_string(),
            title: "Hardening".to_string(),
            start_date: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            end_date: Utc.with_ymd_and_hms(2026, 1, 14, 0, 0, 0).unwrap(),
            priority: Priority::P0,
            tasks: vec![
                task("PMAT-1", TaskStatus::Completed),
                task("PMAT-2", TaskStatus::InProgress),
            ],
            definition_of_done: vec![],
            quality_gates: vec![],
        }
    }

    const RENDERED: [OutputFormat; 7] = [
        OutputFormat::Table,
        OutputFormat::Json,
        OutputFormat::Yaml,
        OutputFormat::Markdown,
        OutputFormat::Csv,
        OutputFormat::Summary,
        OutputFormat::Text,
    ];

    #[test]
    fn sprint_formats_are_distinct_not_one_table_repeated() {
        let rendered: Vec<String> = RENDERED
            .iter()
            .map(|f| format_sprint_status(&sprint(), *f).expect("format must render"))
            .collect();
        // table/text/plain are legitimately the same human rendering; every
        // machine format must differ from it and from each other.
        let table = &rendered[0];
        for (fmt, out) in RENDERED.iter().zip(&rendered) {
            if matches!(fmt, OutputFormat::Table | OutputFormat::Text) {
                continue;
            }
            assert_ne!(
                out, table,
                "--format {fmt} must not be the human table verbatim"
            );
        }
    }

    #[test]
    fn sprint_yaml_is_yaml() {
        let out = format_sprint_status(&sprint(), OutputFormat::Yaml).unwrap();
        assert!(out.contains("version: 1.2.0"), "{out}");
        assert!(out.contains("title: Hardening"), "{out}");
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&out).expect("valid YAML");
        assert!(parsed.get("tasks").is_some(), "{out}");
    }

    #[test]
    fn sprint_csv_has_a_header_and_a_row_per_task() {
        let out = format_sprint_status(&sprint(), OutputFormat::Csv).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0],
            "sprint,task_id,status,complexity,priority,description"
        );
        assert_eq!(lines.len(), 3, "header + one row per task, got {out}");
        assert!(lines[1].starts_with("1.2.0,PMAT-1,Completed"), "{out}");
    }

    #[test]
    fn sprint_markdown_is_markdown() {
        let out = format_sprint_status(&sprint(), OutputFormat::Markdown).unwrap();
        assert!(out.starts_with("# Sprint 1.2.0: Hardening"), "{out}");
        assert!(out.contains("| Task | Status | Description |"), "{out}");
    }

    #[test]
    fn sprint_summary_is_one_line() {
        let out = format_sprint_status(&sprint(), OutputFormat::Summary).unwrap();
        assert_eq!(out.lines().count(), 1, "{out}");
        assert!(out.contains("1/2 completed, 1 in progress"), "{out}");
    }

    #[test]
    fn sprint_junit_is_rejected_rather_than_silently_ignored() {
        let err = format_sprint_status(&sprint(), OutputFormat::Junit)
            .expect_err("junit must not render a human table");
        let msg = format!("{err:#}");
        assert!(msg.contains("junit"), "the error must name the format: {msg}");
    }

    #[test]
    fn task_formats_are_distinct_not_one_table_repeated() {
        let t = task("PMAT-9", TaskStatus::InProgress);
        let table = format_task_status(&t, OutputFormat::Table).unwrap();
        for fmt in [
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Markdown,
            OutputFormat::Csv,
            OutputFormat::Summary,
        ] {
            let out = format_task_status(&t, fmt).unwrap();
            assert_ne!(
                out, table,
                "--format {fmt} must not be the human table verbatim"
            );
        }
    }

    #[test]
    fn task_yaml_csv_and_summary_carry_the_task() {
        let t = task("PMAT-9", TaskStatus::InProgress);
        let yaml = format_task_status(&t, OutputFormat::Yaml).unwrap();
        assert!(yaml.contains("id: PMAT-9"), "{yaml}");

        let csv = format_task_status(&t, OutputFormat::Csv).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(
            lines[0],
            "id,status,complexity,priority,assignee,started_at,completed_at,description"
        );
        assert!(lines[1].starts_with("PMAT-9,InProgress"), "{csv}");

        let summary = format_task_status(&t, OutputFormat::Summary).unwrap();
        assert_eq!(summary, "PMAT-9 [InProgress] Ship the thing");
    }

    #[test]
    fn task_junit_is_rejected() {
        assert!(
            format_task_status(&task("PMAT-9", TaskStatus::Planned), OutputFormat::Junit).is_err()
        );
    }

    #[test]
    fn csv_fields_with_commas_are_quoted() {
        let mut t = task("PMAT-9", TaskStatus::Planned);
        t.description = "fix a, b and \"c\"".to_string();
        let csv = format_task_status(&t, OutputFormat::Csv).unwrap();
        assert!(csv.contains("\"fix a, b and \"\"c\"\"\""), "{csv}");
    }

    /// PMAT-688: once the sweep corpus carried a roadmap, `roadmap status
    /// --color always` became reachable and emitted the same bytes as
    /// `--color auto` — the human table went through no colour helper. The
    /// table (the default and `text` format) must carry an escape when
    /// colours are forced, and none when they are off; machine formats stay
    /// untouched either way.
    #[test]
    fn sprint_table_carries_colour_when_forced() {
        let sprint = sprint();
        let plain = {
            let _off = crate::cli::colors::ForcedColor::off();
            format_sprint_status(&sprint, OutputFormat::Table).expect("table")
        };
        let _on = crate::cli::colors::ForcedColor::on();
        let coloured = format_sprint_status(&sprint, OutputFormat::Table).expect("table");
        assert!(!plain.contains("\x1b["), "{plain}");
        assert!(coloured.contains("\x1b["), "{coloured}");
        let stripped = regex::Regex::new("\x1b\\[[0-9;]*m")
            .expect("static regex must compile")
            .replace_all(&coloured, "");
        assert_eq!(stripped, plain, "colour must add escapes only");
        for machine in [OutputFormat::Json, OutputFormat::Csv, OutputFormat::Markdown] {
            let out = format_sprint_status(&sprint, machine).expect("machine format");
            assert!(!out.contains("\x1b["), "{machine:?} must never carry an escape");
        }
    }
}
