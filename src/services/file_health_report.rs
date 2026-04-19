/// Aggregate report for all files in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealthReport {
    pub project_path: PathBuf,
    pub total_files: usize,
    pub total_lines: usize,
    pub average_health: u8,
    pub average_grade: HealthGrade,
    pub critical_files: Vec<FileHealthMetrics>,
    pub problem_files: Vec<FileHealthMetrics>,
    pub warning_files: Vec<FileHealthMetrics>,
    pub healthy_files_count: usize,
    pub is_compliant: bool,
    pub recommendations: Vec<String>,
}

impl FileHealthReport {
    /// Create a new report from analyzed files
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_files(project_path: PathBuf, files: Vec<FileHealthMetrics>) -> Self {
        let total_files = files.len();
        let total_lines: usize = files.iter().map(|f| f.lines).sum();

        let average_health = if total_files > 0 {
            (files.iter().map(|f| f.health_score as u32).sum::<u32>() / total_files as u32) as u8
        } else {
            100
        };

        let average_grade = HealthGrade::from_score(average_health);

        let mut critical_files: Vec<FileHealthMetrics> = files
            .iter()
            .filter(|f| f.health_score < 50)
            .cloned()
            .collect();
        critical_files.sort_by_key(|a| a.health_score);

        let mut problem_files: Vec<FileHealthMetrics> = files
            .iter()
            .filter(|f| f.health_score >= 50 && f.health_score < 70)
            .cloned()
            .collect();
        problem_files.sort_by_key(|a| a.health_score);

        let warning_files: Vec<FileHealthMetrics> = files
            .iter()
            .filter(|f| f.health_score >= 70 && f.health_score < 80)
            .cloned()
            .collect();

        let healthy_files_count = files.iter().filter(|f| f.health_score >= 80).count();

        let is_compliant = critical_files.is_empty();

        let mut recommendations = Vec::new();

        // Generate recommendations based on findings
        if !critical_files.is_empty() {
            recommendations.push(format!(
                "CRITICAL: {} files have health score <50 and require immediate attention",
                critical_files.len()
            ));

            for file in critical_files.iter().take(5) {
                recommendations.push(format!(
                    "  - Split {} ({} lines, health: {})",
                    file.path.display(),
                    file.lines,
                    file.health_score
                ));
            }
        }

        if !problem_files.is_empty() {
            recommendations.push(format!(
                "WARNING: {} files have health score 50-69 and should be refactored",
                problem_files.len()
            ));
        }

        // TLR recommendations
        let low_tlr_files: Vec<_> = files.iter().filter(|f| f.tlr < f.required_tlr).collect();
        if !low_tlr_files.is_empty() {
            recommendations.push(format!(
                "TEST COVERAGE: {} files have TLR below required threshold",
                low_tlr_files.len()
            ));
        }

        Self {
            project_path,
            total_files,
            total_lines,
            average_health,
            average_grade,
            critical_files,
            problem_files,
            warning_files,
            healthy_files_count,
            is_compliant,
            recommendations,
        }
    }
}
