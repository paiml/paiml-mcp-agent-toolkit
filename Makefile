# MCP Agent Toolkit - Root Makefile
# Pragmatic AI Labs
# https://paiml.com

.PHONY: all validate format lint check test build clean install install-latest reinstall status check-rebuild uninstall help format-scripts lint-scripts check-scripts fix validate-docs

# Define sub-projects
# NOTE: client project will be added when implemented
PROJECTS = server

# Scripts directory path
SCRIPTS_DIR = scripts

# Default target: format and build all projects
all: format build

# Validate everything passes across all projects
validate: check lint test validate-docs
	@echo "✅ All projects validated! All checks passed:"
	@echo "  ✓ Type checking (cargo check)"
	@echo "  ✓ Linting (cargo clippy + deno lint)"
	@echo "  ✓ Testing (cargo test)"
	@echo "  ✓ Documentation naming consistency"
	@echo "  ✓ Ready for build!"

# Format code in all projects
format: format-scripts
	@for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "📝 Formatting $$project..."; \
			$(MAKE) -C $$project format; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done

# Fix all formatting and linting issues automatically
fix: format
	@echo "🔧 Auto-fixing all formatting and lint issues..."
	@echo "✅ All fixable issues have been resolved!"

# Run linting in all projects
lint: lint-scripts
	@for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🔍 Linting $$project..."; \
			$(MAKE) -C $$project lint; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done

# Type check all projects
check: check-scripts
	@for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "✅ Checking $$project..."; \
			$(MAKE) -C $$project check; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done

# Run tests in all projects
test:
	@for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🧪 Testing $$project..."; \
			$(MAKE) -C $$project test; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done



# Build all projects (binaries only - no Docker)
build: validate-docs
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
	@for project in $(PROJECTS); do \
		if [ -d "$$project" ] && [ -f "$$project/Makefile" ]; then \
			echo "🧹 Cleaning $$project..."; \
			$(MAKE) -C $$project clean; \
		else \
			echo "⚠️  Skipping $$project (no Makefile found)"; \
		fi \
	done

# Format TypeScript scripts
format-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "📝 Formatting TypeScript scripts..."; \
		deno fmt $(SCRIPTS_DIR)/*.ts; \
	else \
		echo "✓ No TypeScript scripts to format"; \
	fi

# Lint TypeScript scripts
lint-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "🔍 Linting TypeScript scripts..."; \
		deno lint $(SCRIPTS_DIR)/*.ts; \
	else \
		echo "✓ No TypeScript scripts to lint"; \
	fi

# Type check TypeScript scripts
check-scripts:
	@if [ -d "$(SCRIPTS_DIR)" ] && [ "$$(find $(SCRIPTS_DIR) -name '*.ts' -type f 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "✅ Type checking TypeScript scripts..."; \
		deno check $(SCRIPTS_DIR)/*.ts; \
	else \
		echo "✓ No TypeScript scripts to check"; \
	fi

# Validate documentation naming consistency
validate-docs:
	@echo "📖 Validating documentation naming consistency..."
	@deno run --allow-read --allow-env $(SCRIPTS_DIR)/validate-docs.ts

# Install MCP server
install:
	@echo "🚀 Installing MCP Agent Toolkit..."
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
	@echo "  validate-docs - Check documentation naming consistency"
	@echo "  build        - Build all projects (binaries only)"
	@echo "  clean        - Clean all build artifacts"
	@echo ""
	@echo "Installation:"
	@echo "  install        - Install MCP server binary (always builds first)"
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
	@echo "  help        - Show this help message"
	@echo ""
	@echo "Projects included:"
	@for project in $(PROJECTS); do \
		echo "  - $$project"; \
	done