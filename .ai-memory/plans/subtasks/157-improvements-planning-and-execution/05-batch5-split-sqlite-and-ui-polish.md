# Subtask 05: Batch 5 Execution (Improvements 25–30: Split-SQLite, Telemetry & UI Polish)

## Improvements In Scope
- **Imp-25**: PRAGMA WAL & Cache Optimization (`src-tauri/src/modules/sqlite_manager.rs`)
- **Imp-26**: Strict FIFO Queue Ordering (`src-tauri/src/modules/repo_db/tree.rs`)
- **Imp-27**: Speed Error SQLite Telemetry Logger (`src-tauri/src/modules/speed_errors.rs`)
- **Imp-28**: Levenshtein CLI Command Suggester (`src-tauri/src/modules/failed_commands.rs`)
- **Imp-29**: Borderless Transparent Inline Selects (`src/components/navbar/NavMenu.tsx`)
- **Imp-30**: Scoreboard Emerald Formula Styling (`src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`)

## Status
`[COMPLETED]`

## Verification Summary
- **Imp-25**: Added `PRAGMA cache_size = -64000;` to `src-tauri/src/modules/db_orm/connection.rs` for 64MB SQLite page caching.
- **Imp-26**: Enforced strict FIFO queue ordering `ORDER BY created_at ASC, id ASC` in `src-tauri/src/modules/repo_db/restore.rs`.
- **Imp-27**: Created `src-tauri/src/modules/speed_errors.rs` providing asynchronous telemetry logging for diagnostic codes (E1001–E9001).
- **Imp-28**: Created `src-tauri/src/modules/failed_commands.rs` implementing Levenshtein distance $\le 2$ command suggester.
- **Imp-29**: Verified borderless transparent inline `<select>` styling in segmented capsules in `NavMenu.tsx` and `TreePanel.tsx`.
- **Imp-30**: Verified emerald mono formula styling and dark card container in `AlgorithmScoreCell.tsx`.
