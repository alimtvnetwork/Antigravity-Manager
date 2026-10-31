# Master Plan: UI and Image Copy Functionality Fixes

**Task Slug**: `ui-and-image-copy-functionality-fixes`  
**Status**: ACTIVE  
**Ledger Reference**: `.ai-memory/temp-agents/ui-and-image-copy-functionality-fixes/ledger.md`  
**Spec References**:  
- `02-spec/21-app/ui-and-image-copy-functionality-fixes/01-architecture-spec.md`  
- `02-spec/21-app/ui-and-image-copy-functionality-fixes/02-component-and-ui-spec.md`  

---

## 1. Workstream Breakdown

| Subtask ID | Title | Primary Files | Target Outcome |
|---|---|---|---|
| **Subtask-01** | Clipboard Pure Image & Batch Copy | `src/components/instances/PromptTreeView/clipboard.ts`<br/>`src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Pure PNG clipboard write without multi-MIME collision; offscreen canvas PNG transcoding; dedicated "Copy Image (PNG)" button in gallery header. |
| **Subtask-02** | Immediate Send vs Enqueue & Image Forwarding | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`<br/>`src/components/instances/PromptTreeView/DetailHeader.tsx`<br/>`src/services/instanceService/lifecycle.ts` | Immediate send execution in running session; clear separation between Send and Enqueue; forward image attachments in send/run/rerun. |
| **Subtask-03** | Turn Status Badges & Queued Sync | `src/components/instances/PromptTreeView/ConversationNode.tsx`<br/>`src-tauri/src/modules/repo_db/tree_collect.rs`<br/>`src-tauri/src/commands/cmd_running_up_next.rs` | Display RUNNING (pulse) and QUEUED badges directly beside prompt turns in tree; reflect in-flight transcript turns in up-next count. |
| **Subtask-04** | Telegram Machine Aliasing & Theme Contrast | `src-tauri/src/modules/telegram_inbound/node_label.rs`<br/>`src-tauri/src/modules/telegram_inbound/formatting.rs`<br/>`src/components/instances/PromptTreeView/markdown.tsx` | Resolve IP `192.168.1.3` to `w1` from `~/.gitmap/gitmap-ssh-nodes.json` with `(collected from gitmap)`; place Instance on next line in Telegram; polish theme contrast. |

---

## 2. Execution Phases

### Phase 1: Specifications & Subtasks (Current)
- [x] Pre-flight Git pull cleanly verified.
- [x] Author architecture and UI specs under `02-spec/21-app/ui-and-image-copy-functionality-fixes/`.
- [x] Author master plan and 4 granular subtask plan files.
- [x] Register subtasks in SQLite database (`agent-task.db`).

### Phase 2: Parallel Implementation Waves
- **Wave 1 (Frontend Core)**:
  - Worker 01: Subtask-01 (Clipboard pure image & gallery copy button).
  - Worker 02: Subtask-02 (Immediate send vs enqueue & image forwarding).
- **Wave 2 (Backend & Visual Refinements)**:
  - Worker 01: Subtask-03 (Turn status badges beside prompt turns).
  - Worker 02: Subtask-04 (Telegram machine aliasing & theme contrast).

### Phase 3: Verification & Pre-flight Gates
- [x] Run targeted frontend tests & build (`npm run build`).
- [x] Run Rust verification (`cargo fmt -- --check && cargo test --lib modules::repo_db modules::telegram_inbound`).
- [x] Pre-flight gates verified.

