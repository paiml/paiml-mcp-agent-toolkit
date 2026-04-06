#![cfg_attr(coverage_nightly, coverage(off))]
//! Formatting utilities for Mermaid diagram output

use super::types::MermaidGenerator;
use crate::models::dag::{EdgeType, NodeType};

impl MermaidGenerator {
    #[inline]
    /// Returns the appropriate Mermaid arrow syntax for an edge type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    /// use pmat::models::dag::EdgeType;
    ///
    /// let generator = MermaidGenerator::new(MermaidOptions::default());
    /// assert_eq!(generator.get_edge_arrow(&EdgeType::Calls), "-->");
    /// assert_eq!(generator.get_edge_arrow(&EdgeType::Imports), "-.->");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_edge_arrow(&self, edge_type: &EdgeType) -> &'static str {
        match edge_type {
            EdgeType::Calls => "-->",
            EdgeType::Imports => "-.->",
            EdgeType::Inherits => "-->|inherits|",
            EdgeType::Implements => "-->|implements|",
            EdgeType::Uses => "---",
        }
    }

    #[inline]
    /// Returns a color for visualizing complexity levels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
    ///
    /// let generator = MermaidGenerator::new(MermaidOptions::default());
    /// assert_eq!(generator.get_complexity_color(2), "#90EE90"); // Light green
    /// assert_eq!(generator.get_complexity_color(5), "#FFD700"); // Gold
    /// assert_eq!(generator.get_complexity_color(10), "#FFA500"); // Orange
    /// assert_eq!(generator.get_complexity_color(15), "#FF6347"); // Tomato
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_complexity_color(&self, complexity: u32) -> &'static str {
        match complexity {
            1..=3 => "#90EE90",  // Light green for low complexity
            4..=7 => "#FFD700",  // Gold for medium complexity
            8..=12 => "#FFA500", // Orange for high complexity
            _ => "#FF6347",      // Tomato for very high complexity
        }
    }

    pub(super) fn get_node_stroke_style(&self, node_type: &NodeType) -> (&'static str, u32) {
        match node_type {
            NodeType::Function => (",stroke:#333,stroke-dasharray: 5 5", 2), // Dashed border for functions
            NodeType::Trait => (",stroke:#663399", 3), // Purple border for traits
            NodeType::Interface => (",stroke:#4169E1", 3), // Blue border for interfaces
            _ => ("", 2),                              // Default for others
        }
    }

    #[inline]
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn sanitize_id(&self, id: &str) -> String {
        debug_assert!(!id.is_empty(), "id must not be empty");
        // First replace common multi-character patterns
        let sanitized = id.replace("::", "_").replace(['/', '.', '-', ' '], "_");

        // Then replace any remaining non-alphanumeric characters with underscores
        let sanitized: String = sanitized
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        // Ensure it starts with a letter or underscore
        if sanitized.is_empty() {
            "_empty".to_string()
        } else if sanitized
            .chars()
            .next()
            .expect("sanitized is non-empty (checked at line 347)")
            .is_numeric()
        {
            format!("_{sanitized}")
        } else {
            sanitized
        }
    }

    #[inline]
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn escape_mermaid_label(&self, label: &str) -> String {
        // For IntelliJ compatibility, use simple character replacements instead of HTML entities
        label
            .replace('&', " and ")
            .replace('"', "'")
            .replace('<', "(")
            .replace('>', ")")
            .replace('|', " - ")
            .replace('[', "(")
            .replace(']', ")")
            .replace('{', "(")
            .replace('}', ")")
            .replace('\n', " ")
    }
}
