# Subtask 03: Minor Version Bump Ceremony, Documentation Synchronization & Release Verification Gate

**Plan ID:** `.ai-memory/plans/subtasks/50-instant-rerun-enqueue-and-minor-release/03-subtask-minor-bump-and-release.md`  
**Parent Task:** `50-instant-rerun-enqueue-and-minor-release`  
**Worker Role:** Worker 03 (Release Governance & Verification Specialist)  
**Status:** Ready for Execution  
**Target Version:** `4.268.0` (Minor Release from `4.267.0`)  
**Specification Reference:** `02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/03-migration-and-verification-spec.md`  

---

## 1. Context & Objectives

This subtask executes the official release ceremony for Antigravity Manager (`agm`) **`v4.268.0`**, validating the core functionality delivered by Worker 01 and Worker 02 (Instant Rerun Enqueue, Reactive Scheduler Wakeup, Sub-10ms Enqueue IPC, Zero-Delay Idle Dispatch, and Rerun Enqueue Invariant), executing atomic version manifest synchronization, synchronizing changelogs and documentation, running preflight quality gates, creating git tags, and completing the release commit.

### Core Targets:
1. **Target Version**: `4.268.0` (bumped via `npm run bump minor`).
2. **15 Manifests & Source Files**:
   - `package.json`
   - `package-lock.json` (root and packages mirror)
   - `version.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.lock`
   - `Casks/antigravity-tools.rb`
   - `README.md`
   - `README_EN.md`
   - `src/components/layout/MiniView.tsx`
   - `src/pages/Settings.tsx`
   - `releases-manifest.json`
   - `CHANGELOG.md`
   - `CHANGELOG_EN.md`
   - `src-tauri/hooks.nsh`
3. **Documentation & Changelogs**:
   - `CHANGELOG_EN.md`: Awk-compatible release heading: `    *   **v4.268.0 (2026-10-25)**:`.
   - `CHANGELOG.md`: Markdown release heading: `## [v4.268.0] - 2026-10-25`.
   - `README.md`: Summary entry under `## 🔄 What's New`.
   - `README_EN.md`: Summary entry under `## 📝 Changelog`.
   - Contributor attribution strictly to `@aukgit` (`(Thanks to @aukgit)`).
4. **Preflight Quality Gates**:
   - Frontend Unit Tests: `npx tsx src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`
   - Rust Code Formatting: `cd src-tauri && cargo fmt -- --check`
   - Rust Compilation & Clippy: `cd src-tauri && cargo clippy --all-targets --all-features` (or `cargo check --lib`)
   - Frontend Production Build: `npm run build`
5. **Git Tagging & Commit**:
   - Git Tag: `v4.268.0`
   - Git Commit: `chore(release): bump version to 4.268.0 (instant rerun enqueue & latency elimination)` via `gitmap cpr`

---

## 2. Technical Invariants & Guardrails

1. **Rerun Enqueue Invariant**:
   - 100% of rerun rounds (1..N) must dispatch strictly via `enqueue_prompt`.
   - Dispatches must NEVER call `send_prompt_now`. Active tasks must not be preempted or cancelled by rerun operations.
2. **Awk-Compatible Invariant**:
   - In `CHANGELOG_EN.md`, the heading line must conform character-for-character to `    *   **v4.268.0 (2026-10-25)**:`.
   - The GitHub Actions release parser (`awk -v ver="$VERSION"`) matches `^[[:space:]]*[*+-][[:space:]]+\*\*ver` and extracts the section up to the next version heading.
3. **15-Manifest Synchronization Invariant**:
   - Zero version drift across all 15 target files. Running the manifest verification snippet must report 0 mismatches.
4. **Sole Contributor Attribution Invariant**:
   - Attribution in all changelogs, READMEs, and release notes must be strictly to `@aukgit` (`(Thanks to @aukgit)`).
   - Zero other `@...` GitHub user handles may appear anywhere in release text.
5. **Strictly Relative Git Paths Invariant**:
   - Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`, `/home/...`) in specs, plans, commit messages, or changelogs.
   - All paths must be relative Git paths (e.g., `src/...`, `src-tauri/...`, `02-spec/...`, `.ai-memory/...`).
6. **No Silent Errors Invariant**:
   - Every fallible operation in Rust must propagate with context via `AppError` or call `record_ignored` with a justification comment.
   - Every fallible operation in TypeScript must route through `useErrorStore.getState().trackWarning` or `captureError`.
7. **Caller IDE Self-Preservation Invariant**:
   - Automated tests and CLI operations must never stop, kill, or restart the active developer workspace instance.

---

## 3. Step-by-Step Implementation Tasks

### Step 1: Pre-Bump Unit Test & Functional Verification
Run the targeted rerun action unit tests to verify the Rerun Enqueue Invariant and attachment formatting:
```bash
npx tsx src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts
```
**Assertion Verification:**
- Assert `mockInvokeCalls.filter(c => c.cmd === 'send_prompt_now').length === 0`.
- Assert `mockInvokeCalls.filter(c => c.cmd === 'enqueue_prompt').length === 3` for multi-round rerun tests.
- All test suites pass cleanly with exit code 0.

### Step 2: Atomic Minor Version Bump Execution
Run the atomic version bump script:
```bash
npm run bump minor
```
**Expected Terminal Output:**
- `Starting version synchronization: 4.267.0 -> 4.268.0`
- `✓ Synchronized: package.json -> 4.268.0`
- `✓ Synchronized: package-lock.json -> 4.268.0`
- `✓ Synchronized: version.json -> 4.268.0`
- `✓ Synchronized: src-tauri/Cargo.toml -> 4.268.0`
- `✓ Synchronized: src-tauri/tauri.conf.json -> 4.268.0`
- `✓ Synchronized: src-tauri/Cargo.lock -> 4.268.0`
- `✓ Synchronized: Casks/antigravity-tools.rb -> 4.268.0`
- `✓ Synchronized: README.md -> 4.268.0`
- `✓ Synchronized: README_EN.md -> 4.268.0`
- `✓ Synchronized: src/components/layout/MiniView.tsx -> 4.268.0`
- `✓ Synchronized: releases-manifest.json -> 4.268.0`
- `✓ Synchronized: CHANGELOG.md -> 4.268.0`
- `✓ Synchronized: CHANGELOG_EN.md -> 4.268.0`
- `✓ Synchronized: src-tauri/hooks.nsh -> 4.268.0`

### Step 3: Manifest Verification Audit
Run the atomic manifest verification command to assert that all 15 locations reflect `4.268.0`:
```powershell
node -e "
const fs = require('fs');
const v = '4.268.0';
const checks = [
  ['package.json', JSON.parse(fs.readFileSync('package.json')).version],
  ['version.json', JSON.parse(fs.readFileSync('version.json')).version],
  ['tauri.conf.json', JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json')).version],
  ['releases-manifest.json', JSON.parse(fs.readFileSync('releases-manifest.json')).latest_version],
  ['Cargo.toml', fs.readFileSync('src-tauri/Cargo.toml', 'utf8').match(/\[package\][\s\S]*?version\s*=\s*\"([^\"]+)\"/)[1]],
  ['hooks.nsh', fs.readFileSync('src-tauri/hooks.nsh', 'utf8').match(/StrCpy \$0 \"([^\"]+)\"/)[1]]
];
let failed = false;
for (const [file, val] of checks) {
  if (val !== v) { console.error('Mismatch in ' + file + ': ' + val); failed = true; }
}
if (!failed) console.log('All manifests verified at ' + v);
else process.exit(1);
"
```

### Step 4: Documentation & Changelog Content Refinement
Edit and populate the generated skeletons in `CHANGELOG_EN.md`, `CHANGELOG.md`, `README.md`, and `README_EN.md`:

#### 1. In `CHANGELOG_EN.md`:
Under `*   **Version History**:`, refine the auto-inserted block to:
```markdown
    *   **v4.268.0 (2026-10-25)**:
        -   **[Instant Rerun Enqueue & Latency Elimination] Reactive Scheduler Wakeup, Sub-10ms Enqueue IPC & Zero-Delay Idle Dispatch (Thanks to @aukgit)**:
            -   **Description**: Eliminated queue dispatch latency for rerun and enqueued prompts across Antigravity Manager: replaced the 30-second scheduler polling sleep in `src-tauri/src/modules/scheduler.rs` with an event-driven `tokio::sync::Notify` reactive trigger (`wake_prompt_queue_scheduler`), waking the scheduler loop within sub-5ms upon any database insert; decoupled synchronous Chrome DevTools Protocol (CDP) retry loops from `enqueue_prompt_for_instance` in `src-tauri/src/modules/repo_db/agy.rs` to ensure sub-10ms IPC completion; dispatched prompts immediately (<50ms) when the target instance workspace is idle; strictly enforced the Rerun Enqueue Invariant across `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` and unit tests ensuring 100% of rerun rounds (1..N) stage via `enqueue_prompt` with 0 calls to `send_prompt_now`; and synchronized all 15 repository version manifests to `4.268.0`. (Thanks to @aukgit)
```

#### 2. In `CHANGELOG.md`:
Add entry at the top:
```markdown
## [v4.268.0] - 2026-10-25

### Added
- Reactive prompt queue scheduler wakeup via `tokio::sync::Notify` in `src-tauri/src/modules/scheduler.rs`, waking the background polling loop in <5ms whenever a prompt is enqueued (Thanks to @aukgit)
- Instant dispatch protocol in `src-tauri/src/modules/repo_db/agy.rs` and `dispatch.rs` for idle instance workspaces, dispatching enqueued prompts in <50ms without waiting for a scheduler polling cycle (Thanks to @aukgit)

### Fixed
- Eliminated 1.6s to 30s queue delays when rerunning or enqueuing prompts by removing synchronous CDP retry loops from the IPC path and terminating idle scheduler sleep (Thanks to @aukgit)
- Enforced strict Rerun Enqueue Invariant in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` and `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`, guaranteeing all rerun rounds (1..N) dispatch via `enqueue_prompt` and never preempt active work with `send_prompt_now` (Thanks to @aukgit)

### Changed
- Atomic minor version bump to `4.268.0` across 15 project manifests and code files via `scripts/bump-version.mjs` (Thanks to @aukgit)

---
```

#### 3. In `README.md` (under `## 🔄 What's New`):
Insert the new version block:
```markdown
- **v4.268.0** (2026-10-25):
  - **Instant Rerun Enqueue & Latency Elimination**: Eliminated queue latency for rerun and enqueued prompts by replacing the 30-second scheduler polling backoff in `src-tauri/src/modules/scheduler.rs` with an event-driven `tokio::sync::Notify` trigger (`wake_prompt_queue_scheduler`), waking the background loop in <5ms; decoupled synchronous CDP retries from `enqueue_prompt_for_instance` in `src-tauri/src/modules/repo_db/agy.rs` for sub-10ms IPC return; dispatched enqueued prompts immediately (<50ms) when target workspaces are idle; enforced the Rerun Enqueue Invariant ensuring all rerun rounds dispatch via `enqueue_prompt` without preemptive task cancellation; and synchronized all 15 project manifests to `4.268.0`. (Thanks to @aukgit)
```

#### 4. In `README_EN.md` (under `## 📝 Changelog`):
Insert the top summary quote:
```markdown
> Latest version **v4.268.0**: Instant Rerun Enqueue & Latency Elimination — Eliminated queue latency for rerun and enqueued prompts by replacing the 30s scheduler sleep in `src-tauri/src/modules/scheduler.rs` with event-driven `tokio::sync::Notify` reactive trigger waking in <5ms; decoupled synchronous CDP retries from `enqueue_prompt_for_instance` in `src-tauri/src/modules/repo_db/agy.rs` for sub-10ms IPC return; dispatched enqueued prompts immediately (<50ms) when target workspaces are idle; enforced Rerun Enqueue Invariant ensuring all rerun rounds dispatch via `enqueue_prompt` without preemptive task cancellation; and synchronized all 15 project manifests to `4.268.0`. (Thanks to @aukgit)
```

### Step 5: Preflight Rust & Frontend Quality Gates
Execute the comprehensive preflight quality gate:
```bash
# 1. Rust code formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy & compilation check
cargo clippy --all-targets --all-features

# 3. Return to root and execute frontend production build
cd ..
npm run build
```

### Step 6: Atomic Commit & Git Tag Creation
Stage and commit all changes, then apply the annotated tag:
```bash
# 1. Stage changes
git add package.json package-lock.json version.json src-tauri/Cargo.toml src-tauri/Cargo.lock \
        src-tauri/tauri.conf.json src-tauri/hooks.nsh Casks/antigravity-tools.rb \
        README.md README_EN.md CHANGELOG.md CHANGELOG_EN.md releases-manifest.json \
        src/components/layout/MiniView.tsx src/pages/Settings.tsx \
        src/components/instances/PromptTreeView/ \
        src-tauri/src/modules/scheduler.rs src-tauri/src/modules/repo_db/ \
        02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/ \
        .ai-memory/plans/subtasks/50-instant-rerun-enqueue-and-minor-release/

# 2. Atomic GitMap commit (or git commit)
git commit -m "chore(release): bump version to 4.268.0 (instant rerun enqueue & latency elimination)"

# 3. Create exact annotated tag
git tag v4.268.0
```

### Step 7: Post-Release Verification & Gate Confirmation
- Verify `git status` shows a clean working tree.
- Verify `git tag -l "v4.268.0"` confirms tag existence.
- Confirm ready for push to `main` branch (`git push origin main && git push origin v4.268.0`).

---

## 4. Verification Evidence & Quality Matrix

| Order | Verification Gate | Exact Command | Success Criteria |
|---|---|---|---|
| 1 | Rerun Test Suite | `npx tsx src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` | 14+ tests pass, 0 failures, 0 `send_prompt_now` |
| 2 | Version Bump Script | `npm run bump minor` | Returns 0, synchronized 15 targets to `4.268.0` |
| 3 | Manifest Parity Audit | Node verification snippet | All 15 manifests equal `4.268.0` |
| 4 | Rust Format Gate | `cd src-tauri && cargo fmt -- --check` | Exit code 0, zero formatting diffs |
| 5 | Rust Clippy Gate | `cd src-tauri && cargo clippy --all-targets --all-features` | Exit code 0, zero errors or warnings |
| 6 | Frontend Build Gate | `npm run build` | Clean `dist/` compilation, zero TS errors |
| 7 | Attribution Invariant | `git diff HEAD | grep "@"` | Sole attribution to `@aukgit`, zero third-party handles |
| 8 | Relative Paths Invariant | Check diff for absolute paths | Zero absolute filesystem paths |
| 9 | Git Tag Parity | `git tag -l "v4.268.0"` | Tag exists and matches changelog heading |

---

## 5. Rollback & Failure Recovery Ladder

If any gate in Step 5 fails or an unexpected error occurs during manifest synchronization:
1. **Manifest Rollback**:
   - If version bump produces a mismatch: `git checkout -- .` to restore the pre-bump working tree state.
2. **Clippy/Fmt Failure**:
   - Run `cd src-tauri && cargo fmt` to automatically resolve formatting anomalies.
   - For clippy warnings, resolve with grounded, type-safe fixes preserving `AppError` propagation.
3. **Frontend Build Failure**:
   - Inspect TypeScript compiler output. Fix missing types or broken imports without resorting to `any`.
4. **Tag Cleanup (if needed before push)**:
   - If tag `v4.268.0` was applied prematurely: `git tag -d v4.268.0`.
