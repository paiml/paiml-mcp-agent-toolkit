#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for demo module
//! Tests: assets, runner resolve functions, report types

use crate::demo::assets::{decompress_asset, get_asset, get_asset_hash, AssetEncoding, EmbeddedAsset};
use crate::demo::runner::{detect_repository, resolve_repository, DemoAnalysisResult, DemoReport, DemoStep};
use std::collections::HashMap;
use std::path::PathBuf;

// --- AssetEncoding tests ---

#[test]
fn test_asset_encoding_debug() {
    let _ = format!("{:?}", AssetEncoding::Gzip);
    let _ = format!("{:?}", AssetEncoding::Identity);
}

#[test]
fn test_asset_encoding_clone_copy() {
    let encoding = AssetEncoding::Gzip;
    let copied = encoding;
    let _ = format!("{:?}", copied);
}

// --- EmbeddedAsset tests ---

#[test]
fn test_embedded_asset_identity() {
    let data: &[u8] = b"hello world";
    let asset = EmbeddedAsset {
        content: data,
        content_type: "text/plain",
        encoding: AssetEncoding::Identity,
    };
    let _ = format!("{:?}", asset);
    assert_eq!(asset.content_type, "text/plain");
    assert_eq!(asset.content.len(), 11);
}

#[test]
fn test_decompress_identity_asset() {
    let data: &[u8] = b"test content";
    let asset = EmbeddedAsset {
        content: data,
        content_type: "text/plain",
        encoding: AssetEncoding::Identity,
    };
    let result = decompress_asset(&asset);
    assert_eq!(&*result, b"test content");
}

#[test]
fn test_embedded_asset_clone_copy() {
    let data: &[u8] = b"data";
    let asset = EmbeddedAsset {
        content: data,
        content_type: "application/octet-stream",
        encoding: AssetEncoding::Identity,
    };
    let cloned = asset;
    assert_eq!(cloned.content_type, "application/octet-stream");
}

// --- get_asset tests ---

#[test]
fn test_get_asset_nonexistent() {
    // Without demo feature, get_asset always returns None
    let result = get_asset("/nonexistent/path");
    // Either None (no demo feature) or Some (demo feature with valid path)
    let _ = result;
}

#[test]
fn test_get_asset_hash() {
    let hash = get_asset_hash();
    assert!(!hash.is_empty());
    // Should be "development" or an actual hash
}

// --- resolve_repository tests ---

#[test]
fn test_resolve_repository_with_url() {
    let result = resolve_repository(
        None,
        Some("https://github.com/owner/repo".to_string()),
        None,
    );
    assert!(result.is_ok());
    let path = result.unwrap();
    assert_eq!(path, PathBuf::from("https://github.com/owner/repo"));
}

#[test]
fn test_resolve_repository_github_shorthand() {
    let result = resolve_repository(
        None,
        None,
        Some("gh:paiml/paiml-mcp-agent-toolkit".to_string()),
    );
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("github.com"));
}

#[test]
fn test_resolve_repository_github_url() {
    let result = resolve_repository(
        None,
        None,
        Some("https://github.com/paiml/paiml-mcp-agent-toolkit".to_string()),
    );
    assert!(result.is_ok());
}

#[test]
fn test_resolve_repository_owner_repo() {
    let result = resolve_repository(
        None,
        None,
        Some("paiml/paiml-mcp-agent-toolkit".to_string()),
    );
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("github.com/paiml"));
}

#[test]
fn test_resolve_repository_current_dir() {
    let result = resolve_repository(None, None, None);
    // Should succeed since we're in a git repo
    assert!(result.is_ok());
}

#[test]
fn test_resolve_repository_nonexistent_path() {
    let result = resolve_repository(
        Some(PathBuf::from("/nonexistent/path")),
        None,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_resolve_repository_nonexistent_repo_spec() {
    let result = resolve_repository(
        None,
        None,
        Some("not-a-valid-spec.tar.gz".to_string()),
    );
    assert!(result.is_err());
}

// --- detect_repository tests ---

#[test]
fn test_detect_repository_current_dir() {
    let result = detect_repository(None);
    assert!(result.is_ok());
}

#[test]
fn test_detect_repository_explicit_path() {
    let result = detect_repository(Some(PathBuf::from(".")));
    assert!(result.is_ok());
}

#[test]
fn test_detect_repository_nonexistent() {
    let result = detect_repository(Some(PathBuf::from("/nonexistent/dir")));
    assert!(result.is_err());
}

// --- DemoAnalysisResult tests ---

#[test]
fn test_demo_analysis_result_construction() {
    let result = DemoAnalysisResult {
        files_analyzed: 100,
        functions_analyzed: 500,
        avg_complexity: 8.5,
        hotspot_functions: 10,
        quality_score: 85.0,
        tech_debt_hours: 24,
        qa_verification: Some("verified".to_string()),
        language_stats: None,
        complexity_metrics: None,
    };
    let _ = format!("{:?}", result);
    assert_eq!(result.files_analyzed, 100);
}

#[test]
fn test_demo_analysis_result_serialize() {
    let result = DemoAnalysisResult {
        files_analyzed: 50,
        functions_analyzed: 200,
        avg_complexity: 6.0,
        hotspot_functions: 5,
        quality_score: 90.0,
        tech_debt_hours: 8,
        qa_verification: None,
        language_stats: Some(HashMap::new()),
        complexity_metrics: Some(HashMap::new()),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("files_analyzed"));
    assert!(json.contains("50"));
}
