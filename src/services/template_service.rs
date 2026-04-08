#![cfg_attr(coverage_nightly, coverage(off))]
//! Template generation and rendering service.
//!
//! This module provides code generation capabilities through a flexible template
//! system. It supports multiple languages and project types, with intelligent
//! context generation and parameter validation.
//!
//! # Template Categories
//!
//! - **Project Templates**: Full project scaffolding (CLI, library, web)
//! - **Context Templates**: AI-ready code context generation
//! - **Configuration Templates**: CI/CD, Docker, deployment configs
//! - **Code Templates**: Common patterns and boilerplate
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::template_service::{generate_template, TemplateServerTrait};
//! use serde_json::{Map, Value};
//!
//! # async fn example<T: TemplateServerTrait>(server: &T) -> Result<(), Box<dyn std::error::Error>> {
//! // Generate a Rust CLI project template
//! let mut params = Map::new();
//! params.insert("project_name".to_string(), Value::String("my_app".to_string()));
//! params.insert("description".to_string(), Value::String("My CLI app".to_string()));
//!
//! let template = generate_template(
//!     server,
//!     "project/rust/cli",
//!     params
//! ).await?;
//!
//! println!("Generated: {}", template.filename);
//! println!("Checksum: {}", template.checksum);
//! # Ok(())
//! # }
//! ```ignore

use crate::models::error::TemplateError;
use crate::models::template::{GeneratedTemplate, TemplateResource};
use crate::services::context::{analyze_project, format_context_as_markdown};
use crate::services::renderer;
use crate::TemplateServerTrait;
use serde_json::Map;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;

// Result types for CLI operations

#[derive(Debug)]
/// Result of scaffold operation.
pub struct ScaffoldResult {
    pub files: Vec<GeneratedFile>,
    pub errors: Vec<ScaffoldError>,
}

#[derive(Debug)]
/// Generated file.
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub checksum: String,
}

#[derive(Debug)]
/// Error type for scaffold operations.
pub struct ScaffoldError {
    pub template: String,
    pub error: String,
}

#[derive(Debug)]
/// Result of search operation.
pub struct SearchResult {
    pub template: crate::models::template::TemplateResource,
    pub relevance: f32,
    pub matches: Vec<String>,
}

#[derive(Debug)]
/// Result of validation operation.
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug)]
/// Error type for validation operations.
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

// Template generation: generate_template, generate_context, get_template_content, scaffold_project
include!("template_service_generation.rs");

// Template listing and search: list_templates, list_all_resources, search_templates
include!("template_service_search.rs");

// URI parsing and parameter validation: parse_template_uri, validate_parameters, validate_template
include!("template_service_validation.rs");

// Unit tests and property tests
include!("template_service_tests.rs");
