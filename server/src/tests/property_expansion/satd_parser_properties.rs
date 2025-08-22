//! Property-based tests for SATD (Self-Admitted Technical Debt) parser
//! 
//! This module verifies that the SATD parser handles all comment formats
//! robustly without panicking.

use proptest::prelude::*;
use std::panic;

/// SATD marker types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatdMarker {
    Todo,
    Fixme,
    Hack,
    Xxx,
    Optimize,
    Bug,
    Refactor,
    Note,
}

impl SatdMarker {
    pub fn as_str(&self) -> &str {
        match self {
            SatdMarker::Todo => "TODO",
            SatdMarker::Fixme => "FIXME",
            SatdMarker::Hack => "HACK",
            SatdMarker::Xxx => "XXX",
            SatdMarker::Optimize => "OPTIMIZE",
            SatdMarker::Bug => "BUG",
            SatdMarker::Refactor => "REFACTOR",
            SatdMarker::Note => "NOTE",
        }
    }
}

/// Comment style
#[derive(Debug, Clone)]
pub enum CommentStyle {
    SingleLine,      // //
    MultiLine,       // /* */
    DocSingleLine,   // ///
    DocInner,        // //!
}

impl CommentStyle {
    pub fn wrap(&self, content: &str) -> String {
        match self {
            CommentStyle::SingleLine => format!("// {}", content),
            CommentStyle::MultiLine => format!("/* {} */", content),
            CommentStyle::DocSingleLine => format!("/// {}", content),
            CommentStyle::DocInner => format!("//! {}", content),
        }
    }
}

/// SATD detection result
#[derive(Debug, Clone)]
pub struct SatdDetection {
    pub marker: SatdMarker,
    pub description: String,
    pub line: usize,
    pub confidence: f64,
}

/// SATD detector
pub struct SatdDetector {
    markers: Vec<SatdMarker>,
}

impl SatdDetector {
    pub fn new() -> Self {
        Self {
            markers: vec![
                SatdMarker::Todo,
                SatdMarker::Fixme,
                SatdMarker::Hack,
                SatdMarker::Xxx,
                SatdMarker::Optimize,
                SatdMarker::Bug,
                SatdMarker::Refactor,
                SatdMarker::Note,
            ],
        }
    }
    
    /// Parse a comment for SATD markers
    pub fn parse_comment(&self, comment: &str) -> Option<SatdDetection> {
        // Remove comment delimiters
        let cleaned = self.clean_comment(comment);
        
        // Look for markers
        for marker in &self.markers {
            let marker_str = marker.as_str();
            if let Some(pos) = cleaned.to_uppercase().find(marker_str) {
                // Extract description after marker
                let desc_start = pos + marker_str.len();
                let description = if desc_start < cleaned.len() {
                    cleaned[desc_start..].trim_start_matches(':').trim().to_string()
                } else {
                    String::new()
                };
                
                return Some(SatdDetection {
                    marker: *marker,
                    description,
                    line: 0, // Would be set by caller
                    confidence: self.calculate_confidence(&cleaned, *marker),
                });
            }
        }
        
        None
    }
    
    /// Clean comment by removing delimiters
    fn clean_comment(&self, comment: &str) -> String {
        let mut cleaned = comment.trim();
        
        // Remove single-line comment markers
        if cleaned.starts_with("//!") {
            cleaned = &cleaned[3..];
        } else if cleaned.starts_with("///") {
            cleaned = &cleaned[3..];
        } else if cleaned.starts_with("//") {
            cleaned = &cleaned[2..];
        }
        
        // Remove multi-line comment markers
        if cleaned.starts_with("/*") && cleaned.ends_with("*/") {
            cleaned = &cleaned[2..cleaned.len() - 2];
        }
        
        cleaned.trim().to_string()
    }
    
    /// Calculate confidence score for detection
    fn calculate_confidence(&self, comment: &str, marker: SatdMarker) -> f64 {
        let mut confidence = 0.5; // Base confidence
        
        // Boost confidence if marker is at the beginning
        if comment.to_uppercase().starts_with(marker.as_str()) {
            confidence += 0.2;
        }
        
        // Boost confidence if followed by colon
        let marker_pattern = format!("{}:", marker.as_str());
        if comment.to_uppercase().contains(&marker_pattern) {
            confidence += 0.15;
        }
        
        // Boost confidence for common patterns
        let common_patterns = ["fix this", "needs refactoring", "temporary", "hack"];
        for pattern in &common_patterns {
            if comment.to_lowercase().contains(pattern) {
                confidence += 0.1;
                break;
            }
        }
        
        confidence.min(1.0)
    }
    
    /// Parse source code for all SATD occurrences
    pub fn parse_source(&self, source: &str) -> Vec<SatdDetection> {
        let mut detections = Vec::new();
        
        for (line_num, line) in source.lines().enumerate() {
            if let Some(mut detection) = self.parse_comment(line) {
                detection.line = line_num + 1;
                detections.push(detection);
            }
        }
        
        detections
    }
}

/// Generate a comment with markers
fn generate_comment_with_markers(
    style: CommentStyle,
    markers: Vec<SatdMarker>,
    noise: &str,
) -> String {
    if markers.is_empty() {
        style.wrap(noise)
    } else {
        let marker_text = markers
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        let content = if noise.is_empty() {
            marker_text
        } else {
            format!("{}: {}", marker_text, noise)
        };
        
        style.wrap(&content)
    }
}

// Arbitrary implementations for property testing
impl Arbitrary for SatdMarker {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(SatdMarker::Todo),
            Just(SatdMarker::Fixme),
            Just(SatdMarker::Hack),
            Just(SatdMarker::Xxx),
            Just(SatdMarker::Optimize),
            Just(SatdMarker::Bug),
            Just(SatdMarker::Refactor),
            Just(SatdMarker::Note),
        ].boxed()
    }
}

impl Arbitrary for CommentStyle {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(CommentStyle::SingleLine),
            Just(CommentStyle::MultiLine),
            Just(CommentStyle::DocSingleLine),
            Just(CommentStyle::DocInner),
        ].boxed()
    }
}

prop_compose! {
    /// Strategy for generating comment blocks
    fn arb_comment_block()
        (style in any::<CommentStyle>(),
         markers in prop::collection::vec(any::<SatdMarker>(), 0..5),
         noise in "[a-zA-Z0-9 .,!?\\-_()\\[\\]{}@#$%^&*+=|\\\\/<>~`'\";:]{0,100}")
        -> String
    {
        generate_comment_with_markers(style, markers, &noise)
    }
}

proptest! {
    /// Property: SATD parser never panics
    #[test]
    fn satd_parser_totality(comment in arb_comment_block()) {
        let detector = SatdDetector::new();
        
        let result = panic::catch_unwind(|| {
            detector.parse_comment(&comment)
        });
        
        prop_assert!(result.is_ok(), "SATD parser panicked on: {}", comment);
    }
    
    /// Property: Known markers are always detected
    #[test]
    fn satd_detection_soundness(
        prefix in "[a-zA-Z0-9 ]{0,20}",
        marker in any::<SatdMarker>(),
        suffix in "[a-zA-Z0-9 ]{0,20}"
    ) {
        let comment = format!("{} {} {}", prefix, marker.as_str(), suffix);
        let detector = SatdDetector::new();
        let detected = detector.parse_comment(&comment);
        
        // Property: Known markers are always detected
        prop_assert!(
            detected.is_some(),
            "Failed to detect marker '{}' in comment: {}",
            marker.as_str(),
            comment
        );
        
        // Property: Correct marker is identified
        if let Some(detection) = detected {
            prop_assert_eq!(
                detection.marker, marker,
                "Wrong marker detected"
            );
        }
    }
    
    /// Property: Empty comments don't produce detections
    #[test]
    fn satd_empty_comment_no_detection(style in any::<CommentStyle>()) {
        let empty_comment = style.wrap("");
        let detector = SatdDetector::new();
        let detected = detector.parse_comment(&empty_comment);
        
        prop_assert!(
            detected.is_none(),
            "Empty comment produced detection: {:?}",
            detected
        );
    }
    
    /// Property: Case insensitive detection
    #[test]
    fn satd_case_insensitive(
        marker in any::<SatdMarker>(),
        use_lowercase in prop::bool::ANY
    ) {
        let marker_text = if use_lowercase {
            marker.as_str().to_lowercase()
        } else {
            marker.as_str().to_uppercase()
        };
        
        let comment = format!("// {}: Need to fix this", marker_text);
        let detector = SatdDetector::new();
        let detected = detector.parse_comment(&comment);
        
        prop_assert!(
            detected.is_some(),
            "Case variation not detected: {}",
            comment
        );
    }
    
    /// Property: Multiple markers in one comment
    #[test]
    fn satd_multiple_markers(markers in prop::collection::vec(any::<SatdMarker>(), 2..5)) {
        let marker_text = markers
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        let comment = format!("// {}: Multiple issues here", marker_text);
        let detector = SatdDetector::new();
        let detected = detector.parse_comment(&comment);
        
        // Should detect at least the first marker
        prop_assert!(
            detected.is_some(),
            "Failed to detect any marker in: {}",
            comment
        );
        
        if let Some(detection) = detected {
            // The detected marker should be one of the input markers
            prop_assert!(
                markers.contains(&detection.marker),
                "Detected unexpected marker"
            );
        }
    }
    
    /// Property: Confidence score is bounded
    #[test]
    fn satd_confidence_bounded(comment in arb_comment_block()) {
        let detector = SatdDetector::new();
        
        if let Some(detection) = detector.parse_comment(&comment) {
            prop_assert!(
                detection.confidence >= 0.0 && detection.confidence <= 1.0,
                "Confidence out of bounds: {}",
                detection.confidence
            );
        }
    }
    
    /// Property: Parser handles Unicode
    #[test]
    fn satd_unicode_handling(
        marker in any::<SatdMarker>(),
        unicode_text in "[\u{0080}-\u{10FFFF}]{0,20}"
    ) {
        let comment = format!("// {}: {}", marker.as_str(), unicode_text);
        let detector = SatdDetector::new();
        
        let result = panic::catch_unwind(|| {
            detector.parse_comment(&comment)
        });
        
        prop_assert!(
            result.is_ok(),
            "Parser failed on Unicode: {}",
            comment
        );
    }
    
    /// Property: Source parsing consistency
    #[test]
    fn satd_source_parsing_consistency(
        lines in prop::collection::vec(arb_comment_block(), 1..20)
    ) {
        let source = lines.join("\n");
        let detector = SatdDetector::new();
        
        let result = panic::catch_unwind(|| {
            detector.parse_source(&source)
        });
        
        prop_assert!(result.is_ok(), "Source parsing panicked");
        
        if let Ok(detections) = result {
            // Line numbers should be valid
            for detection in detections {
                prop_assert!(
                    detection.line > 0 && detection.line <= lines.len(),
                    "Invalid line number: {}",
                    detection.line
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_satd_detection_basic() {
        let detector = SatdDetector::new();
        
        // Test TODO detection
        let todo_comment = "// TODO: Fix this later";
        let detection = detector.parse_comment(todo_comment);
        assert!(detection.is_some());
        assert_eq!(detection.unwrap().marker, SatdMarker::Todo);
        
        // Test FIXME detection
        let fixme_comment = "/* FIXME: Memory leak here */";
        let detection = detector.parse_comment(fixme_comment);
        assert!(detection.is_some());
        assert_eq!(detection.unwrap().marker, SatdMarker::Fixme);
        
        // Test no marker
        let normal_comment = "// This is a normal comment";
        let detection = detector.parse_comment(normal_comment);
        assert!(detection.is_none());
    }
    
    #[test]
    fn test_comment_cleaning() {
        let detector = SatdDetector::new();
        
        // Single-line comments
        assert_eq!(detector.clean_comment("// Hello"), "Hello");
        assert_eq!(detector.clean_comment("/// Doc comment"), "Doc comment");
        assert_eq!(detector.clean_comment("//! Inner doc"), "Inner doc");
        
        // Multi-line comments
        assert_eq!(detector.clean_comment("/* Multi */"), "Multi");
        assert_eq!(detector.clean_comment("/* \n Multi \n */"), "Multi");
    }
    
    #[test]
    fn test_confidence_calculation() {
        let detector = SatdDetector::new();
        
        // High confidence: marker at beginning with colon
        let high_conf = "TODO: This needs fixing";
        let conf = detector.calculate_confidence(high_conf, SatdMarker::Todo);
        assert!(conf > 0.8);
        
        // Medium confidence: marker present but not at beginning
        let med_conf = "This is a TODO item";
        let conf = detector.calculate_confidence(med_conf, SatdMarker::Todo);
        assert!(conf > 0.4 && conf < 0.8);
    }
    
    #[test]
    fn test_source_parsing() {
        let detector = SatdDetector::new();
        
        let source = r#"
// TODO: Implement this function
fn foo() {
    // Normal comment
    /* FIXME: Handle error properly */
    println!("Hello");
    // HACK: Temporary solution
}
"#;
        
        let detections = detector.parse_source(source);
        assert_eq!(detections.len(), 3);
        
        // Check markers were detected correctly
        let markers: Vec<_> = detections.iter().map(|d| d.marker).collect();
        assert!(markers.contains(&SatdMarker::Todo));
        assert!(markers.contains(&SatdMarker::Fixme));
        assert!(markers.contains(&SatdMarker::Hack));
    }
}