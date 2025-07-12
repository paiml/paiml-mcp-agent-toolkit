//! Integration tests for analyze deep-context CLI command (Issue #33)
//! 
//! These tests verify that the analyze deep-context command properly finds
//! and reports context issues, addressing GitHub issue #33.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Create a test project with multiple related files
fn create_test_project(dir: &TempDir) {
    // Create main module
    let main_path = dir.path().join("main.rs");
    fs::write(&main_path, r#"
mod user;
mod database;
mod api;

use user::User;
use database::Database;
use api::ApiServer;

fn main() {
    let db = Database::new();
    let user = User::new("Alice", "alice@example.com");
    let server = ApiServer::new(db);
    server.register_user(user);
}
"#).unwrap();

    // Create user module
    let user_path = dir.path().join("user.rs");
    fs::write(&user_path, r#"
pub struct User {
    pub name: String,
    pub email: String,
    id: Option<u64>,
}

impl User {
    pub fn new(name: &str, email: &str) -> Self {
        User {
            name: name.to_string(),
            email: email.to_string(),
            id: None,
        }
    }
    
    pub fn set_id(&mut self, id: u64) {
        self.id = Some(id);
    }
    
    pub fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}
"#).unwrap();

    // Create database module
    let db_path = dir.path().join("database.rs");
    fs::write(&db_path, r#"
use crate::user::User;

pub struct Database {
    users: Vec<User>,
    next_id: u64,
}

impl Database {
    pub fn new() -> Self {
        Database {
            users: Vec::new(),
            next_id: 1,
        }
    }
    
    pub fn add_user(&mut self, mut user: User) -> u64 {
        user.set_id(self.next_id);
        self.next_id += 1;
        let id = self.next_id - 1;
        self.users.push(user);
        id
    }
    
    pub fn find_user_by_email(&self, email: &str) -> Option<&User> {
        self.users.iter().find(|u| u.email == email)
    }
}
"#).unwrap();

    // Create API module
    let api_path = dir.path().join("api.rs");
    fs::write(&api_path, r#"
use crate::user::User;
use crate::database::Database;

pub struct ApiServer {
    db: Database,
}

impl ApiServer {
    pub fn new(db: Database) -> Self {
        ApiServer { db }
    }
    
    pub fn register_user(&mut self, user: User) -> Result<u64, String> {
        if !user.validate_email() {
            return Err("Invalid email".to_string());
        }
        
        if self.db.find_user_by_email(&user.email).is_some() {
            return Err("User already exists".to_string());
        }
        
        Ok(self.db.add_user(user))
    }
}
"#).unwrap();
}

#[test]
fn test_analyze_deep_context_finds_relationships() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir);
    
    // Run analyze deep-context
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "deep-context", "--format", "json"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("user"))
        .stdout(predicate::str::contains("database"))
        .stdout(predicate::str::contains("api"));
}

#[test]
fn test_analyze_deep_context_with_specific_file() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir);
    
    // Run analyze deep-context on specific file
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "deep-context", "--file", "user.rs", "--format", "json"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("User"))
        .stdout(predicate::str::contains("email"));
}

#[test]
fn test_analyze_deep_context_human_format() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir);
    
    // Run analyze deep-context with human format
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "deep-context"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Deep Context Analysis"))
        .stdout(predicate::str::contains("Files analyzed"));
}

#[test]
fn test_analyze_deep_context_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    
    // Run analyze deep-context on empty directory
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "deep-context"]);
    
    // Should succeed but indicate no files found
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0").or(predicate::str::contains("No files")));
}

#[test]
fn test_analyze_deep_context_with_output_file() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir);
    
    let output_path = temp_dir.path().join("context.json");
    
    // Run analyze deep-context with output file
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.current_dir(&temp_dir)
        .args(["analyze", "deep-context", "--format", "json", "-o", output_path.to_str().unwrap()]);
    
    cmd.assert().success();
    
    // Verify output file was created
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("user"));
    assert!(content.contains("database"));
}