/// Format a ranked list of files as a table
/// Formats ranked files as a table for display
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::ranking_utils::format_ranked_files_table;
/// use pmat::services::defect_analyzer::RankedFile;
/// use std::path::PathBuf;
///
/// let files = vec![
///     RankedFile {
///         path: PathBuf::from("src/main.rs"),
///         rank: 1,
///         score: 8.5,
///         defects: vec![],
///     }
/// ];
///
/// let table = format_ranked_files_table(&files);
/// assert!(table.contains("RANK"));
/// assert!(table.contains("src/main.rs"));
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_ranked_files_table(ranked_files: &[RankedFile]) -> String {
    debug_assert!(!ranked_files.is_empty(), "ranked_files must not be empty");
    let mut output = String::new();

    // Header
    output.push_str("RANK  SCORE   FILE                                              DEFECTS  CRITICAL  HIGH  MEDIUM  LOW\n");
    output.push_str("----  ------  ------------------------------------------------  -------  --------  ----  ------  ---\n");

    for file in ranked_files {
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for defect in &file.defects {
            match defect.severity {
                Severity::Critical => critical += 1,
                Severity::High => high += 1,
                Severity::Medium => medium += 1,
                Severity::Low => low += 1,
            }
        }

        output.push_str(&format!(
            "{:<4}  {:>6.1}  {:<48}  {:>7}  {:>8}  {:>4}  {:>6}  {:>3}\n",
            file.rank,
            file.score,
            file.path
                .display()
                .to_string()
                .chars()
                .take(48)
                .collect::<String>(),
            file.defects.len(),
            critical,
            high,
            medium,
            low
        ));
    }

    output
}
