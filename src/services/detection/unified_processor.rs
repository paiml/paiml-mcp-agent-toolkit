// UnifiedDetectionProcessor — high-level detection orchestrator
// Included from mod.rs — shares parent module scope

/// High-level unified detection processor
pub struct UnifiedDetectionProcessor {
    registry: DetectionRegistry,
}

impl UnifiedDetectionProcessor {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            registry: DetectionRegistry::new(),
        }
    }

    /// Detect duplicates in files
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn detect_duplicates(
        &self,
        files: Vec<std::path::PathBuf>,
    ) -> Result<duplicates::DuplicateDetectionResult> {
        let input = DetectionInput::MultipleFiles(files);
        let config = DetectionConfig {
            detector_specific: DetectorSpecificConfig::Duplicates(
                duplicates::DuplicateConfig::default(),
            ),
            ..Default::default()
        };

        match self.registry.detect("duplicates", input, config).await? {
            DetectionOutput::Duplicates(result) => Ok(result),
            _ => Err(anyhow::anyhow!(
                "Invalid output type for duplicates detector"
            )),
        }
    }

    /// Detect SATD in project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn detect_satd(&self, project_path: &Path) -> Result<satd::SATDAnalysisResult> {
        let input = DetectionInput::ProjectDirectory(project_path.to_path_buf());
        let config = DetectionConfig {
            detector_specific: DetectorSpecificConfig::SATD(satd::SATDConfig::default()),
            ..Default::default()
        };

        match self.registry.detect("satd", input, config).await? {
            DetectionOutput::SATD(result) => Ok(result),
            _ => Err(anyhow::anyhow!("Invalid output type for SATD detector")),
        }
    }

    /// Analyze polyglot architecture
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_polyglot(
        &self,
        project_path: &Path,
    ) -> Result<polyglot::PolyglotAnalysis> {
        let input = DetectionInput::ProjectDirectory(project_path.to_path_buf());
        let config = DetectionConfig {
            detector_specific: DetectorSpecificConfig::Polyglot(polyglot::PolyglotConfig::default()),
            ..Default::default()
        };

        match self.registry.detect("polyglot", input, config).await? {
            DetectionOutput::Polyglot(result) => Ok(result),
            _ => Err(anyhow::anyhow!("Invalid output type for polyglot detector")),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Available detectors.
    pub fn available_detectors(&self) -> Vec<&str> {
        self.registry.list_detectors()
    }
}

impl Default for UnifiedDetectionProcessor {
    fn default() -> Self {
        Self::new()
    }
}
