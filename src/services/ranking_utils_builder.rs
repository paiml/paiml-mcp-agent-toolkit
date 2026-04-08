impl AnalysisResultBuilder {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Create a new instance.
    pub fn new(file_path: PathBuf) -> Self {
        let absolute_path = file_path.clone();
        Self {
            file_path,
            absolute_path,
            line_start: 1,
            line_end: None,
            column_start: 1,
            column_end: None,
            metrics: BTreeMap::new(),
            description: String::new(),
            entity_name: None,
            entity_type: None,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With line range.
    pub fn with_line_range(mut self, start: u32, end: Option<u32>) -> Self {
        self.line_start = start;
        self.line_end = end;
        self
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With column range.
    pub fn with_column_range(mut self, start: u32, end: Option<u32>) -> Self {
        self.column_start = start;
        self.column_end = end;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add metric.
    pub fn add_metric(mut self, key: impl Into<String>, value: MetricValue) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add metric int.
    pub fn add_metric_int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.metrics.insert(key.into(), MetricValue::Integer(value));
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add metric float.
    pub fn add_metric_float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), MetricValue::Float(value));
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With entity.
    pub fn with_entity(mut self, name: impl Into<String>, entity_type: impl Into<String>) -> Self {
        self.entity_name = Some(name.into());
        self.entity_type = Some(entity_type.into());
        self
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Build and return the final result.
    pub fn build(self) -> AnalysisResult {
        AnalysisResult {
            file_path: self.file_path,
            absolute_path: self.absolute_path,
            line_range: LineRange {
                start: LineInfo {
                    line: self.line_start,
                    column: self.column_start,
                    byte_offset: 0, // Not computed here
                },
                end: self.line_end.map(|line| LineInfo {
                    line,
                    column: self.column_end.unwrap_or(1),
                    byte_offset: 0,
                }),
            },
            metrics: self.metrics,
            context: AnalysisContext {
                description: self.description,
                entity_name: self.entity_name,
                entity_type: self.entity_type,
            },
        }
    }
}
