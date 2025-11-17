#!/bin/bash
set -e

echo "🍺 Testing PMAT Homebrew Formula"
echo "================================="

# Check if brew is available
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew not available in this environment"
    echo "✅ Formula ready for testing on macOS/Linux with Homebrew"
    exit 0
fi

echo "📦 Homebrew version:"
brew --version

echo ""
echo "🧪 Testing formula syntax..."
brew audit --strict homebrew/pmat.rb

echo ""
echo "🔍 Installing from local formula..."
brew install --build-from-source homebrew/pmat.rb

echo ""
echo "✅ Testing installed binary..."
pmat --version
pmat --help | head -5

echo ""
echo "🧹 Cleaning up..."
brew uninstall pmat

echo ""
echo "🎉 Formula test completed successfully!"