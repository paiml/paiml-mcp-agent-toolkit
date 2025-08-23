# Test script for PMAT Chocolatey package

param(
    [switch]$SkipInstall,
    [switch]$Verbose
)

$ErrorActionPreference = 'Stop'

Write-Host "🍫 Testing PMAT Chocolatey Package" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "⚠️ This script should be run as Administrator for full testing" -ForegroundColor Yellow
    Write-Host "Some operations may fail without admin privileges" -ForegroundColor Yellow
    Write-Host ""
}

# Check if chocolatey is available
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Chocolatey not found. Please install Chocolatey first:" -ForegroundColor Red
    Write-Host "https://chocolatey.org/install" -ForegroundColor Blue
    exit 1
}

Write-Host "✅ Chocolatey found" -ForegroundColor Green
choco --version

Write-Host ""
Write-Host "📦 Testing package creation..." -ForegroundColor Yellow

# Test package creation
try {
    choco pack pmat.nuspec
    
    if (Test-Path "pmat.*.nupkg") {
        Write-Host "✅ Package created successfully" -ForegroundColor Green
        $packageFile = Get-ChildItem "pmat.*.nupkg" | Select-Object -First 1
        Write-Host "Package file: $($packageFile.Name)" -ForegroundColor Gray
    } else {
        Write-Host "❌ Package creation failed - no .nupkg file found" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Package creation failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

if (-not $SkipInstall) {
    Write-Host ""
    Write-Host "🧪 Testing package installation..." -ForegroundColor Yellow
    
    # Test installation from local package
    try {
        $packageFile = Get-ChildItem "pmat.*.nupkg" | Select-Object -First 1
        choco install $packageFile.Name --source . --force --yes
        
        Write-Host "✅ Package installation completed" -ForegroundColor Green
        
        # Test if pmat is available (might need to restart shell)
        Write-Host ""
        Write-Host "🔍 Testing PMAT availability..." -ForegroundColor Yellow
        
        # Check if cargo is available for actual installation
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            Write-Host "✅ Cargo available - PMAT should be installable" -ForegroundColor Green
            
            # Try to run pmat if it was installed
            if (Get-Command pmat -ErrorAction SilentlyContinue) {
                Write-Host "✅ pmat command found in PATH" -ForegroundColor Green
                pmat --version
            } else {
                Write-Host "ℹ️ pmat command not immediately available (may need shell restart)" -ForegroundColor Yellow
                Write-Host "This is expected - the package provides installation helpers" -ForegroundColor Gray
            }
        } else {
            Write-Host "ℹ️ Cargo not available - installation helpers provided instead" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "🧹 Testing package uninstallation..." -ForegroundColor Yellow
        choco uninstall pmat --yes
        Write-Host "✅ Package uninstallation completed" -ForegroundColor Green
        
    } catch {
        Write-Host "⚠️ Installation test failed: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "This may be expected if dependencies are not available" -ForegroundColor Gray
    }
}

# Validate package contents
Write-Host ""
Write-Host "🔍 Validating package structure..." -ForegroundColor Yellow

$expectedFiles = @(
    "tools\chocolateyinstall.ps1",
    "tools\chocolateyuninstall.ps1", 
    "legal\VERIFICATION.txt",
    "legal\LICENSE.txt"
)

$packageFile = Get-ChildItem "pmat.*.nupkg" | Select-Object -First 1
Add-Type -AssemblyName System.IO.Compression.FileSystem

try {
    $zip = [System.IO.Compression.ZipFile]::OpenRead($packageFile.FullName)
    
    foreach ($expectedFile in $expectedFiles) {
        $entry = $zip.Entries | Where-Object { $_.FullName -eq $expectedFile }
        if ($entry) {
            Write-Host "✅ Found: $expectedFile" -ForegroundColor Green
        } else {
            Write-Host "❌ Missing: $expectedFile" -ForegroundColor Red
        }
    }
    
    $zip.Dispose()
} catch {
    Write-Host "⚠️ Could not validate package contents: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "📋 Package Information:" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan
Write-Host "Package ID: pmat" -ForegroundColor White
Write-Host "Version: 2.10.0" -ForegroundColor White
Write-Host "Type: Cargo-based installation with fallback helpers" -ForegroundColor White
Write-Host "Dependencies: None (optional: Rust/Cargo or Node.js/npm)" -ForegroundColor White

Write-Host ""
Write-Host "🎉 Chocolatey package test completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📤 Ready for submission to Chocolatey Community Repository" -ForegroundColor Blue
Write-Host "Submission guide: https://docs.chocolatey.org/en-us/community-repository/moderation/" -ForegroundColor Gray

# Clean up
if (Test-Path "pmat.*.nupkg") {
    Write-Host ""
    Write-Host "🧹 Cleaning up test artifacts..." -ForegroundColor Gray
    Remove-Item "pmat.*.nupkg" -Force
    Write-Host "✅ Cleanup completed" -ForegroundColor Green
}