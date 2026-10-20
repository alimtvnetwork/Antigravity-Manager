# Master Plan: Button Functionality and Testing Instructions

**Task Slug**: `button-functionality-and-testing-instructions`  
**Status**: ACTIVE  
**Spec References**:  
- `02-spec/21-app/button-functionality-and-testing-instructions/01-architecture-spec.md`  
- `02-spec/21-app/button-functionality-and-testing-instructions/02-ui-components-and-clipboard-spec.md`  
- `02-spec/21-app/button-functionality-and-testing-instructions/03-fleet-ssh-and-reset-spec.md`  

---

## 1. Workstream Breakdown

| Subtask ID | Title | Primary Files | Target Outcome |
|---|---|---|---|
| **Subtask-01** | Send & Enqueue Buttons Repair | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`<br/>`src/components/instances/PromptTreeView/DetailHeader.tsx`<br/>`src/services/instanceService/lifecycle.ts` | Immediate Send and Enqueue buttons operational with full image forwarding and status handling. |
| **Subtask-02** | Image Clipboard & Batch Copy | `src/components/instances/PromptTreeView/clipboard.ts`<br/>`src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Pure PNG clipboard write for OneNote/Antigravity; batch copy all images; "Copy" button label. |
| **Subtask-03** | Prompt Tree Title & Header Polish | `src/components/instances/PromptTreeView/DetailHeader.tsx`<br/>`src/components/instances/PromptTreeView/TurnDetailPane.tsx` | Cut off turn badges preceding the title; enlarge title typography. |
| **Subtask-04** | Prompt Tree Cache & DB Reset | `src/components/instances/PromptTreeView/DetailHeader.tsx`<br/>`src-tauri/src/commands/cmd_repo_db.rs`<br/>`src-tauri/src/modules/repo_db/mod.rs` | Complete database wipe and transcript regeneration from disk. |
| **Subtask-05** | Fleet Nodes SSH & GitMap Account Linkage | `src-tauri/src/modules/ssh_manager.rs`<br/>`src/components/fleet/FleetNodesTable.tsx` | Fix Git link and connected accounts on main/w1 mesh nodes. |
| **Subtask-06** | E2E Testing, Telegram Test & Release | `scripts/test-instance-e2e.ps1`<br/>`src-tauri/src/modules/telegram_inbound/` | End-to-end verification, Telegram prompt test, preflight gates, and minor release bump. |

---

## 2. Execution Phases

### Phase 1: Planning & Discovery (A = 3 Research Subagents)
- Dispatched 3 parallel discovery subagents to audit frontend actions, clipboard transcoding, and backend reset/SSH routes.

### Phase 2: Implementation Waves
- **Wave 1 (Frontend Actions & Clipboard)**:
  - Subtask-01: Send & Enqueue buttons fix.
  - Subtask-02: Image clipboard & multi-image batch copy.
  - Subtask-03: DetailHeader title clean-up & enlargement.
- **Wave 2 (Backend Reset & Fleet SSH)**:
  - Subtask-04: Tree reset & SQLite db regeneration.
  - Subtask-05: Fleet SSH account & GitMap linkage.
- **Wave 3 (E2E Verification & Release)**:
  - Subtask-06: End-to-end testing with new instance, Telegram prompt testing, and minor bump release ceremony.
