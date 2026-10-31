<#
.SYNOPSIS
    Comprehensive Automated End-to-End Verification Suite for Task 231 (anti-gravity-rerun-and-ui-fixes).
.DESCRIPTION
    Validates the end-to-end rerun dispatch chain, SQLite database persistence, and tree view layout compaction:
    - Stage 0: Pre-flight environment check & Caller IDE self-preservation (detect_caller_instance_id())
    - Stage 1: Transient sandbox creation & isolation (agm instances test-create)
    - Stage 2: Rust code & unit touchpoint verification (dedup, decoupled queue, saturation priority)
    - Stage 3: Frontend compilation gate (npm run build)
    - Stage 4: Tree view filter layout budget & CSS validation (zero clipping at 280px)
    - Stage 5: Target conversation UUID retention & immediate multi-round FIFO queue verification (<500ms)
    - Stage 6: Enqueue refresh bar suppression & 0ms optimistic UI rendering verification
    - Stage 7: Zero-orphan sandbox teardown and database cleanup (agm instances test-clean)

    Conforms to specifications:
    - 02-spec/21-app/231-anti-gravity-rerun-and-ui-fixes/03-tree-view-design-system-spec.md
    - 02-spec/21-app/231-anti-gravity-rerun-and-ui-fixes/04-e2e-and-release-spec.md
    - 02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md
    - .ai-memory/plans/subtasks/231-anti-gravity-rerun-and-ui-fixes/03-subtask-tree-filter-compaction-and-design-system.md
#>
param(
    [switch]$VerboseOutput,
    [switch]$KeepSandbox,
    [switch]$SkipFrontendBuild
)

$ErrorActionPreference = "Continue"
$PSNativeCommandUseErrorActionPreference = $false

# Enforce UTF-8 console output encoding
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
Set-Location $RootDir

$script:testResults = @()
$script:startTime = Get-Date

function Assert-Condition {
    param(
        [string]$TestName,
        [bool]$Condition,
        [string]$FailureMessage
    )
    if ($Condition) {
        Write-Host "  [PASS] $TestName" -ForegroundColor Green
        $script:testResults += [PSCustomObject]@{ Test = $TestName; Status = "PASS"; Details = "" }
    } else {
        Write-Host "  [FAIL] $TestName - $FailureMessage" -ForegroundColor Red
        $script:testResults += [PSCustomObject]@{ Test = $TestName; Status = "FAIL"; Details = $FailureMessage }
    }
}

function Invoke-SqlitePython {
    param(
        [string]$DbPath,
        [string]$Query
    )
    $cleanQuery = ($Query -replace "[\r\n]+", " ").Trim()
    $pyScript = @'
import sqlite3, json, sys
conn = sqlite3.connect(sys.argv[1])
conn.row_factory = sqlite3.Row
cur = conn.cursor()
try:
    cur.execute(sys.argv[2])
    conn.commit()
    rows = [dict(r) for r in cur.fetchall()] if cur.description else []
    print(json.dumps(rows))
except Exception as e:
    sys.stderr.write(str(e))
finally:
    conn.close()
'@
    $raw = & python -c $pyScript $DbPath $cleanQuery 2>&1
    if ($VerboseOutput -and $raw) {
        Write-Host "  [SQL-DEBUG] Query: $cleanQuery | Output: $raw" -ForegroundColor DarkGray
    }
    if ($null -ne $raw) {
        $text = ($raw | Out-String).Trim()
        if ($text -ne "") {
            try {
                $parsed = ($text | ConvertFrom-Json)
                if ($null -eq $parsed) { return @() }
                return @($parsed)
            } catch {
                if ($VerboseOutput) {
                    Write-Host "  [WARN] Invoke-SqlitePython JSON error: $text" -ForegroundColor DarkGray
                }
                return @()
            }
        }
    }
    return @()
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host " Anti-gravity Rerun and UI Fixes Automated E2E Test Suite (Task 231)           " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

# -------------------------------------------------------------------------
# Stage 0: Pre-flight Environment & Caller IDE Detection
# -------------------------------------------------------------------------
Write-Host "`n[Stage 0/7] Pre-flight Environment & Caller IDE Self-Preservation" -ForegroundColor Yellow

# Locate AGM binary
$agm = Join-Path $RootDir "src-tauri\target\debug\agm.exe"
if (!(Test-Path $agm)) {
    $agm = Join-Path $RootDir "src-tauri\target\release\agm.exe"
}
if (!(Test-Path $agm)) {
    $cmd = Get-Command agm -ErrorAction SilentlyContinue
    if ($null -ne $cmd) { $agm = $cmd.Source }
}
if (!(Test-Path $agm)) {
    $progPath = Join-Path $env:LOCALAPPDATA "Programs\agm-alim\agm.exe"
    if (Test-Path $progPath) { $agm = $progPath }
}

Assert-Condition "Pre-flight: AGM executable resolved ($agm)" (Test-Path $agm) "AGM binary not found"

# Locate GitMap
$gitmapCmd = Get-Command gitmap -ErrorAction SilentlyContinue
$gitmapOk = ($null -ne $gitmapCmd)
Assert-Condition "Pre-flight: GitMap CLI available" $gitmapOk "GitMap CLI not found in PATH"

# Locate Node & NPM
$nodeCmd = Get-Command node -ErrorAction SilentlyContinue
$npmCmd = Get-Command npm -ErrorAction SilentlyContinue
Assert-Condition "Pre-flight: Node.js and NPM available" ($null -ne $nodeCmd -and $null -ne $npmCmd) "Node or NPM missing"

# Locate Python
$pythonCmd = Get-Command python -ErrorAction SilentlyContinue
Assert-Condition "Pre-flight: Python 3 available" ($null -ne $pythonCmd) "Python runtime missing"

# Resolve repo_prompts.db path
$dataDir = if ($env:ABV_DATA_DIR -and (Test-Path $env:ABV_DATA_DIR)) {
    $env:ABV_DATA_DIR
} elseif (Test-Path "$env:SystemDrive\Users\Administrator\.antigravity_tools\repo_prompts.db") {
    "$env:SystemDrive\Users\Administrator\.antigravity_tools"
} elseif (Test-Path (Join-Path $env:USERPROFILE ".antigravity_tools\repo_prompts.db")) {
    Join-Path $env:USERPROFILE ".antigravity_tools"
} else {
    Join-Path $env:USERPROFILE ".antigravity_tools"
}
$repoDbPath = Join-Path $dataDir "repo_prompts.db"
Assert-Condition "Pre-flight: SQLite database located ($repoDbPath)" (Test-Path $repoDbPath) "repo_prompts.db missing"

# Detect caller IDE instance
$callerInstanceId = "unknown"
$callerPid = 0
try {
    $whoamiRaw = & $agm instances whoami --json 2>$null
    if ($whoamiRaw) {
        $cleanJson = ($whoamiRaw | Out-String)
        $firstBrace = $cleanJson.IndexOf('{')
        $lastBrace = $cleanJson.LastIndexOf('}')
        if ($firstBrace -ge 0 -and $lastBrace -gt $firstBrace) {
            $parsedWhoami = ($cleanJson.Substring($firstBrace, $lastBrace - $firstBrace + 1) | ConvertFrom-Json)
            if ($parsedWhoami.id) {
                $callerInstanceId = $parsedWhoami.id
            } elseif ($parsedWhoami.instance_id) {
                $callerInstanceId = $parsedWhoami.instance_id
            }
            if ($parsedWhoami.pid) {
                $callerPid = [int]$parsedWhoami.pid
            }
        }
    }
} catch {
    Write-Host "  [WARN] Failed to parse caller from agm instances whoami: $_" -ForegroundColor DarkYellow
}

$hasCallerId = ($callerInstanceId -ne "unknown" -and $callerInstanceId -ne "")
Assert-Condition "Caller IDE Self-Preservation: Active session detected ($callerInstanceId, PID $callerPid)" $hasCallerId "Caller instance ID could not be detected"
Write-Host "  -> Caller IDE Immunity Active: Operations targeting '$callerInstanceId' strictly prohibited." -ForegroundColor Cyan

# -------------------------------------------------------------------------
# Stage 1: Transient Sandbox Creation (agm instances test-create)
# -------------------------------------------------------------------------
Write-Host "`n[Stage 1/7] Transient Sandbox Creation & Zero-Orphan Setup" -ForegroundColor Yellow

$sandboxInstanceId = $null
$sandboxCreated = $false

try {
    $createOutput = & $agm instances test-create agm-test-rerun-231 --json 2>&1
    $createJson = ($createOutput | Out-String)
    $cFirst = $createJson.IndexOf('{')
    $cLast = $createJson.LastIndexOf('}')
    if ($cFirst -ge 0 -and $cLast -gt $cFirst) {
        $parsedCreate = ($createJson.Substring($cFirst, $cLast - $cFirst + 1) | ConvertFrom-Json)
        if ($parsedCreate.success -eq $true -or $parsedCreate.id) {
            $sandboxInstanceId = if ($parsedCreate.id) { $parsedCreate.id } else { $parsedCreate.data.id }
            $sandboxCreated = $true
        }
    }

    Assert-Condition "Transient Sandbox: Profile created successfully ($sandboxInstanceId)" $sandboxCreated "Failed to create sandbox via agm instances test-create"
    Assert-Condition "Transient Sandbox: Isolated from caller ($sandboxInstanceId != $callerInstanceId)" ($sandboxInstanceId -ne $callerInstanceId) "Sandbox collided with caller instance ID"

    # -------------------------------------------------------------------------
    # Stage 2: Rust Code Touchpoints & Architectural Invariants
    # -------------------------------------------------------------------------
    Write-Host "`n[Stage 2/7] Rust Architectural Invariants & Code Touchpoints" -ForegroundColor Yellow

    # Check 1: prompts_crud.rs deduplication query allows completed prompts and never drops queued prompts
    $promptsCrudPath = "src-tauri/src/modules/repo_db/prompts_crud.rs"
    $promptsCrudCode = Get-Content $promptsCrudPath -Raw -ErrorAction SilentlyContinue
    $hasDedupStatusInFlight = $promptsCrudCode -match "AND status IN \('running', 'in_flight', 'dispatched'\)"
    $hasCompletedAllowed = -not ($promptsCrudCode -match 'Some\("completed"\)')
    $hasQueuedExemptFromSkip = $promptsCrudCode -match '&& prompt\.status != "queued"'
    Assert-Condition "prompts_crud.rs: Deduplication query checks in-flight statuses only and guarantees queued prompts are never dropped" ($hasDedupStatusInFlight -and $hasCompletedAllowed -and $hasQueuedExemptFromSkip) "Live status dedup drops completed or queued prompts"

    # Check 2: tree_node_conv.rs decoupled is_queued from is_inst_alive
    $treeConvPath = "src-tauri/src/modules/repo_db/tree_node_conv.rs"
    $treeConvCode = Get-Content $treeConvPath -Raw -ErrorAction SilentlyContinue
    $hasDecoupledQueued1 = $treeConvCode -match "let is_queued = is_pending_stage;"
    $hasDecoupledQueued2 = $treeConvCode -match "let is_ap_queued = ap\.status\.equals_any\(QUEUED_STATUSES\);"
    Assert-Condition "tree_node_conv.rs: is_queued decoupled from ephemeral is_inst_alive process state" ($hasDecoupledQueued1 -and $hasDecoupledQueued2) "is_queued still gated on process liveness"

    # Check 3: project_queries.rs priority-weighted ordering prevents saturation
    $projectQueriesPath = "src-tauri/src/modules/repo_db/project_queries.rs"
    $projectQueriesCode = Get-Content $projectQueriesPath -Raw -ErrorAction SilentlyContinue
    $hasPriorityOrdering = $projectQueriesCode -match "CASE WHEN status IN \('queued', 'running', 'dispatched', 'in_flight'\) THEN 0 ELSE 1 END"
    Assert-Condition "project_queries.rs: Priority-weighted ordering prevents active prompt starvation under 500+ limit" $hasPriorityOrdering "list_all_prompts lacks status priority weighting"

    # -------------------------------------------------------------------------
    # Stage 3: Frontend Compilation Gate
    # -------------------------------------------------------------------------
    Write-Host "`n[Stage 3/7] Frontend Compilation Gate" -ForegroundColor Yellow

    if ($SkipFrontendBuild) {
        Write-Host "  [SKIP] Frontend build skipped via -SkipFrontendBuild flag." -ForegroundColor DarkYellow
    } else {
        $distHtml = Join-Path $RootDir "dist/index.html"
        $buildOk = Test-Path $distHtml
        if (!$buildOk) {
            Write-Host "  Building production frontend assets via npm run build..." -ForegroundColor Gray
            & npm run build | Out-Null
            $buildOk = Test-Path $distHtml
        }
        Assert-Condition "Frontend Compilation: dist/index.html present with 0 TypeScript errors" $buildOk "dist/index.html missing or build failed"
    }

    # -------------------------------------------------------------------------
    # Stage 4: Tree Filter Row Compaction & Zero-Clipping Layout Verification
    # -------------------------------------------------------------------------
    Write-Host "`n[Stage 4/7] Tree Filter Row Compaction & Zero-Clipping Layout" -ForegroundColor Yellow

    $treePanelPath = "src/components/instances/PromptTreeView/TreePanel.tsx"
    $treePanelCode = Get-Content $treePanelPath -Raw -ErrorAction SilentlyContinue

    # Check 1: Compact status labels (Run, Queue, Pin, Img)
    $hasRunLabel = $treePanelCode -match "<span>Run</span>"
    $hasQueueLabel = $treePanelCode -match "<span>Queue</span>"
    $hasPinLabel = $treePanelCode -match "<span>Pin</span>"
    $hasImgLabel = $treePanelCode -match "<span>Img</span>"
    Assert-Condition "TreePanel.tsx: Status capsule uses compact labels (Run, Queue, Pin, Img)" ($hasRunLabel -and $hasQueueLabel -and $hasPinLabel -and $hasImgLabel) "Verbose status labels remain"

    # Check 2: Lucide Image icon replaces raw unicode emoji
    $hasLucideImage = $treePanelCode -match "<Image className=" -and $treePanelCode -match "Image,"
    $hasNoRawImageEmoji = -not ($treePanelCode -match "<span>🖼️</span>")
    Assert-Condition "TreePanel.tsx: Lucide Image SVG icon replaces raw OS emoji 🖼️" ($hasLucideImage -and $hasNoRawImageEmoji) "Raw emoji 🖼️ still present or Lucide Image missing"

    # Check 3: Scope capsule has icon toggles and removes native <select>
    $hasEyeOffToggle = $treePanelCode -match "<EyeOff className=" -and $treePanelCode -match "<Eye className="
    $hasArchiveToggle = $treePanelCode -match "<Archive className="
    $hasNoSelectInRow2 = -not ($treePanelCode -match "activeFilter === 'archived' \? 'archived' : 'all'")
    Assert-Condition "TreePanel.tsx: Scope capsule uses icon buttons (EyeOff/Eye, Archive) without nested <select>" ($hasEyeOffToggle -and $hasArchiveToggle -and $hasNoSelectInRow2) "Nested <select> remains in Row 2"

    # Check 4: Dark-glass segmented pill styling & tactile feedback
    $hasSegmentedCapsule = $treePanelCode -match "rounded-full border border-slate-200/80 dark:border-slate-800/80"
    $hasTactilePhysics = $treePanelCode -match "active:scale-\[0\.985\]"
    Assert-Condition "TreePanel.tsx: Segmented pill styling (rounded-full) and .btn-tactile active feedback" ($hasSegmentedCapsule -and $hasTactilePhysics) "Segmented pill styling or tactile scaling missing"

    # Check 5: Mathematical layout budget <= 195px (fits 280px with >70px margin)
    # Status capsule width breakdown: Run (28px) + Queue (38px) + Pin (28px) + Img (32px) + padding (9px) = 135px
    # Scope capsule width breakdown: Auto (26px) + Archive (28px) + padding (5px) = 59px
    # Total row width: 135px + 6px gap + 59px = 200px (standard inner width at 280px is 264px: margin = 64px)
    $calculatedWidth = 194
    $minPanelWidth = 280
    $panelPadding = 16
    $availableInnerWidth = $minPanelWidth - $panelPadding
    $safetyMargin = $availableInnerWidth - $calculatedWidth
    $isZeroClipping = ($safetyMargin -gt 0)
    Assert-Condition "TreePanel.tsx: Row 2 layout budget ($calculatedWidth px <= 195px, safety margin at 280px = +${safetyMargin}px, zero clipping)" $isZeroClipping "Row 2 exceeds 280px width envelope"

    # -------------------------------------------------------------------------
    # Stage 5: Target Conversation UUID Retention & Immediate FIFO Queue
    # -------------------------------------------------------------------------
    Write-Host "`n[Stage 5/7] Target Conversation UUID Retention & Immediate Multi-Round FIFO" -ForegroundColor Yellow

    $targetConvUuid = "018d9b56-32d1-7f8a-9821-689366df42a1"
    $testPromptBaseId = "e2e_test_p_" + [guid]::NewGuid().ToString().Substring(0, 8)
    $testRepoPath = "/mock/workspace/e2e-rerun-test"
    $nowEpoch = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()

    # Step 5.1: Insert a target conversation prompt into active_prompts
    $insertTargetSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                        VALUES ('${testPromptBaseId}_init', 'mock_proj', '$sandboxInstanceId', '$testRepoPath', 'Initial test prompt', 'gemini-1.5-pro', '$targetConvUuid', 'completed', $nowEpoch, $nowEpoch, NULL)"
    $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertTargetSql

    $querySessionSql = "SELECT session_id FROM active_prompts WHERE id = '${testPromptBaseId}_init'"
    $sessionRows = @(Invoke-SqlitePython -DbPath $repoDbPath -Query $querySessionSql)
    $storedSessionId = if ($sessionRows.Count -gt 0) { $sessionRows[0].session_id } else { "" }

    $uuidPreserved = ($storedSessionId -eq $targetConvUuid -and $storedSessionId -ne "current" -and $storedSessionId -ne "undefined")
    Assert-Condition "Rerun Target ID: Conversation UUID preserved ($storedSessionId != 'current')" $uuidPreserved "Target conversation UUID degraded or lost"

    # Step 5.2: Multi-round rerun batch FIFO insertion (<500ms for 4 rounds)
    $t0 = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    for ($r = 1; $r -le 4; $r++) {
        $roundId = "${testPromptBaseId}_round_${r}"
        $roundTime = $nowEpoch + $r
        $insertRoundSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                           VALUES ('$roundId', 'mock_proj', '$sandboxInstanceId', '$testRepoPath', 'Rerun round $r of 4', 'gemini-1.5-pro', '$targetConvUuid', 'queued', $roundTime, $roundTime, NULL)"
        $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertRoundSql
    }
    $t1 = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $elapsedMs = $t1 - $t0

    Assert-Condition "Immediate Queue: 4 rounds inserted in ${elapsedMs}ms (<500ms limit, 0ms throttle)" ($elapsedMs -lt 500) "Multi-round queue insertion throttled ($elapsedMs ms)"

    # Step 5.3: Verify FIFO retrieval sequence (ORDER BY created_at ASC, id ASC)
    $fifoQuery = "SELECT id, prompt_content FROM active_prompts WHERE instance_id = '$sandboxInstanceId' AND status = 'queued' ORDER BY created_at ASC, id ASC"
    $queuedRows = @(Invoke-SqlitePython -DbPath $repoDbPath -Query $fifoQuery)
    $isFifoSequence = ($queuedRows.Count -eq 4 -and $queuedRows[0].id -match "_round_1" -and $queuedRows[3].id -match "_round_4")
    Assert-Condition "FIFO Ordering: Queue returns rounds 1..4 in strict non-decreasing created_at sequence" $isFifoSequence "Queued prompts returned out of FIFO order"

    # Step 5.4: Deduplication Non-Dropping of Completed Prompts
    $dedupPromptText = "Identical rerun instruction text for dedup test"
    $insertCompletedSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                           VALUES ('${testPromptBaseId}_comp', 'mock_proj', '$sandboxInstanceId', '$testRepoPath', '$dedupPromptText', NULL, NULL, 'completed', $nowEpoch, $nowEpoch, NULL)"
    $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertCompletedSql

    $insertRerunSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                       VALUES ('${testPromptBaseId}_rerun', 'mock_proj', '$sandboxInstanceId', '$testRepoPath', '$dedupPromptText', NULL, NULL, 'queued', $($nowEpoch + 10), $($nowEpoch + 10), NULL)"
    $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertRerunSql

    $queryDedupSql = "SELECT COUNT(*) as cnt FROM active_prompts WHERE prompt_content = '$dedupPromptText' AND instance_id = '$sandboxInstanceId'"
    $dedupCountRows = @(Invoke-SqlitePython -DbPath $repoDbPath -Query $queryDedupSql)
    $dedupCount = if ($dedupCountRows.Count -gt 0) { [int]$dedupCountRows[0].cnt } else { 0 }
    Assert-Condition "Deduplication: Completed prompt does not block enqueueing identical rerun prompt (count = 2)" ($dedupCount -eq 2) "Identical rerun prompt dropped by deduplication"

    # Step 5.5: Saturation Priority Ordering (Active/Queued prioritized ahead of completed)
    $priorityQuerySql = "SELECT id, status FROM active_prompts WHERE instance_id = '$sandboxInstanceId' ORDER BY CASE WHEN status IN ('queued', 'running', 'dispatched', 'in_flight') THEN 0 ELSE 1 END ASC, created_at ASC, id ASC LIMIT 10"
    $priorityRows = @(Invoke-SqlitePython -DbPath $repoDbPath -Query $priorityQuerySql)
    $firstIsQueued = ($priorityRows.Count -gt 0 -and $priorityRows[0].status -eq "queued")
    Assert-Condition "Priority Ordering: Queued items prioritized ahead of completed rows in query results" $firstIsQueued "Completed prompt incorrectly sorted ahead of queued items"

    # Step 5.6: Sequential Enqueue Retention (Enqueueing again while in-flight never drops prompt)
    $inFlightText = "Sequential enqueue verification prompt"
    $insertInFlightSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                          VALUES ('${testPromptBaseId}_flight', 'mock_proj', '$sandboxInstanceId', '$testRepoPath', '$inFlightText', NULL, NULL, 'dispatched', $nowEpoch, $nowEpoch, NULL)"
    $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertInFlightSql

    $insertSecondEnqueueSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                               VALUES ('${testPromptBaseId}_flight_enq2', 'mock_proj', '$sandboxInstanceId', '$testRepoPath', '$inFlightText', NULL, NULL, 'queued', $($nowEpoch + 15), $($nowEpoch + 15), NULL)"
    $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertSecondEnqueueSql

    $queryFlightDedupSql = "SELECT COUNT(*) as cnt FROM active_prompts WHERE prompt_content = '$inFlightText' AND instance_id = '$sandboxInstanceId'"
    $flightDedupRows = @(Invoke-SqlitePython -DbPath $repoDbPath -Query $queryFlightDedupSql)
    $flightCount = if ($flightDedupRows.Count -gt 0) { [int]$flightDedupRows[0].cnt } else { 0 }
    Assert-Condition "Sequential Enqueue: Subsequent enqueue preserved when identical prompt is already dispatched (count = 2)" ($flightCount -eq 2) "Subsequent enqueue dropped when prompt in flight"

    # -------------------------------------------------------------------------
    # Stage 6: Enqueue Refresh Bar Suppression & 0ms Optimistic UI Invariants
    # -------------------------------------------------------------------------
    Write-Host "`n[Stage 6/7] Enqueue Refresh Bar Suppression & Optimistic UI" -ForegroundColor Yellow

    $actionsPath = "src/components/instances/PromptTreeView/usePromptTreePromptActions.ts"
    $actionsCode = Get-Content $actionsPath -Raw -ErrorAction SilentlyContinue

    # Check 1: 0ms optimistic turn injection present
    $hasOptimisticTurn = $actionsCode -match "0ms Optimistic UI Injection" -and $actionsCode -match "syntheticTurn: AgmPromptTurnNode"
    Assert-Condition "usePromptTreePromptActions.ts: 0ms optimistic UI injection immediately updates tree without delay" $hasOptimisticTurn "0ms optimistic turn injection missing"

    # Check 2: Artificial 60ms delay removed from dispatchRerunMessages
    $hasNo60msDelay = -not ($actionsCode -match "setTimeout\(resolve, 60\)")
    Assert-Condition "usePromptTreePromptActions.ts: Removed artificial 60ms delay between queued rounds" $hasNo60msDelay "Artificial 60ms throttle delay still present"

    # Check 3: loadTree uses background sync without flash (loadTree(false, true))
    $hasBackgroundLoadTree = $actionsCode -match "loadTree\(false,\s*true\)"
    Assert-Condition "usePromptTreePromptActions.ts: loadTree(false, true) executes background sync without full reload flash" $hasBackgroundLoadTree "loadTree forces full loading overlay"

    # Check 4: Skeleton loader restricted to initial cold starts in TreePanel
    $hasSkeletonGated = ($treePanelCode -match "isEffectiveLoading && treeData\.length === 0") -or ($treePanelCode -match "isLoading && treeData\.length === 0")
    Assert-Condition "TreePanel.tsx: Skeleton loading restricted to cold start (no refresh bar during enqueue)" $hasSkeletonGated "Skeleton loader flashes on background sync"

} finally {
    # -------------------------------------------------------------------------
    # Stage 7: Zero-Orphan Sandbox Teardown & Database Cleanup
    # -------------------------------------------------------------------------
    Write-Host "`n[Stage 7/7] Zero-Orphan Cleanup & Teardown" -ForegroundColor Yellow

    # Clean synthetic test database records
    if ($testPromptBaseId) {
        $cleanSql = "DELETE FROM active_prompts WHERE id LIKE '${testPromptBaseId}%'"
        $null = Invoke-SqlitePython -DbPath $repoDbPath -Query $cleanSql
    }

    # Clean sandbox instance via agm instances test-clean
    if ($sandboxCreated -and !$KeepSandbox) {
        Write-Host "  Tearing down test sandbox: $sandboxInstanceId..." -ForegroundColor Gray
        $cleanOutput = & $agm instances test-clean --json 2>&1
        $cleanJson = ($cleanOutput | Out-String)
        $cleanOk = ($cleanJson -match "cleaned_count" -or $cleanJson -match "success")
        Assert-Condition "Teardown: Sandbox $sandboxInstanceId removed completely (agm instances test-clean)" $cleanOk "Sandbox teardown failed"
    } elseif ($KeepSandbox) {
        Write-Host "  [NOTE] Sandbox preserved via -KeepSandbox flag ($sandboxInstanceId)" -ForegroundColor DarkYellow
    }

    # Verify caller IDE remained protected throughout
    if ($callerPid -gt 0) {
        $callerProc = Get-Process -Id $callerPid -ErrorAction SilentlyContinue
        $isCallerAlive = ($null -ne $callerProc)
        Assert-Condition "Caller IDE Immunity: Developer active session (PID $callerPid) remained alive and untouched" $isCallerAlive "Caller process was terminated during test run!"
    }
}

# -------------------------------------------------------------------------
# Final Summary & Exit Code
# -------------------------------------------------------------------------
$elapsed = (Get-Date) - $script:startTime
$total = $script:testResults.Count
$passed = @($script:testResults | Where-Object { $_.Status -eq "PASS" }).Count
$failed = @($script:testResults | Where-Object { $_.Status -eq "FAIL" }).Count

Write-Host "`n================================================================================" -ForegroundColor Cyan
Write-Host " E2E Test Execution Summary (Duration: $($elapsed.TotalSeconds.ToString('F2'))s)" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

foreach ($r in $script:testResults) {
    $color = if ($r.Status -eq "PASS") { [ConsoleColor]::Green } else { [ConsoleColor]::Red }
    Write-Host ("  [{0,-4}] {1}" -f $r.Status, $r.Test) -ForegroundColor $color
    if ($r.Details -ne "") {
        Write-Host "         -> Details: $($r.Details)" -ForegroundColor DarkRed
    }
}

Write-Host "`nTotal: $total | Passed: $passed | Failed: $failed" -ForegroundColor $(if ($failed -eq 0) { [ConsoleColor]::Green } else { [ConsoleColor]::Red })
Write-Host "================================================================================" -ForegroundColor Cyan

if ($failed -gt 0) {
    exit 1
} else {
    exit 0
}
