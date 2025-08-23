$ErrorActionPreference = 'Stop'

$packageName = 'pmat'
$version = '2.10.0'
$url64 = "https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download/v$version/pmat-x86_64-pc-windows-msvc.exe"

$packageArgs = @{
  packageName   = $packageName
  fileType      = 'exe'
  url64bit      = $url64
  softwareName  = 'PMAT*'
  
  checksum64    = 'SHA256_PLACEHOLDER'  # Will need to be updated
  checksumType64= 'sha256'
  
  silentArgs    = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-'
  validExitCodes= @(0)
}

# Try to install via cargo first (if Rust is available)
try {
  $cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
  if ($cargoPath) {
    Write-Host "🦀 Rust/Cargo detected, installing from crates.io..." -ForegroundColor Green
    & cargo install pmat --version $version --force
    
    if ($LASTEXITCODE -eq 0) {
      Write-Host "✅ PMAT installed successfully via Cargo!" -ForegroundColor Green
      Write-Host "🤖 Try: pmat agent mcp-server" -ForegroundColor Cyan
      return
    }
  }
} catch {
  Write-Host "⚠️ Cargo installation failed, trying binary installation..." -ForegroundColor Yellow
}

# Fallback to binary installation
Write-Host "📦 Installing PMAT v$version from binary..." -ForegroundColor Green

# For now, provide installation instructions since we don't have pre-built Windows binaries yet
Write-Host "🦀 To install PMAT on Windows:" -ForegroundColor Yellow
Write-Host "1. Install Rust from https://rustup.rs/" -ForegroundColor White
Write-Host "2. Run: cargo install pmat" -ForegroundColor White
Write-Host "3. Or download from: https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v$version" -ForegroundColor White

# Create a batch file to make pmat available in PATH
$batchContent = @"
@echo off
echo PMAT requires Rust/Cargo to be installed.
echo Please install Rust from https://rustup.rs/
echo Then run: cargo install pmat
pause
"@

$installPath = Join-Path $env:ProgramFiles "PMAT"
if (!(Test-Path $installPath)) {
  New-Item -ItemType Directory -Path $installPath -Force
}

$batchFile = Join-Path $installPath "pmat.bat"
$batchContent | Out-File -FilePath $batchFile -Encoding ASCII

# Add to PATH
$envPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($envPath -notlike "*$installPath*") {
  [Environment]::SetEnvironmentVariable("Path", "$envPath;$installPath", "Machine")
}

Write-Host "📝 Created installation guide at: $batchFile" -ForegroundColor Green