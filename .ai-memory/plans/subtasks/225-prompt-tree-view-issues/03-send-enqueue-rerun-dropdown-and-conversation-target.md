# Subtask 03: Combined [Send, Enqueue, Rerun] Dropdown, Conversation Target Routing Fix, CLI Error Management & Release Verification

- **Parent Specification:** `02-spec/21-app/225-prompt-tree-view-issues/03-migration-and-verification-spec.md`
- **Master Audit Ledger:** `02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md`
- **Master Plan:** `.ai-memory/plans/pending/225-prompt-tree-view-issues.md`
- **Subtask Plan:** `.ai-memory/plans/subtasks/225-prompt-tree-view-issues/03-send-enqueue-rerun-dropdown-and-conversation-target.md`
- **Task ID:** `225`
- **Subtask ID:** `03`
- **Status:** `READY_FOR_EXECUTION`
- **Date:** 2026-10-24

---

## 1. Problem Statement & Scope

Task 225 mandates resolving critical user experience, dispatch correctness, error observability, and release verification defects in the Prompt Tree View:
1. **Dropdown Fragmentation**: The combined `[Send | Enqueue | Rerun]` split-button was previously disassembled or missing from the instruction editing pane (`PromptInstructionView.tsx`), forcing users to rely only on header buttons.
2. **Conversation Misrouting Defect**: Selecting "Current Conversation" bypassed live IDE CDP injection and spawned a headless CLI agent or created a new session, while selecting "New Conversation" injected into the active chat without opening a new tab.
3. **Error Management Tooling & Skill**: Runtime errors in the dashboard must be inspectable and cleanable via native CLI commands (`agm errors list`, `agm errors clear`), and registered as a reusable autonomous skill in `.agents/skills/agm-error-management/skill.md`.
4. **End-to-End Testing & Release Protocol**: Full Cargo build, compilation gate checks, PowerShell E2E testing (`scripts/test-agm-gitmap-running-backup-e2e.ps1`), 5-step account switching revalidation, and atomic minor version bump (`npm run bump minor`).

---

## 2. Target Files for Modification

| Component | Target File | Planned Scope of Change |
| :--- | :--- | :--- |
| **Frontend UI** | `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Rebuild segmented `[Send \| Enqueue \| Rerun]` pill capsule dropdown in instruction view footer. |
| **Frontend UI** | `src/components/instances/PromptTreeView/DetailHeader.tsx` | Ensure segmented pill split-button `[Action Button \| Chevron]` remains synchronized with instruction view. |
| **Frontend Actions** | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Pass explicit `target_mode: 'current' \| 'new'` and `conversation_id` without synthetic degradation. |
| **Frontend State** | `src/components/instances/PromptTreeView/usePromptTreeState.ts` | Ensure `primaryDispatchAction`, `conversationTargetMode`, and `newConversationTitle` are cleanly synchronized across views. |
| **Frontend Service** | `src/services/instanceService/lifecycle.ts` | Ensure `sendPromptNow` and `enqueuePrompt` propagate `conversationId` and destination mode cleanly to Tauri IPC. |
| **Backend Commands** | `src-tauri/src/commands/instance.rs` | Update `send_prompt_now` and `enqueue_prompt` parameter handling and audit logging. |
| **Backend Dispatch** | `src-tauri/src/modules/repo_db/agy.rs` | Fix CDP bypass guard: use CDP when instance is running; differentiate `current` vs `new` chat in CDP; fallback to `agy` CLI only when headless. |
| **Backend CDP** | `src-tauri/src/modules/instance/cdp_onboarding.rs` | Add `submit_prompt_via_cdp_with_mode` supporting "New Chat" triggering before insertion when `target_mode == 'new'`. |
| **CLI Tooling** | `src-tauri/src/bin/agm/error_cmds.rs` | Implement and verify `agm errors list`, `agm errors clear`, `agm errors fix`, and `agm errors count`. |
| **Agentic Skill** | `.agents/skills/agm-error-management/skill.md` | Register autonomous error inspection and self-healing skill for AI companions. |

---

## 3. Concrete Step-by-Step Implementation Roadmap

### Phase 1: Rebuild Combined Action Dropdown (`PromptInstructionView.tsx` & `DetailHeader.tsx`)
- [ ] In `src/components/instances/PromptTreeView/PromptInstructionView.tsx`:
  - Locate the footer controls beneath the instruction editing `<textarea>` (lines 961-985).
  - Wrap existing `Reset` and `Save Changes` buttons into a responsive, flex-aligned action bar.
  - Mount the contiguous segmented pill capsule on the right:
    - Left button: Armed primary action (`Send (Preempt)`, `Enqueue (FIFO)`, or `Rerun Turn (×N)`).
    - Right button: Chevron toggling the dark-glass dispatch popover flyout.
  - Add popover flyout containing:
    - Mode Switcher Segmented Pill: `[ Send (Preempt) | Enqueue (FIFO) | Rerun Turn ]`.
    - Destination Selector: `Current Conv` vs. `New Conv` (using transparent borderless styling: `bg-transparent border-0 focus:ring-0 focus:outline-none`).
    - Inline Title Input: Appears when `New Conv` is selected.
    - Suffix Preset Dropdown: Quick-selection of confirmation presets.
- [ ] Connect state handlers between `PromptInstructionView.tsx` and `PromptTreeViewModal.tsx`:
  - Pass `primaryDispatchAction`, `setPrimaryDispatchAction`, `conversationTargetMode`, `setConversationTargetMode`, `onResend`, `onEnqueue`, and `onRerun`.
  - Bind keyboard accelerators: `N` for Send Now, `Q` for Enqueue FIFO, `R` for Rerun Turn.

### Phase 2: Resolve Current vs. New Conversation Misrouting
- [ ] Update `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
  - Ensure `dispatchConversationId(conversationTargetMode, conv?.conversation_id, ...)` accurately sets `targetConvId`:
    - If `conversationTargetMode === 'new'`, returns `'new'`.
    - If `conversationTargetMode === 'current'`, returns `conv?.conversation_id` or `'current'`.
- [ ] Refactor `src-tauri/src/modules/instance/cdp_onboarding.rs`:
  - Implement `submit_prompt_via_cdp_with_mode`:
    - Accept `open_new_chat_first: bool`.
    - If `open_new_chat_first` is `true`: Locate and click `button[aria-label="New Chat"]` or `button.codicon-plus`, then wait 200ms for DOM settlement.
    - If `open_new_chat_first` is `false`: Focus the active composer and insert text immediately.
- [ ] Refactor `src-tauri/src/modules/repo_db/agy.rs`:
  - Remove the flawed `!resume_existing && !caller_named_conversation` guard in `send_prompt_now_for_instance`.
  - If instance is running, invoke `submit_prompt_via_cdp_with_mode`.
  - Keep `spawn_or_replace_prompt_via_agy` strictly as a fallback when the IDE is not running or CDP fails.
  - Apply the identical fix to `enqueue_prompt_for_instance` (lines 1055-1086).

### Phase 3: Enhance CLI Error Management & Register Agentic Skill
- [ ] Verify native CLI error commands in `src-tauri/src/bin/agm/error_cmds.rs`:
  - `agm errors list [--limit N] [--severity S] [--json]`: Reads records from `errors.db` and renders ASCII table or JSON.
  - `agm errors clear`: Deletes resolved or all error records from `errors.db`.
  - `agm errors fix [--dry-run]`: Runs automated self-healing diagnostic against stale lockfiles, permissions, and SQLite WAL anomalies.
  - `agm errors count`: Returns aggregated counts by severity level.
- [ ] Author autonomous agent skill at `.agents/skills/agm-error-management/skill.md`:
  - Include YAML frontmatter (`name: agm-error-management`, description).
  - Document diagnostic workflows, parameter mappings, and automated self-healing triggers.

### Phase 4: Compiler Gates, E2E Testing, Switching Logic & Release Ceremony
- [ ] Execute Rust pre-flight checks:
  ```pwsh
  cd src-tauri
  cargo fmt -- --check
  cargo clippy --all-targets --all-features
  ```
- [ ] Execute Frontend build:
  ```pwsh
  npm run build
  ```
- [ ] Run PowerShell E2E test suite:
  ```pwsh
  .\scripts\test-agm-gitmap-running-backup-e2e.ps1 -VerboseOutput
  ```
  Verify all 8 gates pass with 100% green status.
- [ ] Revalidate switching logic:
  - Verify 5-step switch lifecycle (`switch_account`).
  - Verify minimal normalized multiplicative scoring ($\div 1000$).
  - Verify stale lock eviction (> 6h).
  - Verify zero prompt loss across multimodal prompt assets.
- [ ] Execute release ceremony:
  - Run `npm run bump minor` for atomic version synchronization.
  - Audit attribution to strictly `@aukgit` (`(Thanks to @aukgit)`).
  - Verify `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, and `README_EN.md` are synchronized.
  - Verify strictly relative Git paths across all committed files.

---

## 4. Concrete Verification Commands & Test Scripts

### Verification Commands:
1. **CLI Error Commands Verification**:
   ```pwsh
   .\src-tauri\target\debug\agm.exe errors count
   .\src-tauri\target\debug\agm.exe errors list --limit 10 --json
   .\src-tauri\target\debug\agm.exe errors clear
   ```
2. **PowerShell E2E Suite Execution**:
   ```pwsh
   powershell.exe -ExecutionPolicy Bypass -File .\scripts\test-agm-gitmap-running-backup-e2e.ps1 -VerboseOutput
   ```
3. **Compiler Diagnostics & Frontend Gate**:
   ```pwsh
   cd src-tauri && cargo clippy --all-targets --all-features
   cd .. && npm run build
   ```

---

## 5. Strict Pass Criteria & Acceptance Gate

- [ ] **AC-UI-01**: Segmented `[Send | Enqueue | Rerun]` pill capsule is present, functional, and visually stable in both `PromptInstructionView.tsx` and `DetailHeader.tsx`.
- [ ] **AC-ROUTING-01**: Selecting "Current Conversation" and clicking Send injects prompt directly into active IDE chat via CDP without opening a new session.
- [ ] **AC-ROUTING-02**: Selecting "New Conversation" and clicking Send triggers the "New Chat" button via CDP before typing prompt.
- [ ] **AC-CLI-01**: `agm errors list` and `agm errors clear` execute cleanly from the terminal and return valid outputs.
- [ ] **AC-SKILL-01**: Skill `.agents/skills/agm-error-management/skill.md` is authored, formatted with valid YAML frontmatter, and indexed.
- [ ] **AC-TEST-01**: All 8 gates of `scripts/test-agm-gitmap-running-backup-e2e.ps1` exit with status 0.
- [ ] **AC-SWITCH-01**: Unified 5-step switch lifecycle, scoring, and zero prompt loss verified against canonical specifications.
- [ ] **AC-REL-01**: `npm run bump minor` synchronizes all manifests with zero absolute filesystem paths.
