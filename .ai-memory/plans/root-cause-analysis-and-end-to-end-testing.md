# Master Plan: Root Cause Analysis and End-to-End Testing

## Plan Information
- **Title**: Root Cause Analysis and End-to-End Testing: Prompt Tree Discrepancies, Timestamp Alignment, Image Previews, and Cross-Machine Relay Parity
- **Slug**: root-cause-analysis-and-end-to-end-testing
- **Author**: @aukgit
- **Created**: 2026-10-19
- **Status**: Completed
- **Specification**: `02-spec/21-app/root-cause-analysis-and-end-to-end-testing/01-architecture-spec.md`

---

## Subtasks Breakdown

- [x] **Subtask 1**: Prompt Tree Selection Stability, Anti-Flapping & Media Preview (`src/components/instances/PromptTreeView/`)
  - Subtask Plan: `.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/01-prompt-tree-stability-and-media-preview.md`
  - Owned Files: `usePromptTreeState.ts`, `usePromptTreeData.ts`, `PromptInstructionView.tsx`, `DetailHeader.tsx`, `clipboard.ts`, `markdown.tsx`
  - Status: Completed & Verified Green
- [x] **Subtask 2**: Backend Turn Counts, Timestamps & Project Summary Alignment (`src/pages/instances/`, `repo_db/`)
  - Subtask Plan: `.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/02-backend-turn-counts-and-timestamps.md`
  - Owned Files: `InstanceCardProjects.tsx`, `DetailsModal.tsx`, `src-tauri/src/modules/repo_db/transcript.rs`
  - Status: Completed & Verified Green
- [x] **Subtask 3**: Telegram Prompt Acknowledgment, Date/Time & GitMap SSH Mesh (`src-tauri/src/modules/telegram_inbound/`, `supabase_sync/`)
  - Subtask Plan: `.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/03-telegram-acknowledgment-and-gitmap-parity.md`
  - Owned Files: `injection.rs`, `send_flow.rs`, `ui_remote.rs`, `open_shot.rs`, `query.rs`, `useFleetMachines.ts`
  - Status: Completed & Verified Green

---

## Non-Negotiable Invariants & Guardrails
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String` strictly followed.
- **Strictly Relative Git Paths**: Zero absolute filesystem paths in code, specs, and commit history.
- **Developer Session Immunity**: Never terminated or disrupted PID 32 (`Antigravity.exe`).
- **Strictly Attribute @aukgit**: Attributed solely `@aukgit`.
- **Preflight Verification**: `cargo fmt`, `cargo clippy`, `cargo test`, `npm run build`, and Python E2E suite all 100% green.
