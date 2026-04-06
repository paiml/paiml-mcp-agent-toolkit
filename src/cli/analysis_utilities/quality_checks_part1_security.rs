// Security checking functions - extracted from quality_checks_part1.rs (CB-040)
async fn check_security(project_path: &Path) -> Result<Vec<QualityViolation>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();
    let patterns = get_security_patterns();

    use tokio::fs;

    if let Ok(mut entries) = fs::read_dir(project_path).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && is_source_file(&path) {
                check_file_security(&path, &patterns, &mut violations).await?;
            }
        }
    }

    Ok(violations)
}
