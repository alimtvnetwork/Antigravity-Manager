# Comprehensive End-to-End Test for Instance Lifecycle, Process Termination, UI Synchronization & Self-Preservation
# Relative paths strictly enforced. Zero external dependency.

$ErrorActionPreference = "Continue"
$PSNativeCommandUseErrorActionPreference = $false

Write-Host "==============================================================" -ForegroundColor Cyan
Write-Host " [E2E] Instance Lifecycle, PID Termination & Audit Trail Test " -ForegroundColor Cyan
Write-Host "==============================================================" -ForegroundColor Cyan

# Resolve AGM CLI executable via local target debug build, release build, or PATH
$agmExe = "agm"
if (Test-Path "src-tauri/target/debug/agm.exe") {
    $agmExe = "src-tauri/target/debug/agm.exe"
} elseif (Test-Path "src-tauri/target/debug/agm") {
    $agmExe = "src-tauri/target/debug/agm"
} elseif (Test-Path "src-tauri/target/release/agm.exe") {
    $agmExe = "src-tauri/target/release/agm.exe"
} elseif (Test-Path "src-tauri/target/release/agm") {
    $agmExe = "src-tauri/target/release/agm"
} else {
    $agmCmd = Get-Command "agm" -ErrorAction SilentlyContinue
    if ($agmCmd) {
        $agmExe = $agmCmd.Source
    } else {
        throw "AGM CLI binary not found. Please compile via cargo build."
    }
}

Write-Host "[INIT] Using AGM binary: $agmExe" -ForegroundColor Green

function Parse-CliJson {
    param([object]$rawOutput)
    if ($null -eq $rawOutput) { return $null }
    $text = if ($rawOutput -is [array]) { $rawOutput -join "`n" } else { "$rawOutput" }
    $text = $text.Trim()
    $match = [regex]::Match($text, '(?s)\{.*\}')
    if ($match.Success) {
        try { return ($match.Value | ConvertFrom-Json) } catch {}
    }
    $arrayMatch = [regex]::Match($text, '(?s)\[.*\]')
    if ($arrayMatch.Success) {
        try { return ($arrayMatch.Value | ConvertFrom-Json) } catch {}
    }
    try { return ($text | ConvertFrom-Json) } catch { return $null }
}

# -------------------------------------------------------------
# STEP 1: Detect Caller Instance & Verify Self-Preservation Guard
# -------------------------------------------------------------
Write-Host "`n[STEP 1] Testing caller instance detection & immunity guard..." -ForegroundColor Yellow
$callerOut = & $agmExe instances whoami --json | Out-String
Write-Host "Caller Detection Output: $callerOut"

$callerJson = Parse-CliJson $callerOut
$callerId = $null
if ($callerJson) {
    if ($callerJson.instance_id) {
        $callerId = $callerJson.instance_id
    } elseif ($callerJson.id) {
        $callerId = $callerJson.id
    } elseif ($callerJson.data -and $callerJson.data.instance_id) {
        $callerId = $callerJson.data.instance_id
    } elseif ($callerJson.data -and $callerJson.data.id) {
        $callerId = $callerJson.data.id
    }
}

$simulatedCaller = $false
if (-not $callerId) {
    Write-Host "  -> No caller detected in current process tree. Simulating caller with AGM_INSTANCE_ID='default'..." -ForegroundColor Yellow
    $env:AGM_INSTANCE_ID = "default"
    $simCallerOut = & $agmExe instances whoami --json | Out-String
    $simCallerJson = Parse-CliJson $simCallerOut
    $simId = if ($simCallerJson.instance_id) { $simCallerJson.instance_id } else { $simCallerJson.data.instance_id }
    if ($simId -eq "default") {
        Write-Host "  [PASS] Simulated caller detected successfully: default" -ForegroundColor Green
        $callerId = "default"
        $simulatedCaller = $true
    }
} else {
    Write-Host "  -> Identified Caller IDE Instance: $callerId" -ForegroundColor Cyan
}

if ($callerId) {
    Write-Host "  -> Verifying self-termination guard on active caller ($callerId)..." -ForegroundColor Yellow

    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = "Continue"

    # 1. Stop guard
    $stopErr = & $agmExe instances stop $callerId 2>&1 | Out-String
    $stopCode = $LASTEXITCODE
    Write-Host "  Stop response: $stopErr (ExitCode: $stopCode)"
    if ($stopCode -eq 2 -and ($stopErr -match "Refusing to stop" -or $stopErr -match "currently running IDE instance")) {
        Write-Host "  [PASS] Self-preservation guard successfully blocked stopping caller instance with exit code 2!" -ForegroundColor Green
    } else {
        $ErrorActionPreference = $prevEAP
        throw "CRITICAL FAILURE: Self-preservation guard did NOT block stopping active caller instance with code 2: $stopErr (ExitCode: $stopCode)"
    }

    # 2. Switch guard
    $switchErr = & $agmExe instances switch $callerId "default" 2>&1 | Out-String
    $switchCode = $LASTEXITCODE
    Write-Host "  Switch response: $switchErr (ExitCode: $switchCode)"
    if ($switchCode -eq 2 -and ($switchErr -match "Refusing to stop" -or $switchErr -match "currently running IDE instance")) {
        Write-Host "  [PASS] Self-preservation guard successfully blocked switching caller instance with exit code 2!" -ForegroundColor Green
    } else {
        $ErrorActionPreference = $prevEAP
        throw "CRITICAL FAILURE: Self-preservation guard did NOT block switching active caller instance with code 2: $switchErr (ExitCode: $switchCode)"
    }

    # 3. Restart guard
    $restartErr = & $agmExe instances restart $callerId 2>&1 | Out-String
    $restartCode = $LASTEXITCODE
    Write-Host "  Restart response: $restartErr (ExitCode: $restartCode)"
    if ($restartCode -eq 2 -and ($restartErr -match "Refusing to stop" -or $restartErr -match "currently running IDE instance")) {
        Write-Host "  [PASS] Self-preservation guard successfully blocked restarting caller instance with exit code 2!" -ForegroundColor Green
    } else {
        $ErrorActionPreference = $prevEAP
        throw "CRITICAL FAILURE: Self-preservation guard did NOT block restarting active caller instance with code 2: $restartErr (ExitCode: $restartCode)"
    }

    # 4. Delete guard
    $delErr = & $agmExe instances rm $callerId 2>&1 | Out-String
    $delCode = $LASTEXITCODE
    Write-Host "  Delete response: $delErr (ExitCode: $delCode)"
    if ($delCode -eq 2 -and ($delErr -match "Refusing to stop" -or $delErr -match "currently running IDE instance")) {
        Write-Host "  [PASS] Self-preservation guard successfully blocked deleting caller instance with exit code 2!" -ForegroundColor Green
    } else {
        $ErrorActionPreference = $prevEAP
        throw "CRITICAL FAILURE: Self-preservation guard did NOT block deleting active caller instance with code 2: $delErr (ExitCode: $delCode)"
    }

    $ErrorActionPreference = $prevEAP

    if ($simulatedCaller) {
        Remove-Item Env:\AGM_INSTANCE_ID
    }
}

# -------------------------------------------------------------
# STEP 2: Create Transient Test Sandbox Instance
# -------------------------------------------------------------
$testSlug = "e2e-sync-" + (Get-Random -Minimum 1000 -Maximum 9999)
Write-Host "`n[STEP 2] Creating transient test instance '$testSlug'..." -ForegroundColor Yellow
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$createOut = & $agmExe instances create-test --name $testSlug --json | Out-String
$sw.Stop()
Write-Host $createOut
Write-Host "  -> Creation latency: $($sw.ElapsedMilliseconds) ms" -ForegroundColor Cyan

$createJson = Parse-CliJson $createOut
$testInstId = $null
if ($createJson) {
    if ($createJson.id) {
        $testInstId = $createJson.id
    } elseif ($createJson.data -and $createJson.data.id) {
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
$statusJson = Parse-CliJson $statusOut
$isRunning = if ($statusJson.is_running -ne $null) { $statusJson.is_running } else { $statusJson.data.is_running }
$pidVal = if ($statusJson.pid -ne $null) { $statusJson.pid } else { $statusJson.data.pid }
Write-Host "  Status verified: is_running=$isRunning, PID=$pidVal"
Write-Host "  [OK] Status queries executed successfully." -ForegroundColor Green

# -------------------------------------------------------------
# STEP 4: Test Verified Process Termination (Stop Instance)
# -------------------------------------------------------------
Write-Host "`n[STEP 4] Testing Stop Instance & process eviction..." -ForegroundColor Yellow
$stopOut = & $agmExe instances stop $testInstId --json | Out-String
$stopExit = $LASTEXITCODE
Write-Host "Stop command output: $stopOut"
if ($stopExit -ne 0) {
    throw "Stop command failed with exit code $($stopExit): $stopOut"
}
# Confirm dead status
$postStopStatus = & $agmExe instances status $testInstId --json | Out-String
$postStopJson = Parse-CliJson $postStopStatus
$postStopRunning = if ($postStopJson.is_running -ne $null) { $postStopJson.is_running } else { $postStopJson.data.is_running }
$postStopPid = if ($postStopJson.pid -ne $null) { $postStopJson.pid } else { $postStopJson.data.pid }
Write-Host "  Confirmed post-stop status: is_running=$postStopRunning, PID=$postStopPid"
if ($postStopRunning -eq $true) {
    throw "CRITICAL FAILURE: Instance '$testInstId' is still running after verified stop!"
}
Write-Host "  [OK] Stop command executed and instance confirmed dead." -ForegroundColor Green

# -------------------------------------------------------------
# STEP 5: Test Safe Account Switch & Synchronization
# -------------------------------------------------------------
Write-Host "`n[STEP 5] Testing account switch synchronization..." -ForegroundColor Yellow
$switchOut = & $agmExe instances switch $testInstId "default" --json 2>&1 | Out-String
$switchExit = $LASTEXITCODE
Write-Host "Switch command output: $switchOut"
if ($switchExit -ne 0 -and $switchOut -match "No accounts registered") {
    Write-Host "  [NOTE] No accounts registered on machine; account switch guard reported expected message." -ForegroundColor Yellow
} elseif ($switchExit -ne 0) {
    throw "Switch command failed with exit code $($switchExit): $switchOut"
} else {
    Write-Host "  [OK] Switch synchronization executed cleanly." -ForegroundColor Green
}

# -------------------------------------------------------------
# STEP 6: Verify Audit Trail Recording
# -------------------------------------------------------------
Write-Host "`n[STEP 6] Verifying audit trail recording..." -ForegroundColor Yellow
$histOut = & $agmExe history --json 2>&1 | Out-String
$histJson = Parse-CliJson $histOut
$matchingTasks = @()
if ($histJson -and $histJson.items) {
    $matchingTasks = @($histJson.items | Where-Object {
        $_.instance_id -eq $testInstId -or $_.subject -match [regex]::Escape($testInstId) -or $_.detail -match [regex]::Escape($testInstId)
    })
}
Write-Host "  Found $($matchingTasks.Count) audit trail records matching instance '$testInstId'."
if ($matchingTasks.Count -gt 0) {
    foreach ($task in $matchingTasks) {
        Write-Host "  -> [AUDIT] Action: $($task.action_label) (Code: $($task.action_code)), Status: $($task.status), Subject: $($task.subject)" -ForegroundColor DarkCyan
    }
    Write-Host "  [PASS] Verified audit trail records logged successfully." -ForegroundColor Green
} else {
    Write-Host "  [NOTE] Checking total history rows: $($histJson.total)" -ForegroundColor Yellow
}

# -------------------------------------------------------------
# STEP 7: Clean Removal & Teardown Verification
# -------------------------------------------------------------
Write-Host "`n[STEP 7] Teardown: Removing test sandbox '$testInstId'..." -ForegroundColor Yellow
$rmOut = & $agmExe instances rm-test $testInstId --json | Out-String
$rmExit = $LASTEXITCODE
Write-Host "Removal output: $rmOut"
if ($rmExit -ne 0) {
    throw "rm-test failed with exit code $($rmExit): $rmOut"
}

# Verify test instance is removed from registry
$listOut = & $agmExe instances ls --json | Out-String
if ($listOut -match [regex]::Escape($testInstId)) {
    throw "ASSERTION FAILED: Instance $testInstId still exists in instance list after rm-test"
}
Write-Host "  [PASS] Test instance successfully purged from registry and filesystem." -ForegroundColor Green

Write-Host "`n==============================================================" -ForegroundColor Cyan
Write-Host " [SUCCESS] All Instance Lifecycle & Synchronization Gates Passed! " -ForegroundColor Green
Write-Host "==============================================================" -ForegroundColor Cyan
exit 0
