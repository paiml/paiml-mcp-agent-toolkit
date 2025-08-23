#!/bin/bash
# Automated AUR submission script for PMAT

set -e

echo "🏛️ Submitting PMAT v2.10.0 to AUR"
echo "=================================="

# Run setup first
./aur-setup.sh

cd aur-pmat

echo ""
echo "📝 Preparing commit..."

# Add files
git add PKGBUILD .SRCINFO

# Check if there are changes to commit
if git diff --cached --quiet; then
    echo "⚠️ No changes detected - package may already be up to date"
    git status
    exit 0
fi

echo ""
echo "📋 Changes to be committed:"
git diff --cached --name-only

echo ""
echo "🚀 Committing and pushing to AUR..."

# Commit with descriptive message
git commit -m "pmat: initial upload - v2.10.0

PMAT is a zero-config AI context generation and code quality toolkit 
with Claude Code Agent Mode for continuous quality monitoring.

Features:
- Claude Code Agent Mode with MCP protocol integration
- AI context generation optimized for LLM workflows  
- Code complexity analysis with Toyota Way standards
- Technical debt detection and quality gates
- Multi-language support (30+ languages via tree-sitter)
- Production-ready systemd service deployment

Homepage: https://github.com/paiml/paiml-mcp-agent-toolkit
License: MIT"

# Push to AUR
git push origin master

echo ""
echo "🎉 Successfully submitted to AUR!"
echo ""
echo "Package now available at:"
echo "  https://aur.archlinux.org/packages/pmat"
echo ""
echo "Users can install with:"
echo "  yay -S pmat"
echo "  # or"  
echo "  git clone https://aur.archlinux.org/pmat.git"
echo "  cd pmat"
echo "  makepkg -si"
echo ""
echo "✅ Arch Linux AUR distribution complete!"