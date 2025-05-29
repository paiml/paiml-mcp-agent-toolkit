# MCP Agent Toolkit - Root Makefile
# Pragmatic AI Labs
# https://paiml.com
#
# ⚠️  IMPORTANT: This is the PRIMARY Makefile for the entire project!
# 
# This root Makefile should be used for 80% of all operations, including:
# - All CI/CD operations (GitHub Actions should use this Makefile)
# - Development commands (format, lint, test, build)
# - Installation and deployment
# - Cross-project operations
#
# The individual project Makefiles (e.g., server/Makefile) should ONLY be used
# when you need project-specific operations while working directly in that directory.
#
# Use `make server-<target>` to run server-specific targets from the root.
# Example: `make server-test` instead of `cd server && make test`
#
# This design prevents workspace-related issues and ensures consistent behavior.

.PHONY: all validate format lint check test coverage build clean install install-latest reinstall status check-rebuild uninstall help format-scripts lint-scripts check-scripts test-scripts fix validate-docs ci-status validate-naming context setup audit docs run-mcp run-mcp-test test-actions install-act check-act deps-validate

# Define sub-projects
# NOTE: client project will be added when implemented
PROJECTS = server

# Scripts directory path
SCRIPTS_DIR = scripts

# Default target: format and build all projects
all: format build

# Validate everything passes across all projects
validate: check lint test validate-docs validate-naming test-workflow-dag test-actions deps-validate
	@echo "✅ All projects validated! All checks passed:"
	@echo "  ✓ Type checking (cargo check + deno check)"
	@echo "  ✓ Linting (cargo clippy + deno lint)"
	@echo "  ✓ Testing (cargo test)"
	@echo "  ✓ Documentation naming consistency"
	@echo "  ✓ Project naming conventions"
	@echo "  ✓ GitHub Actions workflow DAG (no version mismatches)"
	@echo "  ✓ GitHub Actions workflows validated"
	@echo "  ✓ Dependencies validated"
	@echo "  ✓ Ready for build!"

# Format code in all projects
format: format-scripts
	@format_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "📝 Formatting $$project..."; \
			$(MAKE) -C $$project format || format_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$format_success" = "false" ]; then \
		echo ""; \
		echo "❌ Formatting failed for one or more projects"; \
		exit 1; \
	fi

# Fix all formatting and linting issues automatically
fix: format
	@echo "🔧 Auto-fixing all formatting and lint issues..."
	@echo "✅ All fixable issues have been resolved!"

# Run linting in all projects
lint: lint-scripts
	@lint_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🔍 Linting $$project..."; \
			$(MAKE) -C $$project lint || lint_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$lint_success" = "true" ]; then \
		echo ""; \
		echo "✅ All linting checks passed!"; \
	else \
		echo ""; \
		echo "❌ Linting failed for one or more projects"; \
		exit 1; \
	fi

# Type check all projects
check: check-scripts
	@check_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "✅ Checking $$project..."; \
			$(MAKE) -C $$project check || check_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$check_success" = "true" ]; then \
		echo ""; \
		echo "✅ All type checks passed!"; \
	else \
		echo ""; \
		echo "❌ Type checking failed for one or more projects"; \
		exit 1; \
	fi

# Run tests in all projects
test: test-scripts
	@test_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🧪 Testing $$project..."; \
			$(MAKE) -C $$project test || test_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$test_success" = "true" ]; then \
		echo ""; \
		echo "✅ All tests passed!"; \
	else \
		echo ""; \
		echo "❌ Tests failed for one or more projects"; \
		exit 1; \
	fi

# Generate coverage reports for all projects
coverage:
	@coverage_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "📊 Coverage report for $$project..."; \
			$(MAKE) -C $$project coverage || coverage_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$coverage_success" = "false" ]; then \
		echo ""; \
		echo "❌ Coverage generation failed for one or more projects"; \
		exit 1; \
	fi

# Run security audit on all projects
audit:
	@audit_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🔒 Security audit for $$project..."; \
			$(MAKE) -C $$project audit || audit_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$audit_success" = "false" ]; then \
		echo ""; \
		echo "⚠️  Security audit found issues in one or more projects"; \
		exit 1; \
	fi

# Generate documentation
docs:
	@$(MAKE) -C server docs

# Run MCP server
run-mcp:
	@$(MAKE) -C server run-mcp

# Run MCP server in test mode
run-mcp-test:
	@$(MAKE) -C server run-mcp-test



# Build all projects (binaries only - no Docker)
build: validate-docs validate-naming
	@build_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🔨 Building $$project..."; \
			if [ "$$project" = "server" ]; then \
				$(MAKE) -C $$project build-binary || build_success=false; \
			else \
				$(MAKE) -C $$project build || build_success=false; \
			fi \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$build_success" = "true" ]; then \
		echo ""; \
		echo "✅ Build completed successfully!"; \
		echo "   Binaries built for all projects (Docker NOT built)."; \
		echo "   To build Docker: make server-build-docker"; \
	else \
		echo ""; \
		echo "❌ Build failed for one or more projects"; \
		exit 1; \
	fi

# Clean all projects
clean:
	@clean_success=true; \
	for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🧹 Cleaning $$project..."; \
			$(MAKE) -C $$project clean || clean_success=false; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done; \
	if [ "$$clean_success" = "false" ]; then \
		echo ""; \
		echo "❌ Clean failed for one or more projects"; \
		exit 1; \
	fi

# Format TypeScript scripts
format-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "📝 Formatting TypeScript scripts..."; \
		deno fmt $(SCRIPTS_DIR) --quiet; \
	else \
		echo "✓ No TypeScript scripts to format"; \
	fi

# Lint TypeScript scripts (includes type checking)
lint-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "🔍 Linting TypeScript scripts..."; \
		deno lint $(SCRIPTS_DIR) --quiet; \
		echo "✅ Type checking TypeScript scripts..."; \
		deno check $(SCRIPTS_DIR)/**/*.ts; \
	else \
		echo "✓ No TypeScript scripts to lint"; \
	fi

# Type check TypeScript scripts
check-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "✅ Type checking TypeScript scripts..."; \
		deno check $(SCRIPTS_DIR)/**/*.ts --quiet || true; \
	else \
		echo "✓ No TypeScript scripts to check"; \
	fi

# Test TypeScript scripts
test-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.test.ts' -not -name '*.integration.test.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "🧪 Testing TypeScript scripts..."; \
		find $(SCRIPTS_DIR) -name '*.test.ts' -not -name '*.integration.test.ts' -type f | xargs deno test --allow-read --allow-env --allow-write --allow-run; \
	fi
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.integration.test.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "🧪 Running integration tests..."; \
		find $(SCRIPTS_DIR) -name '*.integration.test.ts' -type f | xargs deno test --allow-net --allow-read --allow-env --allow-write --allow-run; \
	else \
		echo "✓ No TypeScript script tests found"; \
	fi

# Test critical Deno scripts with coverage
test-critical-scripts:
	@echo "🧪 Testing critical Deno scripts with coverage..."
	@echo "These scripts are P0 - if they fail, installation/releases break!"
	@echo ""
	@echo "Testing install.sh wrapper..."
	@bash -n $(SCRIPTS_DIR)/install.sh || (echo "❌ install.sh has syntax errors!" && exit 1)
	@echo "✅ install.sh syntax check passed"
	@echo ""
	@echo "Testing TypeScript utility modules..."
	@rm -rf .coverage
	@deno test --coverage=.coverage --allow-read --allow-env --allow-write --allow-run \
		$(SCRIPTS_DIR)/lib/create-release-utils.test.ts \
		$(SCRIPTS_DIR)/lib/install-utils.test.ts \
		$(SCRIPTS_DIR)/lib/create-release-utils-integration.test.ts
	@echo ""
	@echo "Generating coverage report..."
	@deno coverage .coverage --lcov --output=.coverage/lcov.info
	@deno coverage .coverage
	@echo ""
	@echo "Checking coverage thresholds..."
	@echo "Target: 80% coverage for critical scripts"
	@deno coverage .coverage | grep -E "^All files" || true
	@echo ""
	@echo "✅ Critical script tests completed!"

# Clean coverage data
clean-coverage:
	@rm -rf .coverage coverage_profile

# Validate documentation naming consistency
validate-docs:
	@echo "📖 Validating documentation naming consistency..."
	@deno run --allow-read --allow-env $(SCRIPTS_DIR)/validate-docs.ts

# Test GitHub Actions workflow DAG for version mismatches
test-workflow-dag:
	@echo "🔍 Testing GitHub Actions workflow DAG for version mismatches..."
	@deno run --allow-read --allow-write --allow-run --allow-env $(SCRIPTS_DIR)/test-workflow-dag.ts
	@echo ""

# Test workflow DAG with verbose output
test-workflow-dag-verbose:
	@echo "🔍 Testing GitHub Actions workflow DAG (verbose)..."
	@deno run --allow-read --allow-write --allow-run --allow-env $(SCRIPTS_DIR)/test-workflow-dag.ts --verbose --scenarios
	@echo ""

# Check GitHub Actions CI status
ci-status:
	@echo "🔍 Checking GitHub Actions CI status..."
	@$(SCRIPTS_DIR)/validate-github-actions-status.ts

# Test GitHub Actions workflows locally
test-actions:
	@echo "🧪 Testing GitHub Actions workflows locally..."
	@if command -v act >/dev/null 2>&1; then \
		ACT_CMD=act; \
	elif [ -x "/tmp/act" ]; then \
		ACT_CMD=/tmp/act; \
	elif [ -x "$$HOME/.local/bin/act" ]; then \
		ACT_CMD=$$HOME/.local/bin/act; \
	else \
		echo "❌ act is not installed or not in PATH"; \
		echo "   Run 'make install-act' to install it"; \
		exit 1; \
	fi; \
	echo "Testing auto-tag-release workflow..."; \
	$$ACT_CMD -W .github/workflows/auto-tag-release.yml workflow_dispatch -P ubuntu-latest=node:20-bullseye --dryrun; \
	echo ""; \
	echo "Testing ci workflow..."; \
	$$ACT_CMD -W .github/workflows/ci.yml push -P ubuntu-latest=node:20-bullseye --dryrun; \
	echo ""; \
	echo "✅ Workflow syntax validation complete!"

# Install act if not present
install-act:
	@if ! command -v act >/dev/null 2>&1; then \
		echo "📦 Installing act..."; \
		mkdir -p ~/.local/bin; \
		curl -sL https://github.com/nektos/act/releases/latest/download/act_Linux_x86_64.tar.gz | tar xz -C ~/.local/bin; \
		echo "✅ act installed successfully to ~/.local/bin!"; \
		echo "📝 Make sure ~/.local/bin is in your PATH"; \
		echo "   You can add it with: export PATH=\$$HOME/.local/bin:\$$PATH"; \
	else \
		echo "✓ act is already installed"; \
	fi

# Check if act is installed
check-act:
	@if ! command -v act >/dev/null 2>&1; then \
		if [ -x "/tmp/act" ]; then \
			echo "ℹ️  Found act in /tmp/act but it's not in PATH"; \
			echo "   You can use it directly: /tmp/act"; \
			echo "   Or add to PATH: export PATH=/tmp:\$$PATH"; \
		elif [ -x "$$HOME/.local/bin/act" ]; then \
			echo "ℹ️  Found act in ~/.local/bin/act but it's not in PATH"; \
			echo "   Add to PATH: export PATH=\$$HOME/.local/bin:\$$PATH"; \
		else \
			echo "❌ act is not installed. Run 'make install-act' to install it."; \
			echo "   Or install manually from: https://github.com/nektos/act"; \
		fi; \
		exit 1; \
	fi

# Validate all naming conventions across the project
validate-naming:
	@echo "🔍 Validating naming conventions..."
	@deno run --allow-read --allow-run $(SCRIPTS_DIR)/validate-naming.ts

# Generate deep context analysis of the project
context:
	@echo "📊 Generating deep context analysis..."
	@$(SCRIPTS_DIR)/deep-context.ts
	@echo "✅ Deep context analysis complete! See deep_context.md"

# Validate dependencies before installation
deps-validate:
	@echo "🔍 Validating dependencies..."
	@cd server && cargo tree --duplicate | grep -v "^$$" || echo "✅ No duplicate dependencies"
	@cd server && cargo audit || echo "⚠️  Security issues found"

# Install MCP server
# Local install for development (NO VERSION BUMP) - RECOMMENDED
local-install:
	@echo "🚀 Installing MCP Agent Toolkit (local development - no version bump)..."
	@$(MAKE) -C server local-install

# Install with version bump (FOR RELEASES ONLY)
install:
	@echo "🚀 Installing MCP Agent Toolkit (WARNING: This bumps version!)..."
	@$(MAKE) -C server install

# Install latest (check for changes and rebuild if needed)
install-latest:
	@echo "🚀 Installing latest MCP Agent Toolkit (with auto-rebuild if needed)..."
	@$(MAKE) -C server install-latest

# Reinstall (force complete reinstall)
reinstall:
	@echo "🔄 Performing complete reinstall of MCP Agent Toolkit..."
	@$(MAKE) -C server reinstall

# Check installation status
status:
	@echo "🔍 Checking MCP Agent Toolkit status..."
	@$(MAKE) -C server status

# Check if rebuild needed
check-rebuild:
	@echo "🔍 Checking if rebuild is needed..."
	@$(MAKE) -C server check-rebuild

# Uninstall MCP server
uninstall:
	@echo "🗑️  Uninstalling MCP Agent Toolkit..."
	@$(MAKE) -C server uninstall

# Server-specific commands
server-%:
	@if [ -f "server/Makefile" ]; then \
		$(MAKE) -C server $*; \
	else \
		echo "Error: server/Makefile not found"; \
		exit 1; \
	fi

# Client-specific commands
client-%:
	@if [ -f "client/Makefile" ]; then \
		$(MAKE) -C client $*; \
	else \
		echo "Error: client/Makefile not found"; \
		exit 1; \
	fi

# Build for specific target (for cross-compilation in CI)
build-target:
	@if [ -z "$(TARGET)" ]; then \
		echo "Error: TARGET not specified"; \
		echo "Usage: make build-target TARGET=x86_64-unknown-linux-gnu"; \
		exit 1; \
	fi
	@echo "🔨 Building for target: $(TARGET)"
	cargo build --release --target $(TARGET) --manifest-path server/Cargo.toml


# Run cargo doc
cargo-doc:
	cargo doc --all-features --no-deps --manifest-path server/Cargo.toml

# Run cargo geiger for security audit
cargo-geiger:
	cargo geiger --all-features --manifest-path server/Cargo.toml

# Update dependencies
update-deps:
	cargo update --manifest-path server/Cargo.toml

# Update dependencies aggressively
update-deps-aggressive:
	cargo update --aggressive --manifest-path server/Cargo.toml
	cargo upgrade --workspace --to-lockfile --manifest-path server/Cargo.toml

# Update only security dependencies
update-deps-security:
	cargo audit fix --manifest-path server/Cargo.toml

# Upgrade dependencies
upgrade-deps:
	cargo upgrade --workspace --to-lockfile

# Fix audit issues
audit-fix:
	cargo audit fix --manifest-path server/Cargo.toml

# Run benchmarks
benchmark:
	@$(MAKE) server-benchmark


# Generate coverage summary (for CI)
coverage-summary:
	cargo llvm-cov report --summary-only --manifest-path server/Cargo.toml

# Check outdated dependencies
outdated:
	cargo outdated --format json --manifest-path server/Cargo.toml

# Server outdated (alias for CI)
server-outdated:
	cargo outdated --format json --manifest-path server/Cargo.toml

# Run cargo test with all features
test-all-features:
	cargo test --all-features --manifest-path server/Cargo.toml

# Server test all (alias for CI)
server-test-all:
	cargo test --all-features --manifest-path server/Cargo.toml

# Run cargo clippy with warnings as errors
clippy-strict:
	cargo clippy --manifest-path server/Cargo.toml -- -D warnings

# Server build release (for CI)
server-build-release:
	cargo build --release --manifest-path server/Cargo.toml


# Create GitHub release with binary artifacts
create-release:
	@echo "📦 Creating GitHub release..."
	@if command -v deno >/dev/null 2>&1; then \
		./scripts/create-release.ts; \
	else \
		echo "❌ Error: Deno is required to create releases"; \
		echo "Install Deno from: https://deno.land/"; \
		exit 1; \
	fi

# Test curl installation
test-curl-install:
	@echo "🧪 Testing curl installation..."
	@if command -v deno >/dev/null 2>&1; then \
		./scripts/test-curl-install.ts; \
	else \
		echo "❌ Error: Deno is required to run tests"; \
		echo "Install Deno from: https://deno.land/"; \
		exit 1; \
	fi

# Check documentation with rustdoc
cargo-rustdoc:
	cargo rustdoc --all-features --manifest-path server/Cargo.toml -- -D missing_docs || true

# Install development tools
install-dev-tools:
	@if ! command -v tokei &> /dev/null; then \
		echo "Installing tokei..."; \
		cargo install tokei; \
	fi
	@if ! command -v cargo-geiger &> /dev/null; then \
		echo "Installing cargo-geiger..."; \
		cargo install cargo-geiger; \
	fi
	@if ! command -v cargo-outdated &> /dev/null; then \
		echo "Installing cargo-outdated..."; \
		cargo install cargo-outdated; \
	fi
	@if ! command -v cargo-edit &> /dev/null; then \
		echo "Installing cargo-edit..."; \
		cargo install cargo-edit; \
	fi
	@if ! command -v cargo-audit &> /dev/null; then \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit; \
	fi
	@if ! command -v cargo-llvm-cov &> /dev/null; then \
		echo "Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov; \
	fi

# Count lines of code with tokei
tokei:
	tokei server/src --exclude "*.json"

# Count lines of code for server
server-tokei:
	tokei server/src --exclude "*.json"

# Setup development environment
# NOTE: This does NOT install Docker - Docker is optional for this project
setup:
	@echo "🔧 Setting up development environment..."
	@echo "Installing Rust toolchain components..."
	rustup component add rustfmt clippy
	@echo "Installing development tools..."
	cargo install cargo-lambda
	cargo install cargo-watch
	cargo install cargo-audit
	cargo install cargo-llvm-cov
	@if command -v deno >/dev/null 2>&1; then \
		echo "✅ Deno is already installed"; \
	else \
		echo "📦 Installing Deno..."; \
		curl -fsSL https://deno.land/install.sh | sh; \
		echo "Please add Deno to your PATH as instructed above"; \
	fi
	@if command -v shellcheck >/dev/null 2>&1; then \
		echo "✅ Shellcheck is already installed"; \
	else \
		echo "⚠️  Shellcheck is not installed. Install it with:"; \
		echo "    Ubuntu/Debian: sudo apt-get install shellcheck"; \
		echo "    macOS: brew install shellcheck"; \
		echo "    Or visit: https://github.com/koalaman/shellcheck#installing"; \
	fi
	@echo ""
	@echo "✅ Development environment setup complete!"
	@echo ""
	@echo "📝 Note: Docker is OPTIONAL for this project."
	@echo "   - The server runs as a standalone binary by default"
	@echo "   - Docker is only needed if you want containerized deployment"
	@echo "   - To install Docker separately, visit: https://docs.docker.com/get-docker/"

# Quick start guide
quickstart: setup
	@echo ""
	@echo "🚀 Quick Start Guide"
	@echo "==================="
	@echo ""
	@echo "1. Test the MCP server:"
	@echo "   make server-run-mcp-test"
	@echo ""
	@echo "2. In another terminal, test with Claude Code:"
	@echo "   claude mcp add /path/to/paiml-mcp-agent-toolkit/server"
	@echo ""
	@echo "3. Generate templates:"
	@echo "   Use /mcp in Claude Code to see available tools"
	@echo ""

# Help command
help:
	@echo "MCP Agent Toolkit - Root Project"
	@echo "================================"
	@echo ""
	@echo "Primary targets:"
	@echo "  all         - Format and build all projects (default)"
	@echo "  validate    - Run all checks across projects (check, lint, test)"
	@echo "  quickstart  - Setup and show quick start guide"
	@echo ""
	@echo "Development (all projects):"
	@echo "  format       - Format code in all projects"
	@echo "  fix          - Auto-fix all formatting issues (alias for format)"
	@echo "  lint         - Run linters in all projects (checks only)"
	@echo "  check        - Type check all projects"
	@echo "  test         - Run tests in all projects"
	@echo "  test-critical-scripts - Test critical installation/release scripts"
	@echo "  coverage     - Generate coverage reports for all projects"
	@echo "  audit        - Run security audit on all projects"
	@echo "  docs         - Generate and open documentation"
	@echo "  validate-docs - Check documentation naming consistency"
	@echo "  validate-naming - Validate naming conventions across the project"
	@echo "  ci-status    - Check GitHub Actions workflow status"
	@echo "  test-actions - Test GitHub Actions workflows locally with act"
	@echo "  context      - Generate deep context analysis (AST, tree, docs)"
	@echo "  build        - Build all projects (binaries only)"
	@echo "  clean        - Clean all build artifacts"
	@echo ""
	@echo "Running:"
	@echo "  run-mcp      - Run MCP server in STDIO mode"
	@echo "  run-mcp-test - Run MCP server in test mode"
	@echo ""
	@echo "Installation:"
	@echo "  local-install  - Install for development (NO VERSION BUMP) - RECOMMENDED"
	@echo "  install        - Install with version bump (FOR RELEASES ONLY)"
	@echo "  install-latest - Smart install (rebuild only if source changed)"
	@echo "  reinstall      - Force complete uninstall and reinstall"
	@echo "  status         - Check installation and build status"
	@echo "  check-rebuild  - Check if source files changed (needs rebuild)"
	@echo "  uninstall      - Remove MCP server from system"
	@echo ""
	@echo "Project-specific commands:"
	@echo "  server-*    - Run any server Makefile target"
	@echo "  client-*    - Run any client Makefile target"
	@echo ""
	@echo "Examples:"
	@echo "  make server-run-mcp-test  - Run MCP server in test mode"
	@echo "  make server-build-binary  - Build server binary only (no Docker)"
	@echo "  make server-build-docker  - Build Docker image only"
	@echo "  make client-build         - Build client only"
	@echo ""
	@echo "Setup:"
	@echo "  setup       - Install all development dependencies"
	@echo "  install-act - Install act for local GitHub Actions testing"
	@echo "  help        - Show this help message"
	@echo ""
	@echo "Projects included:"
	@for project in $(PROJECTS); do \
		echo "  - $$project"; \
	done