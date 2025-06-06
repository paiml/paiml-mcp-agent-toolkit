#[cfg(test)]
mod project_meta_integration_tests {
    use crate::models::project_meta::MetaFileType;
    use crate::services::makefile_compressor::MakefileCompressor;
    use crate::services::project_meta_detector::ProjectMetaDetector;
    use crate::services::readme_compressor::ReadmeCompressor;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_full_metadata_compression_pipeline() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a realistic project structure
        create_test_project(root);

        // Test ProjectMetaDetector
        let detector = ProjectMetaDetector::new();
        let meta_files = detector.detect(root).await;

        assert_eq!(meta_files.len(), 2); // Makefile and README.md

        // Test MakefileCompressor
        let makefile = meta_files
            .iter()
            .find(|f| matches!(f.file_type, MetaFileType::Makefile))
            .unwrap();

        let makefile_compressor = MakefileCompressor::new();
        let compressed_makefile = makefile_compressor.compress(&makefile.content);

        assert_eq!(compressed_makefile.targets.len(), 6); // all, build, test, clean, install, docker-build
        assert_eq!(compressed_makefile.variables.len(), 3); // PROJECT_NAME, VERSION, CARGO
        assert_eq!(
            compressed_makefile.detected_toolchain,
            Some("rust".to_string())
        );
        // Note: Dependencies are extracted from the raw content, not parsed recipes
        // In a real makefile, we'd expect to find these dependencies

        // Test ReadmeCompressor
        let readme = meta_files
            .iter()
            .find(|f| matches!(f.file_type, MetaFileType::Readme))
            .unwrap();

        let readme_compressor = ReadmeCompressor::new();
        let compressed_readme = readme_compressor.compress(&readme.content);

        assert!(compressed_readme.project_description.is_some());
        assert!(compressed_readme
            .project_description
            .as_ref()
            .unwrap()
            .contains("high-performance code analysis tool"));
        assert!(!compressed_readme.key_features.is_empty());
        assert!(compressed_readme.sections.len() >= 2); // At least Architecture and Features
    }

    #[tokio::test]
    async fn test_metadata_integration() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a minimal project
        create_minimal_project(root);

        // Test that our metadata detection and compression works
        let detector = ProjectMetaDetector::new();
        let meta_files = detector.detect(root).await;

        // Should find both Makefile and README
        assert_eq!(meta_files.len(), 2);

        // Test compression
        let makefile = meta_files
            .iter()
            .find(|f| matches!(f.file_type, MetaFileType::Makefile))
            .unwrap();
        let readme = meta_files
            .iter()
            .find(|f| matches!(f.file_type, MetaFileType::Readme))
            .unwrap();

        let makefile_compressor = MakefileCompressor::new();
        let compressed_makefile = makefile_compressor.compress(&makefile.content);
        assert_eq!(
            compressed_makefile.detected_toolchain,
            Some("rust".to_string())
        );

        let readme_compressor = ReadmeCompressor::new();
        let compressed_readme = readme_compressor.compress(&readme.content);
        assert!(compressed_readme.project_description.is_some());
    }

    fn create_test_project(root: &Path) {
        // Create Makefile
        let makefile_content = r#"
PROJECT_NAME = pmat
VERSION = 0.1.0
CARGO = cargo

all: build test

build:
	$(CARGO) build --release

test:
	$(CARGO) test --all

clean:
	$(CARGO) clean
	rm -rf target/

install:
	cp target/release/$(PROJECT_NAME) /usr/local/bin/

docker-build:
	docker build -t $(PROJECT_NAME):$(VERSION) .

.PHONY: all build test clean install docker-build
"#;
        fs::write(root.join("Makefile"), makefile_content).unwrap();

        // Create README.md
        let readme_content = r#"# PMAT - Performance Monitoring and Analysis Tool

[![CI](https://github.com/user/pmat/workflows/CI/badge.svg)](https://github.com/user/pmat/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance code analysis tool designed for large-scale projects.

## Features

- **Fast Analysis**: Process millions of lines of code in seconds
- **Intelligent Caching**: Smart caching system reduces repeated work
- **Plugin Architecture**: Extend functionality with custom plugins
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Real-time Monitoring**: Watch for changes and update analysis

## Architecture

PMAT is built on a modular architecture consisting of:

### Core Components

1. **Analysis Engine**: The heart of the system that processes code
2. **Cache Manager**: Handles intelligent caching of results
3. **Plugin System**: Allows third-party extensions

### Data Flow

```
Source Code → Parser → AST → Analyzer → Results → Output
                 ↓                           ↑
              Cache ←------------------------┘
```

## Installation

```bash
cargo install pmat
```

## Quick Start

```bash
# Analyze current directory
pmat analyze .

# Watch for changes
pmat watch --path ./src
```

## Configuration

Create a `.pmat.toml` file in your project root:

```toml
[analysis]
exclude = ["target", "node_modules"]
max_file_size = "10MB"

[cache]
enabled = true
ttl = 3600
```

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT - see [LICENSE](LICENSE) for details.
"#;
        fs::write(root.join("README.md"), readme_content).unwrap();

        // Create a simple Rust file
        let rust_content = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        fs::write(root.join("main.rs"), rust_content).unwrap();
    }

    fn create_minimal_project(root: &Path) {
        // Minimal Makefile
        fs::write(root.join("Makefile"), "build:\n\tcargo build").unwrap();

        // Minimal README
        fs::write(
            root.join("README.md"),
            "# Test Project\n\nA simple test project.",
        )
        .unwrap();

        // Minimal source file
        fs::write(root.join("lib.rs"), "pub fn test() {}").unwrap();
    }
}
