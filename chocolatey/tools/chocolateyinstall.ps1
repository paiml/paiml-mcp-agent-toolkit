$ErrorActionPreference = 'Stop'

$packageName = 'pmat'
$version = '2.10.0'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

Write-Host "🤖 Installing PMAT v$version - Pragmatic AI MCP Agent Toolkit" -ForegroundColor Green
Write-Host "=====================================================================" -ForegroundColor Green

# Try to install via cargo first (preferred method)
try {
  $cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
  if ($cargoPath) {
    Write-Host "🦀 Rust/Cargo detected - installing from crates.io..." -ForegroundColor Green
    Write-Host "⏳ Running: cargo install pmat --version $version --force" -ForegroundColor Yellow
    
    & cargo install pmat --version $version --force
    
    if ($LASTEXITCODE -eq 0) {
      Write-Host "✅ PMAT installed successfully via Cargo!" -ForegroundColor Green
      Write-Host ""
      Write-Host "🎯 Quick Start:" -ForegroundColor Cyan
      Write-Host "  pmat --version                    # Verify installation" -ForegroundColor White  
      Write-Host "  pmat context                      # Generate AI context" -ForegroundColor White
      Write-Host "  pmat quality-gate                 # Run quality checks" -ForegroundColor White
      Write-Host "  pmat agent mcp-server             # Start Claude Code Agent Mode" -ForegroundColor White
      Write-Host ""
      Write-Host "🤖 Claude Code Integration:" -ForegroundColor Cyan
      Write-Host "  Add to settings.json:" -ForegroundColor White
      Write-Host '  "mcpServers": { "pmat": { "command": "pmat", "args": ["agent", "mcp-server"] } }' -ForegroundColor Gray
      Write-Host ""
      Write-Host "📚 Documentation: https://github.com/paiml/paiml-mcp-agent-toolkit" -ForegroundColor Blue
      return
    }
  } else {
    Write-Host "ℹ️ Rust/Cargo not found - will provide installation instructions" -ForegroundColor Yellow
  }
} catch {
  Write-Host "⚠️ Cargo installation failed - providing alternative methods..." -ForegroundColor Yellow
}

# Provide comprehensive installation guide
Write-Host ""
Write-Host "🦀 PMAT Installation Options for Windows:" -ForegroundColor Yellow
Write-Host "===========================================" -ForegroundColor Yellow
Write-Host ""
Write-Host "Option 1 (Recommended): Install via Rust/Cargo" -ForegroundColor Green
Write-Host "  1. Install Rust: https://rustup.rs/" -ForegroundColor White
Write-Host "  2. Restart PowerShell/Command Prompt" -ForegroundColor White  
Write-Host "  3. Run: cargo install pmat" -ForegroundColor White
Write-Host ""
Write-Host "Option 2: Use npm package (Node.js ecosystem)" -ForegroundColor Green
Write-Host "  npm install -g pmat-agent" -ForegroundColor White
Write-Host ""
Write-Host "Option 3: Download pre-built binary (when available)" -ForegroundColor Green
Write-Host "  https://github.com/paiml/paiml-mcp-agent-toolkit/releases" -ForegroundColor White
Write-Host ""
Write-Host "Option 4: Use Docker" -ForegroundColor Green
Write-Host "  docker run --rm paiml/pmat:latest pmat --version" -ForegroundColor White

# Create helper scripts
$installPath = Join-Path $env:ProgramFiles "PMAT"
if (!(Test-Path $installPath)) {
  New-Item -ItemType Directory -Path $installPath -Force | Out-Null
}

# Create cargo installation helper
$cargoInstallScript = @"
@echo off
echo ================================================
echo PMAT - Pragmatic AI MCP Agent Toolkit Installer
echo ================================================
echo.
echo This will install PMAT via Rust/Cargo...
echo.

REM Check if cargo is available
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Rust/Cargo not found. Installing Rust first...
    echo Opening Rust installer page...
    start https://rustup.rs/
    echo.
    echo After installing Rust:
    echo 1. Restart this command prompt
    echo 2. Run: cargo install pmat
    pause
    exit /b 1
)

echo Installing PMAT from crates.io...
cargo install pmat --force

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✅ PMAT installed successfully!
    echo.
    echo Quick start:
    echo   pmat --version
    echo   pmat context  
    echo   pmat agent mcp-server
    echo.
) else (
    echo ❌ Installation failed. Please check your internet connection and try again.
)

pause
"@

$cargoScriptPath = Join-Path $installPath "install-via-cargo.bat"  
$cargoInstallScript | Out-File -FilePath $cargoScriptPath -Encoding ASCII

# Create npm installation helper  
$npmInstallScript = @"
@echo off
echo =======================================================
echo PMAT - Alternative Installation via npm (Node.js)
echo =======================================================
echo.

REM Check if npm is available
where npm >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Node.js/npm not found. 
    echo Please install Node.js from: https://nodejs.org/
    pause
    exit /b 1
)

echo Installing PMAT via npm...
npm install -g pmat-agent

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✅ PMAT installed successfully via npm!
    echo.
    echo The binary is available as both 'pmat' and 'pmat-agent'
    echo.
) else (
    echo ❌ npm installation failed.
)

pause
"@

$npmScriptPath = Join-Path $installPath "install-via-npm.bat"
$npmInstallScript | Out-File -FilePath $npmScriptPath -Encoding ASCII

# Add to PATH
$envPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($envPath -notlike "*$installPath*") {
  [Environment]::SetEnvironmentVariable("Path", "$envPath;$installPath", "Machine")
  Write-Host "✅ Added PMAT tools to system PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "📝 Installation helpers created:" -ForegroundColor Green
Write-Host "  $cargoScriptPath" -ForegroundColor White
Write-Host "  $npmScriptPath" -ForegroundColor White
Write-Host ""
Write-Host "💡 You can run these scripts to install PMAT with your preferred method." -ForegroundColor Cyan