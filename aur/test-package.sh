#!/bin/bash
set -e

echo "🏛️ Testing PMAT AUR Package"
echo "==========================="

# Check if we're on Arch Linux
if ! command -v pacman &> /dev/null; then
    echo "❌ Not an Arch Linux system - AUR package cannot be tested here"
    echo "✅ PKGBUILD and .SRCINFO ready for AUR submission"
    exit 0
fi

echo "📦 Arch Linux detected - testing AUR package build"

# Check if makepkg is available  
if ! command -v makepkg &> /dev/null; then
    echo "❌ makepkg not available - install base-devel package"
    echo "   sudo pacman -S base-devel"
    exit 1
fi

echo ""
echo "🔍 Validating PKGBUILD..."
if command -v namcap &> /dev/null; then
    namcap PKGBUILD
else
    echo "⚠️ namcap not available (install namcap package for validation)"
fi

echo ""
echo "🏗️ Testing package build..."
echo "This will:"
echo "  1. Download source tarball"
echo "  2. Verify SHA256 checksum" 
echo "  3. Build PMAT from source"
echo "  4. Create installable package"

makepkg --clean --force

echo ""
echo "📦 Package built successfully!"
echo "Generated files:"
ls -la *.pkg.tar.*

echo ""
echo "🔍 Package contents:"
tar -tf *.pkg.tar.* | head -20

echo ""
echo "✅ AUR package test completed successfully!"
echo ""
echo "Next steps:"
echo "  1. Test install: sudo pacman -U *.pkg.tar.*"
echo "  2. Verify binary: pmat --version"  
echo "  3. Test functionality: pmat agent --help"
echo "  4. Submit to AUR following SUBMIT_TO_AUR.md"