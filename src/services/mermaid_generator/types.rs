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
    /// Advisory only: the renderer draws the graph it is handed and never
    /// consults this. Depth is a property of the traversal, so callers must
    /// prune the graph before rendering (`analyze dag` does, in
    /// `limit_graph_depth`) — setting it here alone changes nothing.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
