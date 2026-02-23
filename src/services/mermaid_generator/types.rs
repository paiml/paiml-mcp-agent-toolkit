#![cfg_attr(coverage_nightly, coverage(off))]
//! Types for the Mermaid diagram generator

use crate::services::semantic_naming::SemanticNamer;

/// Mermaid diagram generator
pub struct MermaidGenerator {
    pub(super) options: MermaidOptions,
    pub(super) namer: SemanticNamer,
}

/// Configuration options for Mermaid diagram generation
#[derive(Default)]
pub struct MermaidOptions {
    pub max_depth: Option<usize>,
    pub filter_external: bool,
    pub group_by_module: bool,
    pub show_complexity: bool,
}

impl MermaidGenerator {
    /// Creates a new `MermaidGenerator` with the given options
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    ///
    /// let options = MermaidOptions::default();
    /// let generator = MermaidGenerator::new(options);
    /// // Generator ready to create Mermaid diagrams
    /// ```
    #[must_use]
    pub fn new(options: MermaidOptions) -> Self {
        Self {
            options,
            namer: SemanticNamer::new(),
        }
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new(MermaidOptions::default())
    }
}
