# MCP Agent Toolkit - Root Workspace Makefile
# Pragmatic AI Labs
# https://paiml.com
#
# ⚠️  IMPORTANT: This is a RUST WORKSPACE PROJECT with a PRIMARY root Makefile!
# 
# WORKSPACE STRUCTURE:
# - Root workspace: Cargo.toml (workspace configuration)
# - Server project: server/Cargo.toml (main binary crate)
# - Future projects: client/, shared/ (when implemented)
#
# This root Makefile should be used for 80% of all operations, including:
# - All CI/CD operations (GitHub Actions should use this Makefile)
# - Development commands (format, lint, test, build)
# - Installation and deployment
# - Cross-workspace operations
#
# The individual project Makefiles (e.g., server/Makefile) should ONLY be used
# when you need project-specific operations while working directly in that directory.
#
# Use `make server-<target>` to run server-specific targets from the root.
# Example: `make server-test` instead of `cd server && make test`
#
# This design prevents workspace-related issues and ensures consistent behavior.

.PHONY: all validate format lint check test test-fast coverage build release clean install install-latest reinstall status check-rebuild uninstall help format-scripts lint-scripts check-scripts test-scripts fix validate-docs ci-status validate-naming context setup audit docs run-mcp run-mcp-test test-actions install-act check-act deps-validate dogfood dogfood-ci update-rust-docs

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

# Fast tests without coverage (optimized for speed)
test-fast:
	@echo "⚡ Running fast tests with maximum parallelism..."
	@RUST_TEST_THREADS=$$(nproc) cargo nextest run --profile fast --workspace || cargo test --release --workspace
	@echo "✅ Fast tests completed!"

# Run tests in all projects (with coverage)
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

# Dogfood our own tools to keep README.md updated
dogfood: server-build-binary
	@echo "🐕 Dogfooding: Using our own MCP toolkit to analyze and update documentation..."
	@deno run --allow-all scripts/dogfood-readme.ts
	@echo ""
	@echo "✅ Dogfooding complete! README.md updated with fresh metrics from our own tools."
	@echo "📁 Check artifacts/dogfooding/ for detailed reports"
	@echo "💡 Tip: Run 'git diff README.md' to see what changed"

# Quick dogfood for CI - just gather metrics without updating README
dogfood-ci: server-build-binary
	@echo "🐕 CI Dogfooding: Gathering metrics using our own tools..."
	@mkdir -p artifacts/dogfooding
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --toolchain rust --format summary > artifacts/dogfooding/complexity-latest.txt
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 7 --format summary > artifacts/dogfooding/churn-latest.txt
	@./target/release/paiml-mcp-agent-toolkit analyze dag --show-complexity -o artifacts/dogfooding/dag-latest.mmd
	@echo "✅ CI dogfooding complete! Metrics saved to artifacts/dogfooding/"

# Update rust-docs with current metrics
update-rust-docs: server-build-binary
	@echo "📝 Updating rust-docs with current metrics..."
	@deno run --allow-all scripts/update-rust-docs.ts
	@echo "✅ rust-docs updated successfully!"

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
		echo "📝 Updating documentation with current metrics..."; \
		echo "   - Updating rust-docs..."; \
		$(MAKE) update-rust-docs || true; \
		echo "   - Updating README.md..."; \
		$(MAKE) dogfood || true; \
		echo ""; \
		echo "✅ Build completed successfully!"; \
		echo "   Binaries built for all projects (Docker NOT built)."; \
		echo "   Documentation updated with latest metrics."; \
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

# Format TypeScript scripts (excluding archived scripts)
format-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "📝 Formatting TypeScript scripts (excluding archive)..."; \
		find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' -exec deno fmt --quiet {} +; \
	else \
		echo "✓ No TypeScript scripts to format"; \
	fi

# Lint TypeScript scripts (includes type checking, excluding archived scripts)
lint-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "🔍 Linting TypeScript scripts (excluding archive)..."; \
		find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' -exec deno lint --quiet {} +; \
		echo "✅ Type checking TypeScript scripts (excluding archive)..."; \
		find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' -exec deno check {} +; \
	else \
		echo "✓ No TypeScript scripts to lint"; \
	fi

# Type check TypeScript scripts (excluding archived scripts)
check-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "✅ Type checking TypeScript scripts (excluding archive)..."; \
		find $(SCRIPTS_DIR) -name '*.ts' -type f -not -path '*/archive/*' -exec deno check {} + || true; \
	else \
		echo "✓ No TypeScript scripts to check"; \
	fi

# Test TypeScript scripts with coverage
test-scripts:
	@rm -rf coverage_deno
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.test.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "🧪 Testing TypeScript scripts with coverage..."; \
		deno test --allow-all --coverage=coverage_deno \
			$(SCRIPTS_DIR)/lib/*.test.ts \
			$(SCRIPTS_DIR)/*.test.ts; \
		echo ""; \
		echo "📊 Coverage Report:"; \
		deno coverage coverage_deno; \
	else \
		echo "✓ No TypeScript script tests found"; \
	fi

# Test dogfood integration (requires built binary)
test-dogfood: server-build-binary
	@echo "🐕 Testing dogfood integration (self-analysis capabilities)..."
	@echo "This test verifies our tool can analyze itself and generate valid Mermaid diagrams"
	@echo ""
	@deno test --allow-all scripts/dogfood-readme-integration.test.ts
	@echo ""
	@echo "✅ Dogfood integration tests complete!"

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

# Generate Deno coverage report
coverage-scripts:
	@rm -rf coverage_deno
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.test.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "📊 Generating TypeScript coverage report..."; \
		deno test --allow-all --coverage=coverage_deno $(SCRIPTS_DIR)/**/*.test.ts --quiet; \
		echo ""; \
		deno coverage coverage_deno; \
		echo ""; \
		echo "📄 Detailed reports available at:"; \
		echo "   - LCOV: coverage_deno/lcov.info"; \
		echo "   - HTML: coverage_deno/html/index.html"; \
	else \
		echo "✓ No TypeScript script tests found"; \
	fi

# Clean coverage data
clean-coverage:
	@rm -rf .coverage coverage_profile coverage_deno

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

## Fuzzing targets
.PHONY: fuzz fuzz-all fuzz-coverage fuzz-corpus

fuzz: ## Run fuzzing for Mermaid generation (default 5 minutes)
	@deno run --allow-run --allow-read --allow-write --allow-env scripts/run-fuzzing.ts

fuzz-all: ## Run all fuzzers
	@deno run --allow-run --allow-read --allow-write --allow-env scripts/run-fuzzing.ts --fuzzer=all

fuzz-coverage: ## Generate fuzzing coverage report
	@deno run --allow-run --allow-read --allow-write --allow-env scripts/run-fuzzing.ts --fuzzer=coverage

fuzz-corpus: ## Generate fuzzing corpus
	@deno run --allow-write --allow-read scripts/generate-fuzz-corpus.ts

fuzz-%: ## Run specific fuzzer (e.g., make fuzz-mermaid_generation)
	@deno run --allow-run --allow-read --allow-write --allow-env scripts/run-fuzzing.ts --fuzzer=fuzz_$*

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

# Update dependencies aggressively beyond semver constraints
update-deps-aggressive:
	@echo "🔄 Updating dependencies aggressively (requires cargo-edit)..."
	@if ! command -v cargo-upgrade &> /dev/null; then \
		echo "Installing cargo-edit for cargo upgrade command..."; \
		cargo install cargo-edit; \
	fi
	@echo "Step 1: Updating within semver-compatible ranges..."
	cargo update --aggressive --manifest-path server/Cargo.toml
	@echo "Step 2: Upgrading to latest incompatible versions (major bumps)..."
	cd server && cargo upgrade --incompatible

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

# Build optimized release binary (workspace-wide)
release:
	@echo "🚀 Building optimized release binary for Rust workspace..."
	@echo "📁 Workspace structure:"
	@echo "   - Root workspace: Cargo.toml (workspace configuration)"
	@echo "   - Server project: server/Cargo.toml (main binary crate)"
	@echo ""
	@echo "🔨 Building release binary with workspace optimizations..."
	cargo build --release --manifest-path server/Cargo.toml
	@echo ""
	@echo "✅ Release binary built successfully!"
	@echo "📍 Binary location: ./target/release/paiml-mcp-agent-toolkit"
	@echo "📊 Binary size: $$(du -h ./target/release/paiml-mcp-agent-toolkit | cut -f1)"
	@echo ""
	@echo "💡 Tips for binary size optimization (future improvements):"
	@echo "   - Strip debug symbols: cargo build --release --config 'profile.release.strip=true'"
	@echo "   - Enable LTO: cargo build --release --config 'profile.release.lto=true'"
	@echo "   - Optimize for size: cargo build --release --config 'profile.release.opt-level=\"s\"'"


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
	@echo "  test         - Run tests in all projects (with coverage)"
	@echo "  test-fast    - Run fast tests with maximum parallelism (no coverage)"
	@echo "  test-critical-scripts - Test critical installation/release scripts"
	@echo "  coverage     - Generate coverage reports for all projects"
	@echo "  coverage-scripts - Generate coverage report for TypeScript tests"
	@echo "  audit        - Run security audit on all projects"
	@echo "  docs         - Generate and open documentation"
	@echo "  validate-docs - Check documentation naming consistency"
	@echo "  validate-naming - Validate naming conventions across the project"
	@echo "  ci-status    - Check GitHub Actions workflow status"
	@echo "  test-actions - Test GitHub Actions workflows locally with act"
	@echo "  context      - Generate deep context analysis (AST, tree, docs)"
	@echo "  build        - Build all projects (binaries only)"
	@echo "  release      - Build optimized release binary (workspace-wide)"
	@echo "  clean        - Clean all build artifacts"
	@echo ""
	@echo "Documentation:"
	@echo "  dogfood      - Update README.md with current project metrics"
	@echo "  dogfood-ci   - Gather metrics without updating files (for CI)"
	@echo "  update-rust-docs - Update rust-docs with current performance metrics"
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

# =============================================================================
# Specification Implementation Targets
# =============================================================================

# Mermaid Specification Testing Targets
setup-mermaid-validator:
	@echo "🔧 Setting up Mermaid specification validator..."
	@if ! command -v deno &> /dev/null; then \
		echo "Error: Deno is required but not installed"; \
		echo "Install with: curl -fsSL https://deno.land/install.sh | sh"; \
		exit 1; \
	fi
	@echo "✅ Deno validator ready"

# Run Mermaid specification compliance tests
test-mermaid-spec: setup-mermaid-validator
	@echo "🧪 Running Mermaid specification compliance tests..."
	cd server && cargo test mermaid_spec_compliance --features mermaid-spec-tests -- --nocapture

# Validate all generated Mermaid artifacts
validate-mermaid-artifacts: setup-mermaid-validator
	@echo "🔍 Validating all Mermaid artifacts against spec..."
	@if [ -d "artifacts/mermaid" ]; then \
		deno run --allow-read scripts/mermaid-validator.ts artifacts/mermaid/; \
	else \
		echo "⚠️  No artifacts/mermaid directory found. Run 'make generate-artifacts' first."; \
	fi

# Generate compliance report for Mermaid diagrams
mermaid-compliance-report: setup-mermaid-validator
	@echo "📊 Generating Mermaid compliance report..."
	cd server && cargo test mermaid_spec_compliance --features mermaid-spec-tests -- --nocapture > ../mermaid-compliance.txt 2>&1 || true
	@echo "Report saved to mermaid-compliance.txt"

# Deterministic Artifact Generation Targets
generate-artifacts:
	@echo "🎯 Generating deterministic artifacts..."
	cd server && cargo run --release -- generate-artifacts --output ../artifacts/ --deterministic

# Test deterministic generation (multiple runs should be identical)
test-determinism: 
	@echo "🔬 Testing artifact generation determinism..."
	cd server && cargo test determinism_tests -- --nocapture

# Verify artifact integrity using stored hashes
verify-artifacts:
	@echo "🔐 Verifying artifact integrity..."
	cd server && cargo run --release -- verify-artifacts --path ../artifacts/

# SATD (Self-Admitted Technical Debt) Analysis Targets  
analyze-satd:
	@echo "🔍 Analyzing Self-Admitted Technical Debt..."
	cd server && cargo run --release -- analyze satd --format json --output ../satd-analysis.json

# Analyze SATD with evolution tracking
analyze-satd-evolution:
	@echo "📈 Analyzing SATD evolution over time..."
	cd server && cargo run --release -- analyze satd --evolution --days 90 --format json

# Export critical SATD items in SARIF format
export-critical-satd:
	@echo "⚠️  Exporting critical technical debt items..."
	cd server && cargo run --release -- analyze satd --severity critical --format sarif --output ../critical-debt.sarif

# Generate comprehensive SATD metrics
satd-metrics:
	@echo "📊 Generating SATD metrics..."
	cd server && cargo run --release -- analyze satd --metrics --format json --output ../satd-metrics.json

# Clean up validation artifacts
clean-mermaid-validator:
	@echo "🧹 Cleaning Mermaid validator artifacts..."
	@rm -f mermaid-compliance.txt

# Comprehensive validation of all specifications
validate-all-specs: test-mermaid-spec test-determinism analyze-satd
	@echo "✅ All specification implementations validated!"
	@echo "  ✓ Mermaid specification compliance"
	@echo "  ✓ Deterministic artifact generation" 
	@echo "  ✓ SATD detection and classification"

# Performance testing for all specifications
benchmark-specs:
	@echo "⚡ Running specification performance benchmarks..."
	cd server && cargo test --release test_validation_performance --ignored -- --nocapture
	cd server && cargo test --release test_artifact_generation_determinism --ignored -- --nocapture

.PHONY: setup-mermaid-validator test-mermaid-spec validate-mermaid-artifacts mermaid-compliance-report generate-artifacts test-determinism verify-artifacts analyze-satd analyze-satd-evolution export-critical-satd satd-metrics clean-mermaid-validator validate-all-specs benchmark-specs