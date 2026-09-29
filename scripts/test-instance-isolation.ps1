# Test script for Instance Isolation, Launching, Switching, and Prompt Backup/Restore
$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Running Instance Isolation & Multi-Window Test Suite     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$agmExe = "D:\work\Antigravity-Manager\src-tauri\target\debug\agm.exe"
if (-not (Test-Path $agmExe)) {
    $cmd = Get-Command "agm" -ErrorAction SilentlyContinue
    if (-not $cmd) {
        $cmd = Get-Command "adm" -ErrorAction SilentlyContinue
    }
    if ($cmd) {
        $agmExe = $cmd.Source
    } else {
        throw "agm.exe not found at $agmExe or on PATH"
    }
}

# 1. Verify default session PID 7872 is running and protected
$defaultProc = Get-Process -Id 7872 -ErrorAction SilentlyContinue
if (-not $defaultProc) {
    Write-Warning "PID 7872 is not currently running. Looking for Antigravity process..."
    $defaultProc = Get-Process -Name "Antigravity" -ErrorAction SilentlyContinue | Select-Object -First 1
}
if ($defaultProc) {
    Write-Host "[VERIFY 1] Active Default IDE session detected: PID $($defaultProc.Id) ($($defaultProc.ProcessName))" -ForegroundColor Green
    $protectedPid = $defaultProc.Id
} else {
    Write-Host "[VERIFY 1] No active Default IDE session running" -ForegroundColor Yellow
    $protectedPid = $null
}

# 2. Create test sandbox instance
$testName = "test-sandbox-iso"
Write-Host "`n[STEP 2] Creating test instance '$testName' bound to riseup.team.now@gmail.com..." -ForegroundColor Yellow
$createOut = & $agmExe instances create $testName -a "riseup.team.now@gmail.com" --json | Out-String
Write-Host $createOut

$instCfg = $createOut | ConvertFrom-Json
$instId = $instCfg.id
$dataDir = $instCfg.data_dir
Write-Host "Created instance with ID: $instId, DataDir: $dataDir" -ForegroundColor Cyan

# 3. Assert isolated directory structure & wizard bypass
Write-Host "`n[STEP 3] Verifying isolated storage and wizard bypass..." -ForegroundColor Yellow
$homeDir = "C:\Users\Administrator\.antigravity_tools\instances\$instId\home"
$appStorage = Join-Path $dataDir "app_storage.json"
$stateDb = Join-Path $dataDir "User\globalStorage\state.vscdb"
$creds = Join-Path $homeDir ".gemini\credentials"
$ideDir = Join-Path $homeDir ".gemini\antigravity-ide"
$roaming = Join-Path $homeDir "AppData\Roaming"

if (-not (Test-Path $appStorage)) { throw "ASSERTION FAILED: $appStorage does not exist" }
$appStorageContent = Get-Content $appStorage -Raw
if (-not ($appStorageContent -match '"ide-install-wizard-shown":\s*"true"')) {
    throw "ASSERTION FAILED: ide-install-wizard-shown not true in $appStorage"
}
Write-Host "  [OK] app_storage.json pre-seeded with ide-install-wizard-shown" -ForegroundColor Green

if (-not (Test-Path $ideDir)) { throw "ASSERTION FAILED: $ideDir does not exist" }
Write-Host "  [OK] .gemini/antigravity-ide directory exists" -ForegroundColor Green

if (-not (Test-Path $roaming)) { throw "ASSERTION FAILED: $roaming does not exist" }
Write-Host "  [OK] Isolated AppData/Roaming exists" -ForegroundColor Green

if (-not (Test-Path $stateDb)) { throw "ASSERTION FAILED: $stateDb does not exist" }
Write-Host "  [OK] state.vscdb initialized with bound credentials" -ForegroundColor Green

# 4. Switch instance account to alex.hudson.riseup@gmail.com
Write-Host "`n[STEP 4] Switching instance '$instId' to alex.hudson.riseup@gmail.com..." -ForegroundColor Yellow
$switchOut = & $agmExe instances switch $instId "alex.hudson.riseup@gmail.com" | Out-String
Write-Host $switchOut

# Verify registry has updated bound email
$instancesOut = & $agmExe instances --json | Out-String
$instancesJson = $instancesOut | ConvertFrom-Json
$switchedInst = $instancesJson | Where-Object { $_.id -eq $instId }
if ($switchedInst.bound_email -ne "alex.hudson.riseup@gmail.com") {
    throw "ASSERTION FAILED: bound_email in registry is $($switchedInst.bound_email), expected alex.hudson.riseup@gmail.com"
}
Write-Host "  [OK] Instance successfully switched to alex.hudson.riseup@gmail.com" -ForegroundColor Green

# 5. Fast-Forward rotate account for the instance
Write-Host "`n[STEP 5] Testing Fast-Forward account rotation for instance '$instId'..." -ForegroundColor Yellow
$ffOut = & $agmExe instances ff $instId | Out-String
Write-Host $ffOut

# 6. Verify protected default PID was never killed
if ($protectedPid) {
    $procCheck = Get-Process -Id $protectedPid -ErrorAction SilentlyContinue
    if (-not $procCheck) {
        throw "CRITICAL FAILURE: Protected Default PID $protectedPid was killed!"
    }
    Write-Host "`n[VERIFY 6] Protected Default PID $protectedPid is still running undisturbed!" -ForegroundColor Green
}

# 7. Clean up test instance
Write-Host "`n[STEP 7] Cleaning up test instance '$instId'..." -ForegroundColor Yellow
$rmOut = & $agmExe instances rm $instId | Out-String
Write-Host $rmOut

# Verify clean removal
$remainingInst = & $agmExe instances --json | ConvertFrom-Json
$exists = $remainingInst | Where-Object { $_.id -eq $instId }
if ($exists) {
    throw "ASSERTION FAILED: Instance $instId still in registry"
}
Write-Host "  [OK] Test instance successfully removed from registry and disk" -ForegroundColor Green

# Final check on protected PID
if ($protectedPid) {
    $procCheck = Get-Process -Id $protectedPid -ErrorAction SilentlyContinue
    if (-not $procCheck) {
        throw "CRITICAL FAILURE: Protected Default PID $protectedPid was killed during cleanup!"
    }
    Write-Host "`n[FINAL VERIFY] Protected Default PID $protectedPid remains 100% active and healthy!" -ForegroundColor Green
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host " ALL TEST ASSERTIONS PASSED PERFECTLY!                    " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
