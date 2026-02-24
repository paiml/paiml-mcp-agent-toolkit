#![cfg_attr(coverage_nightly, coverage(off))]
//! README file compression for efficient context generation
//!
//! This module intelligently compresses README files by identifying and
//! preserving the most important sections while filtering out less relevant
//! content. It uses importance scoring to ensure critical project information
//! is retained while reducing file size for AI context windows.
//!
//! # Compression Strategy
//!
//! The compressor assigns importance scores to different section types:
//! - **High Priority (0.9)**: Overview, Architecture, API, Core Concepts
//! - **Medium Priority (0.6)**: Features, Usage, Installation, Configuration
//! - **Low Priority (0.3)**: Examples, Troubleshooting, FAQ
//! - **Filtered (0.1)**: Badges, License, Contributing, Changelog

use crate::models::project_meta::{CompressedReadme, CompressedSection};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::collections::HashMap;
use tracing::debug;

pub struct ReadmeCompressor {
    section_importance: HashMap<String, f32>,
    #[allow(dead_code)]
    max_section_tokens: usize,
}

impl Default for ReadmeCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Section {
    title: String,
    level: u8,
    paragraphs: Vec<String>,
    lists: Vec<List>,
    #[allow(dead_code)]
    code_snippets: Vec<String>,
}

#[derive(Debug)]
struct List {
    items: Vec<String>,
}

// Core implementation: new, compress, scoring, truncation, feature extraction
include!("readme_compressor_impl.rs");

// Markdown parsing: handle_heading, handle_text, handle_list_end, parse_markdown_sections
include!("readme_compressor_parser.rs");

// Tests
include!("readme_compressor_tests.rs");
