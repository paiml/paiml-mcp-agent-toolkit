// Classification logic for FileClassifier
// Included from file_classifier.rs - do NOT add `use` imports or `#!` attributes here.

impl FileClassifier {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a file should be parsed, with option to include large files
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn should_parse_with_options(
        &self,
        path: &Path,
        content: &[u8],
        include_large_files: bool,
    ) -> ParseDecision {
        // Fast path: empty files
        if content.is_empty() {
            return ParseDecision::Skip(SkipReason::EmptyFile);
        }

        // Fast path: file size check
        if content.len() > self.max_file_size {
            return ParseDecision::Skip(SkipReason::FileTooLarge);
        }

        // Check for large files that are likely minified/generated
        // Skip this check if include_large_files is true
        if !include_large_files && content.len() > LARGE_FILE_THRESHOLD {
            return ParseDecision::Skip(SkipReason::LargeFile);
        }

        // Check if build artifact
        if self.is_build_artifact(path) {
            return ParseDecision::Skip(SkipReason::BuildArtifact);
        }

        // Fast path: vendor directory detection
        if self.skip_vendor && self.is_vendor_path(path) {
            return ParseDecision::Skip(SkipReason::VendorDirectory);
        }

        // Content-based detection (deterministic)
        let sample = &content[..content.len().min(1024)];

        // Check if binary content
        if self.is_binary(sample) {
            return ParseDecision::Skip(SkipReason::BinaryContent);
        }

        // Line length check (prevents parser OOM) - check before minified detection
        if let Ok(text) = std::str::from_utf8(content) {
            if text.lines().any(|l| l.len() > self.max_line_length) {
                return ParseDecision::Skip(SkipReason::LineTooLong);
            }
        }

        // Check if minified
        if self.is_minified(sample) {
            return ParseDecision::Skip(SkipReason::MinifiedContent);
        }

        ParseDecision::Parse
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn should_parse(&self, path: &Path, content: &[u8]) -> ParseDecision {
        self.should_parse_with_options(path, content, false)
    }

    fn is_vendor_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check path patterns
        if self
            .vendor_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
        {
            return true;
        }

        // Check filename patterns
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            for pattern in &VENDOR_RULES.file_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&name_str) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn is_binary(&self, sample: &[u8]) -> bool {
        // Check for null bytes (common in binary files)
        if sample.contains(&0) {
            return true;
        }

        // Check for high proportion of non-printable characters
        let non_printable = sample
            .iter()
            .filter(|&&b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t')
            .count();

        non_printable as f64 / sample.len() as f64 > 0.3
    }

    fn is_minified(&self, sample: &[u8]) -> bool {
        // Check content signatures
        for sig in &VENDOR_RULES.content_signatures {
            if sample.starts_with(sig) {
                return true;
            }
        }

        // Entropy-based detection: minified JS has ~6.5 bits/char
        let entropy = calculate_shannon_entropy(sample);

        // Also check for lack of newlines (common in minified code)
        let newline_count = sample.iter().filter(|&&b| b == b'\n').count();
        let newline_ratio = newline_count as f64 / sample.len() as f64;

        entropy > MINIFIED_ENTROPY_THRESHOLD || newline_ratio < 0.001
    }

    fn is_build_artifact(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check against build artifact patterns
        BUILD_PATTERNS
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }
}

/// Calculate Shannon entropy of a byte sequence
fn calculate_shannon_entropy(data: &[u8]) -> f64 {
    let mut frequencies = [0u32; 256];
    for &byte in data {
        frequencies[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &frequencies {
        if count > 0 {
            let p = f64::from(count) / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}
