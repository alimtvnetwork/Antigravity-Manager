# Master Plan: Telegram Backup and Fleet Management Enhancements

**Task Slug**: `telegram-backup-and-fleet-management-enhancements`  
**Status**: IN_PROGRESS  
**Spec References**:  
- `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/01-architecture-and-fleet-spec.md`  
- `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/02-tree-view-and-clipboard-spec.md`  
- `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/03-telegram-backup-and-media-spec.md`  

---

## 1. Workstream Breakdown

| Subtask ID | Title | Primary Files | Target Outcome |
|---|---|---|---|
| **Subtask-01** | Fleet Caching, Lead Alignment & Cross-Machine Dispatch | `src/components/instances/fleet/useFleetMachines.ts`<br/>`src/components/instances/fleet/FleetMachineRow.tsx`<br/>`src/components/instances/FleetMachinesTable.tsx`<br/>`src-tauri/src/modules/ssh_manager/exec.rs` | 2-tier cache prevents stuck skeleton; `w1` assigned Lead/Local with top primacy; remote prompt dispatch enabled in fleet table. |
| **Subtask-02** | Tree View Spacing, Terminal Indentation, Image Filter & Modernized Dropdown | `src/components/instances/PromptTreeView/ProjectNode.tsx`<br/>`src/components/instances/PromptTreeView/ConversationNode.tsx`<br/>`src/components/instances/PromptTreeView/TreePanel.tsx`<br/>`src/components/instances/PromptTreeView/ModalHeader.tsx` | Expanded top-level project breathing room; terminal branch glyphs (`├──`, `└──`); image filter button; normalized Hide Auto button; dark-glass Sync dropdown. |
| **Subtask-03** | Composite PNG Clipboard Repair & Send Status Correction | `src/components/instances/PromptTreeView/clipboard.ts`<br/>`src/components/instances/PromptTreeView/DetailHeader.tsx`<br/>`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Eliminate canvas tainting in composite image copy; dual-write to OS and browser clipboard; correct prompt queue status on Send button. |
| **Subtask-04** | Telegram Backup, Restore & Media Enhancements | `src-tauri/src/modules/telegram_inbound/config.rs`<br/>`src-tauri/src/modules/telegram_inbound/commands.rs`<br/>`src-tauri/src/modules/telegram_inbound/processing.rs`<br/>`src-tauri/src/modules/telegram_inbound/reports_b.rs` | Register `/backup` & `/restore` slash commands; add interactive Telegram inline buttons; support media sending. |
| **Subtask-05** | E2E Testing, Pre-flight Quality Gates & Release Ceremony | `scripts/test-instance-e2e.ps1`<br/>`package.json`<br/>`changelog.md`<br/>`readme.md` | All test suites green, transient sandbox pass, fmt + clippy + build pass, minor release bump and CI/CD verified via `gitmap pe -t`. |

---

## 2. Execution Phases

### Phase 1: Planning & Spec Authoring (Steps 1..100)
- Ingest GitMap IP and cluster node mappings.
- Author specifications in `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/`.
- Initialize master plan and subtasks in `.ai-memory/plans/`.

### Phase 2: Implementation Waves (Steps 101..250)
- **Wave 1 (Fleet & Lead Alignment)**: Implement fleet caching, lead node assignment to `w1`, and cross-machine prompt sending.
- **Wave 2 (Tree View UX & Clipboard)**: Implement project spacing, branch indentation glyphs, image filter, modernized sync dropdown, and composite PNG clipboard fix.
- **Wave 3 (Telegram Backup & Media)**: Register `/backup` & `/restore` commands and inline keyboards in Telegram.

### Phase 3: Verification & Release (Steps 251..400)
- Run unit test suites (`telegram_inbound`, `repo_db`).
- Test transient sandbox lifecycle (`agm instances test-create` / `test-clean`).
- Execute pre-flight gates (`cargo fmt`, `cargo clippy`, `npm run build`).
- Perform minor version bump to `v4.243.0`, commit atomically via `gitmap cpf`, and verify CI/CD via `gitmap pe -t`.
