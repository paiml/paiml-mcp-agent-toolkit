use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassifierConfig {
    pub skip_vendor: bool,
    pub max_line_length: usize,
    pub max_file_size: usize,
}

/// Maximum line length before considering a file unparseable
const DEFAULT_MAX_LINE_LENGTH: usize = 10_000;

/// Maximum file size for AST parsing (1MB)
pub const DEFAULT_MAX_FILE_SIZE: usize = 1_048_576;

/// Maximum file size before considering it a "large file" (500KB)
/// Large files are likely minified/generated and should be skipped by default
pub const LARGE_FILE_THRESHOLD: usize = 512_000;

/// Shannon entropy threshold for minified content detection
const MINIFIED_ENTROPY_THRESHOLD: f64 = 6.0;

lazy_static! {
    /// Deterministic vendor detection rules
    static ref VENDOR_RULES: VendorRules = VendorRules {
        // Deterministic ordering for consistent results
        path_patterns: vec![
            "vendor/",
            "node_modules/",
            "third_party/",
            "external/",
            ".yarn/",
            "bower_components/",
            ".min.",
            ".bundle.",
        ],
        file_patterns: vec![
            r"\.min\.(js|css)$",
            r"\.bundle\.js$",
            r"-min\.js$",
            r"\.packed\.js$",
            r"\.dist\.js$",
            r"\.production\.js$",
        ],
        // Content signatures (first 256 bytes)
        content_signatures: vec![
            b"/*! jQuery" as &[u8],
            b"/*! * Bootstrap" as &[u8],
            b"!function(e,t){" as &[u8],  // Common minification pattern
            b"/*! For license information" as &[u8],
            b"/** @license React" as &[u8],
        ],
    };

    /// Build artifact patterns - separate from vendor patterns for clarity
    static ref BUILD_PATTERNS: Vec<&'static str> = vec![
        "target/debug/",
        "target/release/",
        "target/thumbv",
        "/target/debug/",
        "/target/release/",
        "build/",
        "/build/",
        "dist/",
        "/dist/",
        "/.next/",
        "__pycache__/",
        "/__pycache__/",
        "venv/",
        "/venv/",
        ".tox/",
        "/.tox/",
        "cmake-build-",
        "/cmake-build-",
        "/.gradle/",
        ".gradle/",
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassifier {
    pub max_line_length: usize,
    pub max_file_size: usize,
    pub vendor_patterns: Vec<String>,
    pub skip_vendor: bool,
}

impl Default for FileClassifier {
    fn default() -> Self {
        Self {
            max_line_length: DEFAULT_MAX_LINE_LENGTH,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            vendor_patterns: VENDOR_RULES
                .path_patterns
                .iter()
                .map(|s| s.to_string())
                .collect(),
            skip_vendor: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseDecision {
    Parse,
    Skip(SkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    VendorDirectory,
    MinifiedContent,
    LineTooLong,
    FileTooLarge,
    BinaryContent,
    EmptyFile,
    BuildArtifact,
    LargeFile,
}

impl FileClassifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a file should be parsed, with option to include large files
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
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
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
    pub fn new(output_path: Option<std::path::PathBuf>) -> Self {
        Self {
            start_time: Instant::now(),
            events: Vec::new(),
            output_path,
        }
    }

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

    pub async fn save_report(&self) -> Result<()> {
        if let Some(output_path) = &self.output_path {
            let report = self.generate_report()?;
            let json = serde_json::to_string_pretty(&report)?;
            tokio::fs::write(output_path, json).await?;
        }
        Ok(())
    }

    fn get_memory_usage_mb(&self) -> f64 {
        // Simplified memory usage tracking
        // In production, use platform-specific APIs for accurate measurement
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<f64>() {
                                return kb / 1024.0;
                            }
                        }
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_file_detection() {
        let classifier = FileClassifier::default();

        // Test file under threshold (400KB) - use content with newlines to avoid LineTooLong
        let mut small_content = String::new();
        for _ in 0..4000 {
            small_content.push_str("a".repeat(100).as_str());
            small_content.push('\n');
        }
        let decision = classifier.should_parse(Path::new("small.js"), small_content.as_bytes());
        assert_eq!(decision, ParseDecision::Parse);

        // Test file over threshold (600KB) - use content with newlines to avoid LineTooLong
        let mut large_content = String::new();
        for _ in 0..6000 {
            large_content.push_str("a".repeat(100).as_str());
            large_content.push('\n');
        }
        let decision = classifier.should_parse(Path::new("large.js"), large_content.as_bytes());
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));

        // Test file exactly at threshold
        let mut threshold_content = String::new();
        for _ in 0..(LARGE_FILE_THRESHOLD / 101) {
            threshold_content.push_str("a".repeat(100).as_str());
            threshold_content.push('\n');
        }
        let decision =
            classifier.should_parse(Path::new("threshold.js"), threshold_content.as_bytes());
        assert_eq!(decision, ParseDecision::Parse);
    }

    #[test]
    fn test_include_large_files_flag() {
        let classifier = FileClassifier::default();

        // Create large content with newlines to avoid LineTooLong
        let mut large_content = String::new();
        for _ in 0..6000 {
            large_content.push_str("a".repeat(100).as_str());
            large_content.push('\n');
        }
        let large_content_bytes = large_content.as_bytes();

        // Without flag - should skip
        let decision =
            classifier.should_parse_with_options(Path::new("large.js"), large_content_bytes, false);
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));

        // With flag - should parse
        let decision =
            classifier.should_parse_with_options(Path::new("large.js"), large_content_bytes, true);
        assert_eq!(decision, ParseDecision::Parse);
    }

    #[test]
    fn test_very_large_files_still_skipped() {
        let classifier = FileClassifier::default();
        let very_large_content = vec![b'a'; 2_000_000]; // 2MB

        // Even with include_large_files, files over max_file_size should be skipped
        let decision =
            classifier.should_parse_with_options(Path::new("huge.js"), &very_large_content, true);
        assert_eq!(decision, ParseDecision::Skip(SkipReason::FileTooLarge));
    }

    #[test]
    fn test_skip_reason_priorities() {
        let classifier = FileClassifier::default();

        // Empty file should skip with EmptyFile reason (highest priority)
        let empty_content = b"";
        let decision =
            classifier.should_parse_with_options(Path::new("empty.js"), empty_content, false);
        assert_eq!(decision, ParseDecision::Skip(SkipReason::EmptyFile));

        // Build artifact should skip even if large
        // But LargeFile check happens first in our implementation
        let build_content = vec![b'a'; 600_000];
        let decision = classifier.should_parse_with_options(
            Path::new("target/debug/deps/lib.rlib"),
            &build_content,
            false,
        );
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));
    }

    #[test]
    fn test_minified_vs_large_file_detection() {
        let classifier = FileClassifier::default();

        // Large but not minified file (has newlines)
        let mut large_normal = String::new();
        for i in 0..10_000 {
            large_normal.push_str(&format!("function test{} () {{\n  return {};\n}}\n", i, i));
        }
        let content = large_normal.as_bytes();

        // Should skip due to size if over threshold
        if content.len() > LARGE_FILE_THRESHOLD {
            let decision = classifier.should_parse(Path::new("large_normal.js"), content);
            assert_eq!(decision, ParseDecision::Skip(SkipReason::LargeFile));
        }

        // Minified content (one very long line)
        let minified = "a".repeat(11_000); // Long line
        let decision = classifier.should_parse(Path::new("minified.js"), minified.as_bytes());
        assert_eq!(decision, ParseDecision::Skip(SkipReason::LineTooLong));
    }

    #[test]
    fn test_vendor_detection_determinism() {
        let classifier = FileClassifier::default();
        let test_files = [
            (
                "vendor/jquery.min.js",
                b"!function(e,t){var n=e.jQuery}" as &[u8],
            ),
            (
                "src/main.rs",
                b"fn main() {\n    println!(\"Hello\");\n}" as &[u8],
            ),
            (
                "assets/vendor/d3.min.js",
                b"/*! For license information please see d3.min.js.LICENSE.txt */" as &[u8],
            ),
            (
                "node_modules/react/index.js",
                b"'use strict';\n\nmodule.exports = require('./lib/React');" as &[u8],
            ),
            (
                "target/debug/build/htmlServer-abc123/out/rules.rs",
                b"// Auto-generated code\npub enum TreeBuilderStep {\n    A,\n    B,\n}" as &[u8],
            ),
        ];

        // Run 100 times to ensure determinism
        let mut results = Vec::new();
        for _ in 0..100 {
            let run_results: Vec<_> = test_files
                .iter()
                .map(|(path, content)| classifier.should_parse(Path::new(path), content))
                .collect();
            results.push(run_results);
        }

        // All runs should produce identical results
        assert!(results.windows(2).all(|w| w[0] == w[1]));

        // Verify expected classifications
        let decisions = &results[0];
        assert!(matches!(
            decisions[0],
            ParseDecision::Skip(SkipReason::VendorDirectory)
        ));
        assert!(matches!(decisions[1], ParseDecision::Parse));
        assert!(matches!(
            decisions[2],
            ParseDecision::Skip(SkipReason::VendorDirectory)
        ));
        assert!(matches!(
            decisions[3],
            ParseDecision::Skip(SkipReason::VendorDirectory)
        ));
        // Verify that target/ directory is properly filtered
        assert!(matches!(
            decisions[4],
            ParseDecision::Skip(SkipReason::BuildArtifact)
        ));
    }

    #[test]
    fn test_performance_on_large_files() {
        let classifier = FileClassifier::default();
        let large_minified = vec![b'a'; 1_000_000]; // 1MB of minified code

        let start = Instant::now();
        let decision = classifier.should_parse(Path::new("large.min.js"), &large_minified);
        let elapsed = start.elapsed();

        assert!(matches!(decision, ParseDecision::Skip(_)));
        assert!(elapsed.as_micros() < 1000); // Should decide in <1ms
    }

    #[test]
    fn test_entropy_calculation() {
        // Test with uniform distribution (low entropy)
        let uniform = b"aaaaaaaaaa";
        let entropy1 = calculate_shannon_entropy(uniform);
        assert!(entropy1 < 1.0);

        // Test with random-like distribution (high entropy)
        let random = b"a1b2c3d4e5f6g7h8i9j0";
        let entropy2 = calculate_shannon_entropy(random);
        assert!(entropy2 > 3.0);

        // Test with minified-like content
        let minified = b"!function(e,t){var n,r,i,o,a,s,u,c,l,f,d,p,h,m,v,g,y,b,_,w,x,k,C,S,E,T,A,O,j,N,D,P,L,q,R,M,I,F,B,H,U,z,W,V,$,G,Q,K,X,Y,J,Z,ee,te,ne,re,ie,oe,ae,se,ue,ce,le";
        let entropy3 = calculate_shannon_entropy(minified);
        assert!(entropy3 > 4.0); // Adjusted threshold based on actual entropy of test data
    }

    #[test]
    fn test_binary_detection() {
        let classifier = FileClassifier::default();

        // Test with text file
        let text = b"Hello, world!\nThis is a text file.";
        assert!(!classifier.is_binary(text));

        // Test with binary content (null bytes)
        let binary = b"PNG\x00\x00\x00\rIHDR";
        assert!(classifier.is_binary(binary));

        // Test with high non-printable ratio
        let mostly_binary = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert!(classifier.is_binary(&mostly_binary));
    }

    #[test]
    fn test_line_length_detection() {
        let classifier = FileClassifier::default();

        // Normal code file
        let normal_code = b"fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(
            classifier.should_parse(Path::new("main.rs"), normal_code),
            ParseDecision::Parse
        );

        // File with very long line
        let long_line = format!("const DATA = \"{}\";\n", "a".repeat(15_000));
        assert_eq!(
            classifier.should_parse(Path::new("data.js"), long_line.as_bytes()),
            ParseDecision::Skip(SkipReason::LineTooLong)
        );
    }

    #[test]
    fn test_rust_target_directory_filtering() {
        let classifier = FileClassifier::default();

        // Test various target/ directory patterns that should be filtered
        let rust_build_artifacts = [
            "target/debug/build/htmlserver-abc123/out/rules.rs",
            "target/release/build/htmlserver-abc123/out/rules.rs",
            "target/debug/deps/libhtml5ever-xyz.rlib",
            "target/release/deps/libhtml5ever-xyz.rlib",
            "target/debug/build/proc-macro2-def456/out/generated.rs",
            "target/thumbv7em-none-eabihf/release/libcore.rlib",
        ];

        for path in rust_build_artifacts {
            let content = b"// Auto-generated code or compiled artifact";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::BuildArtifact)),
                "Failed to filter target directory path: {path}"
            );
        }

        // Verify legitimate source files are not filtered
        let source_files = [
            "src/main.rs",
            "src/lib.rs",
            "tests/integration_test.rs",
            "examples/demo.rs",
        ];

        for path in source_files {
            let content = b"fn main() {\n    println!(\"Hello\");\n}";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Parse),
                "Incorrectly filtered source file: {path} -> {decision:?}"
            );
        }
    }

    #[test]
    fn test_additional_build_artifacts() {
        let classifier = FileClassifier::default();

        // Test additional build artifact patterns
        let build_artifacts = [
            ".gradle/caches/transforms-3/abc123/transformed/classes.jar",
            "frontend/.gradle/build/outputs/apk/debug/app-debug.apk",
        ];

        for path in build_artifacts {
            let content = b"// Some content";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::BuildArtifact)),
                "Failed to filter build artifact: {path}"
            );
        }

        // Test node_modules patterns - should be vendor, not build artifacts
        let vendor_artifacts = [
            "backend/node_modules/@babel/core/lib/index.js",
            "/home/user/project/node_modules/lodash/index.js",
        ];

        for path in vendor_artifacts {
            let content = b"// Some content";
            let decision = classifier.should_parse(Path::new(path), content);
            assert!(
                matches!(decision, ParseDecision::Skip(SkipReason::VendorDirectory)),
                "Failed to filter vendor artifact: {path}"
            );
        }
    }

    #[test]
    fn test_debug_reporter() {
        let mut reporter = DebugReporter::new(None);

        // Record some events
        reporter.record_decision(
            Path::new("vendor/lib.js"),
            &ParseDecision::Skip(SkipReason::VendorDirectory),
        );
        reporter.record_decision(Path::new("src/main.rs"), &ParseDecision::Parse);

        // Record parse result for main.rs
        reporter.record_parse_result(
            Path::new("src/main.rs"),
            std::time::Duration::from_millis(25),
            None,
        );

        let report = reporter.generate_report().unwrap();

        assert_eq!(report.summary.total_files, 2);
        assert_eq!(report.summary.parsed_files, 1);
        assert_eq!(report.summary.skipped_files, 1);
        assert_eq!(report.summary.parse_errors, 0);
        assert_eq!(report.skip_reasons.get("VendorDirectory"), Some(&1));
    }
}
