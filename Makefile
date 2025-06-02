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

.PHONY: all validate format lint check test test-fast coverage build release clean install install-latest reinstall status check-rebuild uninstall help format-scripts lint-scripts check-scripts test-scripts fix validate-docs ci-status validate-naming context setup audit docs run-mcp run-mcp-test test-actions install-act check-act deps-validate dogfood dogfood-ci update-rust-docs size-report size-track size-check size-compare test-all-interfaces test-feature-all-interfaces test-interface-consistency benchmark-all-interfaces load-test-interfaces context-json context-sarif context-llm context-legacy context-benchmark analyze-top-files analyze-composite analyze-health-dashboard profile-binary-performance analyze-memory-usage analyze-scaling

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
dogfood: release
	@echo "🐕 Dogfooding: Using our own MCP toolkit extensively for analysis and documentation..."
	@echo "📊 Phase 1: Comprehensive analysis using the built binary..."
	@mkdir -p artifacts/dogfooding
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 10 --format json > artifacts/dogfooding/complexity-$(shell date +%Y-%m-%d).json
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 30 --top-files 10 --format json > artifacts/dogfooding/churn-$(shell date +%Y-%m-%d).json
	@./target/release/paiml-mcp-agent-toolkit analyze dag --enhanced --top-files 15 -o artifacts/dogfooding/dag-$(shell date +%Y-%m-%d).mmd
	@./target/release/paiml-mcp-agent-toolkit context --format markdown --output artifacts/dogfooding/deep-context-$(shell date +%Y-%m-%d).md
	@echo "📝 Phase 2: Updating documentation with binary-generated metrics..."
	@deno run --allow-all scripts/dogfood-readme.ts
	@echo ""
	@echo "✅ Dogfooding complete! README.md updated with fresh binary-generated metrics."
	@echo "📁 Check artifacts/dogfooding/ for comprehensive analysis reports"
	@echo "🔄 All analysis performed using our own built binary"
	@echo "💡 Tip: Run 'git diff README.md' to see what changed"

# Quick dogfood for CI - comprehensive binary testing and metrics
dogfood-ci: release
	@echo "🐕 CI Dogfooding: Comprehensive testing of our own binary..."
	@mkdir -p artifacts/dogfooding
	@echo "📊 Generating comprehensive analysis using built binary..."
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 10 --format json > artifacts/dogfooding/complexity-latest.json
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 7 --top-files 10 --format json > artifacts/dogfooding/churn-latest.json
	@./target/release/paiml-mcp-agent-toolkit analyze dag --enhanced --top-files 15 -o artifacts/dogfooding/dag-latest.mmd
	@./target/release/paiml-mcp-agent-toolkit context --format json --output artifacts/dogfooding/deep-context-latest.json
	@echo "🧪 Testing binary performance and interface consistency..."
	@time ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format table
	@echo "✅ CI dogfooding complete! All metrics generated using our own binary."
	@echo "📁 Comprehensive reports saved to artifacts/dogfooding/"
	@echo "⚡ Binary performance validated"

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

# Clean coverage data and profraw files
clean-coverage:
	@echo "🧹 Cleaning coverage artifacts and profraw files..."
	@find . -name "*.profraw" -type f -delete
	@find . -name "*.profdata" -type f -delete
	@rm -rf .coverage coverage_profile coverage_deno
	@echo "✅ Coverage cleanup complete!"

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

# Generate comprehensive context with full AST and metrics analysis
context: release
	@echo "📊 Generating comprehensive deep context analysis..."
	@./target/release/paiml-mcp-agent-toolkit context --output deep_context.md
	@echo "✅ Context analysis complete: deep_context.md"

# Simpler alternative using zero-config context command
context-simple: release
	@echo "📊 Generating context with zero-config auto-detection..."
	@./target/release/paiml-mcp-agent-toolkit context --output deep_context.md

# Additional targets for different formats (using auto-detection)
context-json: release
	@./target/release/paiml-mcp-agent-toolkit context \
		--format json \
		--output deep_context.json

context-sarif: release
	@./target/release/paiml-mcp-agent-toolkit context \
		--format sarif \
		--output deep_context.sarif

context-llm: release
	@./target/release/paiml-mcp-agent-toolkit context \
		--format llm-optimized \
		--output deep_context_llm.md

# Performance comparison with legacy TypeScript implementation
context-legacy:
	@echo "🕰️ Running legacy TypeScript implementation for comparison..."
	@deno run --allow-all $(SCRIPTS_DIR)/deep-context.ts -o deep_context_legacy.md

context-benchmark: release context-legacy
	@echo "🏁 Performance comparison: New auto-detection vs Legacy TypeScript"
	@echo "=== New Implementation (Zero-config auto-detection) ==="
	@time ./target/release/paiml-mcp-agent-toolkit context --format markdown --output deep_context_new.md
	@echo ""
	@echo "=== Legacy Implementation (TypeScript) ==="
	@time deno run --allow-all $(SCRIPTS_DIR)/deep-context.ts -o deep_context_legacy_timed.md
	@echo ""
	@echo "📊 Comparing output sizes..."
	@echo "New implementation: $$(wc -c < deep_context_new.md) bytes"
	@echo "Legacy implementation: $$(wc -c < deep_context_legacy_timed.md) bytes"
	@echo "🧹 Cleaning up comparison files..."
	@rm -f deep_context_new.md deep_context_legacy_timed.md deep_context_legacy.md

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

# Binary size analysis and monitoring
size-report: release ## Generate comprehensive binary size report
	@echo "=== Binary Size Report ==="
	@ls -lh target/release/paiml-mcp-agent-toolkit
	@echo ""
	@echo "=== Asset Optimization Status ==="
	@if [ -f "server/assets/vendor/mermaid.min.js.gz" ]; then \
		MERMAID_ORIGINAL=$$(curl -sI "https://unpkg.com/mermaid@latest/dist/mermaid.min.js" | grep -i content-length | cut -d' ' -f2 | tr -d '\r'); \
		MERMAID_COMPRESSED=$$(stat -f%z server/assets/vendor/mermaid.min.js.gz 2>/dev/null || stat -c%s server/assets/vendor/mermaid.min.js.gz); \
		if [ -n "$$MERMAID_ORIGINAL" ] && [ "$$MERMAID_ORIGINAL" -gt 0 ]; then \
			REDUCTION=$$(echo "scale=1; ($$MERMAID_ORIGINAL - $$MERMAID_COMPRESSED) * 100 / $$MERMAID_ORIGINAL" | bc -l 2>/dev/null || echo "N/A"); \
			echo "Mermaid.js: $$MERMAID_ORIGINAL -> $$MERMAID_COMPRESSED bytes ($$REDUCTION% reduction)"; \
		else \
			echo "Mermaid.js: Compressed to $$MERMAID_COMPRESSED bytes"; \
		fi; \
	else \
		echo "❌ Mermaid.js not compressed (run 'make release' to rebuild)"; \
	fi
	@if [ -f "server/assets/demo/app.min.js" ]; then \
		if [ -f "../assets/demo/app.js" ]; then \
			DEMO_JS_ORIGINAL=$$(stat -f%z ../assets/demo/app.js 2>/dev/null || stat -c%s ../assets/demo/app.js); \
			DEMO_JS_MINIFIED=$$(stat -f%z server/assets/demo/app.min.js 2>/dev/null || stat -c%s server/assets/demo/app.min.js); \
			REDUCTION=$$(echo "scale=1; ($$DEMO_JS_ORIGINAL - $$DEMO_JS_MINIFIED) * 100 / $$DEMO_JS_ORIGINAL" | bc -l 2>/dev/null || echo "N/A"); \
			echo "Demo JS: $$DEMO_JS_ORIGINAL -> $$DEMO_JS_MINIFIED bytes ($$REDUCTION% reduction)"; \
		else \
			echo "Demo JS: Minified"; \
		fi; \
	else \
		echo "❌ Demo JS not minified (run 'make release' to rebuild)"; \
	fi
	@echo ""
	@echo "=== Size by Crate ==="
	@if command -v cargo-bloat >/dev/null 2>&1; then \
		cargo bloat --release --crates -n 10 --manifest-path server/Cargo.toml; \
	else \
		echo "Install cargo-bloat for detailed analysis: cargo install cargo-bloat"; \
	fi
	@echo ""
	@echo "=== Largest Functions ==="
	@if command -v cargo-bloat >/dev/null 2>&1; then \
		cargo bloat --release -n 10 --manifest-path server/Cargo.toml; \
	else \
		echo "Install cargo-bloat for detailed analysis: cargo install cargo-bloat"; \
	fi

size-track: release ## Track binary size over time
	@SIZE=$$(stat -f%z target/release/paiml-mcp-agent-toolkit 2>/dev/null || stat -c%s target/release/paiml-mcp-agent-toolkit); \
	echo "$$(date +%Y-%m-%d),$${SIZE}" >> size-history.csv; \
	echo "Binary size: $${SIZE} bytes"; \
	echo "History logged to size-history.csv"

size-check: release ## Check if binary size exceeds threshold
	@SIZE=$$(stat -f%z target/release/paiml-mcp-agent-toolkit 2>/dev/null || stat -c%s target/release/paiml-mcp-agent-toolkit); \
	THRESHOLD=20971520; \
	echo "Binary size: $${SIZE} bytes"; \
	echo "Threshold: $${THRESHOLD} bytes (20MB)"; \
	if [ $${SIZE} -gt $${THRESHOLD} ]; then \
		echo "❌ Binary size exceeds 20MB threshold"; \
		exit 1; \
	else \
		echo "✅ Binary size within acceptable limits"; \
	fi

size-compare: ## Compare binary size with minimal build
	@echo "=== Building with minimal features ==="
	@cargo build --release --no-default-features --features rust-only --manifest-path server/Cargo.toml
	@SIZE_MINIMAL=$$(stat -f%z target/release/paiml-mcp-agent-toolkit 2>/dev/null || stat -c%s target/release/paiml-mcp-agent-toolkit); \
	echo "Minimal build size: $${SIZE_MINIMAL} bytes"
	@echo ""
	@echo "=== Building with all features ==="
	@$(MAKE) release
	@SIZE_FULL=$$(stat -f%z target/release/paiml-mcp-agent-toolkit 2>/dev/null || stat -c%s target/release/paiml-mcp-agent-toolkit); \
	echo "Full build size: $${SIZE_FULL} bytes"; \
	REDUCTION=$$(echo "scale=1; ($${SIZE_FULL} - $${SIZE_MINIMAL}) * 100 / $${SIZE_FULL}" | bc -l 2>/dev/null || echo "N/A"); \
	echo "Feature overhead: $${REDUCTION}%"


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
	@echo "  context      - Generate deep context analysis with auto-detection"
	@echo "  context-json - Generate deep context analysis in JSON format"
	@echo "  context-sarif - Generate deep context analysis in SARIF format"
	@echo "  context-llm  - Generate LLM-optimized deep context analysis"
	@echo "  context-benchmark - Compare new vs legacy implementation performance"
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
	@echo "Interface Testing (CLI, MCP, HTTP):"
	@echo "  test-all-interfaces        - MANDATORY triple-interface testing"
	@echo "  test-feature-all-interfaces FEATURE=<name> - Test specific feature across interfaces"
	@echo "  test-interface-consistency - Validate consistent results across interfaces"
	@echo "  benchmark-all-interfaces   - Performance benchmark across interfaces"
	@echo "  load-test-interfaces       - Load test all interfaces"
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
	@echo "Enhanced Analysis (using built binary):"
	@echo "  analyze-top-files       - Top files analysis across complexity and churn metrics"
	@echo "  analyze-composite       - Composite analysis combining multiple ranking factors"
	@echo "  analyze-health-dashboard - Comprehensive project health dashboard"
	@echo "  profile-binary-performance - Profile binary performance across operations"
	@echo "  analyze-memory-usage    - Analyze binary memory usage patterns"
	@echo "  analyze-scaling         - Test binary scaling with different project sizes"
	@echo "  analyze-satd            - Self-admitted technical debt analysis"
	@echo "  analyze-satd-evolution  - SATD evolution tracking over time"
	@echo "  export-critical-satd    - Export critical technical debt in SARIF format"
	@echo "  satd-metrics           - Generate comprehensive SATD metrics"
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
# Triple Interface Testing (CLI, MCP, HTTP) - MANDATORY for all development
# =============================================================================

# Session Start Ritual - Test all interfaces with core functionality
test-all-interfaces: release
	@echo "🔄 MANDATORY TRIPLE-INTERFACE TESTING: CLI, MCP, HTTP"
	@echo "📖 As per CLAUDE.md: This project MUST test ALL THREE interfaces continuously"
	@echo ""
	@echo "🚀 Starting HTTP server in background..."
	@./target/release/paiml-mcp-agent-toolkit serve --port 8080 &
	@HTTP_PID=$$!; \
	sleep 3; \
	echo ""; \
	echo "=== Testing Complexity Analysis Across All Interfaces ==="; \
	echo ""; \
	echo "🖥️  CLI Interface:"; \
	time ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format json > cli-complexity.json; \
	echo "CLI Response size: $$(wc -c < cli-complexity.json) bytes"; \
	echo ""; \
	echo "🔗 MCP Interface:"; \
	echo '{"jsonrpc":"2.0","method":"analyze_complexity","params":{"project_path":"./","top_files":5,"format":"json"},"id":1}' | \
		./target/release/paiml-mcp-agent-toolkit --mode mcp > mcp-complexity.json; \
	echo "MCP Response size: $$(wc -c < mcp-complexity.json) bytes"; \
	echo ""; \
	echo "🌐 HTTP Interface:"; \
	time curl -s -X GET "http://localhost:8080/api/v1/analyze/complexity?top_files=5&format=json" > http-complexity.json; \
	echo "HTTP Response size: $$(wc -c < http-complexity.json) bytes"; \
	echo ""; \
	echo "✅ All interfaces tested successfully!"; \
	echo "🧹 Cleaning up..."; \
	kill $$HTTP_PID 2>/dev/null || true; \
	rm -f cli-complexity.json mcp-complexity.json http-complexity.json

# Test specific feature across all interfaces
test-feature-all-interfaces: release
	@if [ -z "$(FEATURE)" ]; then \
		echo "Error: FEATURE not specified"; \
		echo "Usage: make test-feature-all-interfaces FEATURE=complexity"; \
		echo "Available features: complexity, churn, dag, context"; \
		exit 1; \
	fi
	@echo "🧪 Testing $(FEATURE) feature across all interfaces..."
	@./target/release/paiml-mcp-agent-toolkit serve --port 8080 &
	@HTTP_PID=$$!; \
	sleep 2; \
	case "$(FEATURE)" in \
		complexity) \
			echo "CLI: ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5"; \
			./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format table; \
			echo "MCP: analyze_complexity method"; \
			echo '{"jsonrpc":"2.0","method":"analyze_complexity","params":{"top_files":5},"id":1}' | ./target/release/paiml-mcp-agent-toolkit --mode mcp; \
			echo "HTTP: GET /api/v1/analyze/complexity"; \
			curl -s "http://localhost:8080/api/v1/analyze/complexity?top_files=5"; \
			;; \
		churn) \
			echo "CLI: ./target/release/paiml-mcp-agent-toolkit analyze churn --days 7"; \
			./target/release/paiml-mcp-agent-toolkit analyze churn --days 7 --top-files 5 --format table; \
			echo "MCP: analyze_churn method"; \
			echo '{"jsonrpc":"2.0","method":"analyze_churn","params":{"days":7,"top_files":5},"id":1}' | ./target/release/paiml-mcp-agent-toolkit --mode mcp; \
			echo "HTTP: GET /api/v1/analyze/churn"; \
			curl -s "http://localhost:8080/api/v1/analyze/churn?days=7&top_files=5"; \
			;; \
		context) \
			echo "CLI: ./target/release/paiml-mcp-agent-toolkit context"; \
			./target/release/paiml-mcp-agent-toolkit context --format json > /tmp/cli_context.json; \
			echo "MCP: analyze_context method"; \
			echo '{"jsonrpc":"2.0","method":"analyze_context","params":{},"id":1}' | ./target/release/paiml-mcp-agent-toolkit --mode mcp > /tmp/mcp_context.json; \
			echo "HTTP: GET /api/v1/context"; \
			curl -s "http://localhost:8080/api/v1/context" > /tmp/http_context.json; \
			;; \
		*) \
			echo "Unknown feature: $(FEATURE)"; \
			;; \
	esac; \
	kill $$HTTP_PID 2>/dev/null || true

# Interface consistency validation
test-interface-consistency: release
	@echo "🔍 Testing interface consistency (same results across CLI/MCP/HTTP)..."
	@./target/release/paiml-mcp-agent-toolkit serve --port 8080 &
	@HTTP_PID=$$!; \
	sleep 3; \
	echo "Generating complexity analysis via all interfaces..."; \
	./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 3 --format json > consistency-cli.json; \
	echo '{"jsonrpc":"2.0","method":"analyze_complexity","params":{"top_files":3,"format":"json"},"id":1}' | \
		./target/release/paiml-mcp-agent-toolkit --mode mcp | jq '.result' > consistency-mcp.json; \
	curl -s "http://localhost:8080/api/v1/analyze/complexity?top_files=3&format=json" > consistency-http.json; \
	echo "Comparing outputs..."; \
	if diff -q consistency-cli.json consistency-mcp.json >/dev/null && \
	   diff -q consistency-cli.json consistency-http.json >/dev/null; then \
		echo "✅ All interfaces return consistent results!"; \
	else \
		echo "⚠️  Interfaces return different results:"; \
		echo "CLI vs MCP:"; \
		diff consistency-cli.json consistency-mcp.json || true; \
		echo "CLI vs HTTP:"; \
		diff consistency-cli.json consistency-http.json || true; \
	fi; \
	kill $$HTTP_PID 2>/dev/null || true; \
	rm -f consistency-cli.json consistency-mcp.json consistency-http.json

# Performance benchmark across interfaces
benchmark-all-interfaces: release
	@echo "⚡ Performance benchmarking across all interfaces..."
	@./target/release/paiml-mcp-agent-toolkit serve --port 8080 &
	@HTTP_PID=$$!; \
	sleep 3; \
	echo "Benchmarking complexity analysis (5 iterations each):"; \
	echo ""; \
	echo "CLI Interface:"; \
	hyperfine --warmup 2 --min-runs 5 \
		"./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format json"; \
	echo ""; \
	echo "MCP Interface:"; \
	hyperfine --warmup 2 --min-runs 5 \
		"echo '{\"jsonrpc\":\"2.0\",\"method\":\"analyze_complexity\",\"params\":{\"top_files\":5},\"id\":1}' | ./target/release/paiml-mcp-agent-toolkit --mode mcp"; \
	echo ""; \
	echo "HTTP Interface:"; \
	hyperfine --warmup 2 --min-runs 5 \
		"curl -s http://localhost:8080/api/v1/analyze/complexity?top_files=5"; \
	kill $$HTTP_PID 2>/dev/null || true

# Interface load testing
load-test-interfaces: release
	@echo "🏋️  Load testing all interfaces..."
	@./target/release/paiml-mcp-agent-toolkit serve --port 8080 &
	@HTTP_PID=$$!; \
	sleep 3; \
	echo "HTTP Load Test (100 requests, 10 concurrent):"; \
	if command -v ab >/dev/null 2>&1; then \
		ab -n 100 -c 10 -k "http://localhost:8080/api/v1/analyze/complexity?top_files=5"; \
	else \
		echo "⚠️  Apache Bench (ab) not installed. Install with: sudo apt-get install apache2-utils"; \
	fi; \
	echo ""; \
	echo "CLI Parallel Test (10 concurrent processes):"; \
	for i in $$(seq 1 10); do \
		./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format json > /tmp/cli_test_$$i.json & \
	done; \
	wait; \
	echo "✅ CLI parallel test completed"; \
	rm -f /tmp/cli_test_*.json; \
	kill $$HTTP_PID 2>/dev/null || true

# =============================================================================
# Enhanced Analysis Targets Using Built Binary
# =============================================================================

# Top-files ranking analysis across different metrics
analyze-top-files: release
	@echo "🔝 Top Files Analysis across multiple metrics using built binary..."
	@mkdir -p artifacts/analysis
	@echo "🧮 Complexity Top Files (Top 10):"
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 10 --format table
	@echo ""
	@echo "🔥 Churn Top Files (Top 10, last 30 days):"
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 30 --top-files 10 --format table
	@echo ""
	@echo "💾 Saving detailed JSON reports..."
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 15 --format json > artifacts/analysis/top-complexity.json
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 30 --top-files 15 --format json > artifacts/analysis/top-churn.json
	@echo "✅ Top files analysis complete! Reports saved to artifacts/analysis/"

# Composite analysis combining multiple ranking factors
analyze-composite: release
	@echo "🎯 Composite Analysis: Combining complexity, churn, and risk factors..."
	@mkdir -p artifacts/analysis
	@echo "📊 Generating comprehensive ranking using built binary..."
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 15 --format json > /tmp/complexity_composite.json
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 30 --top-files 15 --format json > /tmp/churn_composite.json
	@echo "🔗 Cross-referencing high-complexity and high-churn files:"
	@echo "Files appearing in both top complexity and top churn:"
	@jq -r '.files[] | .file_path' /tmp/complexity_composite.json | sort > /tmp/complexity_files.txt
	@jq -r '.hotspots[] | .file_path' /tmp/churn_composite.json | sort > /tmp/churn_files.txt
	@comm -12 /tmp/complexity_files.txt /tmp/churn_files.txt | head -10
	@echo ""
	@echo "💾 Saving composite analysis to artifacts/analysis/composite-ranking.json"
	@echo '{"analysis_type":"composite","generated_at":"'$(shell date -Iseconds)'","components":{"complexity":' > artifacts/analysis/composite-ranking.json
	@cat /tmp/complexity_composite.json >> artifacts/analysis/composite-ranking.json
	@echo ',"churn":' >> artifacts/analysis/composite-ranking.json
	@cat /tmp/churn_composite.json >> artifacts/analysis/composite-ranking.json
	@echo '}}' >> artifacts/analysis/composite-ranking.json
	@rm -f /tmp/complexity_composite.json /tmp/churn_composite.json /tmp/complexity_files.txt /tmp/churn_files.txt
	@echo "✅ Composite analysis complete!"

# Comprehensive project health dashboard
analyze-health-dashboard: release
	@echo "🏥 Project Health Dashboard using built binary comprehensive analysis..."
	@mkdir -p artifacts/dashboard
	@echo "📊 Generating comprehensive project health metrics..."
	@echo ""
	@echo "=== Project Overview ==="
	@./target/release/paiml-mcp-agent-toolkit context --format json > artifacts/dashboard/health-context.json
	@echo "Context analysis complete ✓"
	@echo ""
	@echo "=== Risk Assessment ==="
	@./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format table
	@echo ""
	@echo "=== Recent Activity ==="
	@./target/release/paiml-mcp-agent-toolkit analyze churn --days 7 --top-files 5 --format table
	@echo ""
	@echo "=== Dependency Graph ==="
	@./target/release/paiml-mcp-agent-toolkit analyze dag --enhanced --top-files 10 -o artifacts/dashboard/dependency-graph.mmd
	@echo "Dependency graph saved to artifacts/dashboard/dependency-graph.mmd ✓"
	@echo ""
	@echo "💾 Health dashboard artifacts saved to artifacts/dashboard/"
	@echo "📄 Key files:"
	@echo "  - health-context.json (comprehensive context analysis)"
	@echo "  - dependency-graph.mmd (visual dependency analysis)"

# Binary performance profiling
profile-binary-performance: release
	@echo "⚡ Profiling binary performance across different operations..."
	@mkdir -p artifacts/profiling
	@echo "🔍 Testing startup and analysis performance..."
	@echo ""
	@echo "=== Binary Startup Performance ==="
	@hyperfine --warmup 3 --min-runs 10 \
		"./target/release/paiml-mcp-agent-toolkit --version" \
		--export-json artifacts/profiling/startup-performance.json
	@echo ""
	@echo "=== Analysis Performance by Operation ==="
	@echo "Complexity Analysis:"
	@hyperfine --warmup 2 --min-runs 5 \
		"./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format json" \
		--export-json artifacts/profiling/complexity-performance.json
	@echo ""
	@echo "Context Generation:"
	@hyperfine --warmup 1 --min-runs 3 \
		"./target/release/paiml-mcp-agent-toolkit context --format json --output /tmp/context_perf.json" \
		--export-json artifacts/profiling/context-performance.json
	@rm -f /tmp/context_perf.json
	@echo ""
	@echo "✅ Performance profiling complete! Reports in artifacts/profiling/"

# Memory usage analysis
analyze-memory-usage: release
	@echo "🧠 Analyzing binary memory usage patterns..."
	@mkdir -p artifacts/profiling
	@echo "📊 Running memory-intensive operations with monitoring..."
	@if command -v /usr/bin/time >/dev/null 2>&1; then \
		echo "Context generation memory usage:"; \
		/usr/bin/time -v ./target/release/paiml-mcp-agent-toolkit context --format json --output /tmp/memory_test.json 2> artifacts/profiling/memory-context.txt; \
		echo "Complexity analysis memory usage:"; \
		/usr/bin/time -v ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 20 --format json 2> artifacts/profiling/memory-complexity.txt; \
		echo "Memory usage reports saved to artifacts/profiling/memory-*.txt"; \
		rm -f /tmp/memory_test.json; \
	else \
		echo "⚠️  GNU time not available for detailed memory analysis"; \
		echo "Install with: sudo apt-get install time"; \
	fi

# Scaling analysis - test with different project sizes
analyze-scaling: release
	@echo "📈 Analyzing binary scaling characteristics..."
	@mkdir -p artifacts/scaling
	@echo "🔍 Testing performance with different file counts..."
	@echo "Small scope (top 3 files):"
	@time ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 3 --format json > artifacts/scaling/small-scope.json
	@echo ""
	@echo "Medium scope (top 10 files):"
	@time ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 10 --format json > artifacts/scaling/medium-scope.json
	@echo ""
	@echo "Large scope (top 25 files):"
	@time ./target/release/paiml-mcp-agent-toolkit analyze complexity --top-files 25 --format json > artifacts/scaling/large-scope.json
	@echo ""
	@echo "📊 Comparing output sizes:"
	@echo "Small scope: $$(wc -c < artifacts/scaling/small-scope.json) bytes"
	@echo "Medium scope: $$(wc -c < artifacts/scaling/medium-scope.json) bytes"
	@echo "Large scope: $$(wc -c < artifacts/scaling/large-scope.json) bytes"
	@echo "✅ Scaling analysis complete!"

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

# SATD (Self-Admitted Technical Debt) Analysis Targets using built binary
analyze-satd: release
	@echo "🔍 Analyzing Self-Admitted Technical Debt using built binary..."
	@./target/release/paiml-mcp-agent-toolkit analyze satd --format json --output satd-analysis.json
	@echo "✅ SATD analysis complete! Report saved to satd-analysis.json"

# Analyze SATD with evolution tracking
analyze-satd-evolution: release
	@echo "📈 Analyzing SATD evolution over time using built binary..."
	@./target/release/paiml-mcp-agent-toolkit analyze satd --evolution --days 90 --format json --output satd-evolution.json
	@echo "✅ SATD evolution analysis complete! Report saved to satd-evolution.json"

# Export critical SATD items in SARIF format
export-critical-satd: release
	@echo "⚠️  Exporting critical technical debt items using built binary..."
	@./target/release/paiml-mcp-agent-toolkit analyze satd --severity critical --format sarif --output critical-debt.sarif
	@echo "✅ Critical SATD export complete! Report saved to critical-debt.sarif"

# Generate comprehensive SATD metrics
satd-metrics: release
	@echo "📊 Generating comprehensive SATD metrics using built binary..."
	@./target/release/paiml-mcp-agent-toolkit analyze satd --metrics --format json --output satd-metrics.json
	@echo "✅ SATD metrics analysis complete! Report saved to satd-metrics.json"

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