// DetectionRegistry implementation methods
// Included from mod.rs — shares parent module scope

impl DetectionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            detectors: std::collections::HashMap::new(),
        };

        // Register all available detection strategies
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        // Register duplicate detector
        self.register("duplicates", Arc::new(duplicates::DuplicateDetector::new()));

        // Register SATD detector
        self.register("satd", Arc::new(satd::SATDDetector::new()));

        // Register polyglot analyzer
        self.register("polyglot", Arc::new(polyglot::PolyglotDetector::new()));
    }

    pub fn register(
        &mut self,
        name: &str,
        detector: Arc<
            dyn Detector<
                Input = DetectionInput,
                Output = DetectionOutput,
                Config = DetectionConfig,
            >,
        >,
    ) {
        self.detectors.insert(name.to_string(), detector);
    }

    #[must_use]
    pub fn get_detector(
        &self,
        name: &str,
    ) -> Option<
        Arc<
            dyn Detector<
                Input = DetectionInput,
                Output = DetectionOutput,
                Config = DetectionConfig,
            >,
        >,
    > {
        self.detectors.get(name).cloned()
    }

    #[must_use]
    pub fn list_detectors(&self) -> Vec<&str> {
        self.detectors
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Run detection using the specified detector
    pub async fn detect(
        &self,
        detector_name: &str,
        input: DetectionInput,
        config: DetectionConfig,
    ) -> Result<DetectionOutput> {
        if let Some(detector) = self.get_detector(detector_name) {
            detector.detect(input, config).await
        } else {
            Err(anyhow::anyhow!("Unknown detector: {detector_name}"))
        }
    }
}

impl Default for DetectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            max_files: None,
            parallel_processing: true,
            output_format: OutputFormat::Json,
            detector_specific: DetectorSpecificConfig::Duplicates(
                duplicates::DuplicateConfig::default(),
            ),
        }
    }
}
