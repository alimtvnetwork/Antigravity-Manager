# AI Implementation Instructions: Auto-Switcher Candidate Scoring Fallback & CLI Update Downgrade/Revert Remediation

- **Task Identifier**: `auto-switch-and-rotate-fix`
- **Specification Reference**: `02-spec/21-app/auto-switch-and-rotate-fix/02-component-spec.md`
- **Applicable Subtasks**:
  - `.ai-memory/plans/subtasks/auto-switch-and-rotate-fix/03-auto-switcher-candidate-scoring-and-fallback-fix.md`
  - `.ai-memory/plans/subtasks/auto-switch-and-rotate-fix/04-agm-update-downgrade-revert-fix.md`
- **Status**: `Ready for Execution`

---

## 1. Non-Negotiable Operational Directives

AI workers executing this task must strictly adhere to these architectural invariants:

1. **Relative Git Paths Only**: All file paths in documentation, code imports, comments, and instructions MUST be relative to the repository root (e.g., `src-tauri/src/modules/...`). Never emit absolute filesystem paths (such as `D:\...` or `C:\...`).
2. **Positive Booleans Only**: Every boolean identifier, field, and argument must have positive polarity (`is_active`, `is_relaunch`, `is_period_finished`, `has_valid_quota`, `is_downgrade`). Never introduce inverted or double-negative flags (`is_disabled`, `unhealthy`, `is_not_empty`).
3. **Zero Git Commands**: Do NOT execute git commands (`git add`, `git commit`, `git checkout`, `git diff`). You are editing and verifying source files only. Commits and release tags are managed by the lead orchestrator.
4. **No Raw Search Tools**: Do NOT invoke `rg`, `ripgrep`, `grep`, or `Select-String`. Use `view_file` or GitMap commands for discovery.
5. **Zero Silent Errors**: All fallible operations must be mapped to `AppError` or recorded via `crate::error::record_ignored` with an explicit `// Justification:` comment. Never leave a bare `let _ = ...`, `.ok()`, or empty `catch {}`.
6. **Zero-Build Policy During Editing**: Do NOT run full builds (`cargo build`, `tauri build`) while editing. Only run targeted unit tests, `cargo fmt -- --check`, and `cargo clippy --all-targets --all-features` during pre-flight verification.

---

## 2. Problem Summary & Root Cause Breakdown

### Problem 1: Auto-Switcher Rotation Lockup on Sub-100% Accounts
- **Symptom**: When candidate accounts in the fleet have healthy quota below 100% (e.g., 90%, 80%, 65%), the auto-switcher daemon fails to rotate a depleted instance. The instance remains stuck on an empty account.
- **Root Cause A (`selection.rs:254`)**: Step 3 fallback was guarded by `available_pool.is_empty() && cooldown_pool.is_empty()`. If any account was in cooldown, `cooldown_pool` was not empty, so Step 3 was never evaluated.
- **Root Cause B (`selection.rs:294` / `scoring.rs:205`)**: Step 3 called `score_candidate_account` instead of `score_candidate_account_fallback`. In `scoring.rs:205`, `score_candidate_account` strictly returns `0.0` for any account with `q_4h < 100.0`. Thus, all fallback candidates received a score of `0.0`.
- **Root Cause C (`selection.rs:370-528`)**: In `select_and_verify_next_best_profile`, when candidates were evaluated, if zero 100% accounts existed and fallback staging was bypassed or incomplete, the function returned `Ok(None)`. The caller in `daemon.rs` took no action.

### Problem 2: CLI Update Downgrade / Revert Argument Dropping
- **Symptom**: Running `agm update to v4.162` or `agm update 4.162` ignores the target version, queries `releases/latest`, falsely reports `[*] Already up to date! (v4.185.0)`, and refuses to downgrade.
- **Root Cause A (`update_cmds.rs:15-35`)**: `cmd_update` only parsed boolean flags (`--json`, `--check`, `--force`, `all`, `gitmap`, `ssh`). Positional version arguments (`"to"`, `"v4.162"`, `"4.162"`) were completely ignored.
- **Root Cause B (`update_cmds.rs:151-153`)**: Release metadata was unconditionally fetched from `releases/latest` rather than `releases/tags/v{version}`.
- **Root Cause C (`update_cmds.rs:217`)**: `is_up_to_date = tag_name == crate::common::VERSION;` halted execution when the latest version matched the currently installed version, preventing downgrade to older versions.
- **Root Cause D (`install.ps1:1455-1470`)**: The PowerShell installer executed an early return if `TargetVersion -le curr`, blocking explicit downgrade requests when `-Update` was provided.

---

## 3. Surgical Implementation Guide Across 5 Key Files

### File 1: `src-tauri/src/modules/auto_switcher/selection.rs`

#### Edit 1.1: Enable Step 3 Fallback when Available Pool is Empty
Locate line 254:
```rust
// BEFORE:
if available_pool.is_empty() && cooldown_pool.is_empty() {
```
Replace with:
```rust
// AFTER:
if available_pool.is_empty() {
```
*Rationale*: If no immediately available 100% account exists in `available_pool`, healthy partial-quota accounts (above the low threshold) must be evaluated as candidates before falling back to cooldown-violating accounts.

#### Edit 1.2: Ensure Fallback Scoring is Invoked in Step 3
Locate line 294:
```rust
// Ensure it calls score_candidate_account_fallback:
let score = score_candidate_account_fallback(acc, target_model, now_sec);
```
*Rationale*: `score_candidate_account_fallback` scales the weekly score proportionally by `(quota / 100.0)`, returning a positive score rather than clamping to `0.0`.

#### Edit 1.3: Hardening `select_and_verify_next_best_profile`
Locate line 388-396:
```rust
// Ensure log clarifies that candidates can include partial-quota accounts:
if candidates.is_empty() {
    logger::log_info(
        "[AutoSwitcher] No eligible candidate profiles found meeting quota threshold requirement.",
    );
    return Ok(None);
}
```
In lines 504-515, ensure fallback candidates are staged reliably:
```rust
if fresh_4h_quota > threshold && fresh_4h_quota > highest_fallback_quota {
    highest_fallback_quota = fresh_4h_quota;
    best_fallback = Some(ProfileCandidate {
        instance_id: candidate.instance_id,
        account_id: candidate.account_id,
        email: candidate.email,
        quota_percent: fresh_4h_quota,
        score: candidate.score,
    });
}
```
And verify that line 517 selects `best_fallback` when no 100% account was confirmed:
```rust
if let Some(fallback) = best_fallback {
    logger::log_info(&format!(
        "[AutoSwitcher] No 100.0% quota profile found; selecting best available candidate '{}' with {:.1}% quota.",
        fallback.email, fallback.quota_percent
    ));
    return Ok(Some(fallback));
}
```

---

### File 2: `src-tauri/src/modules/auto_switcher/scoring.rs`

#### Edit 2.1: Audit Scoring Formulas & Invariants
Verify `score_candidate_account` (`scoring.rs:191-210`):
- Strict 100% gate applies to primary selection: returns `0.0` if `q_4h < 100.0 && !is_period_finished`.
- Returns `calculate_weekly_base_score` when quota is 100%.

Verify `score_candidate_account_fallback` (`scoring.rs:216-238`):
- Evaluates `q_4h`: if `is_period_finished`, sets to `100.0`.
- Returns `0.0` if `q_4h <= 0.0`.
- Multiplies fractional quota by base weekly score: `(q_4h / 100.0) * base_score`.

Ensure unit test coverage in `scoring.rs` confirms:
1. `score_candidate_account` returns `0.0` for 95% quota account.
2. `score_candidate_account_fallback` returns `> 0.0` for 95% quota account.

---

### File 3: `src-tauri/src/bin/agm/update_cmds.rs`

#### Edit 3.1: Implement Version Token Extractor
Add helper function in `update_cmds.rs`:
```rust
/// Extract target version string from command-line arguments.
/// Supports: 'to v4.162', 'to 4.162', '4.162', 'v4.162', '--version 4.162', '-v 4.162'.
pub(crate) fn extract_target_version(args: &[String]) -> Option<String> {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        let clean = arg.trim();
        if clean.eq_ignore_ascii_case("to") {
            if let Some(next_arg) = iter.next() {
                let v = next_arg.trim().trim_start_matches(['v', 'V']);
                if !v.is_empty() && v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    return Some(v.to_string());
                }
            }
        } else if clean == "--version" || clean == "-v" {
            if let Some(next_arg) = iter.next() {
                let v = next_arg.trim().trim_start_matches(['v', 'V']);
                if !v.is_empty() && v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    return Some(v.to_string());
                }
            }
        } else if !clean.starts_with('-')
            && !clean.eq_ignore_ascii_case("all")
            && !clean.eq_ignore_ascii_case("gitmap")
            && !clean.eq_ignore_ascii_case("gm")
            && !clean.eq_ignore_ascii_case("ssh")
            && !clean.eq_ignore_ascii_case("export-zip")
            && !clean.eq_ignore_ascii_case("help")
        {
            let v = clean.trim_start_matches(['v', 'V']);
            if v.contains('.') && v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                return Some(v.to_string());
            }
        }
    }
    None
}
```

#### Edit 3.2: Dynamic Release Metadata Resolution
In `cmd_update`, call `extract_target_version`:
```rust
let requested_version = extract_target_version(args);
let has_explicit_target = requested_version.is_some();

let (api_url, updater_cdn_url) = if let Some(ref target) = requested_version {
    (
        format!("https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/tags/v{}", target),
        format!("https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v{}/updater.json", target),
    )
} else {
    (
        "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest".to_string(),
        "https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest/download/updater.json".to_string(),
    )
};
```

#### Edit 3.3: SemVer Comparison & Downgrade Logic
Update line 217 in `update_cmds.rs`:
```rust
let is_up_to_date = tag_name == crate::common::VERSION;
let is_downgrade = has_explicit_target && !is_up_to_date;

// When an explicit version target is passed and it does NOT equal current version,
// allow the operation to proceed (downgrade or upgrade) without requiring --force.
let should_proceed_update = (!is_up_to_date || is_force || has_explicit_target) && !is_check;
```
When invoking `delegate_updater::run` or `run_cli_update`, forward `tag_name` (which now correctly contains the requested version `4.162`):
```rust
let mut cli_update_args = vec![
    "--force".to_string(),
    "--no-launch".to_string(),
    "--version".to_string(),
    tag_name.clone(),
];
```

---

### File 4: `src-tauri/src/modules/delegate_updater/args.rs` & `run.rs`

#### Edit 4.1: Ensure `parse_delegate_args` Ingests Version
In `src-tauri/src/modules/delegate_updater/args.rs:6-58`:
Ensure `--version <ver>` correctly sets `opts.target_version = Some(ver)`.
If a positional token matches version pattern without flag, also capture it into `opts.target_version`.

#### Edit 4.2: Ensure `run_cli_update` Passes Version to PowerShell
In `src-tauri/src/modules/delegate_updater/run.rs:216-220`:
Ensure `ver_arg` is constructed and appended to PowerShell invocation:
```rust
let ver_arg = opts
    .target_version
    .as_ref()
    .map(|v| format!("-Version \"{}\"", v))
    .unwrap_or_default();
```
When `install.ps1` is executed, `-Version "<target>"` must be explicitly passed.

---

### File 5: `install.ps1`

#### Edit 5.1: Honor Explicit Version / Pinned Downgrades
Locate line 1455-1470 in `install.ps1`:
```powershell
# BEFORE:
if ($Update -and -not $Force) {
    $curr = Get-InstalledVersion
    if ($curr) {
        try {
            if ((Convert-ToSemVer $TargetVersion) -le (Convert-ToSemVer $curr)) {
                Write-Success "Already on the latest version ($curr)."
                return
            }
        } catch {
            if ($curr -eq $TargetVersion) {
                Write-Success "Already on the latest version ($curr)."
                return
            }
        }
    }
}
```
Replace with:
```powershell
# AFTER:
if ($Update -and -not $Force -and -not $isPinned) {
    $curr = Get-InstalledVersion
    if ($curr) {
        try {
            if ((Convert-ToSemVer $TargetVersion) -le (Convert-ToSemVer $curr)) {
                Write-Success "Already on the latest version ($curr)."
                return
            }
        } catch {
            if ($curr -eq $TargetVersion) {
                Write-Success "Already on the latest version ($curr)."
                return
            }
        }
    }
}
```
*Rationale*: When `$isPinned` is `$true` (the user or CLI explicitly specified `-Version "4.162"`), the installer must not short-circuit with "Already on the latest version". Operator intent to install the requested version must take precedence.

---

## 4. Verification & Quality Gates

Run these focused pre-flight checks once edits are applied:

1. **Rust Formatting**:
   ```pwsh
   cd src-tauri; cargo fmt -- --check
   ```
2. **Clippy Static Analysis**:
   ```pwsh
   cd src-tauri; cargo clippy --all-targets --all-features
   ```
3. **Auto-Switcher Targeted Unit Tests**:
   ```pwsh
   cd src-tauri; cargo test --package antigravity-tools-lib --lib modules::auto_switcher
   ```
4. **AGM Binary Unit Tests**:
   ```pwsh
   cd src-tauri; cargo test --bin agm
   ```

---

## 5. Evidence Verification Checklist

- [ ] `selection.rs`: Step 3 fallback triggers when `available_pool.is_empty()`.
- [ ] `selection.rs`: Step 3 invokes `score_candidate_account_fallback`.
- [ ] `scoring.rs`: `score_candidate_account_fallback` computes `(quota / 100.0) * base_score`.
- [ ] `selection.rs`: `select_and_verify_next_best_profile` returns `Ok(Some(fallback))` when quota > threshold.
- [ ] `update_cmds.rs`: `extract_target_version` successfully extracts `"4.162"` from `agm update to v4.162`.
- [ ] `update_cmds.rs`: Requests target tag `releases/tags/v4.162` rather than `releases/latest`.
- [ ] `update_cmds.rs`: Allows downgrade without exiting early on `is_up_to_date`.
- [ ] `install.ps1`: Does not block downgrade when `$isPinned` is true.
- [ ] Zero absolute paths in code or documentation.
- [ ] Positive booleans strictly enforced.
