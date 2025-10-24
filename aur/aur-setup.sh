#!/bin/bash
# Script to set up PMAT AUR repository for submission

set -e

echo "🏛️ Setting up PMAT AUR Submission"
echo "================================="

# Check for SSH key for AUR
if [ ! -f ~/.ssh/id_rsa ] && [ ! -f ~/.ssh/id_ed25519 ]; then
    echo "❌ No SSH key found for AUR submission"
    echo "Generate one with: ssh-keygen -t ed25519 -C 'your-email@example.com'"
    echo "Then add the public key to your AUR account at: https://aur.archlinux.org/account/"
    exit 1
fi

echo "✅ SSH key found"

# Check AUR account setup
echo ""
echo "🔑 Testing AUR SSH access..."
if ssh -T aur@aur.archlinux.org 2>&1 | grep -q "successfully authenticated"; then
    echo "✅ AUR SSH access confirmed"
else
    echo "❌ AUR SSH access failed"
    echo "Make sure you've:"
    echo "  1. Created an AUR account at https://aur.archlinux.org/register/"
    echo "  2. Added your SSH public key to your AUR account"
    echo "  3. Your SSH key is properly configured"
    exit 1
fi

# Clone or prepare AUR repository
echo ""
echo "📂 Setting up AUR repository..."
if [ -d "aur-pmat" ]; then
    echo "Directory aur-pmat already exists - updating"
    cd aur-pmat
    git pull origin master || true
    cd ..
else
    echo "Cloning AUR repository for pmat package..."
    git clone ssh://aur@aur.archlinux.org/pmat.git aur-pmat || {
        echo "Creating new AUR repository (first submission)"
        mkdir aur-pmat
        cd aur-pmat
        git init
        git remote add origin ssh://aur@aur.archlinux.org/pmat.git
        cd ..
    }
fi

# Copy package files
echo ""
echo "📋 Copying package files..."
cp PKGBUILD aur-pmat/
cp .SRCINFO aur-pmat/

echo ""
echo "✅ AUR setup completed!"
echo ""
echo "Next steps:"
echo "  cd aur-pmat"
echo "  git add PKGBUILD .SRCINFO"
echo "  git commit -m 'pmat: initial upload - v2.171.1'"
echo "  git push origin master"
echo ""
echo "Or run the full submission process:"
echo "  ./submit-to-aur.sh"