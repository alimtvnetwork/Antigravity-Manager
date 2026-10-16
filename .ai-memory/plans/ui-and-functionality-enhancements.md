# Master Plan: UI and Functionality Enhancements

- **Slug**: `ui-and-functionality-enhancements`
- **Specification Directory**: `02-spec/21-app/ui-and-functionality-enhancements/`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/ui-and-functionality-enhancements/`
- **Benchmark Branch**: `backup/ui-and-functionality-enhancements-benchmark`
- **Target Version**: `4.207.0` (Minor Bump)

---

## 1. High-Level Objectives

1. **Window Maximize Interaction**: Fix header/navbar double-click to toggle maximize/unmaximize without being swallowed by Win32 mouse-down drag loops.
2. **Quota Bar Precision & Polish**: Tone down excessive neon glow, reduce progress bar width by 10% to reclaim table real estate, and space 4 checkpoints at equal 20% intervals with unified consistent color.
3. **Resizable Email Column**: Provide dynamic column width adjustment with min/max bounds (`[220px, 550px]`), persistent storage, and an interactive drag handle on the column header.
4. **Header Status Pill Compaction**: Merge loose `IDLE` status indicator and `Log` button into a single unified 28px dark-glass segmented pill capsule.
5. **Prompt Tree View Toolbar Redesign**: Compact `Copy All`, `Send`, `Enqueue`, and `Clear` into a contiguous segmented capsule; merge `Send` and `Enqueue` into a primary split button with dropdown; introduce a "New Conversation" prompt dispatch workflow with custom conversation title input.
6. **RCA Documentation**: Record exhaustive root cause analysis for the resolution of historical prompt sending and enqueue failures.
7. **Fleet Remote Telemetry (JSON)**: Ensure pure JSON telemetry payload exchange over Telegram, Email, and Supabase for cross-node instance discovery and collision-shielded account fast-forwarding.
8. **Minor Release Ceremony**: Execute Pre-flight Quality Gates, bump to `v4.207.0`, synchronize READMEs and changelogs with `@aukgit` attribution, and commit/push via `gitmap cpf`.

---

## 2. Decomposed Subtasks Breakdown

- `01-titlebar-maximize-and-compact-status-capsule.md`: TitleBar double-click maximize fix & `RecordingIndicator` compact pill styling.
- `02-quota-bar-glow-width-and-circle-consistency.md`: QuotaProgressBar glow reduction, 10% width shrinkage, 4 equal-distance checkpoints.
- `03-resizable-email-column-ui.md`: AccountTable resizable Email column, drag handle, min/max bounds, localStorage persistence.
- `04-prompt-tree-toolbar-compaction-and-new-conversation.md`: PromptTreeView toolbar compaction, unified Send/Enqueue split dropdown, New Conversation creation flow.
- `05-fleet-remote-json-connectivity-and-rca.md`: Pure JSON fleet telemetry ingestion across channels & comprehensive prompt send/enqueue RCA.
- `06-version-bump-and-release-ceremony.md`: Preflight checks, `npm run bump minor` (v4.207.0), changelog & README sync, atomic GitMap commit.

---

## 3. Wave Execution Schedule

```text
Wave 1 (Phase 1: Discovery & Planning):
- Worker 01: UI Discovery (TitleBar, QuotaProgressBar, AccountTable, PromptTreeView) [COMPLETED]
- Worker 02: Backend Discovery (Send/Enqueue RCA, Fleet JSON Sync, Release Ceremony) [COMPLETED]

Wave 2 (Phase 1B: Spec Authoring & Subtask Generation):
- Worker 01: Author 01-architecture-and-ui-spec.md and UI Subtasks (01, 02, 03, 04)
- Worker 02: Author 02-backend-fleet-and-rca-spec.md and Backend Subtasks (05, 06)

Wave 3 (Phase 2: Code Execution):
- Worker 01: Implement UI Improvements (TitleBar maximize, RecordingIndicator pill, QuotaProgressBar, AccountTable resize, PromptTreeView toolbar)
- Worker 02: Implement Fleet JSON Sync & RCA spec verification

Wave 4 (Phase 3: Pre-flight, Bump & Release):
- Lead: Pre-flight checks (cargo fmt, cargo clippy, npm run build), npm run bump minor, changelog/README sync, gitmap cpf commit & push
```
