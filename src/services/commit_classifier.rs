#![cfg_attr(coverage_nightly, coverage(off))]
//! Commit message classifier using Naive Bayes
//!
//! Classifies commit messages into categories like ASTTransform, TraitBounds, etc.
//! Uses a pre-trained model exported from `models/train_classifier.py`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Naive Bayes classifier for commit messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitClassifier {
    /// Word to index mapping
    vocabulary: HashMap<String, usize>,
    /// Class names
    classes: Vec<String>,
    /// Log prior probabilities per class
    class_priors: HashMap<String, f64>,
    /// Log feature probabilities: class -> [log P(word|class) for each word in vocab]
    feature_log_probs: HashMap<String, Vec<f64>>,
    /// Model metadata
    #[serde(default)]
    metadata: ClassifierMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassifierMetadata {
    pub train_samples: usize,
    pub vocab_size: usize,
    pub alpha: f64,
}

/// Classification result
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Predicted class
    pub class: String,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// All class scores (for debugging)
    pub scores: HashMap<String, f64>,
}

impl CommitClassifier {
    /// Load classifier from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read model file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse model: {}", e))
    }

    /// Load the embedded sovereign stack classifier
    pub fn load_sovereign_stack() -> Result<Self, String> {
        // Try multiple locations
        let locations = [
            "models/sovereign-stack-classifier.json",
            "../models/sovereign-stack-classifier.json",
        ];

        for loc in &locations {
            if Path::new(loc).exists() {
                return Self::load(loc);
            }
        }

        // Try from executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let model_path = exe_dir.join("models/sovereign-stack-classifier.json");
                if model_path.exists() {
                    return Self::load(&model_path);
                }
            }
        }

        Err("Could not find sovereign-stack-classifier.json".to_string())
    }

    /// Tokenize text into words
    fn tokenize(text: &str) -> Vec<String> {
        let text = text.to_lowercase();
        // Remove patterns that aren't useful
        let text = regex::Regex::new(r"co-authored-by:.*")
            .expect("valid co-authored-by regex")
            .replace_all(&text, "");
        let text = regex::Regex::new(r"[a-f0-9]{40}")
            .expect("valid SHA regex")
            .replace_all(&text, "");
        let text = regex::Regex::new(r"refs?\s+\w+-\d+")
            .expect("valid refs regex")
            .replace_all(&text, "");

        // Extract words
        let word_re = regex::Regex::new(r"[a-z]+").expect("valid word regex");
        let stopwords: std::collections::HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "must", "shall", "can",
            "need", "dare", "ought", "used", "this", "that", "these", "those", "it", "its", "from",
            "by", "as", "not", "all", "each", "every", "both", "few", "more", "most", "other",
            "some", "such", "no", "nor", "only", "own", "same", "so", "than", "too", "very",
        ]
        .into_iter()
        .collect();

        word_re
            .find_iter(&text)
            .map(|m| m.as_str().to_string())
            .filter(|w| w.len() > 2 && !stopwords.contains(w.as_str()))
            .collect()
    }

    /// Classify a commit message
    pub fn classify(&self, text: &str) -> ClassificationResult {
        let tokens = Self::tokenize(text);
        let mut scores: HashMap<String, f64> = HashMap::new();

        // Calculate log probability for each class
        for class in &self.classes {
            let mut score = *self.class_priors.get(class).unwrap_or(&-10.0);

            if let Some(probs) = self.feature_log_probs.get(class) {
                for token in &tokens {
                    if let Some(&idx) = self.vocabulary.get(token) {
                        if idx < probs.len() {
                            score += probs[idx];
                        }
                    }
                }
            }

            scores.insert(class.clone(), score);
        }

        // Find best class
        let (best_class, _best_score) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(c, s)| (c.clone(), *s))
            .unwrap_or_else(|| {
                (
                    self.classes.first().cloned().unwrap_or_default(),
                    f64::NEG_INFINITY,
                )
            });

        // Convert to probabilities for confidence
        let max_score = scores.values().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_scores: HashMap<String, f64> = scores
            .iter()
            .map(|(c, s)| (c.clone(), (s - max_score).exp()))
            .collect();
        let total: f64 = exp_scores.values().sum();
        let confidence = exp_scores.get(&best_class).unwrap_or(&0.0) / total;

        ClassificationResult {
            class: best_class,
            confidence,
            scores,
        }
    }

    /// Get available classes
    pub fn classes(&self) -> &[String] {
        &self.classes
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = CommitClassifier::tokenize("Fix memory leak in parser module");
        assert!(tokens.contains(&"fix".to_string()));
        assert!(tokens.contains(&"memory".to_string()));
        assert!(tokens.contains(&"leak".to_string()));
        assert!(tokens.contains(&"parser".to_string()));
        assert!(tokens.contains(&"module".to_string()));
        // Stopwords should be filtered
        assert!(!tokens.contains(&"in".to_string()));
    }

    #[test]
    fn test_tokenize_removes_patterns() {
        let tokens = CommitClassifier::tokenize(
            "Fix bug\n\nRefs PMAT-123\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
        );
        assert!(tokens.contains(&"fix".to_string()));
        assert!(tokens.contains(&"bug".to_string()));
        // Should not contain filtered patterns
        assert!(!tokens.contains(&"refs".to_string()));
        assert!(!tokens.contains(&"claude".to_string()));
    }

    #[test]
    fn test_classifier_structure() {
        // Test that classifier can be constructed with mock data
        let classifier = CommitClassifier {
            vocabulary: [("fix".to_string(), 0), ("bug".to_string(), 1)]
                .into_iter()
                .collect(),
            classes: vec!["ASTTransform".to_string(), "TraitBounds".to_string()],
            class_priors: [
                ("ASTTransform".to_string(), -0.5),
                ("TraitBounds".to_string(), -0.5),
            ]
            .into_iter()
            .collect(),
            feature_log_probs: [
                ("ASTTransform".to_string(), vec![-1.0, -2.0]),
                ("TraitBounds".to_string(), vec![-2.0, -1.0]),
            ]
            .into_iter()
            .collect(),
            metadata: ClassifierMetadata::default(),
        };

        let result = classifier.classify("fix the bug");
        assert!(!result.class.is_empty());
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_classifier_classes_method() {
        let classifier = CommitClassifier {
            vocabulary: HashMap::new(),
            classes: vec!["Class1".to_string(), "Class2".to_string()],
            class_priors: HashMap::new(),
            feature_log_probs: HashMap::new(),
            metadata: ClassifierMetadata::default(),
        };

        let classes = classifier.classes();
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0], "Class1");
        assert_eq!(classes[1], "Class2");
    }

    #[test]
    fn test_classifier_metadata_default() {
        let meta = ClassifierMetadata::default();
        assert_eq!(meta.train_samples, 0);
        assert_eq!(meta.vocab_size, 0);
        assert_eq!(meta.alpha, 0.0);
    }

    #[test]
    fn test_classification_result_fields() {
        let result = ClassificationResult {
            class: "TestClass".to_string(),
            confidence: 0.85,
            scores: [("TestClass".to_string(), -1.5)]
                .into_iter()
                .collect(),
        };
        assert_eq!(result.class, "TestClass");
        assert_eq!(result.confidence, 0.85);
        assert!(result.scores.contains_key("TestClass"));
    }

    #[test]
    fn test_tokenize_short_words_filtered() {
        let tokens = CommitClassifier::tokenize("a ab abc abcd");
        // Short words (<=2 chars) should be filtered
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"ab".to_string()));
        assert!(tokens.contains(&"abc".to_string()));
        assert!(tokens.contains(&"abcd".to_string()));
    }

    #[test]
    fn test_classify_empty_vocabulary() {
        let classifier = CommitClassifier {
            vocabulary: HashMap::new(),
            classes: vec!["Default".to_string()],
            class_priors: [("Default".to_string(), -1.0)]
                .into_iter()
                .collect(),
            feature_log_probs: [("Default".to_string(), vec![])]
                .into_iter()
                .collect(),
            metadata: ClassifierMetadata::default(),
        };

        let result = classifier.classify("any text here");
        assert_eq!(result.class, "Default");
    }

    #[test]
    fn test_classifier_load_nonexistent_file() {
        let result = CommitClassifier::load("/nonexistent/path/model.json");
        assert!(result.is_err());
    }
}
