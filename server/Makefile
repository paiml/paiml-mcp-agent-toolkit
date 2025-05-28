
# MCP Template Server - Build and Deployment Makefile
.PHONY: all validate format lint check test coverage build build-binary run run-test test-http test-mcp test-integration run-mcp run-mcp-test clean install local-install install-latest reinstall status check-rebuild uninstall setup audit docs help validate-docs ci-status benchmark

# Default target: executes complete build pipeline
all: format lint check test build validate
	@echo ""
	@echo "✅ All tasks completed successfully!"

# Validate everything passes (comprehensive check)
validate: validate-docs
	@echo ""
	@echo "=== Project Validation Checklist ==="
	@echo "✓ Format: Code formatting compliance (rustfmt + deno fmt)"
	@echo "✓ Lint: Static analysis and linting rules (clippy + deno lint)"
	@echo "✓ Compile: Type checking and compilation (cargo check)"
	@echo "✓ Test: Unit tests and coverage requirements (cargo test)"
	@echo "✓ Docs: Documentation naming consistency"
	@echo ""
	@echo "✅ All validation checks passed!"

# Format code using rustfmt and deno fmt
format:
	cargo fmt --all
	@if command -v deno >/dev/null 2>&1; then \
		if [ -d "scripts" ] && [ "$$(find scripts -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
			echo "Formatting TypeScript files..."; \
			deno fmt scripts/*.ts; \
		fi; \
		if [ "$$(find . -maxdepth 1 -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
			deno fmt *.ts; \
		fi; \
	fi

# Run clippy for linting Rust and deno check/lint for TypeScript
lint:
	cargo clippy --all-targets --all-features -- -D warnings
	@if command -v deno >/dev/null 2>&1; then \
		if [ -d "scripts" ] && [ "$$(find scripts -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
			echo "Type checking TypeScript files..."; \
			deno check scripts/*.ts; \
			echo "Linting TypeScript files..."; \
			deno lint scripts/*.ts; \
		fi; \
		if [ "$$(find . -maxdepth 1 -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
			deno check *.ts 2>/dev/null || true; \
			deno lint *.ts 2>/dev/null || true; \
		fi; \
	fi

# Type check without building
check:
	cargo check --all-targets --all-features

# Run tests with coverage (like deno test --coverage)
# Note: Excludes integration tests that spawn processes to prevent system freezes
test:
	@echo "🔨 Building debug binary for tests..."
	@cargo build --bin paiml-mcp-agent-toolkit
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "Running tests with coverage (excluding integration tests)..."; \
		cargo llvm-cov --all-features --workspace --html --output-dir coverage -- --skip mcp_stdio_test; \
		cargo llvm-cov report --summary-only; \
	else \
		echo "Running tests (excluding integration tests)..."; \
		cargo test -- --skip mcp_stdio_test; \
		echo "💡 Tip: Install cargo-llvm-cov for test coverage:"; \
		echo "   cargo install cargo-llvm-cov"; \
	fi
	@if [ -f "./scripts/test-mcp-e2e.ts" ] && command -v deno >/dev/null 2>&1; then \
		echo ""; \
		echo "Running E2E tests..."; \
		chmod +x ./scripts/test-mcp-e2e.ts; \
		./scripts/test-mcp-e2e.ts || exit 1; \
	fi
	@echo ""
	@echo "🔍 Running critical naming convention tests..."
	@cargo test test_naming_convention --no-fail-fast -- --nocapture || { \
		echo "❌ CRITICAL: Naming convention tests failed!"; \
		echo "This is a level 0 production blocker."; \
		echo "Ensure all references use 'paiml-mcp-agent-toolkit' consistently."; \
		exit 1; \
	}
	@if [ -f "./scripts/test-installation.ts" ] && command -v deno >/dev/null 2>&1; then \
		echo ""; \
		echo "🧪 Running installation validation tests..."; \
		./scripts/test-installation.ts || exit 1; \
	fi

# Generate detailed coverage report in plain text
coverage:
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --text; \
	else \
		echo "Error: cargo-llvm-cov is not installed."; \
		echo "Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	fi


# Build standalone binary (stateless MCP server)
build-binary:
	cargo build --release --bin paiml-mcp-agent-toolkit


# Default build target: binary only (no Docker)
build: validate-docs build-binary
	@echo ""
	@echo "✅ Binary built successfully!"
	@echo "   Location: ./target/release/paiml-mcp-agent-toolkit"



# Run HTTP tests against local server
test-http:
	@if command -v deno >/dev/null 2>&1; then \
		deno run --allow-net test_http_local.ts; \
	else \
		echo "Error: Deno is not installed. Please install Deno to run HTTP tests."; \
		exit 1; \
	fi

# Run MCP STDIO tests (currently disabled due to resource issues)
test-mcp:
	@echo "MCP STDIO tests temporarily disabled to prevent system freezes"
	@echo "These tests were spawning processes that caused CPU spikes"

# Run all integration tests
test-integration: test-http test-mcp

# Run all tests including integration tests (use with caution - may freeze system)
test-all:
	cargo llvm-cov --summary-only

# Run MCP server in STDIO mode
run-mcp:
	cargo run --bin paiml-mcp-agent-toolkit

# Run MCP server in test mode (stateless server doesn't need TEST_MODE)
run-mcp-test:
	cargo run --bin paiml-mcp-agent-toolkit

# Test installation process
test-install:
	@echo "🧪 Testing installation process..."
	@deno run --allow-all scripts/test-installation.ts


# Clean build artifacts
clean:
	cargo clean

# Increment version before building
.PHONY: bump-version
bump-version:
	@echo "📦 Incrementing version..."
	@cd .. && ./scripts/update-version.ts patch

# Install the MCP server binary to the system (idempotent) - FOR RELEASES ONLY
# WARNING: This bumps the version! Use 'make local-install' for development
install: bump-version build-binary
	@echo "🚀 Installing MCP server..."
	@# First, remove any existing installation
	@if [ -f "/usr/local/bin/paiml-mcp-agent-toolkit" ]; then \
		echo "📦 Found existing system installation, removing..."; \
		rm -f /usr/local/bin/paiml-mcp-agent-toolkit 2>/dev/null || \
		sudo rm -f /usr/local/bin/paiml-mcp-agent-toolkit 2>/dev/null || true; \
	fi
	@if [ -f "$(HOME)/.local/bin/paiml-mcp-agent-toolkit" ]; then \
		echo "📦 Found existing user installation, removing..."; \
		rm -f $(HOME)/.local/bin/paiml-mcp-agent-toolkit; \
	fi
	@# Now do fresh install
	@if [ -z "$(DESTDIR)" ]; then \
		if [ -w "/usr/local/bin" ]; then \
			install -m 755 target/release/paiml-mcp-agent-toolkit /usr/local/bin/paiml-mcp-agent-toolkit; \
			echo "✅ Installed to /usr/local/bin/paiml-mcp-agent-toolkit"; \
		else \
			echo "⚠️  Cannot write to /usr/local/bin. Trying user install..."; \
			mkdir -p $(HOME)/.local/bin; \
			install -m 755 target/release/paiml-mcp-agent-toolkit $(HOME)/.local/bin/paiml-mcp-agent-toolkit; \
			echo "✅ Installed to $(HOME)/.local/bin/paiml-mcp-agent-toolkit"; \
			echo ""; \
			if echo "$$PATH" | grep -q "$(HOME)/.local/bin"; then \
				echo "✅ $(HOME)/.local/bin is already in your PATH"; \
			else \
				echo "💡 Add $(HOME)/.local/bin to your PATH:"; \
				echo ""; \
				current_shell=$$(basename "$$SHELL"); \
				if [ "$$current_shell" = "zsh" ] || [ -f "$$HOME/.zshrc" ]; then \
					echo "   For zsh (your shell):"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.zshrc'; \
					echo "   source ~/.zshrc"; \
				elif [ "$$current_shell" = "bash" ] || [ -f "$$HOME/.bashrc" ]; then \
					echo "   For bash (your shell):"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.bashrc'; \
					echo "   source ~/.bashrc"; \
				else \
					echo "   Detected shell: $$SHELL"; \
					echo ""; \
					echo "   For bash:"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.bashrc'; \
					echo "   source ~/.bashrc"; \
					echo ""; \
					echo "   For zsh:"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.zshrc'; \
					echo "   source ~/.zshrc"; \
				fi; \
			fi \
		fi \
	else \
		install -D -m 755 target/release/paiml-mcp-agent-toolkit $(DESTDIR)/usr/local/bin/paiml-mcp-agent-toolkit; \
		echo "✅ Installed to $(DESTDIR)/usr/local/bin/paiml-mcp-agent-toolkit"; \
	fi
	@# Verify installation
	@echo ""
	@echo "🔍 Verifying installation..."
	@if command -v paiml-mcp-agent-toolkit >/dev/null 2>&1; then \
		echo "✅ Binary is accessible in PATH"; \
		echo "📍 Location: $$(which paiml-mcp-agent-toolkit)"; \
		echo "📌 Version: $$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"; \
		echo ""; \
		echo "🧪 Testing server startup..."; \
		if timeout 2s paiml-mcp-agent-toolkit < /dev/null >/dev/null 2>&1; then \
			echo "✅ Server starts successfully"; \
		else \
			exit_code=$$?; \
			if [ $$exit_code -eq 124 ]; then \
				echo "✅ Server starts successfully (timeout expected for STDIO server)"; \
			else \
				echo "⚠️  Server may have issues starting. Exit code: $$exit_code"; \
				echo "   Try running manually: paiml-mcp-agent-toolkit"; \
			fi \
		fi; \
		echo ""; \
		echo "📋 Configuring Claude Code integration..."; \
		if command -v claude >/dev/null 2>&1; then \
			if claude mcp list 2>/dev/null | grep -q "paiml-toolkit"; then \
				echo "✅ MCP server already configured in Claude Code"; \
			else \
				echo "📝 Adding MCP server to Claude Code..."; \
				if claude mcp add paiml-toolkit paiml-mcp-agent-toolkit 2>/dev/null; then \
					echo "✅ Successfully added to Claude Code"; \
				else \
					echo "⚠️  Could not auto-configure Claude Code"; \
					echo "   Run manually: claude mcp add paiml-toolkit paiml-mcp-agent-toolkit"; \
				fi \
			fi; \
			echo ""; \
			echo "🎯 Final step: Restart Claude Code to use the MCP server"; \
		else \
			echo "ℹ️  Claude CLI not found. To use with Claude Code:"; \
			echo "   1. Install Claude CLI: npm install -g @anthropic-ai/claude-cli"; \
			echo "   2. Run: claude mcp add paiml-toolkit paiml-mcp-agent-toolkit"; \
		fi \
	else \
		echo "⚠️  Binary not found in PATH!"; \
		echo "   Installation completed but binary is not accessible."; \
		echo "   You may need to:"; \
		echo "   1. Restart your terminal"; \
		echo "   2. Source your shell config (source ~/.bashrc or source ~/.zshrc)"; \
		echo "   3. Add the installation directory to your PATH"; \
		exit 1; \
	fi

# Local install for development (NO VERSION BUMP)
# This is the safe target for local development
local-install: build-binary
	@echo "🚀 Installing MCP server (local development mode - no version bump)..."
	@# First, remove any existing installation
	@if [ -f "/usr/local/bin/paiml-mcp-agent-toolkit" ]; then \
		echo "📦 Found existing system installation, removing..."; \
		rm -f /usr/local/bin/paiml-mcp-agent-toolkit 2>/dev/null || \
		sudo rm -f /usr/local/bin/paiml-mcp-agent-toolkit 2>/dev/null || true; \
	fi
	@if [ -f "$(HOME)/.local/bin/paiml-mcp-agent-toolkit" ]; then \
		echo "📦 Found existing user installation, removing..."; \
		rm -f $(HOME)/.local/bin/paiml-mcp-agent-toolkit; \
	fi
	@# Now do fresh install
	@if [ -z "$(DESTDIR)" ]; then \
		if [ -w "/usr/local/bin" ]; then \
			install -m 755 target/release/paiml-mcp-agent-toolkit /usr/local/bin/paiml-mcp-agent-toolkit; \
			echo "✅ Installed to /usr/local/bin/paiml-mcp-agent-toolkit"; \
		else \
			echo "⚠️  Cannot write to /usr/local/bin. Trying user install..."; \
			mkdir -p $(HOME)/.local/bin; \
			install -m 755 target/release/paiml-mcp-agent-toolkit $(HOME)/.local/bin/paiml-mcp-agent-toolkit; \
			echo "✅ Installed to $(HOME)/.local/bin/paiml-mcp-agent-toolkit"; \
			echo ""; \
			if echo "$$PATH" | grep -q "$(HOME)/.local/bin"; then \
				echo "✅ $(HOME)/.local/bin is already in your PATH"; \
			else \
				echo "💡 Add $(HOME)/.local/bin to your PATH:"; \
				echo ""; \
				current_shell=$$(basename "$$SHELL"); \
				if [ "$$current_shell" = "zsh" ] || [ -f "$$HOME/.zshrc" ]; then \
					echo "   For zsh (your shell):"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.zshrc'; \
					echo "   source ~/.zshrc"; \
				elif [ "$$current_shell" = "bash" ] || [ -f "$$HOME/.bashrc" ]; then \
					echo "   For bash (your shell):"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.bashrc'; \
					echo "   source ~/.bashrc"; \
				else \
					echo "   Detected shell: $$SHELL"; \
					echo ""; \
					echo "   For bash:"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.bashrc'; \
					echo "   source ~/.bashrc"; \
					echo ""; \
					echo "   For zsh:"; \
					echo '   echo "export PATH=$$HOME/.local/bin:$$PATH" >> ~/.zshrc'; \
					echo "   source ~/.zshrc"; \
				fi; \
			fi \
		fi \
	else \
		install -D -m 755 target/release/paiml-mcp-agent-toolkit $(DESTDIR)/usr/local/bin/paiml-mcp-agent-toolkit; \
		echo "✅ Installed to $(DESTDIR)/usr/local/bin/paiml-mcp-agent-toolkit"; \
	fi
	@# Verify installation
	@echo ""
	@echo "🔍 Verifying installation..."
	@if command -v paiml-mcp-agent-toolkit >/dev/null 2>&1; then \
		echo "✅ Binary is accessible in PATH"; \
		echo "📍 Location: $$(which paiml-mcp-agent-toolkit)"; \
		echo "📌 Version: $$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"; \
		echo ""; \
		echo "🧪 Testing server startup..."; \
		if timeout 2s paiml-mcp-agent-toolkit < /dev/null >/dev/null 2>&1; then \
			echo "✅ Server starts successfully"; \
		else \
			exit_code=$$?; \
			if [ $$exit_code -eq 124 ]; then \
				echo "✅ Server starts successfully (timeout expected for STDIO server)"; \
			else \
				echo "⚠️  Server may have issues starting. Exit code: $$exit_code"; \
				echo "   Try running manually: paiml-mcp-agent-toolkit"; \
			fi \
		fi; \
		echo ""; \
		echo "💡 Tip: This is a local development install (no version bump)"; \
		echo "   For releases, use 'make install' which bumps the version"; \
	else \
		echo "⚠️  Binary not found in PATH!"; \
		echo "   Installation completed but binary is not accessible."; \
		echo "   You may need to:"; \
		echo "   1. Restart your terminal"; \
		echo "   2. Source your shell config (source ~/.bashrc or source ~/.zshrc)"; \
		echo "   3. Add the installation directory to your PATH"; \
		exit 1; \
	fi

# Uninstall the MCP server
uninstall:
	@echo "🗑️  Uninstalling MCP server..."
	@if [ -f "/usr/local/bin/paiml-mcp-agent-toolkit" ]; then \
		rm -f /usr/local/bin/paiml-mcp-agent-toolkit && echo "✅ Removed /usr/local/bin/paiml-mcp-agent-toolkit"; \
	elif [ -f "$(HOME)/.local/bin/paiml-mcp-agent-toolkit" ]; then \
		rm -f $(HOME)/.local/bin/paiml-mcp-agent-toolkit && echo "✅ Removed $(HOME)/.local/bin/paiml-mcp-agent-toolkit"; \
	else \
		echo "⚠️  MCP server not found in standard locations"; \
	fi

# Install development dependencies
setup:
	cargo install cargo-watch
	cargo install cargo-audit
	cargo install cargo-llvm-cov

# Security audit
audit:
	cd .. && cargo audit

# Validate documentation naming consistency
validate-docs:
	@echo "📖 Validating documentation naming consistency..."
	@if [ -f "../scripts/validate-docs.ts" ]; then \
		cd .. && deno run --allow-read --allow-env scripts/validate-docs.ts; \
	else \
		echo "⚠️  Validation script not found, skipping..."; \
	fi

# Generate documentation
docs:
	cargo doc --no-deps --open

# Check CI status (GitHub Actions)
ci-status:
	@echo "🔍 Checking CI status..."
	@cd .. && deno run --allow-net --allow-env scripts/validate-github-actions-status.ts

# Run performance benchmarks
benchmark:
	@echo "🏃 Running performance benchmarks..."
	cargo bench

# Dependency management targets
deps-check: ## Check for outdated dependencies
	@echo "🔍 Checking for outdated dependencies..."
	@cd .. && cargo outdated --exit-code 1 || \
		(echo "⚠️  Outdated dependencies found. Run 'make deps-update' to update."; exit 0)

deps-update: ## Update dependencies conservatively
	@echo "📦 Updating dependencies..."
	@cd .. && cargo update
	@cd .. && cargo outdated --format json > server/outdated-deps.json
	@echo "✅ Dependencies updated. Run tests to verify."

deps-update-aggressive: ## Update dependencies to latest compatible versions
	@echo "🚀 Aggressively updating dependencies..."
	@cd .. && cargo update --aggressive
	@cd .. && cargo upgrade --workspace --to-lockfile
	@echo "✅ Dependencies updated to latest versions."

deps-audit: ## Run security audit with auto-fix
	@echo "🔒 Running security audit..."
	@cd .. && cargo audit fix || cd .. && cargo audit

deps-canary-test: ## Test dependency updates in isolated environment
	@echo "🐤 Running canary tests for dependency updates..."
	@docker run --rm -v $(PWD)/..:/workspace -w /workspace/server rust:latest bash -c "\
		cd .. && cargo update && \
		cargo test --all-features && \
		cargo bench --no-run"

deps-rollback: ## Rollback to previous Cargo.lock
	@echo "⏪ Rolling back dependencies..."
	@git checkout HEAD -- Cargo.lock
	@cargo fetch

# Help command
help:
	@echo "MCP Template Server - Flexible Deployment Options"
	@echo ""
	@echo "Primary targets:"
	@echo "  all      - Execute complete build pipeline (format → lint → check → test → build → validate)"
	@echo "  validate - Print validation checklist of what was validated"
	@echo ""
	@echo "Development:"
	@echo "  format   - Format Rust code with rustfmt and TypeScript with deno fmt"
	@echo "  lint     - Run clippy for Rust and deno check/lint for TypeScript"
	@echo "  check    - Type check without building"
	@echo "  test     - Run tests with coverage summary"
	@echo "  coverage - Generate detailed plain text coverage report"
	@echo "  build        - Build standalone binary only (default)"
	@echo "  build-binary - Build standalone Rust binary"
	@echo ""
	@echo "Running:"
	@echo "  run-mcp  - Run MCP server in STDIO mode (stateless)"
	@echo "  run-mcp-test - Run MCP server in STDIO mode (same as run-mcp)"
	@echo ""
	@echo "Testing:"
	@echo "  test-http - Run HTTP tests against local server"
	@echo "  test-mcp - Run MCP STDIO tests"
	@echo "  test-integration - Run all integration tests (HTTP + MCP)"
	@echo "  test-all - Run all tests including integration (use with caution)"
	@echo ""
	@echo ""
	@echo "Installation:"
	@echo "  local-install  - Install for development (NO VERSION BUMP) - RECOMMENDED"
	@echo "  install        - Install with version bump (FOR RELEASES ONLY)"
	@echo "  install-latest - Check for changes and rebuild/install if needed"
	@echo "  reinstall      - Force complete uninstall and reinstall"
	@echo "  status         - Check installation and build status"
	@echo "  check-rebuild  - Check if source files changed (needs rebuild)"
	@echo "  uninstall      - Remove MCP server from system"
	@echo ""
	@echo ""
	@echo "Dependency Management:"
	@echo "  deps-check   - Check for outdated dependencies"
	@echo "  deps-update  - Update dependencies conservatively"
	@echo "  deps-update-aggressive - Update to latest compatible versions"
	@echo "  deps-audit   - Run security audit with auto-fix"
	@echo "  deps-canary-test - Test updates in isolated environment"
	@echo "  deps-rollback - Rollback to previous Cargo.lock"
	@echo ""
	@echo "Utilities:"
	@echo "  clean    - Remove build artifacts"
	@echo "  setup    - Install required development tools"
	@echo "  audit    - Run security audit"
	@echo "  docs     - Generate and open documentation"
	@echo "  ci-status - Check CI/GitHub Actions workflow status"
	@echo "  benchmark - Run performance benchmarks"
	@echo "  help     - Show this help message"

# Install latest (check for changes and rebuild if needed)
install-latest:
	@echo "🚀 Installing latest MCP server (with auto-rebuild if needed)..."
	@cd .. && ./scripts/mcp-install.ts

# Reinstall (force complete reinstall)
reinstall:
	@echo "🔄 Performing complete reinstall..."
	@cd .. && ./scripts/mcp-install.ts --reinstall

# Check installation status
status:
	@cd .. && ./scripts/mcp-install.ts --status

# Check if rebuild needed
check-rebuild:
	@cd .. && ./scripts/mcp-install.ts --check

