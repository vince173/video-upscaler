$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

Write-Host ''
Write-Host '=== Installing Microsoft Edge WebDriver ==='
Write-Host ''

# Get latest version
Write-Host '[1/4] Getting latest version...'
Invoke-WebRequest -Uri 'https://msedgedriver.azureedge.net/LATEST_RELEASE' -OutFile 'latest_version.txt'
$version = Get-Content 'latest_version.txt'
Write-Host "  Latest version: $version"

# Detect architecture
Write-Host '[2/4] Detecting system architecture...'
$arch = if ([Environment]::Is64BitOperatingSystem) { '64' } else { '32' }
Write-Host "  Architecture: $arch-bit"

# Download
Write-Host '[3/4] Downloading Edge WebDriver...'
$url = "https://msedgedriver.azureedge.net/$version/edgedriver_win${arch}.zip"
Invoke-WebRequest -Uri $url -OutFile 'msedgedriver.zip'
Write-Host "  Download complete"

# Extract to cargo bin
Write-Host '[4/4] Installing to .cargo/bin...'
$binPath = Join-Path $env:USERPROFILE '.cargo\bin'
New-Item -ItemType Directory -Force -Path $binPath | Out-Null
Expand-Archive -Path 'msedgedriver.zip' -DestinationPath $binPath -Force

# Cleanup
Remove-Item 'latest_version.txt', 'msedgedriver.zip' -ErrorAction SilentlyContinue

# Verify
$msedgedriver = Join-Path $binPath 'msedgedriver.exe'
if (Test-Path $msedgedriver) {
    Write-Host ''
    Write-Host 'SUCCESS: Edge WebDriver installed!' -ForegroundColor Green
    Write-Host "Location: $msedgedriver"
    Write-Host ''
    Write-Host 'You can now run tests: npm run test'
} else {
    Write-Host ''
    Write-Host 'ERROR: Installation failed' -ForegroundColor Red
    exit 1
}
