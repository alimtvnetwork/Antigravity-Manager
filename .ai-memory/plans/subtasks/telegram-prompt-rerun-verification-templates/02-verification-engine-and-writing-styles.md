---
plan: telegram-prompt-rerun-verification-templates
subtask: "02"
title: Verification Chapter Engine, Diverse Writing Styles & Multi-Round Repetition Architecture
domain: frontend-react-tauri
status: pending
citations:
  component_spec: 02-spec/21-app/telegram-prompt-rerun-verification-templates/02-verification-engine-and-writing-styles-spec.md
target_files:
  - src/components/instances/PromptTreeView/verificationTemplates.ts
  - src/components/instances/PromptTreeView/VerificationModal.tsx
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
  - src-tauri/src/modules/repo_db/verification.rs
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/commands/project_cmds.rs
---

# Subtask 02: Verification Chapter Engine, Diverse Writing Styles & Multi-Round Repetition Architecture

## 1. Context & Objectives

The verbatim user instruction for this workstream specifies:
> "Verification would be a segment where we get in, check the instances, and then pick which prompts that needs to send as a verification two times or three times. And that would have a template, usually that can be also changed from the UI... and add more verification steps on writing. Each one of them will have different writings so that the AI follows through, understand where they have made mistakes, and they could fix it."

To eliminate the "superficial affirmation" anti-pattern and provide structured multi-stage auditing, Worker 02 will implement:
1. **Verification Generation Engine Extension (`verificationTemplates.ts`)**:
   - Define the 4 specialized verification writing styles:
     * `root-cause-auditor` (Strict Root-Cause Auditor): Demands 4-part RCA, reproduction traces, surgical minimal fixes, and prevention invariants.
     * `socratic-skeptic` (Socratic Skeptic / Devil's Advocate): Aggressively challenges premature claims of success; poses probing questions on edge cases, race conditions, nulls, timeouts, and unverified assumptions.
     * `spec-gatekeeper` (Acceptance Criteria Gatekeeper): Demands 1:1 bidirectional line-by-line verification against spec checkboxes (`02-spec/...`) and verbatim user prompt.
     * `release-blocker` (Production Release Blocker Inspector): Enforces 14-location SemVer sync, strict `@aukgit` attribution, pre-flight checks, and error visibility.
   - Implement `buildMultiRoundVerificationPrompts(options)` supporting 1x, 2x, and 3x progressive escalation.
2. **Dedicated UI Verification Modal (`VerificationModal.tsx`)**:
   - Multi-select prompt turn list with sequential sequence tokens (`[AGM:P001]`).
   - Repetition round selector pills (`[ 1x | 2x | 3x ]`).
   - Interactive writing style cards with Lucide icons, descriptive subtitles, and active highlight rings.
   - Tabbed live prompt preview showing exact markdown output for each round.
   - Direct FIFO enqueue action with desktop feedback and loading states.
3. **Action Hook Extension (`usePromptTreePromptActions.ts`)**:
   - Add `handleOpenVerificationModal(conv)` and `enqueueVerificationRounds(options)`.
   - Wire error handling via `useErrorStore.getState().captureException`.
4. **Backend Scheduling Module (`src-tauri/src/modules/repo_db/verification.rs`)**:
   - Iterates through rounds $1 \dots N$.
   - Enforces **500ms safety sleep** (`tokio::time::sleep`) between consecutive round submissions to eliminate `.antigravity_resume_task.json` file overwrite collisions.
   - Appends prompts to `active_prompts` maintaining strict FIFO order (`ORDER BY created_at ASC, id ASC`).
   - Re-exports through facade `src-tauri/src/modules/repo_db.rs` and exposes Tauri IPC command `enqueue_verification_rounds_cmd` in `src-tauri/src/commands/project_cmds.rs`.

---

## 2. Verified Architectural Invariants

- **Invariant I1 (Strict FIFO Queue Ordering)**: All verification prompts must be stored in `active_prompts` with monotonically increasing timestamps (`created_at`) and scheduled via `ORDER BY created_at ASC, id ASC`.
- **Invariant I2 (500ms Safety Throttling)**: Sequential rounds (2x/3x) must sleep for at least 500ms between disk write/queue enqueue calls to prevent file system race conditions in `.antigravity_resume_task.json`.
- **Invariant I3 (Zero Silent Errors)**: All fallible operations must be routed into `AppError` in Rust and `useErrorStore` in TypeScript. Bare `catch {}` is strictly forbidden.
- **Invariant I4 (Disjoint File Boundary)**: Worker 02 strictly modifies only its assigned files and does not modify files owned by Worker 01 or Worker 03.
- **Invariant I5 (Strict Relative Paths)**: All file paths referenced in prompt templates and UI modals must be relative to the workspace root.
- **Invariant I6 (Strict `@aukgit` Attribution)**: All release blocker checklists and generated templates must verify exclusive `@aukgit` attribution (`(Thanks to @aukgit)`).

---

## 3. Step-by-Step Implementation Plan for Worker 02

### Step 2.1: Extend Verification Engine (`verificationTemplates.ts`)
- [ ] Export `VerificationWritingStyle` union: `'root-cause-auditor' | 'socratic-skeptic' | 'spec-gatekeeper' | 'release-blocker'`.
- [ ] Export `WRITING_STYLES` descriptor array with id, title, subtitle, description, badge, color, and icon name.
- [ ] Export `MultiRoundVerificationOptions` interface containing style, rounds (`1 | 2 | 3`), prompt texts, conversation title, conversation ID, instance ID, and repo path.
- [ ] Implement style chapter synthesizers:
  - `buildRootCauseAuditorChapters(prompts, round, totalRounds, context)`
  - `buildSocraticSkepticChapters(prompts, round, totalRounds, context)`
  - `buildSpecGatekeeperChapters(prompts, round, totalRounds, context)`
  - `buildReleaseBlockerChapters(prompts, round, totalRounds, context)`
- [ ] Implement `buildMultiRoundVerificationPrompts(options: MultiRoundVerificationOptions): string[]`:
  - Returns an array of $N$ formatted markdown strings for rounds $1 \dots N$.
  - Includes standardized round banner: `# 🛡️ VERIFICATION ROUND X OF Y: [STYLE_NAME]`.
  - Embeds verbatim user prompt turns within `<<<AUDITED_USER_PROMPT_START>>>` delimiters.

### Step 2.2: Implement `VerificationModal.tsx`
- [ ] Create `src/components/instances/PromptTreeView/VerificationModal.tsx`.
- [ ] Structure:
  - Header: Conversation title, instance name badge, sequence token (`[AGM:C001]`), close button (`X` and `Escape` key listener).
  - Section 1 (Prompt Turns Selection):
    - Table / list of prompt turns with individual checkboxes.
    - Quick action buttons: *"Select All"*, *"Last 3"*, *"Last 5"*, *"Clear"*.
    - Sequence code badges (`[AGM:P001]`) and 150-word truncated previews.
  - Section 2 (Repetition Rounds):
    - Segmented pill buttons: `[ 1x Single Audit ]`, `[ 2x Baseline + Stress ]`, `[ 3x Full Release Gate ]`.
  - Section 3 (Writing Style Selection):
    - 4 dark-glass cards with Lucide icons (`Search`, `Zap`, `CheckSquare`, `ShieldAlert`).
    - Active card styling with high-contrast ring and subtle glow.
  - Section 4 (Live Tabbed Markdown Preview):
    - Tabs for `Round 1`, `Round 2`, `Round 3` (depending on selected round count).
    - Read-only markdown code preview showing the exact synthesized payload.
  - Footer:
    - *"Cancel"* button.
    - Primary action button: *"Synthesize & Enqueue (N Rounds)"* with loading spinner.
- [ ] Ensure full keyboard accessibility (Escape to close, autofocus first element).
- [ ] Wrap all fallible logic in `useErrorStore.getState().captureException`.

### Step 2.3: Hook Extensions (`usePromptTreePromptActions.ts`)
- [ ] Add state for `isVerificationModalOpen` and `verificationModalConv`.
- [ ] Add `handleOpenVerificationModal(conv?: AgmConversationNode)`:
  - Defaults to `selectedConversation` if omitted.
  - Opens `VerificationModal`.
- [ ] Add `enqueueVerificationRounds`:
  - Accepts `MultiRoundVerificationOptions`.
  - Invokes Tauri command `enqueue_verification_rounds_cmd`.
  - In non-Tauri browser environments, enqueues via simulated storage or web mock.
  - Emits audit record `AuditAction::VerificationEnqueue`.
  - Displays instant action feedback toast: `"Enqueued X verification round(s) to FIFO scheduler"`.

### Step 2.4: Backend Verification Module (`verification.rs`)
- [ ] Create `src-tauri/src/modules/repo_db/verification.rs`:
  - Define `VerificationEnqueueRequest` and `VerificationEnqueueResult` structs with `#[derive(Serialize, Deserialize)]`.
  - Implement `pub async fn enqueue_verification_rounds(req: VerificationEnqueueRequest) -> Result<VerificationEnqueueResult, AppError>`.
  - Loop over `req.round_prompts`:
    - Insert into `active_prompts` table with `status = 'queued'`.
    - Set strictly incrementing `created_at` timestamp: `now + idx`.
    - If `round_num < total_rounds`, execute `tokio::time::sleep(Duration::from_millis(500)).await`.
  - Call `invalidate_prompt_tree_cache()`.
- [ ] Update `src-tauri/src/modules/repo_db.rs`:
  - Add `pub mod verification;`.
  - Re-export `pub use verification::{enqueue_verification_rounds, VerificationEnqueueRequest, VerificationEnqueueResult};`.
- [ ] Update `src-tauri/src/commands/project_cmds.rs`:
  - Add Tauri command `enqueue_verification_rounds_cmd`.
- [ ] Update `src-tauri/src/lib.rs` (or command handler registry):
  - Ensure `enqueue_verification_rounds_cmd` is registered in `generate_handler!`.

### Step 2.5: Integration into DetailHeader & ContextMenu
- [ ] Update `DetailHeader.tsx`:
  - Add *"Verification Audit..."* action button in DetailHeader toolbar or Send dropdown menu.
  - Clicking button invokes `onOpenVerificationModal()`.
- [ ] Update `ConversationContextMenu.tsx`:
  - Add *"Synthesize Verification Audit (1x/2x/3x)..."* context menu option.
  - Clicking option opens `VerificationModal`.
- [ ] Mount `<VerificationModal />` in `PromptTreeViewModal.tsx`.

---

## 4. Concrete Implementation Details & Code Specifications

### 4.1 Engine Types in `verificationTemplates.ts`
```typescript
export type VerificationWritingStyle =
    | 'root-cause-auditor'
    | 'socratic-skeptic'
    | 'spec-gatekeeper'
    | 'release-blocker';

export interface MultiRoundVerificationOptions {
    style: VerificationWritingStyle;
    rounds: 1 | 2 | 3;
    prompts: string[];
    conversationTitle?: string;
    conversationId?: string;
    instanceId?: string;
    repoPath?: string;
}

export interface WritingStyleDescriptor {
    id: VerificationWritingStyle;
    title: string;
    subtitle: string;
    description: string;
    iconName: 'Search' | 'Zap' | 'CheckSquare' | 'ShieldAlert';
    badge: string;
    color: string;
}

export const WRITING_STYLES: ReadonlyArray<WritingStyleDescriptor> = [
    {
        id: 'root-cause-auditor',
        title: 'Root-Cause Auditor',
        subtitle: 'Failure Forensics & Invariants',
        description: 'Demands 4-part RCA, reproduction traces, surgical minimal fixes, and prevention invariants.',
        iconName: 'Search',
        badge: 'RCA',
        color: 'purple',
    },
    {
        id: 'socratic-skeptic',
        title: 'Socratic Skeptic',
        subtitle: "Devil's Advocate & Stress Probe",
        description: 'Aggressively challenges success claims; probes edge cases, race conditions, nulls, and timeouts.',
        iconName: 'Zap',
        badge: 'Adversary',
        color: 'amber',
    },
    {
        id: 'spec-gatekeeper',
        title: 'Spec Gatekeeper',
        subtitle: '1:1 Acceptance Compliance',
        description: 'Demands line-by-line verification against specification checkboxes and verbatim user instructions.',
        iconName: 'CheckSquare',
        badge: 'Gatekeeper',
        color: 'emerald',
    },
    {
        id: 'release-blocker',
        title: 'Release Blocker',
        subtitle: 'Production Readiness & Gates',
        description: 'Enforces 14-location SemVer sync, strict @aukgit attribution, pre-flight checks, and error visibility.',
        iconName: 'ShieldAlert',
        badge: 'Blocker',
        color: 'rose',
    },
];
```

### 4.2 Multi-Round Generation Function Logic
```typescript
export function buildMultiRoundVerificationPrompts(options: MultiRoundVerificationOptions): string[] {
    const { style, rounds, prompts, conversationTitle, conversationId } = options;
    const cleanPrompts = prompts.map((p) => p.trim()).filter((p) => p.length > 0);
    const roundCount = Math.max(1, Math.min(3, rounds)) as 1 | 2 | 3;
    const result: string[] = [];

    for (let r = 1; r <= roundCount; r++) {
        const roundBanner = [
            `# 🛡️ VERIFICATION ROUND ${r} OF ${roundCount}: [${style.toUpperCase()}]`,
            '',
            '> [!IMPORTANT]',
            `> **Automated Verification Escalation (Round ${r} of ${roundCount})**`,
            `> Conversation: **${conversationTitle || conversationId || 'Current Conversation'}**`,
            r === 1 ? '> **Focus:** Baseline acceptance and direct directive compliance.' : '',
            r === 2 ? '> **Focus:** Adversarial stress testing, edge cases, race conditions, and error branch handling.' : '',
            r === 3 ? '> **Focus:** Production sign-off, SemVer parity, @aukgit attribution, and pre-flight gate clearance.' : '',
        ].filter(Boolean).join('\n');

        let styleChapters = '';
        switch (style) {
            case 'root-cause-auditor':
                styleChapters = buildRootCauseAuditorContent(cleanPrompts, r, roundCount);
                break;
            case 'socratic-skeptic':
                styleChapters = buildSocraticSkepticContent(cleanPrompts, r, roundCount);
                break;
            case 'spec-gatekeeper':
                styleChapters = buildSpecGatekeeperContent(cleanPrompts, r, roundCount);
                break;
            case 'release-blocker':
                styleChapters = buildReleaseBlockerContent(cleanPrompts, r, roundCount);
                break;
        }

        const promptInventory = [
            '### Audited Instruction Turns:',
            '<<<AUDITED_USER_PROMPT_START>>>',
            cleanPrompts.map((p, idx) => `[Turn ${idx + 1}]:\n${p}`).join('\n\n---\n\n'),
            '<<<AUDITED_USER_PROMPT_END>>>',
        ].join('\n');

        result.push([roundBanner, '', styleChapters, '', promptInventory].join('\n'));
    }

    return result;
}
```

### 4.3 Backend `src-tauri/src/modules/repo_db/verification.rs`
```rust
//! src-tauri/src/modules/repo_db/verification.rs
use super::schema::connect_db;
use super::tree::invalidate_prompt_tree_cache;
use crate::error::AppError;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEnqueueRequest {
    pub instance_id: String,
    pub repo_path: String,
    pub conversation_id: String,
    pub style: String,
    pub round_prompts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEnqueueResult {
    pub enqueued_count: usize,
    pub conversation_id: String,
    pub message: String,
}

pub async fn enqueue_verification_rounds(
    req: VerificationEnqueueRequest,
) -> Result<VerificationEnqueueResult, AppError> {
    if req.round_prompts.is_empty() {
        return Err(AppError::Config("No verification round prompts provided".into()));
    }

    let total_rounds = req.round_prompts.len();
    let mut enqueued = 0;

    for (idx, prompt_body) in req.round_prompts.into_iter().enumerate() {
        let round_num = idx + 1;
        let inst_id = req.instance_id.clone();
        let r_path = req.repo_path.clone();
        let conv_id = req.conversation_id.clone();
        let prompt_clone = prompt_body.clone();

        tokio::task::spawn_blocking(move || -> Result<(), AppError> {
            let conn = connect_db().map_err(|e| AppError::Database(e))?;
            let now = Utc::now().timestamp();
            let prompt_id = format!("verify-{}-{}-r{}", conv_id, now, round_num);

            conn.execute(
                "INSERT INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8, ?8)",
                params![
                    prompt_id,
                    conv_id,
                    inst_id,
                    r_path,
                    prompt_clone,
                    "default",
                    conv_id,
                    now + (idx as i64),
                ],
            ).map_err(|e| AppError::Database(format!("Failed to insert verification prompt: {e}")))?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::Process(format!("Tokio spawn failure: {e}")))??;

        enqueued += 1;
        info!("Enqueued verification round {}/{} for conv {}", round_num, total_rounds, req.conversation_id);

        if round_num < total_rounds {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    tokio::task::spawn_blocking(|| {
        invalidate_prompt_tree_cache();
    })
    .await
    .map_err(|e| AppError::Process(format!("Cache invalidation failed: {e}")))?;

    Ok(VerificationEnqueueResult {
        enqueued_count: enqueued,
        conversation_id: req.conversation_id,
        message: format!("Successfully enqueued {} verification round(s)", enqueued),
    })
}
```

---

## 5. Verification & Pre-flight Quality Gates Checklist

Before completing this subtask, Worker 02 must verify:
- [ ] Unit tests pass: `verificationTemplates.test.ts` (covers all 4 styles, 1x/2x/3x rounds).
- [ ] Rust formatting check: `cd src-tauri && cargo fmt -- --check`.
- [ ] Rust Clippy check: `cd src-tauri && cargo clippy --all-targets --all-features`.
- [ ] Frontend build check: `npm run build`.
- [ ] UI verification:
  - Open `VerificationModal` from DetailHeader and ContextMenu.
  - Test round selection: `1x`, `2x`, `3x`.
  - Test style card selection: `root-cause-auditor`, `socratic-skeptic`, `spec-gatekeeper`, `release-blocker`.
  - Verify live markdown preview tabs update accurately.
  - Test enqueue action and verify sequential prompts appear in `active_prompts` SQLite table spaced $\ge 500$ms apart.
