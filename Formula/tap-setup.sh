#!/bin/bash
# Script to set up PMAT Homebrew tap for immediate availability

set -e

echo "🍺 Setting up PMAT Homebrew Tap"
echo "==============================="

# This script shows how users can use our tap before homebrew-core acceptance

cat << 'EOF'
## Using PMAT Homebrew Tap (Before homebrew-core)

### Add the tap:
brew tap paiml/pmat https://github.com/paiml/paiml-mcp-agent-toolkit

### Install PMAT:
brew install paiml/pmat/pmat

### Verify installation:
pmat --version
pmat agent mcp-server --help

### Use with Claude Code:
# Configure in Claude Code settings.json:
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["agent", "mcp-server"],
      "env": {}
    }
  }
}

## Updating:
brew upgrade paiml/pmat/pmat

## Uninstalling:
brew uninstall pmat
brew untap paiml/pmat
EOF

echo ""
echo "✅ Tap setup instructions ready!"
echo "📝 Users can install PMAT via tap while we await homebrew-core approval"