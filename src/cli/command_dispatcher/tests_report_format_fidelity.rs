//! Issue #672: `report` format-fidelity regression tests.
//!
//! These live in their own module because `command_dispatcher/tests.rs` gates
//! every other dispatcher test behind the `broken-tests` quarantine feature,
//! so tests added there never compile and never run.

use super::CommandDispatcher;

#[cfg(test)]
mod tests {
    use super::*;

    /// Issue #672 regression: the dispatcher squashed every non-json
    /// `--output-format` to `OutputFormat::Table`, which
    /// `execute_report_command` mapped back to `ReportOutputFormat::Text`, so
    /// `report --format csv` wrote a plain-text file starting with
    /// "CODE QUALITY REPORT" instead of comma-separated values.
    #[tokio::test]
    async fn test_report_csv_format_is_csv_or_an_explicit_error() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}").expect("internal error");
        let out = temp_dir.path().join("report.csv");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Csv,
            false,
            false,
            false,
            vec!["all".to_string()],
            None,
            Some(out.clone()),
            false,
            false,
            false,
            false,
        )
        .await;

        match result {
            Ok(()) => {
                let content = std::fs::read_to_string(&out).expect("report file");
                assert!(
                    !content.starts_with("CODE QUALITY REPORT"),
                    "--format csv must not emit the plain-text report, got {content:?}"
                );
                assert!(
                    content.starts_with("id,severity,category,file_path"),
                    "--format csv must emit the CSV header, got {content:?}"
                );
            }
            Err(e) => {
                // Without the optional `reporting` feature there is no CSV
                // writer at all; failing loudly is the contract-compliant
                // outcome. Silently writing plain text is not.
                assert!(
                    e.to_string().contains("CSV reporting requires"),
                    "unexpected csv failure: {e}"
                );
                assert!(!out.exists(), "a rejected format must not write a file");
            }
        }
    }

    /// Issue #672: markdown must be markdown, not the same plain text.
    #[tokio::test]
    async fn test_report_markdown_format_emits_actual_markdown() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}").expect("internal error");
        let out = temp_dir.path().join("report.md");

        CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Markdown,
            false,
            false,
            false,
            vec!["all".to_string()],
            None,
            Some(out.clone()),
            false,
            false,
            false,
            false,
        )
        .await
        .expect("markdown report should succeed");

        let content = std::fs::read_to_string(&out).expect("report file");
        assert!(
            content.starts_with("# Code Quality Report"),
            "--format markdown must emit markdown, got {content:?}"
        );
    }

    /// Issue #672: `--format html` used to silently produce markdown.
    #[tokio::test]
    async fn test_report_html_format_is_rejected() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}").expect("internal error");
        let out = temp_dir.path().join("report.html");

        let err = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Html,
            false,
            false,
            false,
            vec!["all".to_string()],
            None,
            Some(out.clone()),
            false,
            false,
            false,
            false,
        )
        .await
        .expect_err("html has no emitter and must be rejected");
        assert!(err.to_string().contains("not implemented"), "{err}");
        assert!(!out.exists(), "rejected format must not write a file");
    }
}
