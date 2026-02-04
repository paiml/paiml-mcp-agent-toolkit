//! Output abstraction for CLI handlers
//!
//! This module provides a trait-based abstraction for handler output,
//! enabling dependency injection and testability for all CLI handlers.
//!
//! # Design Rationale (Five Whys Root Cause)
//!
//! Handlers were untestable because they used `println!`/`eprintln!` directly.
//! By injecting an `OutputWriter`, we can:
//! - Test handlers with a mock writer that captures output
//! - Verify output content in unit tests
//! - Achieve >90% coverage on handler code
//!
//! # Usage
//!
//! ```rust,ignore
//! // Production code
//! let mut writer = StdoutWriter::new();
//! handle_complexity(&mut writer, args).await?;
//!
//! // Test code
//! let mut writer = TestWriter::new();
//! handle_complexity(&mut writer, args).await?;
//! assert!(writer.contains("Analyzing"));
//! ```

use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

/// Trait for handler output - enables testable CLI handlers
pub trait OutputWriter {
    /// Write a status message (typically to stderr)
    /// Used for progress indicators, spinners, etc.
    fn status(&mut self, msg: &str);

    /// Write a result/data message (typically to stdout)
    /// Used for actual command output
    fn result(&mut self, msg: &str);

    /// Write a warning message (typically to stderr with ⚠️)
    fn warning(&mut self, msg: &str);

    /// Write an error message (typically to stderr with ❌)
    fn error(&mut self, msg: &str);

    /// Write a success message (typically to stderr with ✅)
    fn success(&mut self, msg: &str);

    /// Write a debug/info message (typically to stderr with 🔍)
    fn info(&mut self, msg: &str);

    /// Flush output buffers
    fn flush(&mut self);
}

/// Production output writer - writes to stdout/stderr
#[derive(Default)]
pub struct StdoutWriter {
    _private: (),
}

impl StdoutWriter {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl OutputWriter for StdoutWriter {
    fn status(&mut self, msg: &str) {
        eprintln!("{msg}");
    }

    fn result(&mut self, msg: &str) {
        println!("{msg}");
    }

    fn warning(&mut self, msg: &str) {
        eprintln!("⚠️  {msg}");
    }

    fn error(&mut self, msg: &str) {
        eprintln!("❌ {msg}");
    }

    fn success(&mut self, msg: &str) {
        eprintln!("✅ {msg}");
    }

    fn info(&mut self, msg: &str) {
        eprintln!("🔍 {msg}");
    }

    fn flush(&mut self) {
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
    }
}

/// Test output writer - captures all output for assertions
#[derive(Default, Clone)]
pub struct TestWriter {
    pub status_messages: Vec<String>,
    pub results: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub successes: Vec<String>,
    pub infos: Vec<String>,
}

impl TestWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any output contains the given substring
    pub fn contains(&self, needle: &str) -> bool {
        self.all_output().any(|s| s.contains(needle))
    }

    /// Check if result output contains the given substring
    pub fn result_contains(&self, needle: &str) -> bool {
        self.results.iter().any(|s| s.contains(needle))
    }

    /// Check if status output contains the given substring
    pub fn status_contains(&self, needle: &str) -> bool {
        self.status_messages.iter().any(|s| s.contains(needle))
    }

    /// Get all output as an iterator
    pub fn all_output(&self) -> impl Iterator<Item = &String> {
        self.status_messages
            .iter()
            .chain(self.results.iter())
            .chain(self.warnings.iter())
            .chain(self.errors.iter())
            .chain(self.successes.iter())
            .chain(self.infos.iter())
    }

    /// Get combined output as a single string
    pub fn combined_output(&self) -> String {
        let mut output = String::new();
        for msg in self.all_output() {
            let _ = writeln!(output, "{msg}");
        }
        output
    }

    /// Clear all captured output
    pub fn clear(&mut self) {
        self.status_messages.clear();
        self.results.clear();
        self.warnings.clear();
        self.errors.clear();
        self.successes.clear();
        self.infos.clear();
    }

    /// Check if no output was produced
    pub fn is_empty(&self) -> bool {
        self.status_messages.is_empty()
            && self.results.is_empty()
            && self.warnings.is_empty()
            && self.errors.is_empty()
            && self.successes.is_empty()
            && self.infos.is_empty()
    }
}

impl OutputWriter for TestWriter {
    fn status(&mut self, msg: &str) {
        self.status_messages.push(msg.to_string());
    }

    fn result(&mut self, msg: &str) {
        self.results.push(msg.to_string());
    }

    fn warning(&mut self, msg: &str) {
        self.warnings.push(msg.to_string());
    }

    fn error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }

    fn success(&mut self, msg: &str) {
        self.successes.push(msg.to_string());
    }

    fn info(&mut self, msg: &str) {
        self.infos.push(msg.to_string());
    }

    fn flush(&mut self) {
        // No-op for test writer
    }
}

/// Null output writer - discards all output (useful for benchmarks)
#[derive(Default)]
pub struct NullWriter;

impl NullWriter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputWriter for NullWriter {
    fn status(&mut self, _msg: &str) {}
    fn result(&mut self, _msg: &str) {}
    fn warning(&mut self, _msg: &str) {}
    fn error(&mut self, _msg: &str) {}
    fn success(&mut self, _msg: &str) {}
    fn info(&mut self, _msg: &str) {}
    fn flush(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_writer_creation() {
        let writer = StdoutWriter::new();
        // StdoutWriter is a ZST, just verify it can be created
        drop(writer);
    }

    #[test]
    fn test_test_writer_captures_output() {
        let mut writer = TestWriter::new();

        writer.status("Processing...");
        writer.result("42");
        writer.warning("Low memory");
        writer.error("Failed");
        writer.success("Done");
        writer.info("Analyzing");

        assert_eq!(writer.status_messages.len(), 1);
        assert_eq!(writer.results.len(), 1);
        assert_eq!(writer.warnings.len(), 1);
        assert_eq!(writer.errors.len(), 1);
        assert_eq!(writer.successes.len(), 1);
        assert_eq!(writer.infos.len(), 1);

        assert!(writer.contains("Processing"));
        assert!(writer.contains("42"));
        assert!(writer.result_contains("42"));
        assert!(writer.status_contains("Processing"));
    }

    #[test]
    fn test_test_writer_combined_output() {
        let mut writer = TestWriter::new();
        writer.status("A");
        writer.result("B");

        let combined = writer.combined_output();
        assert!(combined.contains("A"));
        assert!(combined.contains("B"));
    }

    #[test]
    fn test_test_writer_clear() {
        let mut writer = TestWriter::new();
        writer.status("msg");
        assert!(!writer.is_empty());

        writer.clear();
        assert!(writer.is_empty());
    }

    #[test]
    fn test_null_writer_discards() {
        let mut writer = NullWriter::new();
        writer.status("ignored");
        writer.result("ignored");
        writer.warning("ignored");
        writer.error("ignored");
        writer.success("ignored");
        writer.info("ignored");
        writer.flush();
        // No assertions - just verifying no panics
    }
}
