// Tests for demo scorer
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("internal error")
    }

    fn create_readme(repo_path: &std::path::Path, content: &str) {
        let readme_path = repo_path.join("README.md");
        fs::write(readme_path, content).expect("internal error");
    }

    fn create_examples_dir(repo_path: &std::path::Path) {
        let examples_dir = repo_path.join("examples");
        fs::create_dir_all(&examples_dir).expect("internal error");

        // Create a sample example file
        fs::write(
            examples_dir.join("basic.rs"),
            r#"
fn main() {
    let result = do_something().expect("Failed to do something");
    println!("Result: {:?}", result);
}

fn do_something() -> Result<i32, String> {
    Ok(42)
}
"#,
        )
        .expect("internal error");
    }

    fn create_cargo_toml(repo_path: &std::path::Path, content: &str) {
        fs::write(repo_path.join("Cargo.toml"), content).expect("internal error");
    }

    const PROFESSIONAL_README: &str = r#"# Project

![Build](https://img.shields.io/badge/build-passing-green)
![Tests](https://img.shields.io/badge/tests-100%25-green)
![Coverage](https://img.shields.io/badge/coverage-85%25-green)
![License](https://img.shields.io/badge/license-MIT-blue)

<img src="docs/logo.svg" alt="Logo" width="200">

## Quick Start

```bash
cargo install myproject
```

## Demo

![Demo](docs/demo.gif)

## Getting Started

1. Install the project
2. Run `myproject --help`
"#;

    const MINIMAL_README: &str = r#"# Project

A project.
"#;

    include!("demo_scorer_tests_basic.inc.rs");
}

mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    include!("demo_scorer_tests_archetype.inc.rs");
    include!("demo_scorer_tests_g1.inc.rs");
    include!("demo_scorer_tests_g2.inc.rs");
    include!("demo_scorer_tests_g3.inc.rs");
    include!("demo_scorer_tests_g4_files.inc.rs");
}
