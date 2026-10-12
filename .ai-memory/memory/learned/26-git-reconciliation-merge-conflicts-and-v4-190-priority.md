# Git Reconciliation, Conflict Resolution, and v4.190.0 Priority Protocol

- **Date:** 2026-10-12
- **Related Commits:** `395f7597`, `92d51e8e`
- **Related Releases:** v4.190.0 (Production Stable Baseline)

## 1. Context & Divergence State

During active development on branch `main`, the remote repository `origin/main` was 7 commits ahead, containing the stable production release `v4.190.0` along with:
1. Account switch fast-forward and CLI downgrade/revert fixes (`183a0a3e`, `b633e9dc`).
2. Delegate updater and update command alignment and rustfmt passes (`de1b28fe`, `064f21ba`).
3. Royal Violet UI design token theme integration across views and modals (`72282ca3`).
4. v4.190.0 version bump across 14 manifests and changelogs (`016d3cb3`, `287bb4af`).

Concurrently, local development contained pending uncommitted work for Plan 157 (`verify-running-prompts-instance-prompts-section`):
- `src-tauri/src/modules/repo_db/`: Default host gemini directory discovery, authoritative `not_fully_idle > 0` supremacy over stale status strings, step completion vs. turn completion differentiation in transcript inspection, and 600s adaptive thinking window.
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`: Deep-link auto-expansion for `initialSelectedProjectId`, running subagent parent preservation in filters, and running project elevation.
- `scripts/test-running-prompts-e2e.ps1`: End-to-end PowerShell verification suite.

To safely resolve the divergence without discarding local work, the local changes were committed atomically as:
- Commit `395f7597`: `feat(repo_db): verify running prompts in instance prompts section and improve liveness detection`

## 2. Conflicts Encountered & Resolution Strategy

When executing `git merge origin/main`, Git encountered a conflict in `package-lock.json` and auto-merged `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`.

### Conflict A: `package-lock.json`
- **Conflict Details:**
  Both branches modified the root package descriptor version:
  - `HEAD` (local): `"version": "4.189.0"` (with duplicate entries pruned).
  - `origin/main` (remote): `"version": "4.190.0"` with legacy duplicate `"version": "4.175.0"` and `"version": "4.166.0"` entries from previous branches.
- **Resolution:**
  1. Strictly enforced **v4.190.0 Top Priority**: The release version monotonic constraint requires the highest production version (`4.190.0`) to win.
  2. Pruned the redundant, duplicated `"version"` fields from both `package-lock.json` root object and `packages[""]` dictionary.
  3. Confirmed zero remaining conflict markers with `git diff --check`.

### Merge Inspection B: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- **Details:**
  Git automatically integrated the two non-conflicting sections:
  - Lines 268-343: `origin/main` applied the Royal Violet palette (`dark:bg-[#171025]`, `dark:border-[#40285e]`).
  - Lines 66-230: Local commit applied deep-link auto-expansion, running subagent parent preservation, and sorting.
- **Resolution:**
  Verified that both changes coexist without syntax or logical collisions. The Royal Violet dark background tokens and border tokens were preserved intact, and deep-link expansion operates seamlessly.

## 3. Strict Invariant: v4.190.0 Logic Priority

As directed by maintainer guidelines and explicit user instruction:
> "The last release was good enough... 4.190 is one of the release that is working fine. So any changes that actually overrides the logic of this should not be taken into priority because the release for the 190 is actually kind of stable. Auto switcher works, switching accounts works, password works... don't take anything other than 190. The 190 would be high priority."

### Preserved Subsystems & Verification:
1. **Auto-Switcher & Scoring (`src/services/instanceService/scoring.ts`, `src-tauri/src/modules/auto_switcher/`):**
   - 100% untouched by local changes; v4.190.0 scoring algorithm, quota thresholds, and candidate ranking remain in full effect.
2. **Account Switching & Fast-Forward (`src-tauri/src/modules/instance/switch.rs`, `process_cache.rs`):**
   - v4.190.0 PID isolation and fast-forward dispatch logic remain completely intact.
3. **CLI Updater Downgrade Pipeline (`src-tauri/src/bin/agm/update_cmds.rs`, `modules/delegate_updater/`):**
   - Explicit version revert and downgrade support from v4.190.0 preserved without alteration.
4. **Desktop UI Theming (`src/styles/ui-tokens.css`, `tailwind.config.js`, `App.css`):**
   - Royal Violet palette configuration remains active across all instance and account components.

## 4. Build & Verification Gate

1. **Frontend Compilation:**
   Executed `npm run build` (`tsc && vite build`).
   Result: Built successfully in 14.99s with code 0 (`dist/` generated cleanly).
2. **Rust Formatting Gate:**
   Executed `cargo fmt -- --check` in `src-tauri`.
   Result: Code formatting clean with code 0.
3. **Merge Commit:**
   Concluded merge with commit `92d51e8e`: `Merge remote-tracking branch 'origin/main' into main (preserving v4.190.0 release priority)`.
