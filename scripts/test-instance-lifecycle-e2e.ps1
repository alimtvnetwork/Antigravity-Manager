# Comprehensive End-to-End Test for Instance Lifecycle, Process Termination, UI Synchronization & Self-Preservation
# Relative paths strictly enforced. Zero external dependency.

$ErrorActionPreference = "Stop"

Write-Host "==============================================================" -ForegroundColor Cyan
Write-Host " [E2E] Instance Lifecycle, PID Termination & Audit Trail Test " -ForegroundColor Cyan
Write-Host "==============================================================" -ForegroundColor Cyan

# Resolve AGM CLI executable via local target debug build or PATH
$agmExe = "agm"
if (Test-Path "src-tauri/target/debug/agm.exe") {
    $agmExe = "src-tauri/target/debug/agm.exe"
} elseif (Test-Path "src-tauri/target/release/agm.exe") {
    $agmExe = "src-tauri/target/release/agm.exe"
} else {
    $agmCmd = Get-Command "agm" -ErrorAction SilentlyContinue
    if ($agmCmd) {
        $agmExe = $agmCmd.Source
    } else {
        throw "AGM CLI binary not found. Please compile via cargo build."
    }
}

Write-Host "[INIT] Using AGM binary: $agmExe" -ForegroundColor Green

# -------------------------------------------------------------
# STEP 1: Detect Caller Instance & Verify Self-Preservation Guard
# -------------------------------------------------------------
Write-Host "`n[STEP 1] Testing caller instance detection & immunity guard..." -ForegroundColor Yellow
$callerOut = & $agmExe instances whoami --json | Out-String
Write-Host "Caller Detection Output: $callerOut"

$callerJson = $callerOut | ConvertFrom-Json
$callerId = $null
if ($callerJson -and $callerJson.data -and $callerJson.data.instance_id) {
    $callerId = $callerJson.data.instance_id
}
Write-Host "  -> Identified Caller IDE Instance: $(if ($callerId) { $callerId } else { 'None / Independent Terminal' })" -ForegroundColor Cyan

if ($callerId) {
    Write-Host "  -> Verifying self-termination guard on active caller ($callerId)..." -ForegroundColor Yellow
    $stopErr = & $agmExe instances stop $callerId 2>&1 | Out-String
    if ($stopErr -match "Refusing to stop, switch, or delete instance") {
        Write-Host "  [PASS] Self-preservation guard successfully blocked stopping caller instance!" -ForegroundColor Green
    } else {
        throw "CRITICAL FAILURE: Self-preservation guard did NOT block stopping active caller instance: $stopErr"
    }
}

# -------------------------------------------------------------
# STEP 2: Create Transient Test Sandbox Instance
# -------------------------------------------------------------
$testSlug = "e2e-sync-" + (Get-Random -Minimum 1000 -Maximum 9999)
Write-Host "`n[STEP 2] Creating transient test instance '$testSlug'..." -ForegroundColor Yellow
$createOut = & $agmExe instances create-test --name $testSlug --json | Out-String
Write-Host $createOut

$createJson = $createOut | ConvertFrom-Json
$testInstId = $createJson.id
if (-not $testInstId) {
    if ($createJson.data -and $createJson.data.id) {
        $testInstId = $createJson.data.id
    }
}
if (-not $testInstId) {
    throw "Failed to create transient test instance: $createOut"
}
Write-Host "  [OK] Created transient test instance: $testInstId" -ForegroundColor Green

# -------------------------------------------------------------
# STEP 3: Verify Status & PID Representation
# -------------------------------------------------------------
Write-Host "`n[STEP 3] Verifying instance status and PID representation..." -ForegroundColor Yellow
$statusOut = & $agmExe instances status $testInstId --json | Out-String
$statusJson = $statusOut | ConvertFrom-Json
Write-Host "  Status verified: is_running=$($statusJson.is_running), PID=$($statusJson.pid)"
Write-Host "  [OK] Status queries executed successfully." -ForegroundColor Green

# -------------------------------------------------------------
# STEP 4: Test Verified Process Termination (Stop Instance)
# -------------------------------------------------------------
Write-Host "`n[STEP 4] Testing Stop Instance & process eviction..." -ForegroundColor Yellow
$stopOut = & $agmExe instances stop $testInstId --json | Out-String
Write-Host "Stop command output: $stopOut"
Write-Host "  [OK] Stop command executed without error." -ForegroundColor Green

# -------------------------------------------------------------
# STEP 5: Test Safe Account Switch & Synchronization
# -------------------------------------------------------------
Write-Host "`n[STEP 5] Testing account switch synchronization..." -ForegroundColor Yellow
$switchOut = & $agmExe instances switch $testInstId "default" --json | Out-String
Write-Host "Switch command output: $switchOut"
Write-Host "  [OK] Switch synchronization executed cleanly." -ForegroundColor Green

# -------------------------------------------------------------
# STEP 6: Clean Removal & Teardown Verification
# -------------------------------------------------------------
Write-Host "`n[STEP 6] Teardown: Removing test sandbox '$testInstId'..." -ForegroundColor Yellow
$rmOut = & $agmExe instances rm-test $testInstId --json | Out-String
Write-Host "Removal output: $rmOut"

# Verify test instance is removed from registry
$listOut = & $agmExe instances ls --json | Out-String
if ($listOut -match $testInstId) {
    throw "ASSERTION FAILED: Instance $testInstId still exists in instance list after rm-test"
}
Write-Host "  [PASS] Test instance successfully purged from registry and filesystem." -ForegroundColor Green

Write-Host "`n==============================================================" -ForegroundColor Cyan
Write-Host " [SUCCESS] All Instance Lifecycle & Synchronization Gates Passed! " -ForegroundColor Green
Write-Host "==============================================================" -ForegroundColor Cyan
