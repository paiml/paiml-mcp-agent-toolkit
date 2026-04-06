// DuplicationDefectAnalyzer: constructors, trait impl, and helper methods for
// code clone detection.

impl DuplicationDefectAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        let config = crate::services::duplicate_detector::DuplicateDetectionConfig::default();
        Self {
            detector: crate::services::duplicate_detector::DuplicateDetectionEngine::new(config),
        }
    }
}

impl Default for DuplicationDefectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DefectAnalyzer for DuplicationDefectAnalyzer {
    type Config = DuplicationConfig;

    async fn analyze(&self, project_path: &Path, config: Self::Config) -> Result<Vec<Defect>> {
        let mut defects = Vec::new();
        let files = discover_source_files(project_path).await?;

        // Prepare file contents
        let mut file_contents = Vec::new();
        for file in files {
            if let Ok(content) = tokio::fs::read_to_string(&file).await {
                let language = self.detect_language(&file);
                file_contents.push((file.clone(), content, language));
            }
        }

        let report = self.detector.detect_duplicates(&file_contents)?;

        // Convert clone groups to defects
        for (group_index, group) in report.groups.iter().enumerate() {
            if group.average_similarity >= config.min_similarity {
                for (instance_index, instance) in group.fragments.iter().enumerate() {
                    defects.push(self.clone_to_defect(
                        group,
                        instance,
                        group_index * 100 + instance_index + 1,
                    ));
                }
            }
        }

        Ok(defects)
    }

    fn category(&self) -> DefectCategory {
        DefectCategory::Duplication
    }

    fn supports_incremental(&self) -> bool {
        false // Requires full codebase analysis
    }
}

impl DuplicationDefectAnalyzer {
    fn detect_language(&self, path: &Path) -> crate::services::duplicate_detector::Language {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        use crate::services::duplicate_detector::Language;

        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") => Language::TypeScript,
            Some("js") => Language::JavaScript,
            Some("py") => Language::Python,
            Some("c") => Language::C,
            Some("cpp" | "cc" | "cxx" | "cu" | "cuh") => Language::Cpp,
            Some("kt") => Language::Kotlin,
            _ => Language::Rust, // Default
        }
    }

    fn clone_to_defect(
        &self,
        group: &CloneGroup,
        instance: &crate::services::duplicate_detector::CloneInstance,
        index: usize,
    ) -> Defect {
        let severity = match &group.clone_type {
            CloneType::Type1 { .. } if group.total_lines > 50 => Severity::High,
            CloneType::Type1 { .. } => Severity::Medium,
            CloneType::Type2 { .. } if group.total_lines > 30 => Severity::Medium,
            CloneType::Type2 { .. } => Severity::Low,
            CloneType::Type3 { .. } => Severity::Low,
        };

        let mut metrics = HashMap::new();
        metrics.insert(
            "similarity".to_string(),
            instance.similarity_to_representative,
        );
        metrics.insert("total_lines".to_string(), group.total_lines as f64);
        metrics.insert("group_size".to_string(), group.fragments.len() as f64);

        Defect {
            id: format!("DUP-{index:04}"),
            severity,
            category: DefectCategory::Duplication,
            file_path: instance.file.clone(),
            line_start: instance.start_line as u32,
            line_end: Some(instance.end_line as u32),
            column_start: None,
            column_end: None,
            message: format!(
                "{:?} code clone ({} lines, {:.0}% similarity) in group {}",
                group.clone_type,
                instance.end_line - instance.start_line + 1,
                instance.similarity_to_representative * 100.0,
                group.id
            ),
            rule_id: format!("duplication-{:?}", group.clone_type).to_lowercase(),
            fix_suggestion: Some(
                "Extract duplicated code into a shared function or module".to_string(),
            ),
            metrics,
        }
    }
}
