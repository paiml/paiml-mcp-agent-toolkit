#!/bin/bash
# Build PMAT Debian package

set -e

echo "🐧 Building PMAT Debian Package"
echo "==============================="

# Check if we're in the right directory
if [ ! -f "DEBIAN/control" ]; then
    echo "❌ Must be run from debian directory"
    echo "Usage: cd debian && ./build-deb.sh"
    exit 1
fi

# Check dependencies
if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "❌ dpkg-deb not found. Install with: sudo apt install dpkg-dev"
    exit 1
fi

# Compress changelog
echo "📝 Compressing changelog..."
if [ -f "usr/share/doc/pmat/changelog.Debian.gz" ]; then
    # Decompress, then recompress properly
    gunzip -f usr/share/doc/pmat/changelog.Debian.gz 2>/dev/null || true
fi

if [ -f "usr/share/doc/pmat/changelog.Debian" ]; then
    gzip -9 usr/share/doc/pmat/changelog.Debian
else
    echo "⚠️ changelog.Debian not found, creating basic one..."
    cat > usr/share/doc/pmat/changelog.Debian << 'EOF'
pmat (2.172.0) unstable; urgency=medium

  * Initial Debian package release
  * Claude Code Agent Mode integration
  * MCP server implementation
  * Multi-ecosystem distribution support

 -- Pragmatic AI Labs <hello@paiml.com>  Fri, 23 Aug 2024 12:00:00 +0000
EOF
    gzip -9 usr/share/doc/pmat/changelog.Debian
fi

# Compress man page
echo "📖 Compressing man page..."
if [ -f "usr/share/man/man1/pmat.1" ]; then
    gzip -9 -f usr/share/man/man1/pmat.1
fi

# Set permissions
echo "🔧 Setting correct permissions..."
find . -type d -exec chmod 755 "{}" \;
find . -type f -exec chmod 644 "{}" \;
chmod +x DEBIAN/postinst DEBIAN/prerm DEBIAN/postrm

# Fix ownership (best effort)
if [ "$EUID" -eq 0 ]; then
    echo "👤 Setting ownership to root:root..."
    chown -R root:root .
else
    echo "ℹ️ Not running as root - permissions may need adjustment"
fi

# Calculate installed size
echo "📏 Calculating package size..."
INSTALLED_SIZE=$(du -ks . | cut -f1)
sed -i "s/^Installed-Size:.*/Installed-Size: $INSTALLED_SIZE/" DEBIAN/control

# Build the package
echo "📦 Building package..."
cd ..
dpkg-deb --build debian pmat_2.172.0_amd64.deb

if [ -f "pmat_2.172.0_amd64.deb" ]; then
    echo "✅ Package built successfully: pmat_2.172.0_amd64.deb"
    echo "📏 Package size: $(du -h pmat_2.172.0_amd64.deb | cut -f1)"

    # Validate package
    echo ""
    echo "🔍 Validating package..."
    dpkg-deb --info pmat_2.172.0_amd64.deb

    echo ""
    echo "📋 Package contents:"
    dpkg-deb --contents pmat_2.172.0_amd64.deb | head -20

    echo ""
    echo "✅ Package validation completed!"
    echo ""
    echo "🧪 Test installation:"
    echo "  sudo dpkg -i pmat_2.172.0_amd64.deb"
    echo ""
    echo "🗑️ Remove package:"
    echo "  sudo apt remove pmat"
    echo ""
    echo "🧹 Purge completely:"  
    echo "  sudo apt purge pmat"
    
else
    echo "❌ Package build failed"
    exit 1
fi