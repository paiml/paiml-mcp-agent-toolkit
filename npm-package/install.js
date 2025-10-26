#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');
const os = require('os');

const VERSION = '2.172.0';
const REPO = 'paiml/paiml-mcp-agent-toolkit';

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();
  
  if (platform === 'linux' && arch === 'x64') return 'x86_64-unknown-linux-gnu';
  if (platform === 'linux' && arch === 'arm64') return 'aarch64-unknown-linux-gnu';
  if (platform === 'darwin' && arch === 'x64') return 'x86_64-apple-darwin';
  if (platform === 'darwin' && arch === 'arm64') return 'aarch64-apple-darwin';
  if (platform === 'win32' && arch === 'x64') return 'x86_64-pc-windows-msvc';
  
  throw new Error(`Unsupported platform: ${platform}-${arch}`);
}

function downloadBinary() {
  return new Promise((resolve, reject) => {
    console.log('📦 Installing PMAT v2.172.0...');
    
    // Check if cargo is available for installation from source
    try {
      execSync('cargo --version', { stdio: 'ignore' });
      console.log('🦀 Cargo detected, installing from crates.io...');
      execSync('cargo install pmat --version 2.172.0 --force', { stdio: 'inherit' });
      
      // Create wrapper script
      const binDir = path.join(__dirname, 'bin');
      if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir, { recursive: true });
      }
      
      const wrapperScript = process.platform === 'win32' 
        ? 'pmat.cmd'
        : 'pmat';
      
      const wrapperContent = process.platform === 'win32'
        ? `@echo off\npmat %*`
        : `#!/bin/bash\nexec pmat "$@"`;
      
      fs.writeFileSync(path.join(binDir, wrapperScript), wrapperContent, { mode: 0o755 });
      
      console.log('✅ PMAT installed successfully!');
      console.log('🤖 Try: pmat agent mcp-server');
      resolve();
    } catch (error) {
      console.log('⚠️  Cargo not found, trying pre-built binaries...');
      
      try {
        const target = getPlatform();
        const url = `https://github.com/${REPO}/releases/download/v${VERSION}/pmat-${target}`;
        
        console.log(`📥 Downloading from: ${url}`);
        
        // For now, fall back to cargo installation instruction
        console.log('🦀 Please install Rust and Cargo, then run:');
        console.log('   cargo install pmat');
        console.log('');
        console.log('🤖 Or download directly from:');
        console.log(`   https://github.com/${REPO}/releases/tag/v${VERSION}`);
        
        resolve();
      } catch (err) {
        reject(err);
      }
    }
  });
}

downloadBinary().catch(error => {
  console.error('❌ Installation failed:', error.message);
  console.log('');
  console.log('💡 Manual installation options:');
  console.log('   1. Install Rust: https://rustup.rs/');
  console.log('   2. Run: cargo install pmat');
  console.log('   3. Or download from GitHub releases');
  process.exit(1);
});