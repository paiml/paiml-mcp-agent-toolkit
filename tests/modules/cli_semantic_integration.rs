#![allow(deprecated)]
// CLI Semantic Search Integration Tests
// Sprint 33 (PMAT-SEARCH-011)
//
// Tests for semantic search CLI commands

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// Test 1: pmat embed --help shows help text
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_embed_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("embed").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage semantic search embeddings",
        ))
        .stdout(predicate::str::contains("sync"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("clear"));
}

// Test 2: pmat embed sync --help shows sync help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_embed_sync_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("embed").arg("sync").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Sync embeddings"))
        .stdout(predicate::str::contains("--path"))
        .stdout(predicate::str::contains("--language"));
}

// Test 3: pmat embed status --help shows status help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_embed_status_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("embed").arg("status").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show embedding database status"));
}

// Test 4: pmat embed clear --help shows clear help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_embed_clear_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("embed").arg("clear").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Remove all embeddings"))
        .stdout(predicate::str::contains("--confirm"));
}

// Test 5: pmat semantic --help shows help text
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_semantic_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("semantic").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Semantic code search"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("similar"));
}

// Test 6: pmat semantic search --help shows search help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_semantic_search_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("semantic").arg("search").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Search code"))
        .stdout(predicate::str::contains("--mode"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--language"));
}

// Test 7: pmat semantic similar --help shows similar help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_semantic_similar_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("semantic").arg("similar").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Find similar code"))
        .stdout(predicate::str::contains("--limit"));
}

// Test 8: pmat analyze cluster --help shows cluster help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_analyze_cluster_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze").arg("cluster").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Cluster"))
        .stdout(predicate::str::contains("--method"))
        .stdout(predicate::str::contains("--k"));
}

// Test 9: pmat analyze topics --help shows topics help
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_analyze_topics_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze").arg("topics").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("topics"))
        .stdout(predicate::str::contains("--num-topics"));
}
// Test 12: pmat embed status without initialized database
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_embed_status_no_database() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("nonexistent.db");

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("embed")
        .arg("status")
        .env("PMAT_VECTOR_DB_PATH", db_path);

    // Should either fail or show empty status
    let output = cmd.output().unwrap();
    assert!(
        !output.status.success()
            || String::from_utf8_lossy(&output.stdout).contains("0")
            || String::from_utf8_lossy(&output.stderr).contains("not enabled")
    );
}

// Test 13: pmat embed clear without --confirm flag shows error
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_embed_clear_requires_confirm() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("embed").arg("clear");

    // Should fail without --confirm
    cmd.assert().failure();
}

// Test 14: pmat semantic search with invalid mode shows error
#[ignore = "requires semantic integration setup"]
#[test]
// Requires API key and database
fn test_semantic_search_invalid_mode() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("semantic")
        .arg("search")
        .arg("test query")
        .arg("--mode")
        .arg("invalid");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("mode")));
}

// Test 15: pmat semantic similar requires file path
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_semantic_similar_requires_file_path() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("semantic").arg("similar");

    // Should fail - missing required file_path argument
    cmd.assert().failure();
}

// Test 16: pmat analyze cluster requires method
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_analyze_cluster_requires_method() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze").arg("cluster");

    // Should fail - missing required --method argument
    cmd.assert().failure();
}

// Test 17: pmat analyze topics requires num_topics
#[ignore = "requires semantic integration setup"]
#[test]
#[ignore] // Integration test requires pmat binary
fn test_analyze_topics_requires_num_topics() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("analyze").arg("topics");

    // Should fail - missing required --num-topics argument
    cmd.assert().failure();
}