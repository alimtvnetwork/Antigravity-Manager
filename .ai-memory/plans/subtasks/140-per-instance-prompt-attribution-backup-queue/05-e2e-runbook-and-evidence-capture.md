# Subtask 05: E2E runbook and evidence capture

Status: pending

| Field | Value |
|---|---|
| Parent plan | `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md` |
| Case definitions (steps, expected results, SQL Q01 to Q17) | `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md` |
| Checklist and ordering | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/04-e2e-multi-instance-test-catalog.md` |
| Machine | Windows, PowerShell 5.1 or 7, run from the repo root (`<repo>`) |
| Evidence folder | `.ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/` (gitignored) |

This is the exact procedure an AI follows on this machine. Read the whole file once before running anything. Paste the helper block in section 3 into one PowerShell session and keep that session for the whole run.

## 1. Safety rules (D10, verbatim)

- Never kill Cursor or `agm-alim`; never switch or close the default instance; never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
- New test instances only with prefix `test-cli-flow-` or `test-diag-` (spec 94 L8); delete only instances this run created.
- Do not click the Google data-collection checkbox; do not launch a replacement AGM GUI.
- Read databases read-only (`file:<path>?mode=ro`); never print emails or tokens in full.
- Record evidence per test: command, exit code, JSON excerpt, SQL row with `instance_id`. Never claim a live reinject was watched unless it was.

Additional rules for this runbook:

1. Never run a command from the banned list in section 3 of the component spec (`agm tif`, `agm instances rm-all`, `agm switch <account>` without an instance, `agm ff` without an instance, `agm instances all ff`, any `agm auto-switch` action other than `status`, `agm rrc` / `agm rrp` / `agm brp` / `agm qs` without an instance on the live data, `agm agy cache-clear`, `agm accounts export`, `scripts/dev-tool-clear.ps1 -CleanInstances`, `scripts/test-instance-e2e.ps1`, `scripts/test-instance-isolation.ps1`).
2. Stop a process only by a PID this run started itself (a timed-out `agm.exe` child, or the `agm qs` loop of E2E-15). Never stop by image name. Test instances are closed only with `agm instances stop <id>`.
3. The only write to a live database is the narrow row cleanup in E2E-19, limited to this run's exact instance ids. Every other write goes to a copy under the run folder.
4. Do not use `Select-String`, `findstr`, or grep tools; text checks use `.Contains()` or the `-match` operator on a string already in memory.
5. Evidence files never contain an absolute path, a drive letter, a username, a full email, or a token. The helper `Protect-Text` rewrites them.

## 2. Stop rules

| Condition | Action |
|---|---|
| A protected PID (Cursor, `agm-alim`, default Antigravity, protected sandbox) is gone or its start time changed | ABORT: stop only this run's test instances (`agm instances stop <id>`), write `ABORTED` lines for the remaining cases, tell the user. Never restart or kill anything protected. |
| E2E-01 FAIL | STOP: run E2E-19, mark every other case BLOCKED with reason `feasibility_failed`, report. |
| A scoped command acted on default or a protected sandbox (Q14 finding inside the command window, or a rotation whose target is not a test instance) | ABORT as above. |
| `ABV_DATA_DIR` is already set in the session at start | STOP before E2E-00 and ask the user; the run cannot tell which data dir is live. |
| The required build for a case is missing (gate G-BUILD) | Mark that case BLOCKED (`needs_subtask_0N`), continue with the next case. |
| A human or UI tool is needed and not available (E2E-01 path 2, E2E-09) | Mark that part BLOCKED (`needs_human_ui`), continue. |

## 3. Session setup and helpers

Run once, from the repo root.

```powershell
$ErrorActionPreference = 'Stop'
if ($env:ABV_DATA_DIR) { throw 'ABV_DATA_DIR is set; stop and ask the user.' }

$RepoRoot   = (Get-Location).Path
$Rand4      = '{0:D4}' -f (Get-Random -Minimum 0 -Maximum 10000)
$RunId      = (Get-Date -Format 'yyyyMMdd-HHmmss') + "-$Rand4"
$RunDir     = Join-Path $RepoRoot ".ai-memory\temp-agents\140-per-instance-e2e-runs\$RunId"
$RunRel     = ".ai-memory/temp-agents/140-per-instance-e2e-runs/$RunId"
$Scratch    = Join-Path $env:TEMP "agm-e2e140-$RunId"
New-Item -ItemType Directory -Force -Path $RunDir, (Join-Path $RunDir 'raw') | Out-Null
foreach ($r in 'repo-a', 'repo-b', 'repo-shared', 'repo-gone') {
    $d = Join-Path $Scratch $r
    New-Item -ItemType Directory -Force -Path $d | Out-Null
    Set-Content -Path (Join-Path $d 'README.md') -Value "E2E140 scratch repo $r for run $RunId. No git folder on purpose."
}

$ProtectedIds = @('cli-switch-proof-6857', 'test-cli-flow-1743', 'clone-from-default-4424', 'new-empty-instance-4425')
$Run = [ordered]@{ run_id = $RunId; rand4 = $Rand4; instances = [ordered]@{}; gates = [ordered]@{} }

$Agm = Join-Path $RepoRoot 'src-tauri\target\debug\agm.exe'
if (-not (Test-Path $Agm)) { $Agm = (Get-Command agm -ErrorAction Stop).Source }
python -c "import sqlite3, json, pathlib; print('py-ok')" | Out-Null

function Protect-Text([string]$s) {
    if ($null -eq $s) { return $null }
    $pairs = @(
        @($env:TEMP, '%TEMP%'), @($env:LOCALAPPDATA, '%LOCALAPPDATA%'), @($env:APPDATA, '%APPDATA%'),
        @($RepoRoot, '<repo>'), @($env:USERPROFILE, '%USERPROFILE%')
    )
    foreach ($p in $pairs) {
        if (-not $p[0]) { continue }
        foreach ($form in @($p[0], $p[0].Replace('\', '\\'), $p[0].Replace('\', '/'))) {
            $s = $s -ireplace [regex]::Escape($form), $p[1]
        }
    }
    $s = $s -replace '([A-Za-z0-9._%+-]{2})[A-Za-z0-9._%+-]*@([A-Za-z0-9-]+\.[A-Za-z0-9.-]+)', '$1***@$2'
    $s = $s -replace 'ya29\.[A-Za-z0-9_\-\.]+', 'ya29.***'
    $s = $s -replace '1//[A-Za-z0-9_\-]{20,}', '1//***'
    $s = $s -replace '("(access_token|refresh_token|id_token|token|api_key)"\s*:\s*")[^"]*"', '$1***"'
    $s = $s -replace '[A-Za-z]:\\', '<drive>\'
    return $s
}

function ConvertFrom-AgmJson([string]$text) {
    if (-not $text) { return $null }
    try { return $text | ConvertFrom-Json } catch {}
    $i = $text.IndexOfAny([char[]]@('{', '['))
    if ($i -lt 0) { return $null }
    try { return $text.Substring($i) | ConvertFrom-Json } catch { return $null }
}

function Invoke-Agm {
    param([string[]]$A, [string]$Cwd = $Scratch, [int]$TimeoutSec = 120, [hashtable]$Env = @{})
    $argLine = ($A | ForEach-Object { if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ } }) -join ' '
    $tag = [guid]::NewGuid().ToString('N').Substring(0, 8)
    $so = Join-Path $RunDir "raw\$tag.out"; $se = Join-Path $RunDir "raw\$tag.err"
    $saved = @{}
    foreach ($k in $Env.Keys) { $saved[$k] = [Environment]::GetEnvironmentVariable($k); [Environment]::SetEnvironmentVariable($k, $Env[$k]) }
    $t0 = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    try {
        $p = Start-Process -FilePath $Agm -ArgumentList $argLine -WorkingDirectory $Cwd -NoNewWindow -PassThru `
            -RedirectStandardOutput $so -RedirectStandardError $se
        $null = $p.Handle
        if ($p.WaitForExit($TimeoutSec * 1000)) { $code = $p.ExitCode } else { Stop-Process -Id $p.Id -Force; $code = 124 }
    } finally {
        foreach ($k in $Env.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k]) }
    }
    $out = Protect-Text ([string](Get-Content $so -Raw -ErrorAction SilentlyContinue))
    $err = Protect-Text ([string](Get-Content $se -Raw -ErrorAction SilentlyContinue))
    Set-Content $so $out; Set-Content $se $err
    [pscustomobject]@{
        cmd = 'agm ' + (Protect-Text $argLine); cwd = (Protect-Text $Cwd); env = @($Env.Keys)
        exit_code = $code; started = $t0; ended = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
        stdout_file = "$RunRel/raw/$tag.out"
        stdout_excerpt = if ($out) { $out.Substring(0, [Math]::Min(2000, $out.Length)) } else { '' }
        stderr_excerpt = if ($err) { $err.Substring(0, [Math]::Min(1000, $err.Length)) } else { '' }
        json = (ConvertFrom-AgmJson $out)
    }
}

$PySql = @'
import os, sqlite3, json, pathlib
db = os.environ["E2E_DB"]; sql = os.environ["E2E_SQL"]
params = json.loads(os.environ.get("E2E_PARAMS") or "{}")
mode = os.environ.get("E2E_MODE", "ro")
con = sqlite3.connect(pathlib.Path(db).resolve().as_uri() + "?mode=" + mode, uri=True, timeout=5)
con.row_factory = sqlite3.Row
cur = con.execute(sql, params)
rows = [dict(r) for r in cur.fetchall()] if cur.description else []
if mode == "rw":
    con.commit()
print(json.dumps({"rows": rows, "changes": con.total_changes}, default=str))
'@

function Invoke-Sql {
    param([string]$Db, [string]$Sql, [hashtable]$Params = @{}, [switch]$Write, [switch]$LiveCleanup)
    if (-not (Test-Path $Db)) { return [pscustomobject]@{ db = (Protect-Text $Db); missing = $true; rows = @() } }
    $full = (Resolve-Path $Db).Path
    if ($Write -and -not $LiveCleanup -and -not $full.StartsWith($RunDir, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to write outside the run folder: $(Protect-Text $full)"
    }
    $env:E2E_DB = $full; $env:E2E_SQL = $Sql
    $env:E2E_PARAMS = ($Params | ConvertTo-Json -Compress)
    $env:E2E_MODE = if ($Write) { 'rw' } else { 'ro' }
    $raw = ($PySql | python -) -join "`n"
    $res = $raw | ConvertFrom-Json
    [pscustomobject]@{ db = (Protect-Text $full); sql = $Sql; params = $Params; rows = @($res.rows); changes = $res.changes }
}

function Get-MainProcs {
    Get-CimInstance Win32_Process -Filter "Name='Cursor.exe' OR Name='agm-alim.exe' OR Name='Antigravity.exe'" |
        Where-Object { [string]$_.CommandLine -notmatch '--type=' } |
        ForEach-Object { [pscustomobject]@{ pid = [int]$_.ProcessId; name = $_.Name; start = $_.CreationDate.ToString('o'); cmd = (Protect-Text ([string]$_.CommandLine)) } }
}

function Get-Instances {
    $r = Invoke-Agm @('instances', 'ls', '--json') -Cwd $Scratch
    @($r.json) | ForEach-Object {
        $c = if ($_.config) { $_.config } else { $_ }
        $pidv = if ($_.pid) { $_.pid } else { $c.pid }
        [pscustomobject]@{ id = $c.id; name = $c.name; seq = $c.seq_num; is_default = $c.is_default; running = $_.is_running; pid = $pidv; data_dir_raw = $c.data_dir }
    }
}

function Test-Safety([string]$CaseId) {
    $now = @(Get-MainProcs)
    $lost = @($Snapshot.procs | Where-Object { $p = $_; -not ($now | Where-Object { $_.pid -eq $p.pid -and $_.start -eq $p.start }) })
    if ($lost.Count -gt 0) {
        Add-Report $CaseId 'ABORTED' ("protected process changed: " + (($lost | ForEach-Object { "$($_.name) $($_.pid)" }) -join ', '))
        foreach ($id in $Run.instances.Values) { Invoke-Agm @('instances', 'stop', $id) | Out-Null }
        throw "ABORT: protected process changed during $CaseId"
    }
    return $true
}

function Get-FlavorDirs([string]$InstanceId) {
    $base = if ($InstanceId -eq 'GLOBAL') { Join-Path $env:USERPROFILE '.gemini' } else { Join-Path $InstancesDir "$InstanceId\home\.gemini" }
    'antigravity', 'antigravity-ide', 'antigravity-cli' | ForEach-Object { Join-Path $base $_ }
}

function Read-Shared([string]$path) {
    try {
        $fs = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
        try { (New-Object IO.StreamReader($fs)).ReadToEnd() } finally { $fs.Dispose() }
    } catch { '' }
}

function Find-Marker([string]$InstanceId, [string]$Marker) {
    $hits = @()
    foreach ($d in Get-FlavorDirs $InstanceId) {
        $db = Join-Path $d 'conversation_summaries.db'
        if (Test-Path $db) {
            $r = Invoke-Sql -Db $db -Sql $Q.Q09 -Params @{ marker = $Marker }
            foreach ($row in $r.rows) { $hits += [pscustomobject]@{ store = 'summaries'; dir = (Protect-Text $d); cid = $row.conversation_id; status = $row.status; not_fully_idle = $row.not_fully_idle; last_modified_time = $row.last_modified_time } }
        }
        $brain = Join-Path $d 'brain'
        if (Test-Path $brain) {
            foreach ($b in Get-ChildItem $brain -Directory) {
                $t = Join-Path $b.FullName '.system_generated\logs\transcript.jsonl'
                if ((Test-Path $t) -and (Read-Shared $t).Contains($Marker)) { $hits += [pscustomobject]@{ store = 'transcript'; dir = (Protect-Text $d); cid = $b.Name } }
            }
        }
    }
    , $hits
}

function Wait-Until([scriptblock]$Cond, [int]$TimeoutSec, [int]$EverySec = 3) {
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) { $r = & $Cond; if ($r) { return $r }; Start-Sleep -Seconds $EverySec }
    return $null
}

function New-Marker([string]$InstanceId, [string]$CaseStep) { "E2E140-$RunId-$InstanceId-$CaseStep" }
function New-PromptText([string]$Marker) {
    "$Marker This is an automated isolation test. Do not edit, create, or delete any file. List the names of the files in this folder, then count from 1 to 40 in words, one per line, then reply DONE $Marker."
}

function New-Case([string]$Id, [string]$Title, [string[]]$Ac, [string[]]$Bugs, [string]$Mode) {
    Test-Safety $Id | Out-Null
    [ordered]@{ id = $Id; title = $Title; ac = $Ac; bugs = $Bugs; mode = $Mode; run_id = $RunId
        started_at = (Get-Date).ToString('o'); since = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
        steps = @(); sql = @(); files = @(); notes = @(); result = $null; reason = $null }
}

function Save-Case($Case, [string]$Result, [string]$Reason) {
    Test-Safety $Case.id | Out-Null
    if ($Run.instances.Contains('A')) {
        $other = if ($Run.instances.Contains('B')) { $Run.instances.B } else { $Run.instances.A }
        $Case.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q14 -Params @{ since = $Case.since; inst = $Run.instances.A; other = $other }
    }
    $Case.result = $Result; $Case.reason = $Reason; $Case.finished_at = (Get-Date).ToString('o')
    Set-Content -Path (Join-Path $RunDir "$($Case.id).json") -Value (Protect-Text ($Case | ConvertTo-Json -Depth 12)) -Encoding UTF8
    Add-Report $Case.id $Result $Reason
}

function Add-Report([string]$CaseId, [string]$Result, [string]$Reason) {
    $line = '{0} | {1} | {2}/{0}.json | {3}' -f $CaseId, $Result, $RunRel, $Reason
    Add-Content -Path (Join-Path $RunDir 'report.txt') -Value $line
}

function Save-Run { Set-Content -Path (Join-Path $RunDir 'run.json') -Value (Protect-Text ($Run | ConvertTo-Json -Depth 6)) -Encoding UTF8 }
```

Load the SQL catalog (same text as section 4.5 of the component spec):

```powershell
$Q = @{
  Q01  = "SELECT id, instance_id, project_id, repo_path, session_id, status, created_at, updated_at FROM active_prompts WHERE instance_id = :inst ORDER BY created_at"
  Q01N = "SELECT id, instance_id, repo_path, session_id, status, status_reason, source_dir, attempts, created_at, updated_at FROM active_prompts WHERE instance_id = :inst ORDER BY created_at"
  Q02  = "SELECT id, instance_id, session_id, status, created_at, updated_at FROM active_prompts WHERE instance_id = :inst AND instr(prompt_content, :marker) > 0"
  Q03  = "SELECT id, instance_id, status FROM active_prompts WHERE instance_id <> :inst AND instr(prompt_content, :marker) > 0"
  Q04  = "SELECT id, instance_id, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at FROM running_projects WHERE instance_id = :inst ORDER BY id"
  Q05  = "SELECT COUNT(*) AS bad FROM running_projects WHERE instance_id = :inst AND substr(id, -length('__' || :inst)) <> '__' || :inst"
  Q06  = "SELECT conversation_id, seq_id, instance_id, project_key, updated_at FROM agm_conversation_sequences WHERE instance_id = :inst AND conversation_id = :cid"
  Q06X = "SELECT conversation_id, seq_id, instance_id FROM agm_conversation_sequences WHERE conversation_id = :cid AND instance_id IN (:inst, :other)"
  Q07  = "SELECT id, backup_batch_id, prompt_id, instance_id, project_path, conversation_id, status, is_restored, restored_at, created_at FROM prompt_backups WHERE instance_id = :inst ORDER BY created_at, id"
  Q08  = "SELECT SUM(CASE WHEN instance_id = :inst THEN 1 ELSE 0 END) AS own_rows, SUM(CASE WHEN instance_id IS NULL OR instance_id <> :inst THEN 1 ELSE 0 END) AS foreign_rows FROM prompt_backups WHERE backup_batch_id = :batch"
  Q09  = "SELECT conversation_id, title, status, not_fully_idle, workspace_uris, last_modified_time FROM conversation_summaries WHERE instr(coalesce(preview, ''), :marker) > 0 OR instr(coalesce(title, ''), :marker) > 0"
  Q10  = "SELECT COUNT(*) AS n FROM conversation_summaries WHERE conversation_id = :cid"
  Q11  = "SELECT COUNT(*) AS n FROM active_prompts WHERE (instance_id IS NULL OR instance_id IN ('', '__default__', 'all')) AND status <> 'orphaned'"
  Q11B = "SELECT COUNT(*) AS n FROM prompt_backups WHERE (instance_id IS NULL OR instance_id IN ('', '__default__', 'all')) AND is_restored = 0"
  Q12  = "SELECT COUNT(*) AS bad FROM active_prompts WHERE instance_id = :inst AND id LIKE 'prompt-%' AND id <> 'prompt-' || instance_id || '-' || session_id"
  Q13  = "SELECT id, instance_id, repo_path, session_id, status, status_reason, source_dir, attempts, updated_at FROM active_prompts WHERE instance_id = :inst AND status IN ('queued', 'dispatching', 'dispatched', 'failed') AND instr(prompt_content, :marker) > 0"
  Q14  = "SELECT id, instance_id, status, updated_at FROM active_prompts WHERE updated_at >= :since AND instance_id NOT IN (:inst, :other) ORDER BY updated_at"
  Q15  = "SELECT id, status, session_id, updated_at FROM active_prompts WHERE instance_id = :inst AND status = 'backed_up' ORDER BY id"
  Q16  = "SELECT id, status, updated_at FROM active_prompts WHERE instance_id = :inst AND status = 'running' ORDER BY id"
  Q16N = "SELECT id, status, updated_at, attempts FROM active_prompts WHERE instance_id = :inst AND status = 'running' ORDER BY id"
  Q17  = "SELECT COUNT(*) AS n FROM active_prompts WHERE instance_id = :other AND session_id = :cid"
}
```

## 4. Run order

Follow the checklist order in subtask 04. In short: E2E-00, then E2E-14 when the build contains subtask 01, then E2E-01 (gate), E2E-02 to E2E-07, E2E-08, E2E-09 (human), E2E-10, E2E-15, E2E-16, E2E-11, E2E-12, E2E-18, E2E-17, E2E-13, E2E-19.

Wrap every case like this so a thrown abort still reaches cleanup:

```powershell
try {
    # case blocks from section 5, in order
} finally {
    # E2E-19 step 1 always runs: stop only this run's instances
    foreach ($id in $Run.instances.Values) { Invoke-Agm @('instances', 'stop', $id) | Out-Null }
    Save-Run
}
```

## 5. Case blocks

Each block below is the minimum script. Expected results and pass/fail rules are in the component spec section 5; record the verdict with `Save-Case`. Where a block says "decide", compare the captured evidence with the spec's expected result and pick PASS, FAIL, BLOCKED, or BASELINE (today-baseline run on an unfixed build).

### E2E-00 Safety snapshot

```powershell
$c = New-Case 'E2E-00' 'Safety snapshot' @('AC-32') @() 'today'
$Snapshot = [ordered]@{ procs = @(Get-MainProcs) }
$insts = @(Get-Instances)
$def = $insts | Where-Object { $_.is_default -or $_.id -eq 'default' } | Select-Object -First 1
$Snapshot.instances = @($insts | Where-Object { $_.id -eq $def.id -or $ProtectedIds -contains $_.id } | Select-Object id, running, pid)
$probe = $insts | Where-Object { $ProtectedIds -contains $_.id -and $_.data_dir_raw } | Select-Object -First 1
$DataDir = Split-Path (Split-Path (Split-Path $probe.data_dir_raw))
$InstancesDir = Join-Path $DataDir 'instances'
$RepoDb = Join-Path $DataDir 'repo_prompts.db'
$bdir = Join-Path $DataDir 'backup-prompts'
$BackupDb = if (Test-Path (Join-Path $bdir 'SQL.db')) { Join-Path $bdir 'SQL.db' } else { Join-Path $bdir 'backup-prompts.db' }
if (Test-Path (Join-Path $Scratch 'data\backup-prompts')) { throw 'scratch cwd has data\backup-prompts; the CLI would use a different backup DB' }
$auto = Invoke-Agm @('auto-switch', 'status', '--json')
$c.steps += $auto
$Run.gates.noisy_autoswitch = ($auto.stdout_excerpt -match '"enabled"\s*:\s*true')
$c.sql += Invoke-Sql -Db $RepoDb -Sql "SELECT instance_id, status, COUNT(*) AS n, MAX(updated_at) AS last FROM active_prompts GROUP BY instance_id, status"
$l1 = Invoke-Sql -Db $RepoDb -Sql $Q.Q11
$l2 = Invoke-Sql -Db $BackupDb -Sql $Q.Q11B
$c.sql += $l1, $l2
$Run.gates.G_LEGACY = (($l1.rows[0].n -eq 0) -and ($l2.missing -or $l2.rows[0].n -eq 0))
$Snapshot.repo_db_last_write = (Get-Item $RepoDb).LastWriteTimeUtc.ToString('o')
$c.snapshot = $Snapshot
Save-Run
Save-Case $c 'PASS' "procs=$($Snapshot.procs.Count) G_LEGACY=$($Run.gates.G_LEGACY)"
```

If `$probe` is empty (no protected sandbox has a `data_dir`), stop and ask the user for the data dir; do not guess.

### Build detection (sandbox only, before E2E-14 and E2E-01)

```powershell
function New-Sandbox([string]$Name, [switch]$Sanitize) {
    $sb = Join-Path $RunDir $Name
    New-Item -ItemType Directory -Force -Path (Join-Path $sb 'backup-prompts'), (Join-Path $sb 'instances') | Out-Null
    Copy-Item $RepoDb (Join-Path $sb 'repo_prompts.db')
    if (Test-Path $BackupDb) { Copy-Item $BackupDb (Join-Path $sb 'backup-prompts\backup-prompts.db') }
    foreach ($f in 'instances.json', 'instances.db') { if (Test-Path (Join-Path $InstancesDir $f)) { Copy-Item (Join-Path $InstancesDir $f) (Join-Path $sb "instances\$f") } }
    if ($Sanitize) {
        $keep = @($Run.instances.Values)
        $in = if ($keep.Count) { "'" + ($keep -join "','") + "'" } else { "''" }
        foreach ($t in 'active_prompts', 'running_projects', 'agm_conversation_sequences') {
            Invoke-Sql -Db (Join-Path $sb 'repo_prompts.db') -Write -Sql "DELETE FROM $t WHERE instance_id IS NULL OR instance_id NOT IN ($in)" | Out-Null
        }
        if (Test-Path (Join-Path $sb 'backup-prompts\backup-prompts.db')) {
            Invoke-Sql -Db (Join-Path $sb 'backup-prompts\backup-prompts.db') -Write -Sql "DELETE FROM prompt_backups WHERE instance_id IS NULL OR instance_id NOT IN ($in)" | Out-Null
        }
    }
    $sb
}
$SbFull = New-Sandbox 'sandbox-full'
$probeRun = Invoke-Agm @('prompts', '--json') -Env @{ ABV_DATA_DIR = $SbFull }
$cols = (Invoke-Sql -Db (Join-Path $SbFull 'repo_prompts.db') -Sql "SELECT name FROM pragma_table_info('active_prompts')").rows.name
$Run.gates.G_BUILD_01 = ($cols -contains 'source_dir') -and ($cols -contains 'attempts')
$trace = Invoke-Agm @('prompts', 'trace', '--help')
$Run.gates.G_BUILD_03 = ($trace.stdout_excerpt -match 'prompts trace')
$SbData = New-Sandbox 'sandbox-data' -Sanitize
$rrp = Invoke-Agm @('rrp', '-i', 'test-cli-flow-1743', '--json') -Env @{ ABV_DATA_DIR = $SbData } -TimeoutSec 300
$Run.gates.G_BUILD_02 = ($rrp.stdout_excerpt -match 'restored_from_backup') -and ($rrp.stdout_excerpt -match 'skipped')
Save-Run
```

The sanitized sandbox is rebuilt with `New-Sandbox 'sandbox-data' -Sanitize` after A and B exist (E2E-08, E2E-10, E2E-16, E2E-17 baselines), so it then keeps their rows.

### E2E-14 Legacy migration on a copy (run here only when `G_BUILD_01` is true; otherwise run the read-only count and record BASELINE)

```powershell
$c = New-Case 'E2E-14' 'Legacy empty instance_id migration on a copy' @('AC-05', 'AC-08') @('B01', 'B18') 'after-01-sandbox'
$c.sql += Invoke-Sql -Db (Join-Path $SbFull 'repo_prompts.db') -Sql $Q.Q11
if (-not $Run.gates.G_BUILD_01) { Save-Case $c 'BASELINE' 'build lacks subtask 01; legacy counts recorded'; }
else {
    $sb = New-Sandbox 'sandbox-data' -Sanitize
    $schema = "CREATE TABLE IF NOT EXISTS conversation_summaries (conversation_id TEXT PRIMARY KEY, title TEXT, preview TEXT, status TEXT, not_fully_idle INTEGER, workspace_uris TEXT, last_modified_time TEXT)"
    foreach ($o in 'test-cli-flow-1743', 'cli-switch-proof-6857') {
        $dst = Join-Path $sb "instances\$o\home\.gemini\antigravity"
        New-Item -ItemType Directory -Force -Path $dst | Out-Null
        $src = Join-Path $InstancesDir "$o\home\.gemini\antigravity\conversation_summaries.db"
        $dstDb = Join-Path $dst 'conversation_summaries.db'
        if (Test-Path $src) { Copy-Item $src $dstDb }
        else { New-Item -ItemType File -Path $dstDb | Out-Null }
        Invoke-Sql -Db $dstDb -Write -Sql $schema | Out-Null
        Invoke-Sql -Db $dstDb -Write -Sql "INSERT OR REPLACE INTO conversation_summaries (conversation_id, status, not_fully_idle) VALUES (:cid, 'IDLE', 0)" -Params @{ cid = "e2e140-$RunId-dup" } | Out-Null
    }
    $o1Db = Join-Path $sb 'instances\test-cli-flow-1743\home\.gemini\antigravity\conversation_summaries.db'
    Invoke-Sql -Db $o1Db -Write -Sql "INSERT OR REPLACE INTO conversation_summaries (conversation_id, status, not_fully_idle) VALUES (:cid, 'IDLE', 0)" -Params @{ cid = "e2e140-$RunId-one" } | Out-Null
    $sbRepo = Join-Path $sb 'repo_prompts.db'
    foreach ($s in @(@('l1', "e2e140-$RunId-one"), @('l2', "e2e140-$RunId-nocid"), @('l3', "e2e140-$RunId-dup"))) {
        Invoke-Sql -Db $sbRepo -Write -Sql "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at) VALUES (:id, 'e2e140', '', :repo, :text, NULL, :cid, 'backed_up', strftime('%s','now'), strftime('%s','now'))" `
            -Params @{ id = "e2e140-$RunId-$($s[0])"; repo = (Join-Path $Scratch 'repo-a'); text = "E2E140-$RunId-legacy-$($s[0])"; cid = $s[1] } | Out-Null
    }
    $c.steps += Invoke-Agm @('prompts', '--json') -Env @{ ABV_DATA_DIR = $sb }
    $c.sql += Invoke-Sql -Db $sbRepo -Sql "SELECT id, instance_id, status, status_reason FROM active_prompts WHERE id LIKE :p" -Params @{ p = "e2e140-$RunId-l%" }
    $c.sql += Invoke-Sql -Db $sbRepo -Sql $Q.Q11
    $c.sql += Invoke-Sql -Db $sbRepo -Sql "SELECT name FROM agm_schema_migrations"
    $c.files += [pscustomobject]@{ path = "$RunRel/sandbox-data/repo_prompts.db.pre-v140.bak"; exists = (Test-Path "$sbRepo.pre-v140.bak") }
    $c.steps += Invoke-Agm @('qs', '--once') -Env @{ ABV_DATA_DIR = $sb } -TimeoutSec 300
    $c.sql += Invoke-Sql -Db $sbRepo -Sql "SELECT id, instance_id, status, status_reason FROM active_prompts WHERE id LIKE :p" -Params @{ p = "e2e140-$RunId-l%" }
    $c.steps += Invoke-Agm @('prompts', '--json') -Env @{ ABV_DATA_DIR = $SbFull }
    $c.sql += Invoke-Sql -Db (Join-Path $SbFull 'repo_prompts.db') -Sql $Q.Q11
    $c.files += [pscustomobject]@{ path = '%USERPROFILE%/.antigravity_tools/repo_prompts.db'; last_write_before = $Snapshot.repo_db_last_write; last_write_after = (Get-Item $RepoDb).LastWriteTimeUtc.ToString('o') }
    # decide per spec E2E-14, then:
    Save-Case $c 'PASS' 'l1 owner O1, l2 and l3 orphaned, Q11 0 on both copies, bak present'
}
```

The `Save-Case` reason above is the text to use on PASS; on FAIL write which expected item failed.

### E2E-01 Feasibility gate

```powershell
$c = New-Case 'E2E-01' 'Antigravity honors the per-instance profile' @('AC-01') @() 'today'
$cr = Invoke-Agm @('instances', 'create', "test-cli-flow-a-$Rand4", '--data-only', '--json')
$c.steps += $cr
if ($cr.exit_code -ne 0 -or -not $cr.json.id) { Save-Case $c 'FAIL' 'could not create A'; throw 'STOP: E2E-01' }
$A = $cr.json.id
if (-not $A.StartsWith('test-cli-flow-')) { throw "unexpected id prefix: $A" }
$Run.instances.A = $A; Save-Run
$c.steps += Invoke-Agm @('instances', 'assign', $A, (Join-Path $Scratch 'repo-a'))
$c.steps += Invoke-Agm @('instances', 'launch', $A) -TimeoutSec 120
$obs = Wait-Until { $o = Invoke-Agm @('observe', $A, '--json'); if ($o.json.is_running) { $o } } 90 3
$c.steps += $obs
$pidA = @($obs.json.pids)[0]
$cmdA = [string](Get-CimInstance Win32_Process -Filter "ProcessId=$pidA").CommandLine
$c.l2_check = [ordered]@{ pid = $pidA; has_instance_data = $cmdA.Contains("instances\$A\data"); has_default_appdata = $cmdA.Contains("$env:APPDATA\Antigravity") }

$m1 = New-Marker $A 'E2E01a'
$Run.gates.G_AGY = [bool]$cr.json.bound_email
if ($Run.gates.G_AGY) {
    $c.steps += Invoke-Agm @('prompt', '-i', $A, (New-PromptText $m1)) -Cwd (Join-Path $Scratch 'repo-a')
    $hitsA = Wait-Until { $h = Find-Marker $A $m1; if ($h.Count) { , $h } } 180 5
    $hitsG = Find-Marker 'GLOBAL' $m1
    $c.path1 = [ordered]@{ marker = $m1; hits_in_A = $hitsA; hits_in_global = $hitsG }
    foreach ($cid in @($hitsA | ForEach-Object { $_.cid } | Select-Object -Unique)) {
        foreach ($d in Get-FlavorDirs 'GLOBAL') {
            $c.sql += Invoke-Sql -Db (Join-Path $d 'conversation_summaries.db') -Sql $Q.Q10 -Params @{ cid = $cid }
        }
        $c.files += [pscustomobject]@{ check = 'global brain folder'; cid = $cid; exists = @(Get-FlavorDirs 'GLOBAL' | Where-Object { Test-Path (Join-Path $_ "brain\$cid") }).Count -gt 0 }
    }
    $Run.cidA1 = @($hitsA | ForEach-Object { $_.cid })[0]
}
Save-Run
```

Path 2 (IDE flavor) needs a human or UI tool: ask them to type `New-PromptText (New-Marker $A 'E2E01b')` into the agent panel of A's window (the window whose title shows repo-a), then run `Find-Marker $A $m2` and `Find-Marker 'GLOBAL' $m2` with the same 180 s wait. Decide per spec E2E-01. On FAIL: `Save-Case $c 'FAIL' '<reason>'`, then run E2E-19 and stop. On PASS with only path 1: `Save-Case $c 'PASS' 'cli flavor; IDE flavor BLOCKED needs_human_ui'`.

### E2E-02 Canonical key resolution

```powershell
$c = New-Case 'E2E-02' 'Canonical key resolution' @('AC-02', 'AC-03', 'AC-04') @('B01', 'B02', 'B03') 'today-baseline'
$crB = Invoke-Agm @('instances', 'create', "test-cli-flow-b-$Rand4", '--data-only', '--json')
$c.steps += $crB; $B = $crB.json.id; $Run.instances.B = $B; Save-Run
$c.steps += Invoke-Agm @('observe', 'default', '--json')
$c.steps += Invoke-Agm @('observe', '__default__', '--json')
$c.steps += Invoke-Agm @('observe', 'nope-0000', '--json')
$c.files += [pscustomobject]@{ path = '<instances>/nope-0000'; exists = (Test-Path (Join-Path $InstancesDir 'nope-0000')) }
$c.steps += Invoke-Agm @('observe', $Rand4, '--json')
if ($Run.gates.G_BUILD_03) {
    $c.steps += Invoke-Agm @('prompts', '-i', 'default', '--json')
    $c.steps += Invoke-Agm @('prompts', '-i', '__default__', '--json')
    $c.steps += Invoke-Agm @('prompt', 'enqueue', '-i', 'nope-0000', '--repo', (Join-Path $Scratch 'repo-a'), (New-PromptText (New-Marker 'nope-0000' 'E2E02d')))
    $c.files += [pscustomobject]@{ path = '<instances>/nope-0000'; exists_after_enqueue = (Test-Path (Join-Path $InstancesDir 'nope-0000')) }
    $before = @((Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $A }).rows, (Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $B }).rows)
    $c.steps += Invoke-Agm @('brp', '-i', $Rand4, '--json')
    $after = @((Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $A }).rows, (Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $B }).rows)
    $c.backup_unchanged = (($before | ConvertTo-Json -Depth 6) -eq ($after | ConvertTo-Json -Depth 6))
}
# decide per spec E2E-02; on an unfixed build use BASELINE
Save-Case $c $(if ($Run.gates.G_BUILD_03) { 'PASS' } else { 'BASELINE' }) 'see steps'
```

Replace the verdict with FAIL when any after-fix expectation in the spec does not hold (for example `exit_code` 0 for `nope-0000`, or the folder exists).

### E2E-03 Single instance running detection

```powershell
$c = New-Case 'E2E-03' 'Running detection, single instance' @('AC-19', 'AC-20') @('B05', 'B11') 'today-baseline'
function Get-Wpr([string]$Inst) {
    if ($Run.gates.G_BUILD_03) { $r = Invoke-Agm @('wpr', '-i', $Inst, '--json') } else { $r = Invoke-Agm @('wpr', '--json') }
    $items = if ($r.json -is [array]) { $r.json } elseif ($r.json.prompts) { $r.json.prompts } elseif ($r.json.rows) { $r.json.rows } else { @($r.json) }
    $rows = @($items) | Where-Object { $_.instance_id -eq $Inst }
    [pscustomobject]@{ step = $r; rows = @($rows) }
}
$idle = Wait-Until { $h = Find-Marker $A $m1; $s = @($h | Where-Object { $_.store -eq 'summaries' }); if ($s.Count -and -not ($s | Where-Object { $_.not_fully_idle -gt 0 })) { $true } } 300 10
Start-Sleep -Seconds 150
$w1 = Get-Wpr $A; $c.steps += $w1.step; $c.wpr_idle = $w1.rows
if ($Run.gates.G_LEGACY) {
    $c.steps += Invoke-Agm @('brp', '-i', $A, '--json') -TimeoutSec 300
    $c.sql += Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $A }
    $c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q02 -Params @{ inst = $A; marker = $m1 }
} else { $c.notes += 'brp step BLOCKED: legacy_rows_present' }
$m3 = New-Marker $A 'E2E03a'
$c.steps += Invoke-Agm @('prompt', '-i', $A, (New-PromptText $m3)) -Cwd (Join-Path $Scratch 'repo-a')
$seen = Wait-Until { $w = Get-Wpr $A; if ($w.rows | Where-Object { $_.status -eq 'running' }) { $w } } 120 2
$c.wpr_running = if ($seen) { $seen.rows } else { 'not observed' }
Wait-Until { $h = Find-Marker $A $m3; $s = @($h | Where-Object { $_.store -eq 'summaries' }); if ($s.Count -and -not ($s | Where-Object { $_.not_fully_idle -gt 0 })) { $true } } 300 10 | Out-Null
Start-Sleep -Seconds 150
$w2 = Get-Wpr $A; $c.steps += $w2.step; $c.wpr_after_idle = $w2.rows
$c.steps += Invoke-Agm @('instances', 'stop', $A)
Wait-Until { $o = Invoke-Agm @('observe', $A, '--json'); if (-not $o.json.is_running) { $true } } 60 3 | Out-Null
$w3 = Get-Wpr $A; $c.steps += $w3.step; $c.wpr_stopped = $w3.rows
$c.steps += Invoke-Agm @('instances', 'launch', $A) -TimeoutSec 120
Wait-Until { $o = Invoke-Agm @('observe', $A, '--json'); if ($o.json.is_running) { $true } } 90 3 | Out-Null
# decide per spec E2E-03
Save-Case $c 'BASELINE' 'replace with PASS or FAIL on a build with subtasks 02 and 03'
```

### E2E-04 to E2E-07

Use the same pattern. Each block:

1. `New-Case` with the ID, title, AC and bug lists from the spec.
2. Launch B once (E2E-04): `Invoke-Agm @('instances','assign',$B,(Join-Path $Scratch 'repo-b'))`, `Invoke-Agm @('instances','launch',$B) -TimeoutSec 120`, wait for `observe` running, record the L2 check as in E2E-01.
3. Send each marked prompt with `Invoke-Agm @('prompt','-i',<inst>,(New-PromptText <marker>)) -Cwd <repo folder>`; record `$Run.cid...` from `Find-Marker`.
4. Run the commands listed in the spec case in order. Use `Get-Wpr` for listing. For today's build filter `wpr`, `running-projects`, and `tree all` JSON by `instance_id` in PowerShell; with `G_BUILD_03` call the `-i` form.
5. Capture the listed queries with `Invoke-Sql` (`$RepoDb` for Q01 to Q06, Q11 to Q17; `$BackupDb` for Q07, Q08, Q11B; summaries DBs through `Find-Marker` and Q10).
6. Files: for E2E-05 copy each version of `repo-shared\.antigravity_resume_task.json` to the run folder as `E2E-05-legacy-<n>.json` after `Protect-Text`, and record `LastWriteTimeUtc` of the legacy and per-instance hand-off files.
7. E2E-06: `agm instances stop $A` before the clone, `Invoke-Agm @('instances','create',"test-cli-flow-c-$Rand4",'--from',$A,'--data-only','--json')`, store `$Run.instances.C`, relaunch A, then Q10 on C's summaries with `$Run.cidA1`. If 0 everywhere, record BLOCKED `clone_does_not_copy_conversations` for the live part.
8. E2E-07 steps 2 to 4 and 5 run only when `G_LEGACY` is true; save `backup_batch_id` from each `brp --json` output (`$r.json.batch_id`, or the first `backup_batch_id` in Q07 created after `$c.since`).
9. `Save-Case` with the verdict per spec.

### E2E-08, E2E-10, E2E-15, E2E-16 (after subtasks 02 and 03)

```powershell
if (-not ($Run.gates.G_BUILD_02 -and $Run.gates.G_BUILD_03)) {
    foreach ($id in 'E2E-08', 'E2E-10', 'E2E-15', 'E2E-16') { Add-Report $id 'BLOCKED' 'needs_subtask_02_and_03' }
}
```

Today, run the sandbox baselines instead: rebuild `$SbData = New-Sandbox 'sandbox-data' -Sanitize` (it now keeps A and B rows), seed the rows the spec names (ids `e2e140-<runid>-q1`, `-b1`, `-r1`, `-b2`, `-q2`, `-f1`, `repo_path` under `$Scratch`), run the listed command with `-Env @{ ABV_DATA_DIR = $SbData }`, read the seeded rows back, and record `BASELINE`.

With the fixed build, E2E-08 is:

```powershell
$c = New-Case 'E2E-08' 'Enqueue via CLI to one instance' @('AC-28', 'AC-14', 'AC-15') @('B13', 'B25') 'after-03'
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q15 -Params @{ inst = $A }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q01N -Params @{ inst = $B }
$m8 = New-Marker $A 'E2E08a'
$c.steps += Invoke-Agm @('prompt', 'enqueue', '-i', $A, '--repo', (Join-Path $Scratch 'repo-a'), (New-PromptText $m8), '--json')
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q13 -Params @{ inst = $A; marker = $m8 }
$c.steps += Invoke-Agm @('qs', $A, '--once') -TimeoutSec 300
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q13 -Params @{ inst = $A; marker = $m8 }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q15 -Params @{ inst = $A }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q01N -Params @{ inst = $B }
$c.hits_A = Wait-Until { $h = Find-Marker $A $m8; if ($h.Count) { , $h } } 180 5
$c.hits_B = Find-Marker $B $m8
$c.hits_global = Find-Marker 'GLOBAL' $m8
# decide per spec E2E-08
Save-Case $c 'PASS' 'one queued row, dispatched once, only A received the marker, A backed_up and B rows unchanged'
```

E2E-10, E2E-15, E2E-16 follow their spec steps with the same helpers. For E2E-15 start the loop as this run's own child and keep its PID:

```powershell
$qs = Start-Process -FilePath $Agm -ArgumentList "qs $A" -WorkingDirectory $Scratch -NoNewWindow -PassThru `
    -RedirectStandardOutput (Join-Path $RunDir 'raw\e2e15-qs.out') -RedirectStandardError (Join-Path $RunDir 'raw\e2e15-qs.err')
$Run.qs_pid = $qs.Id; Save-Run
# poll Q13 every 1 s up to 120 s, then:
Stop-Process -Id $Run.qs_pid -Force
Test-Safety 'E2E-15' | Out-Null
```

For E2E-16 rename with `Rename-Item (Join-Path $Scratch 'repo-gone') 'repo-gone-moved'` after the enqueue and before `qs`.

### E2E-09 UI enqueue (human or UI tool)

1. Gate: `G_BUILD_03` true and the running AGM window is a build with subtask 03 (ask the user or read its version in the About panel). Never start another AGM window.
2. Record `LastWriteTimeUtc` (or absence) of `repo-a\.antigravity_resume_task.json` and `repo-a\.antigravity_resume_task.<A>.json`.
3. Give the human the exact text `New-PromptText (New-Marker $A 'E2E09a')` and the click path from spec E2E-09 step 2. Remind them not to click the Google data-collection checkbox.
4. After they confirm: `Invoke-Agm @('prompts','queue','ls','-i',$A,'--json')`, Q13 with that marker, re-check the two timestamps, compare the row with the E2E-08 row.
5. `Save-Case` PASS, FAIL, or BLOCKED `needs_human_ui`.

### E2E-11 Account switch on A (after subtasks 02 and 03)

```powershell
$c = New-Case 'E2E-11' 'Instance account switch on A' @('AC-17', 'AC-18', 'AC-32', 'AC-12', 'AC-25') @('B15', 'B16', 'B29', 'B30', 'B31', 'B23') 'after-02-03'
$reg = Get-Content (Join-Path $InstancesDir 'instances.json') -Raw | ConvertFrom-Json
$boundIds = @($reg.instances | ForEach-Object { $_.bound_account_id } | Where-Object { $_ })
$accJson = ConvertFrom-AgmJson ((& $Agm accounts --json) | Out-String)
$accList = if ($accJson.accounts) { @($accJson.accounts) } else { @($accJson) }
$cand = $accList | Where-Object { $_.id -and ($boundIds -notcontains $_.id) } | Select-Object -First 1
$accJson = $null; $accList = $null
if (-not $cand) { Save-Case $c 'BLOCKED' 'no_unbound_account'; return }
$c.account_id = $cand.id
$c.account_email_masked = Protect-Text ([string]$cand.email)
$m11 = New-Marker $A 'E2E11a'
$c.steps += Invoke-Agm @('prompt', '-i', $A, (New-PromptText $m11)) -Cwd (Join-Path $Scratch 'repo-a')
$h = Wait-Until { $x = Find-Marker $A $m11; if ($x.Count) { , $x } } 180 5
$cidA = @($h | ForEach-Object { $_.cid })[0]
$pre = [ordered]@{ instances = @(Get-Instances | Select-Object id, running, pid); procs = @(Get-MainProcs) }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q01N -Params @{ inst = $A }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q01N -Params @{ inst = $B }
$c.sql += Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $B }
$c.steps += Invoke-Agm @('instances', 'switch', $A, $c.account_id, '--json') -TimeoutSec 300
Test-Safety 'E2E-11' | Out-Null
$post = [ordered]@{ instances = @(Get-Instances | Select-Object id, running, pid) }
$c.pids = [ordered]@{ before = $pre.instances; after = $post.instances }
# if A is not running again: Invoke-Agm @('instances','launch',$A); if nothing was restored: Invoke-Agm @('rrp','-i',$A,'--json') once
$c.sql += Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $A }
$c.sql += Invoke-Sql -Db $BackupDb -Sql $Q.Q07 -Params @{ inst = $B }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q01N -Params @{ inst = $A }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q01N -Params @{ inst = $B }
$hand = Join-Path $Scratch "repo-a\.antigravity_resume_task.$A.json"
if (Test-Path $hand) { $c.handoff = Protect-Text (Get-Content $hand -Raw) | ConvertFrom-Json }
$c.steps += Invoke-Agm @('prompts', 'trace', '-i', $A, '--conversation', $cidA, '--json')
$c.steps += Invoke-Agm @('history', '--json')
$c.hits_A = Wait-Until { $x = Find-Marker $A $m11; if ($x.Count -gt 1) { , $x } } 180 5
$c.hits_B = Find-Marker $B $m11
$c.watched_reinject = $false
Save-Run
# decide per spec E2E-11
Save-Case $c 'PASS' 'only A closed, A rows attempts<=1, B unchanged, hand-off session equals conversation'
```

The account list is read into memory only (never through `Invoke-Agm`, so it never lands in `raw/`), and only the chosen `id` and the masked email go into the case file. The chosen account is bound to no instance in `instances.json`. Run this block as the body of a script block or function so `return` ends only this case. Set `watched_reinject = $true` only if a human or a screenshot saw the restored prompt arrive.

### E2E-12 Auto-switch

```powershell
$c = New-Case 'E2E-12' 'Auto-switch, scoped to A' @('AC-23', 'AC-24') @('B21', 'B22') 'after-02'
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q15 -Params @{ inst = $A }
$c.sql += Invoke-Sql -Db $RepoDb -Sql $Q.Q15 -Params @{ inst = $B }
$c.steps += Invoke-Agm @('instances', 'ff', $A, '--json') -TimeoutSec 300
Test-Safety 'E2E-12' | Out-Null
$c.steps += Invoke-Agm @('history', '--json')
# read source and target of the newest rotation entry; if the target is default or a protected id: ABORT
# same-instance branch: Q15 for B unchanged, Q01N for A attempts<=1
# cross-instance branch: Q17 with :other = B for each A cid = 0, then relaunch A and re-check
Push-Location (Join-Path $RepoRoot 'src-tauri')
$c.unit = (cargo test --test switch_flow_instance_scope_test 2>&1 | Out-String)
Pop-Location
$c.unit = Protect-Text $c.unit
Save-Case $c 'PASS' 'live branch matched, integration tests green'
```

### E2E-13 Default instance paths (unit only)

```powershell
$c = New-Case 'E2E-13' 'Default instance paths, unit only' @('AC-23', 'AC-25', 'AC-02') @('B21', 'B23') 'unit-only'
Push-Location (Join-Path $RepoRoot 'src-tauri')
$c.unit_switch = Protect-Text (cargo test --test switch_flow_instance_scope_test 2>&1 | Out-String)
$c.unit_key = Protect-Text (cargo test --test canonical_instance_key_test 2>&1 | Out-String)
Pop-Location
Save-Case $c $(if ($c.unit_switch -match 'test result: ok' -and $c.unit_key -match 'test result: ok') { 'PASS' } else { 'FAIL' }) 'targeted test files'
```

When the test files do not exist yet, record BLOCKED `needs_subtask_02`.

### E2E-17 Refusal (sandbox mode)

```powershell
$c = New-Case 'E2E-17' 'Refusal without -i when two instances run' @('AC-26') @('B25', 'B27', 'B31') 'after-03-sandbox'
Push-Location (Join-Path $RepoRoot 'src-tauri')
$c.unit = Protect-Text (cargo test --test cli_instance_parity_test action_commands_refuse_without_instance_when_two_running 2>&1 | Out-String)
Pop-Location
if (-not $Run.gates.G_BUILD_03) { Save-Case $c 'BLOCKED' 'needs_subtask_03' }
elseif ($c.unit -notmatch 'test result: ok\. [1-9]') { Save-Case $c 'BLOCKED' 'refusal unit test not green; live refusal not attempted' }
else {
    $sb = New-Sandbox 'sandbox-data' -Sanitize
    $db = Join-Path $sb 'repo_prompts.db'; $bdb = Join-Path $sb 'backup-prompts\backup-prompts.db'
    $c.sql += Invoke-Sql -Db $db -Sql "SELECT COUNT(*) AS n, MAX(updated_at) AS last FROM active_prompts"
    $c.sql += Invoke-Sql -Db $bdb -Sql "SELECT COUNT(*) AS n FROM prompt_backups"
    foreach ($cmd in @(@('brp'), @('rrp'), @('rrc'))) { $c.steps += Invoke-Agm $cmd -Env @{ ABV_DATA_DIR = $sb } -TimeoutSec 120 }
    $c.steps += Invoke-Agm @('prompt', (New-PromptText (New-Marker 'none' 'E2E17a'))) -Cwd (Join-Path $Scratch 'repo-shared') -Env @{ ABV_DATA_DIR = $sb }
    $c.sql += Invoke-Sql -Db $db -Sql "SELECT COUNT(*) AS n, MAX(updated_at) AS last FROM active_prompts"
    $c.sql += Invoke-Sql -Db $bdb -Sql "SELECT COUNT(*) AS n FROM prompt_backups"
    $ok = -not ($c.steps | Where-Object { $_.exit_code -ne 2 })
    Save-Case $c $(if ($ok) { 'PASS' } else { 'FAIL' }) 'exit codes and counts recorded'
}
```

### E2E-18 Trace

```powershell
$c = New-Case 'E2E-18' 'Trace identity chain' @('AC-30') @() 'after-03'
$t = Invoke-Agm @('prompts', 'trace', '-i', $A, '--conversation', $cidA, '--json'); $c.steps += $t
$keys = 'source_dir', 'conversation_id', 'repo_path', 'row_id', 'status', 'status_history', 'backup_row_id', 'handoff', 'dispatch_result'
$first = @($t.json)[0]
$c.missing_keys = @($keys | Where-Object { -not ($first.PSObject.Properties.Name -contains $_) })
$c.steps += Invoke-Agm @('prompts', 'trace', '-i', $B, '--conversation', $cidA, '--json')
# cross-check row_id in Q01N for A, backup_row_id in Q07 for A, handoff.path exists
Save-Case $c $(if ($c.missing_keys.Count -eq 0) { 'PASS' } else { 'FAIL' }) 'keys and cross-checks'
```

### E2E-19 Cleanup

```powershell
$c = New-Case 'E2E-19' 'Cleanup' @('AC-32') @() 'today'
foreach ($id in $Run.instances.Values) {
    if (-not $id.StartsWith('test-cli-flow-')) { throw "refusing to touch $id" }
    if ($ProtectedIds -contains $id) { throw "refusing to touch protected $id" }
    $c.steps += Invoke-Agm @('instances', 'stop', $id)
    Wait-Until { $o = Invoke-Agm @('observe', $id, '--json'); if (-not $o.json.is_running) { $true } } 60 3 | Out-Null
}
$ids = @($Run.instances.Values)
$params = @{}; $names = @()
for ($i = 0; $i -lt $ids.Count; $i++) { $params["i$i"] = $ids[$i]; $names += ":i$i" }
$in = $names -join ', '
foreach ($t in 'active_prompts', 'running_projects', 'agm_conversation_sequences') {
    $c.sql += Invoke-Sql -Db $RepoDb -Sql "SELECT COUNT(*) AS n FROM $t WHERE instance_id IN ($in)" -Params $params
    $c.sql += Invoke-Sql -Db $RepoDb -Write -LiveCleanup -Sql "DELETE FROM $t WHERE instance_id IN ($in)" -Params $params
}
if (Test-Path $BackupDb) {
    $c.sql += Invoke-Sql -Db $BackupDb -Sql "SELECT COUNT(*) AS n FROM prompt_backups WHERE instance_id IN ($in)" -Params $params
    $c.sql += Invoke-Sql -Db $BackupDb -Write -LiveCleanup -Sql "DELETE FROM prompt_backups WHERE instance_id IN ($in)" -Params $params
}
foreach ($id in $ids) { $c.steps += Invoke-Agm @('instances', 'rm', $id, '--force') }
Remove-Item -Recurse -Force $Scratch, (Join-Path $RunDir 'sandbox-full'), (Join-Path $RunDir 'sandbox-data') -ErrorAction SilentlyContinue
$final = @(Get-Instances)
$c.final_instances = @($final | Where-Object { $_.is_default -or $ProtectedIds -contains $_.id } | Select-Object id, running, pid)
$c.leftover_run_ids = @($final | Where-Object { $ids -contains $_.id } | ForEach-Object { $_.id })
$c.start_instances = $Snapshot.instances
Save-Case $c $(if ($c.leftover_run_ids.Count -eq 0) { 'PASS' } else { 'FAIL' }) 'run instances removed; protected processes unchanged'
```

A protected instance whose registry `pid` differs from the start while its original process (same PID and start time in `Get-MainProcs`) is still alive is a PID refresh, not a failure; note it. A protected process that is gone has already aborted the run in `Test-Safety`.

## 6. Evidence format

One JSON file per case at `<runfolder>/<ID>.json`:

```json
{
  "id": "E2E-07",
  "title": "Backup isolation and idempotence, no steal",
  "ac": ["AC-09", "AC-10"],
  "bugs": ["B06", "B18", "B19", "B20"],
  "mode": "today-baseline",
  "run_id": "<runid>",
  "started_at": "<iso time>",
  "finished_at": "<iso time>",
  "since": 0,
  "steps": [
    { "cmd": "agm brp -i test-cli-flow-a-<rand4>-<ts4> --json", "cwd": "%TEMP%\\agm-e2e140-<runid>\\repo-shared", "exit_code": 0,
      "stdout_file": ".ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/raw/<tag>.out", "stdout_excerpt": "...", "json": {} }
  ],
  "sql": [
    { "db": "%USERPROFILE%\\.antigravity_tools\\backup-prompts\\backup-prompts.db", "sql": "SELECT ... WHERE instance_id = :inst ...",
      "params": { "inst": "test-cli-flow-a-<rand4>-<ts4>" }, "rows": [ { "instance_id": "test-cli-flow-a-<rand4>-<ts4>" } ] }
  ],
  "files": [ { "path": "%TEMP%\\agm-e2e140-<runid>\\repo-shared\\.antigravity_resume_task.json", "exists": true, "last_write": "<iso time>" } ],
  "notes": [],
  "result": "PASS",
  "reason": "S1 equals S2, batch A only A rows, batch B only B rows, 07b two rows"
}
```

Also in the run folder: `run.json` (run id, instance roles and ids, gates), `raw/` (masked stdout and stderr of every `agm` call), `report.txt`.

## 7. Report format

`report.txt` has one line per case, in run order:

```text
E2E-00 | PASS | .ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/E2E-00.json | procs=4 G_LEGACY=True
E2E-01 | PASS | .ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/E2E-01.json | cli flavor; IDE flavor BLOCKED needs_human_ui
E2E-08 | BLOCKED | .ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/E2E-08.json | needs_subtask_02_and_03
```

Result values: `PASS`, `FAIL`, `BLOCKED` (with a reason code: `needs_subtask_0N`, `needs_human_ui`, `legacy_rows_present`, `feasibility_failed`, `clone_does_not_copy_conversations`, `no_bound_account`), `BASELINE` (today-baseline run on an unfixed build; the reason states the observed bug ID), `ABORTED`.

When the run ends, tell the user: the run id, the path of `report.txt` (relative), the count per result, every FAIL with its one-line reason, and whether every protected PID stayed unchanged. Never say a reinject was watched unless `watched_reinject` is true in that case file.

## 8. Done when

- `report.txt` has a line for E2E-00 to E2E-19.
- Every FAIL names the expected item that failed and the evidence file.
- E2E-19 shows no leftover run ids and no protected process change.
