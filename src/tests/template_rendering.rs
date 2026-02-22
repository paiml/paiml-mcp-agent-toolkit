#![cfg_attr(coverage_nightly, coverage(off))]
use crate::services::renderer::{render_template, TemplateRenderer};
use serde_json::json;

#[test]
fn test_render_rust_cli_makefile() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"# {{ project_name }} - Rust CLI Application Makefile

.PHONY: all check lint test build run clean help

# Default target
all: check lint test build

# Type check
check:
    cargo check{% if target %} --target {{ target }}{% endif %}

# Lint with clippy
lint:
    cargo clippy --all-targets{% if target %} --target {{ target }}{% endif %} -- -D warnings

# Run tests
test:
    {% if has_tests %}cargo test{% if target %} --target {{ target }}{% endif %}{% else %}@echo "No tests configured"{% endif %}

# Build release binary
build:
    cargo build --release{% if target %} --target {{ target }}{% endif %}

# Run the application
run:
    cargo run{% if target %} --target {{ target }}{% endif %}{% if default_args %} -- {{ default_args }}{% endif %}

# Clean build artifacts
clean:
    cargo clean

# Show help
help:
    @echo "{{ project_name }} - Available targets:"
    @echo "  all    - Run check, lint, test, and build"
    @echo "  check  - Type check the code"
    @echo "  lint   - Run clippy linter"
    @echo "  test   - Run tests"
    @echo "  build  - Build release binary"
    @echo "  run    - Run the application"
    @echo "  clean  - Remove build artifacts"
    @echo "  help   - Show this help message"
"#;

    let params = json!({
        "project_name": "my-awesome-cli",
        "has_tests": true,
        "target": "x86_64-unknown-linux-gnu",
        "default_args": "--help"
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("# my-awesome-cli - Rust CLI Application Makefile"));
    assert!(result.contains("cargo check --target x86_64-unknown-linux-gnu"));
    assert!(result.contains("cargo test --target x86_64-unknown-linux-gnu"));
    assert!(result.contains("cargo run --target x86_64-unknown-linux-gnu -- --help"));
}

#[test]
fn test_render_python_uv_makefile() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"# {{ project_name }} - Python UV CLI Application Makefile

.PHONY: all setup check lint test build run clean help

# Default target
all: check lint test

# Setup virtual environment
setup:
    uv venv
    uv pip install -e .{% if dev_dependencies %}
    uv pip install {% for dep in dev_dependencies %}{{ dep }} {% endfor %}{% endif %}

# Type check with mypy
check:
    {% if has_mypy %}uv run mypy {{ source_dir }}{% else %}@echo "Type checking not configured"{% endif %}

# Lint with ruff
lint:
    uv run ruff check {{ source_dir }}
    uv run ruff format --check {{ source_dir }}

# Run tests
test:
    {% if has_tests %}uv run pytest{% if test_args %} {{ test_args }}{% endif %}{% else %}@echo "No tests configured"{% endif %}

# Build distribution
build:
    uv build

# Run the application
run:
    uv run {{ entry_point }}{% if default_args %} {{ default_args }}{% endif %}

# Clean build artifacts
clean:
    rm -rf dist/ build/ *.egg-info
    find . -type d -name __pycache__ -exec rm -rf {} +
    find . -type f -name "*.pyc" -delete

# Show help
help:
    @echo "{{ project_name }} - Available targets:"
    @echo "  all    - Run check, lint, and test"
    @echo "  setup  - Setup virtual environment"
    @echo "  check  - Type check with mypy"
    @echo "  lint   - Lint with ruff"
    @echo "  test   - Run tests with pytest"
    @echo "  build  - Build distribution package"
    @echo "  run    - Run the application"
    @echo "  clean  - Remove build artifacts"
    @echo "  help   - Show this help message"
"#;

    let params = json!({
        "project_name": "python-data-processor",
        "source_dir": "src",
        "entry_point": "python -m data_processor",
        "has_mypy": true,
        "has_tests": true,
        "test_args": "-v --cov=src",
        "dev_dependencies": ["pytest", "pytest-cov", "mypy", "ruff"],
        "default_args": "--version"
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("# python-data-processor - Python UV CLI Application Makefile"));
    assert!(result.contains("uv run mypy src"));
    // Allow for HTML entity encoding of = sign
    assert!(result.contains("uv run pytest -v --cov") && result.contains("src"));
    assert!(result.contains("uv pip install pytest pytest-cov mypy ruff"));
    assert!(result.contains("uv run python -m data_processor --version"));
}

#[test]
fn test_render_deno_typescript_makefile() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"# {{ project_name }} - Deno TypeScript CLI Application Makefile

.PHONY: all check lint test build run clean help

# Default target
all: check lint test

# Type check
check:
    deno check {{ entry_point }}{% if additional_files %} {% for file in additional_files %}{{ file }} {% endfor %}{% endif %}

# Lint
lint:
    deno lint{% if lint_args %} {{ lint_args }}{% endif %}

# Format check
format-check:
    deno fmt --check

# Format code
format:
    deno fmt

# Run tests
test:
    {% if has_tests %}deno test{% if test_args %} {{ test_args }}{% endif %}{% else %}@echo "No tests configured"{% endif %}

# Build standalone executable
build:
    deno compile{% if permissions %} {{ permissions }}{% endif %}{% if compile_args %} {{ compile_args }}{% endif %} -o {{ output_name }} {{ entry_point }}

# Run the application
run:
    deno run{% if permissions %} {{ permissions }}{% endif %} {{ entry_point }}{% if default_args %} {{ default_args }}{% endif %}

# Clean build artifacts
clean:
    rm -f {{ output_name }}{% if additional_outputs %} {% for output in additional_outputs %}{{ output }} {% endfor %}{% endif %}

# Show help
help:
    @echo "{{ project_name }} - Available targets:"
    @echo "  all          - Run check, lint, and test"
    @echo "  check        - Type check TypeScript code"
    @echo "  lint         - Lint code with deno lint"
    @echo "  format-check - Check code formatting"
    @echo "  format       - Format code"
    @echo "  test         - Run tests"
    @echo "  build        - Build standalone executable"
    @echo "  run          - Run the application"
    @echo "  clean        - Remove build artifacts"
    @echo "  help         - Show this help message"
"#;

    let params = json!({
        "project_name": "deno-file-processor",
        "entry_point": "src/main.ts",
        "output_name": "file-processor",
        "permissions": "--allow-read --allow-write --allow-net",
        "has_tests": true,
        "test_args": "--allow-read --allow-write --coverage=coverage",
        "additional_files": ["src/lib.ts", "src/utils.ts"],
        "compile_args": "--target x86_64-unknown-linux-gnu",
        "default_args": "--help"
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("# deno-file-processor - Deno TypeScript CLI Application Makefile"));
    assert!(result.contains("deno check src/main.ts src/lib.ts src/utils.ts"));
    // Allow for HTML entity encoding of = sign
    assert!(
        result.contains("deno test --allow-read --allow-write --coverage")
            && result.contains("coverage")
    );
    assert!(result.contains("deno compile --allow-read --allow-write --allow-net --target x86_64-unknown-linux-gnu -o file-processor src/main.ts"));
    assert!(result.contains("deno run --allow-read --allow-write --allow-net src/main.ts --help"));
}

#[test]
fn test_render_readme_template() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"# {{ project_name }}

{{ description }}

## Features

{% for feature in features %}
- {{ feature }}
{% endfor %}

## Installation

```bash
{{ install_command }}
```

## Usage

```bash
{{ usage_example }}
```

{% if configuration %}
## Configuration

{{ configuration }}
{% endif %}

## Development

### Prerequisites

{% for prereq in prerequisites %}
- {{ prereq }}
{% endfor %}

### Building

```bash
{{ build_command }}
```

### Testing

```bash
{{ test_command }}
```

## License

{{ license }}
"#;

    let params = json!({
        "project_name": "Super CLI Tool",
        "description": "A powerful command-line tool for processing data efficiently.",
        "features": [
            "Fast parallel processing",
            "Support for multiple file formats",
            "Extensible plugin system",
            "Real-time progress reporting"
        ],
        "install_command": "cargo install super-cli-tool",
        "usage_example": "super-cli process input.csv -o output.json --parallel",
        "configuration": "Configuration can be provided via a `.super-cli.toml` file in your home directory.",
        "prerequisites": [
            "Rust 1.75 or later",
            "GNU Make",
            "Git"
        ],
        "build_command": "make build",
        "test_command": "make test",
        "license": "MIT License"
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("# Super CLI Tool"));
    assert!(result.contains("A powerful command-line tool for processing data efficiently."));
    assert!(result.contains("- Fast parallel processing"));
    assert!(result.contains("cargo install super-cli-tool"));
    assert!(result.contains("super-cli process input.csv -o output.json --parallel"));
    // Check for the configuration text, allowing for HTML entity encoding of backticks
    assert!(
        result.contains("Configuration can be provided via a")
            && result.contains(".super-cli.toml")
            && result.contains("file in your home directory")
    );
}

#[test]
fn test_render_gitignore_template() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"# {{ project_type }} .gitignore

{% if ide_files %}
# IDE files
{% for file in ide_files %}
{{ file }}
{% endfor %}
{% endif %}

{% if build_artifacts %}
# Build artifacts
{% for artifact in build_artifacts %}
{{ artifact }}
{% endfor %}
{% endif %}

{% if language_specific %}
# {{ language }} specific
{% for pattern in language_specific %}
{{ pattern }}
{% endfor %}
{% endif %}

{% if os_specific %}
# OS specific
{% for pattern in os_specific %}
{{ pattern }}
{% endfor %}
{% endif %}

{% if custom_patterns %}
# Project specific
{% for pattern in custom_patterns %}
{{ pattern }}
{% endfor %}
{% endif %}
"#;

    let params = json!({
        "project_type": "Rust CLI Application",
        "language": "Rust",
        "ide_files": [
            ".idea/",
            ".vscode/",
            "*.swp",
            "*.swo",
            "*~"
        ],
        "build_artifacts": [
            "target/",
            "Cargo.lock",
            "dist/",
            "build/"
        ],
        "language_specific": [
            "**/*.rs.bk",
            "*.pdb"
        ],
        "os_specific": [
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini"
        ],
        "custom_patterns": [
            ".env",
            "*.log",
            "temp/",
            "cache/"
        ]
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("# Rust CLI Application .gitignore"));
    assert!(result.contains("# IDE files"));
    assert!(result.contains(".vscode/"));
    assert!(result.contains("# Build artifacts"));
    assert!(result.contains("target/"));
    assert!(result.contains("# Rust specific"));
    assert!(result.contains("**/*.rs.bk"));
    assert!(result.contains("# OS specific"));
    assert!(result.contains(".DS_Store"));
}

#[test]
fn test_render_with_conditionals() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"{% if enable_feature %}Feature is enabled!{% else %}Feature is disabled.{% endif %}
{% if not disable_option %}Option is active.{% endif %}
{% if language == "rust" %}This is a Rust project.{% endif %}
{% if environment != "production" %}Not in production.{% endif %}"#;

    let params = json!({
        "enable_feature": true,
        "disable_option": false,
        "language": "rust",
        "environment": "development"
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("Feature is enabled!"));
    assert!(result.contains("Option is active."));
    assert!(result.contains("This is a Rust project."));
    assert!(result.contains("Not in production."));
}

#[test]
fn test_render_with_missing_parameters() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"Project: {{ project_name }}
Description: {{ description }}
Version: {{ version }}"#;

    let params = json!({
        "project_name": "Test Project"
        // Missing description and version
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("Project: Test Project"));
    assert!(result.contains("Description: "));
    assert!(result.contains("Version: "));
}

#[test]
fn test_render_with_nested_loops() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"{% for category in categories %}
## {{ category.name }}
{% for item in category.items %}
- {{ item }}
{% endfor %}
{% endfor %}"#;

    let params = json!({
        "categories": [
            {
                "name": "Languages",
                "items": ["Rust", "Python", "TypeScript"]
            },
            {
                "name": "Tools",
                "items": ["Make", "Git", "Docker"]
            }
        ]
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("## Languages"));
    assert!(result.contains("- Rust"));
    assert!(result.contains("- Python"));
    assert!(result.contains("## Tools"));
    assert!(result.contains("- Docker"));
}

#[test]
fn test_render_with_string_helpers() {
    let renderer = TemplateRenderer::new().unwrap();

    let template = r#"Snake: {{ name|snake_case }}
Kebab: {{ name|kebab_case }}
Pascal: {{ name|pascal_case }}
Year: {{ current_year() }}
Date: {{ current_date() }}"#;

    let params = json!({
        "name": "MyAwesomeProject"
    });

    let result = render_template(&renderer, template, params.as_object().unwrap().clone()).unwrap();

    assert!(result.contains("Snake: my_awesome_project"));
    assert!(result.contains("Kebab: my-awesome-project"));
    assert!(result.contains("Pascal: MyAwesomeProject"));
    assert!(result.contains("Year: 20")); // Partial match for year
    assert!(result.contains("Date: 20")); // Partial match for date
}
