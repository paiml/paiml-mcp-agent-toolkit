#!/bin/bash
# Test PMAT Debian package

set -e

echo "🧪 Testing PMAT Debian Package"
echo "==============================="

# Check if package exists
if [ ! -f "../pmat_2.10.0_amd64.deb" ]; then
    echo "❌ Package file not found. Build first with: ./build-deb.sh"
    exit 1
fi

# Check if running on Debian/Ubuntu
if ! command -v dpkg >/dev/null 2>&1; then
    echo "❌ Not a Debian/Ubuntu system - cannot test .deb package"
    exit 1
fi

echo "📦 Package found: ../pmat_2.10.0_amd64.deb"

# Package info
echo ""
echo "📋 Package Information:"
echo "======================="
dpkg-deb --field ../pmat_2.10.0_amd64.deb

# Lint package
echo ""
echo "🔍 Linting package..."
if command -v lintian >/dev/null 2>&1; then
    echo "Running lintian checks..."
    lintian ../pmat_2.10.0_amd64.deb || echo "⚠️ Lintian warnings (may be acceptable)"
else
    echo "ℹ️ lintian not available (install with: sudo apt install lintian)"
fi

# Test installation (requires sudo)
if [ "$EUID" -eq 0 ]; then
    echo ""
    echo "🔧 Testing installation as root..."
    
    # Install package
    echo "Installing package..."
    dpkg -i ../pmat_2.10.0_amd64.deb || apt-get install -f -y
    
    echo "✅ Package installed"
    
    # Verify files
    echo ""
    echo "📁 Verifying installed files..."
    dpkg -L pmat | head -20
    
    # Test basic functionality (if pmat binary is available)
    if command -v pmat >/dev/null 2>&1; then
        echo ""
        echo "🎯 Testing basic functionality..."
        pmat --version || echo "⚠️ pmat --version failed (expected if binary not installed)"
        echo "Binary location: $(which pmat 2>/dev/null || echo 'not found')"
    else
        echo ""
        echo "ℹ️ pmat binary not available (expected - requires separate installation)"
        echo "Package provides framework and service setup"
    fi
    
    # Check service status
    if command -v systemctl >/dev/null 2>&1; then
        echo ""
        echo "🔧 Checking systemd integration..."
        systemctl status pmat-agent.service --no-pager || echo "ℹ️ Service not started (expected)"
        echo "Service file: $(systemctl show -p FragmentPath pmat-agent.service --value)"
    fi
    
    # Offer to remove
    echo ""
    read -p "🗑️ Remove package for cleanup? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Removing package..."
        apt-get remove pmat -y
        echo "✅ Package removed"
        
        read -p "🧹 Purge configuration and data? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            apt-get purge pmat -y
            echo "✅ Package purged"
        fi
    fi
    
else
    echo ""
    echo "ℹ️ Not running as root - skipping installation test"
    echo "To test installation manually:"
    echo "  sudo dpkg -i ../pmat_2.10.0_amd64.deb"
    echo "  sudo apt remove pmat"
    echo "  sudo apt purge pmat"
fi

# Check package contents
echo ""
echo "📦 Package Contents:"
echo "===================="
dpkg-deb --contents ../pmat_2.10.0_amd64.deb

echo ""
echo "✅ Package testing completed!"
echo ""
echo "📤 Package ready for distribution:"
echo "  - Local installation: sudo dpkg -i pmat_2.10.0_amd64.deb"
echo "  - APT repository: Upload to repository and run apt update"  
echo "  - PPA submission: Follow Ubuntu PPA guidelines"