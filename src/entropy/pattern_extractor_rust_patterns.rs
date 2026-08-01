// Regexes are compiled once, not once per call. `group_by_structural_hash` used
// to look at only the first 20 matches per file; removing that cap (so a count
// stops being a cap — see pattern_extractor_utils.rs) makes these run over every
// match in every file, and re-compiling six regexes per file plus four per match
// turned a 1.4s analysis of pmat's own src/ into minutes.
static RE_RESULT_HANDLING: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?m)^\s*(match|if let)\s+.*Result\s*<.*>\s*\{")
        .expect("Hardcoded regex pattern must be valid")
});

// Only match multi-condition validation blocks, not standalone `.len()` /
// `.is_empty()` calls (those are standard Rust idioms). Require at least two
// chained conditions or comparisons on the same line.
static RE_DATA_VALIDATION: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(
        r"(?m)if\s+.*\.(is_empty|len|contains|starts_with|ends_with)\(.*(\&\&|\|\||\.len\(\)\s*[<>=])",
    )
    .expect("Hardcoded regex pattern must be valid")
});

// Standalone `.lock()` calls on mutexes are idiomatic Rust, not duplication.
// Only flag when the same resource management sequence repeats.
static RE_RESOURCE_MANAGEMENT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?m)\.(open|close|lock|unlock|acquire|release)\(\)")
        .expect("Hardcoded regex pattern must be valid")
});

static RE_CONTROL_FLOW: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?m)^\s*}\s*else\s+if\s+").expect("Hardcoded regex pattern must be valid")
});

// Iterator chains — individual iterator combinators are idiomatic Rust, so the
// per-file threshold (see RUST_PATTERN_THRESHOLDS) is what suppresses noise.
static RE_DATA_TRANSFORMATION: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\.(map|filter|filter_map|flat_map|fold|reduce)\(")
        .expect("Hardcoded regex pattern must be valid")
});

// HTTP/API calls (reqwest, fetch, etc.). Excludes bare `.get(` which matches
// HashMap/BTreeMap/Vec accessors; only qualified HTTP patterns.
static RE_API_CALL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?m)(client\.|http\.|fetch\(|\.post\(|\.put\(|\.delete\()")
        .expect("Hardcoded regex pattern must be valid")
});

impl PatternExtractor {
    /// Run one construct's extractor: match, apply its per-file threshold, group.
    ///
    /// Both thresholds come from `RUST_PATTERN_THRESHOLDS` so that the numbers
    /// `EntropyAnalyzer::measurement_note` quotes to the user are the numbers
    /// actually enforced here.
    fn extract_with_threshold(
        &self,
        regex: &regex::Regex,
        file_path: &Path,
        content: &str,
        pattern_type: PatternType,
        collection: &mut PatternCollection,
    ) {
        let threshold = rust_threshold(pattern_type);
        let matches: Vec<_> = regex.find_iter(content).collect();

        if matches.len() >= threshold.min_matches_in_file {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                pattern_type,
                threshold.min_identical,
                collection,
            );
        }
    }

    /// Extract error handling patterns
    fn extract_error_handling_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        self.extract_with_threshold(
            &RE_RESULT_HANDLING,
            file_path,
            content,
            PatternType::ErrorHandling,
            collection,
        );
        Ok(())
    }

    /// Extract data validation patterns
    fn extract_data_validation_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        self.extract_with_threshold(
            &RE_DATA_VALIDATION,
            file_path,
            content,
            PatternType::DataValidation,
            collection,
        );
        Ok(())
    }

    /// Extract resource management patterns
    fn extract_resource_management_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        self.extract_with_threshold(
            &RE_RESOURCE_MANAGEMENT,
            file_path,
            content,
            PatternType::ResourceManagement,
            collection,
        );
        Ok(())
    }

    /// Extract control flow patterns
    fn extract_control_flow_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        self.extract_with_threshold(
            &RE_CONTROL_FLOW,
            file_path,
            content,
            PatternType::ControlFlow,
            collection,
        );
        Ok(())
    }

    /// Extract data transformation patterns
    fn extract_data_transformation_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        self.extract_with_threshold(
            &RE_DATA_TRANSFORMATION,
            file_path,
            content,
            PatternType::DataTransformation,
            collection,
        );
        Ok(())
    }

    /// Extract API call patterns
    fn extract_api_call_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        self.extract_with_threshold(
            &RE_API_CALL,
            file_path,
            content,
            PatternType::ApiCall,
            collection,
        );
        Ok(())
    }
}
