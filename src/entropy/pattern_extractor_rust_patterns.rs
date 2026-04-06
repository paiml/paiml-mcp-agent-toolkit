impl PatternExtractor {
    /// Extract error handling patterns
    fn extract_error_handling_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: Result<T, E> handling
        let result_pattern = Regex::new(r"(?m)^\s*(match|if let)\s+.*Result\s*<.*>\s*\{")
            .expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = result_pattern.find_iter(content).collect();

        if matches.len() > 1 {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                PatternType::ErrorHandling,
                3,
                5,
                collection,
            );
        }

        Ok(())
    }

    /// Extract data validation patterns
    fn extract_data_validation_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: Input validation — only match multi-condition validation blocks,
        // not standalone `.len()` / `.is_empty()` calls (those are standard Rust idioms).
        // Require at least two chained conditions or comparisons on the same line.
        let validation_pattern = Regex::new(
            r"(?m)if\s+.*\.(is_empty|len|contains|starts_with|ends_with)\(.*(\&\&|\|\||\.len\(\)\s*[<>=])",
        )
        .expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = validation_pattern.find_iter(content).collect();

        // Raised threshold: standalone validation calls are idiomatic, not duplication
        if matches.len() > 5 {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                PatternType::DataValidation,
                3,
                3,
                collection,
            );
        }

        Ok(())
    }

    /// Extract resource management patterns
    fn extract_resource_management_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: File/resource management (open/close, lock/unlock)
        // Standalone `.lock()` calls on mutexes are idiomatic Rust, not duplication.
        // Only flag when the same resource management sequence repeats.
        let resource_pattern = Regex::new(r"(?m)\.(open|close|lock|unlock|acquire|release)\(\)")
            .expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = resource_pattern.find_iter(content).collect();

        // Raised threshold: individual .lock()/.open() calls are standard practice
        if matches.len() > 5 {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                PatternType::ResourceManagement,
                3,
                4,
                collection,
            );
        }

        Ok(())
    }

    /// Extract control flow patterns
    fn extract_control_flow_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: Complex if-else chains
        let if_else_pattern =
            Regex::new(r"(?m)^\s*}\s*else\s+if\s+").expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = if_else_pattern.find_iter(content).collect();

        if matches.len() > 2 {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                PatternType::ControlFlow,
                3,
                6,
                collection,
            );
        }

        Ok(())
    }

    /// Extract data transformation patterns
    fn extract_data_transformation_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: Iterator chains — only flag multi-step chains, not individual
        // .map()/.collect() calls which are standard Rust idiom.
        // Match chains of 2+ combinators: e.g. `.iter().map(...).collect()`
        let iter_pattern = Regex::new(r"\.(map|filter|filter_map|flat_map|fold|reduce)\(")
            .expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = iter_pattern.find_iter(content).collect();

        // Raised threshold: individual iterator combinators are idiomatic Rust
        if matches.len() > 8 {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                PatternType::DataTransformation,
                3,
                2,
                collection,
            );
        }

        Ok(())
    }

    /// Extract API call patterns
    fn extract_api_call_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: HTTP/API calls (reqwest, fetch, etc.)
        // Exclude bare `.get(` which matches HashMap/BTreeMap/Vec accessors.
        // Only match qualified HTTP patterns (client., http., fetch(), .post, .put, .delete).
        let api_pattern = Regex::new(r"(?m)(client\.|http\.|fetch\(|\.post\(|\.put\(|\.delete\()")
            .expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = api_pattern.find_iter(content).collect();

        // Raised threshold: isolated HTTP calls are not duplication
        if matches.len() > 3 {
            self.group_by_structural_hash(
                &matches,
                content,
                file_path,
                PatternType::ApiCall,
                3,
                3,
                collection,
            );
        }

        Ok(())
    }
}
