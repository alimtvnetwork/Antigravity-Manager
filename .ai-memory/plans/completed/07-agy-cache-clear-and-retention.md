# Plan (Completed): Antigravity Conversation Pruning, CLI Cache-Clear Engine, Undo Staging & UI Quick-Clean

## Master Verification Outcome: PASSED (All 5 Subtasks Completed)

- **Completed Subtasks:**
  - `01-rust-agy-cleaner-engine`: Implemented `src-tauri/src/modules/agy_cleaner.rs` with cross-platform Antigravity conversation discovery, recency ranking based on `conversation_summaries.db`, safe transactional staging to OS temporary directory, application cache clearing, and full transaction rollback via `manifest.json`.
  - `02-cli-commands-dispatch`: Added CLI subcommands `agy cache-clear`, `ccko` (keep 1), `cckf` (keep 5), `agy undo [tx_id]`, `--preflight` / `--pre` / `--precheck`, `-y` / `--yes`, and rich help documentation with OS temporary staging warnings in `src-tauri/src/modules/cli.rs`. Updated `run.ps1` and `run.sh` CLI forwarders.
  - `03-background-telemetry-cleaner`: Added `ConversationCleanupConfig` to `AppConfig` in `src-tauri/src/models/config.rs` and `src/types/config.ts`. Initialized 1-hour periodic background ticker in `src-tauri/src/lib.rs` and `src-tauri/src/modules/agy_cleaner.rs`. Added interactive configuration card in `src/pages/Settings.tsx`.
  - `04-ui-top-right-quick-clean`: Built interactive modal `src/components/modals/agy-clean-modal.tsx` with live preflight analysis, retention presets (Keep 1, Keep 5, Keep 10, Keep 40, custom), staging notices, and undo capability. Added top-right Quick Clean action button in `src/components/navbar/NavSettings.tsx` and `src/components/navbar/NavDropdowns.tsx`.
  - `05-verification-and-consolidation`: Ran `tsc --noEmit` (0 errors), `cargo fmt` (0 errors), `cargo check` (0 errors), and live CLI dry-run validation (`.\run.ps1 agy help`, `.\run.ps1 agy cache-clear --preflight`, `.\run.ps1 ccko --preflight`).

---

## Architectural Context & Domain Logic
Antigravity stores session data under `~/.gemini/antigravity`:
- `conversations/<id>.db`: SQLite conversation databases.
- `conversation_summaries.db`: Index of conversation titles, timestamps, and steps.
- `brain/<id>/`: Transcripts, tasks, and scratchpad artifacts.
- Cache folders (Electron GPUCache, Code Cache, HTTP storage).

Over time, hundreds of conversations accumulate gigabytes of disk space and slow down IDE indexing. This feature provides a cross-platform (Windows, macOS, Linux) pruning and cache cleaning engine that:
1. Identifies conversations and sorts them by recency.
2. Keeps the newest $K$ conversations (default 10; or 1 for `ccko`, 5 for `cckf`, or custom `--keep N`).
3. Safely moves older conversations and brain folders to a temporary recovery directory with a metadata `manifest.json` before removal.
4. Allows immediate reversion via `agy undo`.
5. Supports dry-run inspection via `--preflight` / `--pre` / `--precheck`.
6. Enforces confirmation prompts `[y/N]` unless `-y` is supplied.
7. Adds an automated 1-hour background telemetry cleaner in Settings keeping conversations capped at 40.
8. Adds a top-right Navbar Quick-Clear action with an impact confirmation modal.

---

## Implementation Details

### 1. Rust Cleaner Engine (`src-tauri/src/modules/agy_cleaner.rs`)
- `scan_conversations()`: Discovers `conversations/*.db` and queries `conversation_summaries.db` for `last_modified_time` sorting.
- `preflight_check(keep_count)`: Calculates preserved vs pruned counts, projected space reclamation, and cache targets without modifying files.
- `prune_and_clean(keep_count)`: Creates staging transaction folder `antigravity-cleaner-backup/tx_<timestamp>` in OS temp, moves older databases and brain directories into staging, flushes application cache folders, and writes `manifest.json`.
- `undo_prune(target_tx_id)`: Restores databases and brain directories back from staging according to `manifest.json` and renames manifest to `manifest.json.reverted`.
- `start_cleanup_daemon()`: Tokio background ticker waking every hour to evaluate `app_config.conversation_cleanup.is_enabled` and prune older conversations beyond `keep_count` (default: 40).

### 2. CLI Dispatcher (`src-tauri/src/modules/cli.rs`)
- Registered subcommands:
  - `antigravity-manager agy cache-clear [--keep <N>]`
  - `antigravity-manager agy cache-clear-keep-one` (alias: `ccko`)
  - `antigravity-manager agy cache-clear-keep-five` (alias: `cckf`)
  - `antigravity-manager agy undo [tx_id]`
  - `antigravity-manager agy help`
- Supported flags: `--preflight`, `--precheck`, `--pre`, `-y`, `--yes`, `--keep <N>`, `-k <N>`.

### 3. Settings & Background Daemon (`src-tauri/src/models/config.rs`, `src/pages/Settings.tsx`)
- Struct `ConversationCleanupConfig`:
  - `is_enabled: bool` (default: false)
  - `interval_hours: u32` (default: 1)
  - `keep_count: usize` (default: 40)
- Settings UI: Dedicated switch toggle, input for retained conversations count, interval, and explanatory notice.

### 4. Navbar Quick Clean & Confirmation Modal (`src/components/modals/agy-clean-modal.tsx`, `src/components/navbar/NavSettings.tsx`)
- Sparkles icon in top-right navbar triggers `AgyCleanModal`.
- Modal displays:
  - Total conversations found
  - Number preserved in Antigravity
  - Number to be staged into OS temporary backup
  - Total cache space to be reclaimed
  - Retention preset chips (Keep 1, Keep 5, Keep 10, Keep 40, Custom)
  - Safety explanation of temporary staging and warning about OS temp clearing
  - "Clean & Prune Now" button and "Undo Last Clean" button
