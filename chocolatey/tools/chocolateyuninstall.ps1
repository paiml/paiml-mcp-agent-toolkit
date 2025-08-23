$ErrorActionPreference = 'Stop'

$packageName = 'pmat'

Write-Host "🗑️ Uninstalling PMAT - Pragmatic AI MCP Agent Toolkit" -ForegroundColor Yellow

# Try to uninstall via cargo first
try {
  $cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
  if ($cargoPath) {
    Write-Host "🦀 Attempting to uninstall via Cargo..." -ForegroundColor Yellow
    & cargo uninstall pmat
    
    if ($LASTEXITCODE -eq 0) {
      Write-Host "✅ PMAT uninstalled successfully via Cargo!" -ForegroundColor Green
    } else {
      Write-Host "⚠️ Cargo uninstall completed with warnings" -ForegroundColor Yellow
    }
  }
} catch {
  Write-Host "ℹ️ Cargo uninstall not applicable" -ForegroundColor Gray
}

# Try to uninstall npm package
try {
  $npmPath = Get-Command npm -ErrorAction SilentlyContinue
  if ($npmPath) {
    Write-Host "📦 Attempting to uninstall npm package..." -ForegroundColor Yellow
    & npm uninstall -g pmat-agent
    
    if ($LASTEXITCODE -eq 0) {
      Write-Host "✅ npm package uninstalled successfully!" -ForegroundColor Green
    }
  }
} catch {
  Write-Host "ℹ️ npm uninstall not applicable" -ForegroundColor Gray
}

# Clean up installation directory
$installPath = Join-Path $env:ProgramFiles "PMAT"
if (Test-Path $installPath) {
  Write-Host "🧹 Cleaning up installation directory..." -ForegroundColor Yellow
  try {
    Remove-Item -Path $installPath -Recurse -Force
    Write-Host "✅ Installation directory cleaned up" -ForegroundColor Green
  } catch {
    Write-Host "⚠️ Could not remove all files from installation directory" -ForegroundColor Yellow
  }
}

# Remove from PATH
$envPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($envPath -like "*$installPath*") {
  $newPath = $envPath.Replace(";$installPath", "").Replace("$installPath;", "").Replace("$installPath", "")
  [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
  Write-Host "✅ Removed PMAT from system PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎯 PMAT uninstallation completed!" -ForegroundColor Green
Write-Host "Thank you for using PMAT - Pragmatic AI MCP Agent Toolkit" -ForegroundColor Cyan