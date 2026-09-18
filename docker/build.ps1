# Build custom Antigravity Manager Docker image
# Usage:
#   .\docker\build.ps1
#   .\docker\build.ps1 -Tag "antigravity-manager:4.18.0"
#   .\docker\build.ps1 -UseMirror   # Regional mirror acceleration
#   .\docker\build.ps1 -Push -Registry "yourname/antigravity-manager"

param(
    [string]$Tag = "antigravity-manager:local",
    [string]$VersionTag = "",
    [ValidateSet("auto", "true", "false")]
    [string]$Mirror = "auto",
    [switch]$UseMirror,
    [switch]$Push,
    [string]$Registry = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host @"
[Error] docker command not found.

Please install and launch Docker Desktop:
  winget install -e --id Docker.DockerDesktop

Restart your machine if needed, open Docker Desktop, and rerun this script.
"@ -ForegroundColor Red
    exit 1
}

if ($UseMirror) { $Mirror = "true" }

$Version = (Get-Content "package.json" -Raw | ConvertFrom-Json).version
if (-not $VersionTag) {
    $VersionTag = "antigravity-manager:$Version"
}

Write-Host "==> Project Root:   $Root" -ForegroundColor Cyan
Write-Host "==> Build Tag:      $Tag" -ForegroundColor Cyan
Write-Host "==> Version Tag:    $VersionTag" -ForegroundColor Cyan
Write-Host "==> Mirror Mode:    $Mirror" -ForegroundColor Cyan
Write-Host ""

$buildArgs = @(
    "build",
    "-f", "docker/Dockerfile",
    "--build-arg", "USE_MIRROR=$Mirror",
    "-t", $Tag,
    "-t", $VersionTag,
    "."
)

Write-Host "==> Executing: docker $($buildArgs -join ' ')" -ForegroundColor Yellow
docker @buildArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host "[Error] Docker image build failed" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "==> Build successful" -ForegroundColor Green
docker images $Tag.Split(":")[0]

if ($Push) {
    if (-not $Registry) {
        Write-Host "[Error] -Push requires -Registry, e.g. yourname/antigravity-manager" -ForegroundColor Red
        exit 1
    }
    $remoteLatest = "${Registry}:latest"
    $remoteVersion = "${Registry}:$Version"
    docker tag $Tag $remoteLatest
    docker tag $VersionTag $remoteVersion
    docker push $remoteLatest
    docker push $remoteVersion
    Write-Host "==> Pushed: $remoteLatest , $remoteVersion" -ForegroundColor Green
}

Write-Host @"

Startup Example:

  docker run -d ``
    --name antigravity-manager ``
    -p 8045:8045 ``
    -e API_KEY=your-api-key ``
    -e WEB_PASSWORD=your-login-password ``
    -v `${HOME}/.antigravity_tools:/root/.antigravity_tools ``
    $Tag

Or using compose:

  docker compose -f docker/docker-compose.yml -f docker/docker-compose.fork.yml up -d --build

"@
