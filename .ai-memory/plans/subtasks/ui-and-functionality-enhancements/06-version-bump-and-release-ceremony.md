# Subtask 06: Minor Version Bump & Release Ceremony (v4.207.0)

- **Task ID**: `Task-06`
- **Subtask Path**: `.ai-memory/plans/subtasks/ui-and-functionality-enhancements/06-version-bump-and-release-ceremony.md`
- **Specification**: `02-spec/21-app/ui-and-functionality-enhancements/02-backend-fleet-and-rca-spec.md`
- **Parent Plan**: `.ai-memory/plans/ui-and-functionality-enhancements.md`
- **Target Version**: `v4.207.0` (Minor Release from `v4.206.0`)
- **Release Channel**: `main` (Production Stable)
- **Status**: `READY_FOR_EXECUTION`
- **Lead / Assignee**: Worker 02 / Release Lead

---

## 1. Objectives & Governance Rules

This subtask governs the strict execution of pre-flight validation, atomic 14-location version bumping, documentation synchronization, and git release ceremony for Antigravity-Manager `v4.207.0`.

### Non-Negotiable Release Invariants (from `AGENTS.md`)
1. **Pre-flight Quality Triad**:
   - `cd src-tauri && cargo fmt -- --check` MUST return 0 diffs.
   - `cd src-tauri && cargo clippy --all-targets --all-features` MUST pass with 0 warnings and 0 errors.
   - `npm run build` MUST compile the React frontend cleanly with 0 TypeScript/Vite errors.
2. **Atomic Version Sync**:
   - Run `npm run bump minor` to advance from `v4.206.0` to `v4.207.0` across all 14 manifest files simultaneously.
3. **Strict Attribution Invariant**:
   - Audit Git history (`<last-tag>..HEAD`). Attribution in `changelog.md` and `changelog_en.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - **FORBIDDEN**: Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes.
4. **README Changelog Synchronization**:
   - For stable releases on `main`, release summaries MUST be synchronized across three files:
     - `changelog.md` and `changelog_en.md`
     - `README.md` (under `## 📝 更新日志`)
     - `README_EN.md` (under `## 📝 Changelog`)
   - Never update only `changelog.md` while leaving `README.md` / `README_EN.md` outdated.
5. **Release Channel Separation**:
   - Stable releases reside exclusively on `main`. Tags must match `changelog.md` headings character-for-character (`v4.207.0`).
6. **Thinking Cache Invalidation**:
   - Keep `SUGGESTION_DELETE_THINKING_STORE = false` in `src/components/common/SuggestionDeleteThinkingModal.tsx` for routine releases.

---

## 2. Release Manifest Locations (14 Locations)

The `npm run bump minor` script automatically synchronizes:
1. `package.json` (`version`)
2. `src-tauri/Cargo.toml` (`[package] version`)
3. `src-tauri/Cargo.lock`
4. `src-tauri/tauri.conf.json` (`version`)
5. `install.ps1` (`$TargetVersion`)
6. `install.sh` (`TARGET_VERSION`)
7. `scripts/test-instance-e2e.ps1`
8. `scripts/test-instance-isolation.ps1`
9. `changelog.md` (Skeleton header insertion)
10. `changelog_en.md` (Skeleton header insertion)
11. `README.md` (Version badge / notes)
12. `README_EN.md` (Version badge / notes)
13. `02-spec/21-app/ui-and-functionality-enhancements/00-master-audit-ledger.md`
14. `.ai-memory/plans/ui-and-functionality-enhancements.md`

---

## 3. Step-by-Step Execution Protocol

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Release Ceremony Workflow                       │
└────────────────────────────────────────────────────────────────────────┘

 Step 1: Pre-flight Verification Gate
   ├── cd src-tauri && cargo fmt -- --check
   ├── cd src-tauri && cargo clippy --all-targets --all-features
   └── npm run build
         │
         ▼ (All 3 Pass Cleanly)
 Step 2: Atomic Version Bump
   └── npm run bump minor  (4.206.0 -> 4.207.0)
         │
         ▼
 Step 3: Changelog & README Authoring
   ├── Populate changelog.md with UI & Backend enhancements (Thanks to @aukgit)
   ├── Populate changelog_en.md with English translations (Thanks to @aukgit)
   ├── Sync README.md under "## 📝 更新日志"
   └── Sync README_EN.md under "## 📝 Changelog"
         │
         ▼
 Step 4: Re-run Pre-flight Gate
   ├── cargo fmt & cargo clippy
   └── npm run build
         │
         ▼
 Step 5: GitMap Atomic Commit & Push
   ├── gitmap cpf "Release: release - bump version to 4.207.0 and update changelog (Thanks to @aukgit)"
   ├── git tag v4.207.0
   └── git push origin v4.207.0
```

---

## 4. Release Note Highlights (Draft for v4.207.0)

- **Window TitleBar Maximize Interaction**: Resolved header double-click event swallowing on Win32 desktops, enabling fluid maximize/restore window toggling without dragging interference.
- **Quota Progress Bar Refinement & Glow Reduction**: Reduced excessive neon glow effects, trimmed bar width by 10% to allocate space for account emails, and aligned 4 checkpoints at equal 20% intervals with unified green color styling.
- **Resizable Email Column**: Added interactive drag handle to the Account Table email header with bound limits `[220px, 550px]` and persistent localStorage state.
- **Compact Header Status Pill**: Merged loose `IDLE` status badge and `Log` button into a single unified 28px dark-glass segmented pill capsule.
- **Prompt Tree View Toolbar Compaction & New Conversation Flow**: Compacted action buttons into a contiguous segmented capsule, unified `Send` and `Enqueue` into a primary split button with dropdown, and introduced an inline "New Conversation" creation dialog with custom conversation title input.
- **Prompt Dispatch & FIFO Queue RCA Codification**: Codified poison-resilient mutexes, panic-safe async wrappers (`run_caught_prompt_task`), live CDP DOM fallback, and automated queue advancement hooks (`drain_next_queued_prompt_for_workspace`).
- **Fleet Remote Telemetry (JSON)**: Standardized multi-machine discovery, instance inventory, and account lease synchronization via pure JSON envelopes (`agm-fleet-sync-v1.0` and `[AGM IN-USE]`) across Supabase, Email, and Telegram channels.

---

## 5. Acceptance Checklist

- [ ] `cargo fmt -- --check` returns 0 formatting errors.
- [ ] `cargo clippy --all-targets --all-features` returns 0 warnings and 0 errors.
- [ ] `npm run build` generates clean production assets without errors.
- [ ] `npm run bump minor` synchronizes version `4.207.0` across all 14 files.
- [ ] `changelog.md` and `changelog_en.md` attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
- [ ] `README.md` and `README_EN.md` updated with v4.207.0 highlights.
- [ ] Git commit created via `gitmap cpf` and tag `v4.207.0` ready for deployment on `main`.
