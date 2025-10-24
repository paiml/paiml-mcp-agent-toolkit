# Build and prepare PMAT Chocolatey package for submission

param(
    [switch]$Test,
    [switch]$Submit,
    [string]$ApiKey = ""
)

$ErrorActionPreference = 'Stop'

Write-Host "🍫 Building PMAT Chocolatey Package" -ForegroundColor Green
Write-Host "===================================" -ForegroundColor Green

# Check if chocolatey is available
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Chocolatey not found. Please install from: https://chocolatey.org/install" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Chocolatey found: " -NoNewline -ForegroundColor Green
choco --version

# Clean up any existing packages
Write-Host ""
Write-Host "🧹 Cleaning up previous builds..." -ForegroundColor Yellow
Get-ChildItem "pmat.*.nupkg" -ErrorAction SilentlyContinue | Remove-Item -Force
Write-Host "✅ Cleanup completed" -ForegroundColor Green

# Build the package
Write-Host ""
Write-Host "📦 Building package..." -ForegroundColor Yellow
try {
    choco pack pmat.nuspec
    
    $packageFile = Get-ChildItem "pmat.*.nupkg" | Select-Object -First 1
    if ($packageFile) {
        Write-Host "✅ Package created: $($packageFile.Name)" -ForegroundColor Green
        Write-Host "📏 Package size: $([math]::Round($packageFile.Length / 1KB, 2)) KB" -ForegroundColor Gray
    } else {
        throw "Package file not found after build"
    }
} catch {
    Write-Host "❌ Package build failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Run tests if requested
if ($Test) {
    Write-Host ""
    Write-Host "🧪 Running package tests..." -ForegroundColor Yellow
    
    try {
        .\test-package.ps1 -SkipInstall
        Write-Host "✅ Package tests passed" -ForegroundColor Green
    } catch {
        Write-Host "❌ Package tests failed: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "Fix issues before submission" -ForegroundColor Yellow
        exit 1
    }
}

# Display package information
Write-Host ""
Write-Host "📋 Package Information:" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan

try {
    $packageInfo = choco info $packageFile.Name --local-only --limit-output
    Write-Host "Package ID: pmat" -ForegroundColor White
    Write-Host "Version: 2.171.1" -ForegroundColor White
    Write-Host "Authors: Pragmatic AI Labs" -ForegroundColor White
    Write-Host "Tags: ai, claude-code, mcp, agent, code-quality" -ForegroundColor White
} catch {
    Write-Host "Could not retrieve package info" -ForegroundColor Yellow
}

# Submit if requested
if ($Submit) {
    if (-not $ApiKey) {
        Write-Host ""
        Write-Host "🔑 API Key required for submission" -ForegroundColor Yellow
        Write-Host "Get your API key from: https://community.chocolatey.org/account" -ForegroundColor Blue
        Write-Host "Then run: .\build-package.ps1 -Submit -ApiKey 'YOUR_API_KEY'" -ForegroundColor Gray
        exit 1
    }
    
    Write-Host ""
    Write-Host "🚀 Submitting to Chocolatey Community Repository..." -ForegroundColor Yellow
    
    try {
        # Set API key
        choco apikey --key $ApiKey --source https://push.chocolatey.org/
        
        # Push package
        choco push $packageFile.Name --source https://push.chocolatey.org/
        
        Write-Host "✅ Package submitted successfully!" -ForegroundColor Green
        Write-Host ""
        Write-Host "📋 Next Steps:" -ForegroundColor Cyan
        Write-Host "1. Package will undergo automated validation" -ForegroundColor White
        Write-Host "2. Community moderators will review (3-7 days)" -ForegroundColor White
        Write-Host "3. You'll receive email notifications about status" -ForegroundColor White
        Write-Host "4. Track progress at: https://community.chocolatey.org/packages/pmat" -ForegroundColor White
        
    } catch {
        Write-Host "❌ Submission failed: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "Check your API key and network connection" -ForegroundColor Yellow
        exit 1
    }
}

# Final instructions
Write-Host ""
Write-Host "🎯 Build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📤 Manual submission:" -ForegroundColor Cyan
Write-Host "choco push $($packageFile.Name) --source https://push.chocolatey.org/" -ForegroundColor Gray
Write-Host ""
Write-Host "🧪 Test locally:" -ForegroundColor Cyan
Write-Host ".\test-package.ps1" -ForegroundColor Gray
Write-Host ""
Write-Host "📚 Submission guide:" -ForegroundColor Cyan
Write-Host "Read SUBMIT_TO_CHOCOLATEY.md for complete details" -ForegroundColor Gray