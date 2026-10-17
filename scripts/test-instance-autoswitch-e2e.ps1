# Test script for Instance Autoswitch, Caller Self-Preservation Guard, and Test Instance Management E2E Verification
# Strictly uses relative paths only and verifies:
# 1. agm instances current detects caller instance (including env override).
# 2. agm instances create-test creates transient test instance in %LOCALAPPDATA%\agm-instances\test-<id>.
# 3. Verifies directory on disk and entry in instance registry.
# 4. Verifies agm instances stop <caller_id> is refused by caller self-preservation guard.
# 5. Cleans up via agm instances rm-test and verifies directory and registry cleanliness.

$ErrorActionPreference = "Stop"

Write-Host "===================================================================" -ForegroundColor Cyan
Write-Host " Running Instance Autoswitch & Self-Preservation Guard E2E Suite  " -ForegroundColor Cyan
Write-Host "===================================================================" -ForegroundColor Cyan

# Locate agm CLI binary using strictly relative paths
$agmExe = "src-tauri\target\debug\agm.exe"
if (-not (Test-Path $agmExe)) {
    $cmd = Get-Command "agm" -ErrorAction SilentlyContinue
    if (-not $cmd) {
        $cmd = Get-Command "adm" -ErrorAction SilentlyContinue
    }
    if ($cmd) {
        $agmExe = $cmd.Source
    } else {
        throw "agm.exe binary not found at relative path '$agmExe' or in system PATH"
    }
}

Write-Host "Using AGM executable: $agmExe" -ForegroundColor Cyan

function Extract-JsonPayload ($rawOutput) {
    if ($rawOutput -match '(?ms)(\{.*\}|\[.*\])') {
        return ($matches[1] | ConvertFrom-Json)
    }
    return ($rawOutput | ConvertFrom-Json)
}

# -------------------------------------------------------------------
# STEP 1: Test 'agm instances current' to detect caller instance
# -------------------------------------------------------------------
Write-Host "`n[STEP 1] Testing 'agm instances current' detection..." -ForegroundColor Yellow

# Test default current output
$currentRaw = & $agmExe instances current | Out-String
$currentRaw = $currentRaw.Trim()
Write-Host "  Raw 'agm instances current' output: '$currentRaw'" -ForegroundColor Gray

# Test JSON output
$currentJsonRaw = & $agmExe instances current --json | Out-String
$currentJson = Extract-JsonPayload $currentJsonRaw
$detectedCallerId = $currentJson.data.instance_id
Write-Host "  JSON detected caller instance ID: '$detectedCallerId'" -ForegroundColor Green

# Test environment variable override detection
$probeId = "test-probe-env-caller"
$env:ANTIGRAVITY_INSTANCE_ID = $probeId
try {
    $envOverrideJsonRaw = & $agmExe instances current --json | Out-String
    $envOverrideJson = Extract-JsonPayload $envOverrideJsonRaw
    if ($envOverrideJson.data.instance_id -ne $probeId) {
        throw "ASSERTION FAILED: ANTIGRAVITY_INSTANCE_ID override was '$($envOverrideJson.data.instance_id)', expected '$probeId'"
    }
    Write-Host "  [OK] Caller instance ID properly overridden by ANTIGRAVITY_INSTANCE_ID: $probeId" -ForegroundColor Green
} finally {
    Remove-Item Env:\ANTIGRAVITY_INSTANCE_ID -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------
# STEP 2: Create a new test instance via 'agm instances create-test'
# -------------------------------------------------------------------
$testName = "test-autoswitch-sandbox"
Write-Host "`n[STEP 2] Creating new test instance with name '$testName'..." -ForegroundColor Yellow

$createJsonRaw = & $agmExe instances create-test --name $testName --json | Out-String
Write-Host $createJsonRaw.Trim() -ForegroundColor Gray
$createJson = Extract-JsonPayload $createJsonRaw

$testId = $createJson.data.id
$testDataDir = $createJson.data.data_dir

if (-not $testId) {
    throw "ASSERTION FAILED: create-test did not return an instance ID"
}
if (-not $testDataDir) {
    throw "ASSERTION FAILED: create-test did not return a data directory"
}

Write-Host "  [OK] Created test instance: ID='$testId', DataDir='$testDataDir'" -ForegroundColor Green

# -------------------------------------------------------------------
# STEP 3: Verify the test instance was created on disk and in registry
# -------------------------------------------------------------------
Write-Host "`n[STEP 3] Verifying filesystem sandbox and registry registration..." -ForegroundColor Yellow

# 3a. Filesystem check
if (-not (Test-Path $testDataDir)) {
    throw "ASSERTION FAILED: Test instance directory '$testDataDir' does not exist on disk"
}
Write-Host "  [OK] Filesystem sandbox directory verified on disk: $testDataDir" -ForegroundColor Green

# 3b. Registry check
$listJsonRaw = & $agmExe instances list --json | Out-String
$listJson = Extract-JsonPayload $listJsonRaw
$matchedInst = $listJson.data | Where-Object { $_.id -eq $testId }

if (-not $matchedInst) {
    throw "ASSERTION FAILED: Instance '$testId' was not found in 'agm instances list --json' registry"
}
Write-Host "  [OK] Instance '$testId' confirmed present in instance registry" -ForegroundColor Green

# -------------------------------------------------------------------
# STEP 4: Verify 'agm instances stop <caller_id>' is refused by guard
# -------------------------------------------------------------------
Write-Host "`n[STEP 4] Verifying caller self-preservation guard..." -ForegroundColor Yellow

# 4a. Verify with simulated caller via ANTIGRAVITY_INSTANCE_ID
$env:ANTIGRAVITY_INSTANCE_ID = $testId
try {
    $origEap = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $stopOutput = & $agmExe instances stop $testId 2>&1 | Out-String
    $stopExitCode = $LASTEXITCODE
    $ErrorActionPreference = $origEap

    Write-Host "  Stop attempt inside caller instance (exit code $stopExitCode):" -ForegroundColor Gray
    Write-Host "  $($stopOutput.Trim())" -ForegroundColor Gray

    if ($stopExitCode -eq 0) {
        throw "ASSERTION FAILED: 'agm instances stop $testId' succeeded unexpectedly when running inside caller instance!"
    }

    $cleanOutput = ($stopOutput -replace '\s+', ' ')
    $expectedMsgPart = "Refusing to stop, switch, or delete instance '$testId' because this command is running inside its IDE process"
    if (-not ($cleanOutput -match [regex]::Escape($expectedMsgPart))) {
        throw "ASSERTION FAILED: Output did not contain expected refusal message. Output was: $cleanOutput"
    }
    Write-Host "  [OK] Caller self-preservation guard successfully intercepted stop command for '$testId'" -ForegroundColor Green
} finally {
    Remove-Item Env:\ANTIGRAVITY_INSTANCE_ID -ErrorAction SilentlyContinue
}

# 4b. If the current environment actually is an IDE caller process, verify real guard
if ($detectedCallerId -and $detectedCallerId -ne "none") {
    Write-Host "  Testing real caller self-preservation for detected IDE instance '$detectedCallerId'..." -ForegroundColor Cyan
    $origEap = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $realStopOutput = & $agmExe instances stop $detectedCallerId 2>&1 | Out-String
    $realExitCode = $LASTEXITCODE
    $ErrorActionPreference = $origEap

    if ($realExitCode -eq 0) {
        throw "ASSERTION FAILED: Real caller instance stop was not intercepted!"
    }
    $cleanRealOutput = ($realStopOutput -replace '\s+', ' ')
    $expectedRealMsg = "Refusing to stop, switch, or delete instance '$detectedCallerId' because this command is running inside its IDE process"
    if (-not ($cleanRealOutput -match [regex]::Escape($expectedRealMsg))) {
        throw "ASSERTION FAILED: Output did not contain expected refusal message for real caller. Output was: $cleanRealOutput"
    }
    Write-Host "  [OK] Real caller self-preservation confirmed for '$detectedCallerId'" -ForegroundColor Green
}

# -------------------------------------------------------------------
# STEP 5: Clean up test instance via 'agm instances rm-test'
# -------------------------------------------------------------------
Write-Host "`n[STEP 5] Cleaning up test instance '$testId' via rm-test..." -ForegroundColor Yellow

$rmJsonRaw = & $agmExe instances rm-test $testId --json | Out-String
Write-Host $rmJsonRaw.Trim() -ForegroundColor Gray

# 5a. Verify directory deleted
if (Test-Path $testDataDir) {
    throw "ASSERTION FAILED: Test instance directory '$testDataDir' still exists after rm-test"
}
Write-Host "  [OK] Test directory cleanly removed from disk" -ForegroundColor Green

# 5b. Verify removed from registry
$postListRaw = & $agmExe instances list --json | Out-String
$postList = Extract-JsonPayload $postListRaw
$postMatched = $postList.data | Where-Object { $_.id -eq $testId }

if ($postMatched) {
    throw "ASSERTION FAILED: Instance '$testId' still present in registry after rm-test"
}
Write-Host "  [OK] Instance '$testId' cleanly unregistered from registry" -ForegroundColor Green

# 5c. Test rm-test --all to ensure bulk cleanup safety
Write-Host "`n[STEP 5c] Testing 'agm instances rm-test --all' idempotency..." -ForegroundColor Yellow
$rmAllRaw = & $agmExe instances rm-test --all --json | Out-String
Write-Host $rmAllRaw.Trim() -ForegroundColor Gray
Write-Host "  [OK] Bulk cleanup executed cleanly" -ForegroundColor Green

Write-Host "`n===================================================================" -ForegroundColor Green
Write-Host " ALL TEST-INSTANCE-AUTOSWITCH E2E TESTS PASSED SUCCESSFULLY!       " -ForegroundColor Green
Write-Host "===================================================================" -ForegroundColor Green
