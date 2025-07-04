#!/bin/sh
# MCP Agent Toolkit Installation Script
#
# This is a standalone POSIX-compliant shell installer that works on Linux, macOS, and Windows (via WSL).
# A TypeScript/Deno version is also available at scripts/install.ts for those who prefer it.
# 
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
#   
# Or to install a specific version:
#   curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh -s v0.1.0

set -euf

# Configuration
REPO="paiml/paiml-mcp-agent-toolkit"
BINARY_NAME="pmat"
INSTALL_DIR="${HOME}/.local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Helper functions
error() {
    printf "${RED}Error: %s${NC}\n" "$1" >&2
    exit 1
}

info() {
    printf "${GREEN}%s${NC}\n" "$1"
}

warn() {
    printf "${YELLOW}%s${NC}\n" "$1"
}

# Cleanup function to remove test artifacts
cleanup_artifacts() {
    # Remove any tar.gz files that might be test artifacts in current directory
    # Only remove files with suspicious names that indicate test failures
    for file in *-platform.tar.gz nonexistent-platform.tar.gz; do
        if [ -f "$file" ]; then
            rm -f "$file" 2>/dev/null || true
        fi
    done
}

# Detect platform (returns full Rust target triple)
detect_platform() {
    os=$(uname -s)
    arch=$(uname -m)
    
    case "$os" in
        Linux*)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-gnu";;
                aarch64) echo "aarch64-unknown-linux-gnu";;
                *)       error "Unsupported Linux architecture: $arch";;
            esac
            ;;
        Darwin*)
            warn "Pre-built binaries are not available for macOS."
            warn "Please install using cargo:"
            warn ""
            warn "  cargo install pmat"
            warn ""
            warn "Or from the latest git version:"
            warn "  cargo install --git https://github.com/paiml/paiml-mcp-agent-toolkit.git pmat"
            exit 0
            ;;
        MINGW*|CYGWIN*|MSYS*)
            echo "x86_64-pc-windows-msvc"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
}

# Get latest version from GitHub
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
install() {
    PLATFORM=$(detect_platform)
    VERSION="${1:-$(get_latest_version)}"
    
    # Remove 'v' prefix if present
    VERSION="${VERSION#v}"
    
    info "Installing ${BINARY_NAME} v${VERSION} for ${PLATFORM}..."
    
    # Construct download URL
    # Note: Release artifacts use full repo name, not just binary name
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/paiml-mcp-agent-toolkit-${PLATFORM}.tar.gz"
    
    # Create temp directory
    TMP_DIR=$(mktemp -d)
    # Enhanced cleanup trap that also removes any stray tar.gz files in CWD
    trap 'rm -rf "$TMP_DIR"; cleanup_artifacts' EXIT
    
    # Download binary
    info "Downloading from ${DOWNLOAD_URL}..."
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/archive.tar.gz"; then
        error "Failed to download binary. Please check if version ${VERSION} exists for ${PLATFORM}."
    fi
    
    # Extract binary
    tar -xzf "$TMP_DIR/archive.tar.gz" -C "$TMP_DIR"
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Install binary
    if [ -f "$TMP_DIR/${BINARY_NAME}" ]; then
        mv "$TMP_DIR/${BINARY_NAME}" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/${BINARY_NAME}"
    else
        error "Binary not found in archive"
    fi
    
    info "Successfully installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
    
    # Check if install dir is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            warn "Warning: ${INSTALL_DIR} is not in your PATH."
            warn "Add the following to your shell profile:"
            warn "  export PATH=\"\$PATH:${INSTALL_DIR}\""
            ;;
    esac
    
    # Verify installation
    if command -v "${BINARY_NAME}" >/dev/null 2>&1; then
        info "Installation complete! Run '${BINARY_NAME} --version' to verify."
    else
        warn "Installation complete, but ${BINARY_NAME} is not in your PATH yet."
        warn "Please restart your shell or add ${INSTALL_DIR} to your PATH."
    fi
}

# Show help
show_help() {
    echo "MCP Agent Toolkit Installer"
    echo ""
    echo "Usage: $0 [OPTIONS] [VERSION]"
    echo ""
    echo "Options:"
    echo "  --help, -h    Show this help message"
    echo ""
    echo "Arguments:"
    echo "  VERSION       Specific version to install (e.g., v0.1.0)"
    echo "                If not specified, installs the latest version"
    echo ""
    echo "Examples:"
    echo "  $0                    # Install latest version"
    echo "  $0 v0.1.0            # Install specific version"
    echo ""
    echo "Environment variables:"
    echo "  INSTALL_DIR   Installation directory (default: ~/.local/bin)"
}

# Main
main() {
    # Check for help flag
    case "${1:-}" in
        --help|-h)
            show_help
            exit 0
            ;;
    esac
    
    info "MCP Agent Toolkit Installer"
    install "$@"
}

main "$@"