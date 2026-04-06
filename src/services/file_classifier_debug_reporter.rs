// Debug reporter for tracking file classification decisions
// Included from file_classifier.rs - do NOT add `use` imports or `#!` attributes here.

fn parse_vmrss_kb_f64(status: &str) -> Option<f64> {
    status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|kb_str| kb_str.parse::<f64>().ok())
}

/// Debug reporter for tracking file classification decisions
#[derive(Debug)]
pub struct DebugReporter {
    start_time: Instant,
    events: Vec<DebugEvent>,
    output_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    pub timestamp_ms: u64,
    pub file: std::path::PathBuf,
    pub decision: ParseDecision,
    pub parse_time_ms: Option<u64>,
    pub error: Option<String>,
    pub memory_usage_mb: f64,
}

impl DebugReporter {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(output_path: Option<std::path::PathBuf>) -> Self {
        Self {
            start_time: Instant::now(),
            events: Vec::new(),
            output_path,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn record_decision(&mut self, file: &Path, decision: &ParseDecision) {
        let event = DebugEvent {
            timestamp_ms: self.start_time.elapsed().as_millis() as u64,
            file: file.to_path_buf(),
            decision: *decision,
            parse_time_ms: None,
            error: None,
            memory_usage_mb: self.get_memory_usage_mb(),
        };
        self.events.push(event);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn record_parse_result(
        &mut self,
        file: &Path,
        parse_time: std::time::Duration,
        error: Option<String>,
    ) {
        let memory_usage = self.get_memory_usage_mb();
        if let Some(event) = self.events.iter_mut().rev().find(|e| e.file == file) {
            event.parse_time_ms = Some(parse_time.as_millis() as u64);
            event.error = error;
            event.memory_usage_mb = memory_usage;
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_report(&self) -> Result<DebugReport> {
        let total_files = self.events.len();
        let parsed_files = self
            .events
            .iter()
            .filter(|e| matches!(e.decision, ParseDecision::Parse))
            .count();
        let skipped_files = total_files - parsed_files;

        let mut skip_reasons = std::collections::HashMap::new();
        for event in &self.events {
            if let ParseDecision::Skip(reason) = event.decision {
                *skip_reasons.entry(format!("{reason:?}")).or_insert(0) += 1;
            }
        }

        let parse_errors = self.events.iter().filter(|e| e.error.is_some()).count();

        let total_time_ms = self.start_time.elapsed().as_millis() as u64;
        let memory_peak_mb = self
            .events
            .iter()
            .map(|e| e.memory_usage_mb)
            .fold(0.0, f64::max);

        Ok(DebugReport {
            summary: DebugSummary {
                total_files,
                parsed_files,
                skipped_files,
                parse_errors,
                total_time_ms,
                memory_peak_mb,
            },
            skip_reasons,
            events: self.events.clone(),
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn save_report(&self) -> Result<()> {
        if let Some(output_path) = &self.output_path {
            let report = self.generate_report()?;
            let json = serde_json::to_string_pretty(&report)?;
            tokio::fs::write(output_path, json).await?;
        }
        Ok(())
    }

    fn get_memory_usage_mb(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                return parse_vmrss_kb_f64(&status).unwrap_or(0.0) / 1024.0;
            }
        }
        0.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugReport {
    pub summary: DebugSummary,
    pub skip_reasons: std::collections::HashMap<String, usize>,
    pub events: Vec<DebugEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSummary {
    pub total_files: usize,
    pub parsed_files: usize,
    pub skipped_files: usize,
    pub parse_errors: usize,
    pub total_time_ms: u64,
    pub memory_peak_mb: f64,
}

struct VendorRules {
    path_patterns: Vec<&'static str>,
    file_patterns: Vec<&'static str>,
    content_signatures: Vec<&'static [u8]>,
}
