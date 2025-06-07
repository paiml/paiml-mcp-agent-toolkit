//! Project Builder Test Utility
//!
//! This module provides a fluent API for building test projects with various
//! languages and configurations for testing the unified analysis engine.

use crate::models::error::PmatError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Builder for creating test projects with various file types and structures
pub struct ProjectBuilder {
    /// Temporary directory for the test project
    temp_dir: TempDir,
    /// Files to create with their content
    files: HashMap<PathBuf, String>,
    /// Directories to create
    directories: Vec<PathBuf>,
    /// Git repository initialization
    init_git: bool,
    /// Package.json for JS/TS projects
    package_json: Option<String>,
    /// Cargo.toml for Rust projects
    cargo_toml: Option<String>,
    /// Requirements.txt for Python projects
    requirements_txt: Option<String>,
    /// Makefile for projects with make
    makefile: Option<String>,
}

impl ProjectBuilder {
    /// Create a new project builder
    pub fn new() -> Result<Self, PmatError> {
        let temp_dir = TempDir::new().map_err(|e| PmatError::Io(e))?;
        
        Ok(Self {
            temp_dir,
            files: HashMap::new(),
            directories: Vec::new(),
            init_git: false,
            package_json: None,
            cargo_toml: None,
            requirements_txt: None,
            makefile: None,
        })
    }

    /// Add a file with content to the project
    pub fn with_file<P: AsRef<Path>>(mut self, path: P, content: &str) -> Self {
        self.files.insert(path.as_ref().to_path_buf(), content.to_string());
        self
    }

    /// Add a directory to the project
    pub fn with_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.directories.push(path.as_ref().to_path_buf());
        self
    }

    /// Initialize as a Git repository
    pub fn with_git(mut self) -> Self {
        self.init_git = true;
        self
    }

    /// Add a Rust project structure
    pub fn with_rust_project(mut self, name: &str) -> Self {
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
tokio = {{ version = "1.0", features = ["full"] }}
"#,
            name
        );
        
        self.cargo_toml = Some(cargo_toml);
        self.directories.push(PathBuf::from("src"));
        self.files.insert(
            PathBuf::from("src/main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#.to_string(),
        );
        self.files.insert(
            PathBuf::from("src/lib.rs"),
            r#"pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Config {
    pub debug: bool,
    pub name: String,
}

impl Config {
    pub fn new(name: String) -> Self {
        Self {
            debug: false,
            name,
        }
    }
}
"#.to_string(),
        );
        self
    }

    /// Add a TypeScript/JavaScript project structure
    pub fn with_typescript_project(mut self, name: &str) -> Self {
        let package_json = format!(
            r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "Test TypeScript project",
  "main": "dist/index.js",
  "scripts": {{
    "build": "tsc",
    "test": "jest"
  }},
  "dependencies": {{
    "express": "^4.18.0"
  }},
  "devDependencies": {{
    "@types/node": "^18.0.0",
    "typescript": "^4.8.0",
    "jest": "^29.0.0"
  }}
}}
"#,
            name
        );
        
        self.package_json = Some(package_json);
        self.directories.push(PathBuf::from("src"));
        self.files.insert(
            PathBuf::from("src/index.ts"),
            r#"interface User {
    id: number;
    name: string;
    email?: string;
}

class UserService {
    private users: User[] = [];

    async addUser(user: User): Promise<void> {
        this.users.push(user);
    }

    async getUser(id: number): Promise<User | undefined> {
        return this.users.find(u => u.id === id);
    }

    async getAllUsers(): Promise<User[]> {
        return [...this.users];
    }
}

export { User, UserService };
"#.to_string(),
        );
        self.files.insert(
            PathBuf::from("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#.to_string(),
        );
        self
    }

    /// Add a Python project structure
    pub fn with_python_project(mut self, name: &str) -> Self {
        self.requirements_txt = Some("fastapi==0.104.1\nuvicorn==0.24.0\npydantic==2.5.0".to_string());
        
        self.directories.push(PathBuf::from(&name));
        self.files.insert(
            PathBuf::from(format!("{}/__init__.py", name)),
            "".to_string(),
        );
        self.files.insert(
            PathBuf::from(format!("{}/main.py", name)),
            r#"from fastapi import FastAPI
from pydantic import BaseModel
from typing import Optional, List

app = FastAPI()

class User(BaseModel):
    id: int
    name: str
    email: Optional[str] = None

class UserService:
    def __init__(self):
        self.users: List[User] = []
    
    async def add_user(self, user: User) -> None:
        self.users.append(user)
    
    async def get_user(self, user_id: int) -> Optional[User]:
        for user in self.users:
            if user.id == user_id:
                return user
        return None
    
    async def get_all_users(self) -> List[User]:
        return self.users.copy()

service = UserService()

@app.post("/users/")
async def create_user(user: User):
    await service.add_user(user)
    return {"message": "User created"}

@app.get("/users/{user_id}")
async def get_user(user_id: int):
    user = await service.get_user(user_id)
    return user or {"error": "User not found"}
"#.to_string(),
        );
        self.files.insert(
            PathBuf::from(format!("{}/utils.py", name)),
            r#"import hashlib
from typing import Any, Dict

def hash_data(data: str) -> str:
    """Generate SHA-256 hash of input data."""
    return hashlib.sha256(data.encode()).hexdigest()

def validate_email(email: str) -> bool:
    """Basic email validation."""
    return "@" in email and "." in email.split("@")[1]

class ConfigManager:
    def __init__(self):
        self.config: Dict[str, Any] = {}
    
    def set(self, key: str, value: Any) -> None:
        self.config[key] = value
    
    def get(self, key: str, default: Any = None) -> Any:
        return self.config.get(key, default)
"#.to_string(),
        );
        self
    }

    /// Add C/C++ project structure
    pub fn with_c_project(mut self, name: &str) -> Self {
        self.makefile = Some(format!(
            r#"CC=gcc
CFLAGS=-Wall -Wextra -std=c99
TARGET={}
SOURCES=main.c utils.c
OBJECTS=$(SOURCES:.c=.o)

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(OBJECTS) -o $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJECTS) $(TARGET)

.PHONY: all clean
"#,
            name
        ));
        
        self.files.insert(
            PathBuf::from("main.c"),
            r#"#include <stdio.h>
#include <stdlib.h>
#include "utils.h"

typedef struct {
    int id;
    char name[100];
    char email[100];
} User;

int main() {
    printf("Hello, C World!\n");
    
    User user = {1, "John Doe", "john@example.com"};
    print_user(&user);
    
    int result = add_numbers(5, 3);
    printf("5 + 3 = %d\n", result);
    
    return 0;
}
"#.to_string(),
        );
        
        self.files.insert(
            PathBuf::from("utils.h"),
            r#"#ifndef UTILS_H
#define UTILS_H

typedef struct {
    int id;
    char name[100];
    char email[100];
} User;

int add_numbers(int a, int b);
void print_user(const User* user);
int validate_email(const char* email);

#endif
"#.to_string(),
        );
        
        self.files.insert(
            PathBuf::from("utils.c"),
            r#"#include <stdio.h>
#include <string.h>
#include "utils.h"

int add_numbers(int a, int b) {
    return a + b;
}

void print_user(const User* user) {
    if (user == NULL) {
        printf("User is NULL\n");
        return;
    }
    
    printf("User ID: %d\n", user->id);
    printf("Name: %s\n", user->name);
    printf("Email: %s\n", user->email);
}

int validate_email(const char* email) {
    if (email == NULL) {
        return 0;
    }
    
    const char* at_sign = strchr(email, '@');
    if (at_sign == NULL) {
        return 0;
    }
    
    const char* dot = strchr(at_sign, '.');
    return dot != NULL && dot > at_sign + 1;
}
"#.to_string(),
        );
        self
    }

    /// Add a complex mixed-language project
    pub fn with_mixed_project(self) -> Self {
        self.with_rust_project("mixed-project")
            .with_file("scripts/build.py", r#"#!/usr/bin/env python3
import subprocess
import sys

def build_rust():
    """Build the Rust component."""
    result = subprocess.run(["cargo", "build"], capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Rust build failed: {result.stderr}")
        return False
    return True

def run_tests():
    """Run all tests."""
    result = subprocess.run(["cargo", "test"], capture_output=True, text=True)
    return result.returncode == 0

if __name__ == "__main__":
    if build_rust():
        print("Build successful")
        if run_tests():
            print("All tests passed")
        else:
            print("Some tests failed")
            sys.exit(1)
    else:
        print("Build failed")
        sys.exit(1)
"#)
            .with_file("frontend/app.js", r#"const express = require('express');
const app = express();
const port = 3000;

app.use(express.json());

// In-memory user store
let users = [];
let nextId = 1;

app.post('/api/users', (req, res) => {
    const { name, email } = req.body;
    
    if (!name || !email) {
        return res.status(400).json({ error: 'Name and email are required' });
    }
    
    const user = {
        id: nextId++,
        name,
        email,
        createdAt: new Date().toISOString()
    };
    
    users.push(user);
    res.status(201).json(user);
});

app.get('/api/users/:id', (req, res) => {
    const id = parseInt(req.params.id);
    const user = users.find(u => u.id === id);
    
    if (!user) {
        return res.status(404).json({ error: 'User not found' });
    }
    
    res.json(user);
});

app.listen(port, () => {
    console.log(`Server running at http://localhost:${port}`);
});
"#)
            .with_directory("frontend")
            .with_directory("scripts")
    }

    /// Build the project and return the path
    pub fn build(self) -> Result<PathBuf, PmatError> {
        let project_path = self.temp_dir.path().to_path_buf();

        // Create directories
        for dir in &self.directories {
            let full_path = project_path.join(dir);
            fs::create_dir_all(&full_path).map_err(PmatError::Io)?;
        }

        // Create files
        for (file_path, content) in &self.files {
            let full_path = project_path.join(file_path);
            
            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(PmatError::Io)?;
            }
            
            fs::write(&full_path, content).map_err(PmatError::Io)?;
        }

        // Create configuration files
        if let Some(cargo_toml) = &self.cargo_toml {
            fs::write(project_path.join("Cargo.toml"), cargo_toml).map_err(PmatError::Io)?;
        }

        if let Some(package_json) = &self.package_json {
            fs::write(project_path.join("package.json"), package_json).map_err(PmatError::Io)?;
        }

        if let Some(requirements_txt) = &self.requirements_txt {
            fs::write(project_path.join("requirements.txt"), requirements_txt).map_err(PmatError::Io)?;
        }

        if let Some(makefile) = &self.makefile {
            fs::write(project_path.join("Makefile"), makefile).map_err(PmatError::Io)?;
        }

        // Initialize git if requested
        if self.init_git {
            use std::process::Command;
            let _ = Command::new("git")
                .args(&["init"])
                .current_dir(&project_path)
                .output();
            
            // Add gitignore
            let gitignore = r#"target/
node_modules/
*.pyc
__pycache__/
*.o
.DS_Store
.env
"#;
            fs::write(project_path.join(".gitignore"), gitignore).map_err(PmatError::Io)?;
        }

        // Prevent the temp directory from being dropped
        std::mem::forget(self.temp_dir);
        
        Ok(project_path)
    }

    /// Get the project path without consuming the builder
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create temporary directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_project() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_file("test.txt", "Hello, world!")
            .with_directory("src")
            .build()
            .unwrap();

        assert!(project.join("test.txt").exists());
        assert!(project.join("src").is_dir());
        
        let content = fs::read_to_string(project.join("test.txt")).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_rust_project() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_rust_project("test-project")
            .build()
            .unwrap();

        assert!(project.join("Cargo.toml").exists());
        assert!(project.join("src").is_dir());
        assert!(project.join("src/main.rs").exists());
        assert!(project.join("src/lib.rs").exists());
    }

    #[test]
    fn test_typescript_project() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_typescript_project("test-ts")
            .build()
            .unwrap();

        assert!(project.join("package.json").exists());
        assert!(project.join("tsconfig.json").exists());
        assert!(project.join("src/index.ts").exists());
    }

    #[test]
    fn test_python_project() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_python_project("test-py")
            .build()
            .unwrap();

        assert!(project.join("requirements.txt").exists());
        assert!(project.join("test-py/__init__.py").exists());
        assert!(project.join("test-py/main.py").exists());
        assert!(project.join("test-py/utils.py").exists());
    }

    #[test]
    fn test_c_project() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_c_project("test-c")
            .build()
            .unwrap();

        assert!(project.join("Makefile").exists());
        assert!(project.join("main.c").exists());
        assert!(project.join("utils.h").exists());
        assert!(project.join("utils.c").exists());
    }

    #[test]
    fn test_mixed_project() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_mixed_project()
            .build()
            .unwrap();

        // Rust components
        assert!(project.join("Cargo.toml").exists());
        assert!(project.join("src/main.rs").exists());
        
        // Python script
        assert!(project.join("scripts/build.py").exists());
        
        // JavaScript frontend
        assert!(project.join("frontend/app.js").exists());
    }

    #[test]
    fn test_git_initialization() {
        let builder = ProjectBuilder::new().unwrap();
        let project = builder
            .with_rust_project("git-test")
            .with_git()
            .build()
            .unwrap();

        assert!(project.join(".gitignore").exists());
        // Note: .git directory may not exist in test environment without git
    }
}