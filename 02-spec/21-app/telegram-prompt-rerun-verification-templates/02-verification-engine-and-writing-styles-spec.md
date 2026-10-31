# Specification: Verification Chapter Engine, Diverse Writing Styles & Multi-Round Repetition Architecture

> **Spec ID:** `02-spec/21-app/telegram-prompt-rerun-verification-templates/02-verification-engine-and-writing-styles-spec.md`  
> **Parent Plan:** `.ai-memory/plans/telegram-prompt-rerun-verification-templates.md`  
> **Associated Subtask:** `.ai-memory/plans/subtasks/telegram-prompt-rerun-verification-templates/02-verification-engine-and-writing-styles.md`  
> **Status:** APPROVED & READY FOR WORKER 02  
> **Author:** Spec Author 02  
> **Primary Attribution:** `@aukgit` (`(Thanks to @aukgit)`)  
> **Target Release:** `v4.223.0`  
> **Target Modules:**
> - `src/components/instances/PromptTreeView/verificationTemplates.ts` (Engine Extension)
> - `src/components/instances/PromptTreeView/VerificationModal.tsx` (New Component)
> - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` (Hook Extension)
> - `src-tauri/src/modules/repo_db/verification.rs` (New Backend Module)
> - `src-tauri/src/modules/repo_db.rs` (Facade Re-export)
> - `src-tauri/src/commands/project_cmds.rs` (Tauri IPC Registration)

---

## User Request (Verbatim)

The user prompt establishing this capability is recorded verbatim and losslessly:

> "Verification would be a segment where we get in, check the instances, and then pick which prompts that needs to send as a verification two times or three times. And that would have a template, usually that can be also changed from the UI... and add more verification steps on writing. Each one of them will have different writings so that the AI follows through, understand where they have made mistakes, and they could fix it."

---

## 1. Executive Summary & Problem Classification

### 1.1 The "Superficial Affirmation" Anti-Pattern
In autonomous software development workflows with agentic models (Claude 3.7 Sonnet/Thinking, Gemini 2.5 Pro, OpenAI o3-mini), requesting a verification audit with a generic or unanchored prompt frequently results in superficial affirmations. When asked a standard question such as *"Did you verify this?"*, the model reads the file high-level, assumes happy-path execution, and outputs *"Yes, everything is verified and working perfectly!"* without:
1. Actually executing targeted unit tests or pre-flight compiler gates.
2. Formulating a structured 4-part Root Cause Analysis (RCA) for touched failure points.
3. Conducting adversarial boundary and concurrency stress testing.
4. Performing 1:1 bidirectional verification against spec checkboxes.

### 1.2 The "Monolithic Single-Shot Verification" Limitation
A single verification round often fails to uncover subtle defects or regression cascades:
- **Round 1 (Direct Compliance):** The AI answers basic acceptance questions, but may overlook edge cases or negative error handling branches.
- **Round 2 (Stress & Adversarial Testing):** Only after the initial implementation is verified can the model be nudged to act as an adversary, seeking out race conditions, null references, and malformed network payloads.
- **Round 3 (Production Readiness & Governance):** Only after edge cases are hardened does the release gate inspect 14-location SemVer synchronization, changelog attribution to `@aukgit`, and error-store visibility.

When forced into a single omnibus verification prompt, LLMs suffer context fatigue, truncating later sections and skimming over release blockers.

### 1.3 The Target Solution: Verification Engine with 4 Writing Styles & Multi-Round Repetition
This specification establishes an interactive, multi-round verification architecture:
1. **Dedicated UI Verification Modal (`VerificationModal.tsx`):** Allows operators to inspect workspace instances, select specific historical prompt turns, choose multi-round repetition (`1x`, `2x`, `3x`), and select from 4 specialized writing styles.
2. **4 Specialized Verification Writing Styles:**
   - `root-cause-auditor` (Strict Root-Cause Auditor): Demands 4-part RCA, reproduction trace, and architectural prevention invariants.
   - `socratic-skeptic` (Socratic Skeptic / Devil's Advocate): Aggressively challenges premature claims of success; poses probing questions on edge cases, race conditions, nulls, timeouts, and unverified assumptions.
   - `spec-gatekeeper` (Acceptance Criteria Gatekeeper): Demands 1:1 bidirectional line-by-line verification against spec checkboxes (`02-spec/...`) and verbatim user prompt.
   - `release-blocker` (Production Release Blocker Inspector): Enforces 14-location SemVer sync, strict `@aukgit` attribution, pre-flight checks, and error visibility.
3. **Multi-Round Repetition Engine (2x / 3x):**
   - Synthesizes distinct, evolving chapters for each round so the AI stays focused on escalating verification depths.
   - Enforces 500ms safety throttling between round dispatches to prevent `.antigravity_resume_task.json` file overwrite collisions.
   - Inserts generated prompts directly into the FIFO scheduling queue (`ORDER BY created_at ASC, id ASC`).

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│               Interactive Verification Modal & Multi-Round Engine Data Flow             │
└────────────────────────────────────────────────────────────────────────────────────────┘

  [PromptTreeView / ContextMenu]
          │  (Opens modal with selected conversation & turns)
          ▼
  ┌────────────────────────────────────────────────────────────────────────────────────┐
  │ VerificationModal.tsx                                                              │
  │ • Checkbox Turn Selector: [✓ Turn 1 | ✓ Turn 2 | Turn 3]                          │
  │ • Round Selector Pills:   [ 1x | 2x | 3x ]                                         │
  │ • Writing Style Cards:    [root-cause-auditor | socratic-skeptic |                 │
  │                            spec-gatekeeper    | release-blocker ]                  │
  │ • Live Tabbed Preview:    Round 1 Preview | Round 2 Preview | Round 3 Preview      │
  └────────────────────────────────────────────────────────────────────────────────────┘
          │
          │  "Synthesize & Enqueue Verification"
          ▼
  ┌────────────────────────────────────────────────────────────────────────────────────┐
  │ verificationTemplates.ts                                                           │
  │ • buildMultiRoundVerificationPrompts(style, turns, rounds, context)               │
  │ • Generates Round 1, Round 2, Round 3 structured Markdown payloads                │
  └────────────────────────────────────────────────────────────────────────────────────┘
          │
          │  IPC: enqueue_verification_rounds
          ▼
  ┌────────────────────────────────────────────────────────────────────────────────────┐
  │ src-tauri/src/modules/repo_db/verification.rs                                      │
  │ • Iterates through rounds 1..N                                                     │
  │ • Enforces 500ms safety sleep between repeated rounds                              │
  │ • Inserts into active_prompts FIFO queue (ORDER BY created_at ASC, id ASC)         │
  │ • Emits audit trail record & invalidates prompt tree cache                         │
  └────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. The 4 Specialized Verification Writing Styles

To ensure the AI understands where mistakes were made and fixes them progressively, the engine defines 4 distinct writing styles. Each style features a unique tone, non-negotiable auditing mandates, targeted evidence contracts, and a specific cognitive focus.

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           4 Specialized Verification Writing Styles                             │
├───────────────────────┬──────────────────────────────┬──────────────────────────────────────────┤
│ Style Identifier      │ Cognitive Role               │ Primary Enforcement Domain               │
├───────────────────────┼──────────────────────────────┼──────────────────────────────────────────┤
│ root-cause-auditor    │ Strict Root-Cause Auditor    │ 4-part RCA, Reproduction, Invariants     │
│ socratic-skeptic      │ Socratic Skeptic / Adversary │ Edge Cases, Race Conditions, Null Safety │
│ spec-gatekeeper       │ Acceptance Gatekeeper        │ 1:1 Spec Checkboxes, Verbatim Fidelity   │
│ release-blocker       │ Production Release Blocker   │ SemVer Sync, @aukgit, Pre-flight Gates   │
└───────────────────────┴──────────────────────────────┴──────────────────────────────────────────┘
```

### 2.1 Style 1: Strict Root-Cause Auditor (`root-cause-auditor`)
- **Role & Persona:** Lead Diagnostic Forensics Engineer.
- **Mental Model:** Uncompromising skepticism toward superficial symptom patches. Demands physical verification of underlying failure mechanisms.
- **Auditing Mandate:**
  1. **4-Part Root Cause Analysis (RCA):**
     * **Problem:** Exact observed failure or regression with bounded reproduction trace.
     * **Root Cause:** Grounded explanation of why the failure occurred (code path, variable state, timing window).
     * **Fix:** Minimal, surgical modification that eliminates the root cause without collateral code churn.
     * **Prevention Invariant:** Structural, testable rule guaranteeing the problem class can never recur.
  2. **Reproduction Evidence:** Must state the exact command, payload, or unit test that triggers the failure before the fix and passes after the fix.
  3. **No Speculative Hypotheses:** Every assertion must reference physical file paths and line ranges (`src/...#L12-L34`).

#### Template Envelope Snippet:
```markdown
# 🚨 VERIFICATION AUDIT: STRICT ROOT-CAUSE ANALYSIS & FAILURE FORENSICS
> **Writing Style:** `root-cause-auditor` | **Round:** {{round}} of {{totalRounds}}

### Mandatory 4-Part RCA Contract:
Every touched module or resolved defect MUST be audited against the following 4-part schema:
1. **Problem Statement:** Cite exact symptoms, error logs, and stack traces.
2. **Root Cause Analysis:** Explain the exact physical mechanism of failure (unhandled branch, race condition, type coercion).
3. **Surgical Remediation:** Confirm that only minimal, necessary lines were changed.
4. **Architectural Prevention Invariant:** Formulate an explicit invariant preventing regression.

| Defect / Touched Module | Root Cause Diagnosis | Minimal Surgical Fix | Prevention Invariant | Status |
| :--- | :--- | :--- | :--- | :---: |
| `src-tauri/src/...` | Exact flaw explanation | Line-level fix reference | Invariant I-X | PASS/FAIL |
```

---

### 2.2 Style 2: Socratic Skeptic / Devil's Advocate (`socratic-skeptic`)
- **Role & Persona:** Adversarial QA Architect & Devil's Advocate.
- **Mental Model:** Treats every claim of "it works" as guilty until proven innocent. Actively crafts destructive edge cases, malicious inputs, and concurrency hazards.
- **Auditing Mandate:**
  1. **Boundary & Stress Probing:**
     * What happens on 0-length inputs, 10MB payloads, 50 concurrent requests, or disconnected network sockets?
     * What happens if the SQLite database is locked (`SQLITE_BUSY`), the file system is read-only, or the process receives `SIGINT`?
  2. **Race Condition & Null Safety:**
     * Are asynchronous promises or background Tokio tasks detached without cancellation tokens?
     * Can `undefined`, `null`, `None`, or empty strings crash downstream DOM renderers or Tauri IPC handlers?
  3. **No Blind Assumptions:** Demands mathematical proof or deterministic logs demonstrating resilience under fault injection.

#### Template Envelope Snippet:
```markdown
# ⚡ ADVERSARIAL VERIFICATION: SOCRATIC SKEPTIC & BOUNDARY STRESS PROBE
> **Writing Style:** `socratic-skeptic` | **Round:** {{round}} of {{totalRounds}}

### Adversarial Challenge Invariant:
Do NOT affirm success. Instead, attempt to break your implementation by answering:
1. **Conformity Under Extremes:** How does this code behave when passed empty strings, unicode emoji storms, or 100,000-character payloads?
2. **Concurrency & Deadlocks:** Can two rapid UI clicks trigger parallel dispatch races, overwriting state?
3. **Failure Isolation:** If an internal helper throws, is the error caught cleanly by AppError/errorStore or does it crash the application?

| Adversarial Scenario | Stress Mechanism | Expected Behavior | Observed Physical Proof | Verdict |
| :--- | :--- | :--- | :--- | :---: |
| Zero / Null Input | `count = 0` / empty string | Graceful error toast | Tested via unit test | IMMUNE |
| Concurrency Race | Double click within 50ms | Guard locks second click | Debounce / state flag | IMMUNE |
```

---

### 2.3 Style 3: Acceptance Criteria Gatekeeper (`spec-gatekeeper`)
- **Role & Persona:** Strict Specification Compliance Officer.
- **Mental Model:** Purely binary, contract-driven verification. Compares code line-by-line against specification documents (`02-spec/...`) and the original user instructions.
- **Auditing Mandate:**
  1. **1:1 Bidirectional Traceability:**
     * Every checkbox in the specification document must map to a concrete code implementation.
     * Every directive in the user's prompt must have an explicit proof citation.
  2. **Zero Orphaned Tasks:** Unimplemented items or deferred tasks must be flagged as non-negotiable failures; partial marks are prohibited.
  3. **Verbatim Fidelity:** UI labels, button texts, sequence tokens (`[AGM:P001]`), and keyboard shortcuts must match specification strings character-for-character.

#### Template Envelope Snippet:
```markdown
# 📋 SPECIFICATION GATEKEEPER: 1:1 ACCEPTANCE CRITERIA AUDIT
> **Writing Style:** `spec-gatekeeper` | **Round:** {{round}} of {{totalRounds}}

### Strict Specification Verification Contract:
Audit the implemented code against every line item of the parent specification:
- Every `- [ ]` checkbox must be either verified as `- [x]` with file citations or reported as FAILED.
- All UI component names, props, and Tauri IPC command signatures must match specification types verbatim.

| Spec Item ID | Specification Directive | Physical Code Location | Line-by-Line Evidence | Compliance |
| :--- | :--- | :--- | :--- | :---: |
| SPEC-2.1 | 4 Writing Styles Defined | `verificationTemplates.ts` | Lines 50-120 | 100% PASS |
| SPEC-2.2 | 500ms Safety Throttling | `verification.rs` | Lines 85-92 | 100% PASS |
```

---

### 2.4 Style 4: Production Release Blocker Inspector (`release-blocker`)
- **Role & Persona:** Principal Release Governance Auditor.
- **Mental Model:** Protects production channels from unverified code, attribution leaks, broken pre-flight gates, and silent error swallowing.
- **Auditing Mandate:**
  1. **14-Location SemVer Sync:** Versions across `package.json`, `Cargo.toml`, `tauri.conf.json`, `npm-shrinkwrap.json`, and all 14 manifests must be identical.
  2. **Strict `@aukgit` Attribution:** Changelogs (`CHANGELOG.md`, `CHANGELOG_EN.md`) and Readmes (`README.md`, `README_EN.md`) must credit exclusively `@aukgit` (`(Thanks to @aukgit)`). Zero other user handles allowed.
  3. **Pre-flight Quality Gates:** Clean run of `cd src-tauri && cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, and `npm run build`.
  4. **Zero Silent Errors (House Rule 2026-10-10):** No bare `let _ = ...`, no bare `catch {}`, no `.unwrap_or_default()` without error routing.

#### Template Envelope Snippet:
```markdown
# 🛑 PRODUCTION RELEASE BLOCKER: GOVERNANCE & QUALITY GATES INSPECTION
> **Writing Style:** `release-blocker` | **Round:** {{round}} of {{totalRounds}}

### Non-Negotiable Release Gates:
Before certifying release readiness, the following gates MUST pass without exception:
1. **Atomic Version Sync:** Check all 14 version manifest locations for `vX.Y.Z` parity.
2. **Strict Attribution Invariant:** Verify that ONLY `@aukgit` is credited in changelogs.
3. **Pre-flight Quality Gates:** Run `cargo fmt`, `cargo clippy`, and `npm run build`.
4. **Error Visibility Invariant:** Confirm all fallible operations route into `AppError` or `useErrorStore`.

| Release Gate | Verification Command | Gate Criteria | Result | Blocker Status |
| :--- | :--- | :--- | :---: | :---: |
| Rust Formatting | `cargo fmt -- --check` | Exit code 0 | PASS | CLEAR |
| Rust Clippy | `cargo clippy --all-features` | Zero warnings | PASS | CLEAR |
| Frontend Build | `npm run build` | Clean bundle, zero TS errors | PASS | CLEAR |
| Error Visibility | Static inspection | Zero bare `catch {}` | PASS | CLEAR |
```

---

## 3. Multi-Round Repetition Architecture (2x / 3x)

### 3.1 Progressive Round Escalation
When the operator selects 2 or 3 verification rounds (`2x` or `3x`), the engine generates distinct prompts for each round. This prevents the AI from repeating identical checks and progressively escalates the depth of audit:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        3-Round Verification Escalation Ladder                          │
└────────────────────────────────────────────────────────────────────────────────────────┘

  Round 1: Baseline Compliance & Acceptance Audit
  ├── Verification Scope: 1:1 directive check against original user prompts
  ├── Focus: Did we build what was asked? Do the components render? Does the API respond?
  └── Outcome: Baseline checklist with line-level file citations

          │  (500ms Safety Throttle)
          ▼
  Round 2: Adversarial Stress, Edge Cases & Error Recovery
  ├── Verification Scope: Negative tests, boundary conditions, concurrency races
  ├── Focus: What breaks under malicious input? Are error branches routed through AppError?
  └── Outcome: Hardening against crashes, memory leaks, and silent failures

          │  (500ms Safety Throttle)
          ▼
  Round 3: Production Sign-Off, Quality Gates & Release Readiness
  ├── Verification Scope: Pre-flight checks (clippy, fmt, build), SemVer sync, @aukgit attribution
  ├── Focus: Is the code clean, zero-warning, properly documented, and ready for deployment?
  └── Outcome: Final binary sign-off verdict [PASS / FAIL]
```

### 3.2 Round Header & Banner Standard
Every generated verification prompt begins with a standardized, prominent banner:
```markdown
# 🛡️ VERIFICATION ROUND {{round}} OF {{totalRounds}}: [{{styleTitle}}]
> [!IMPORTANT]
> **Verification Escalation Directive (Round {{round}} of {{totalRounds}}):**
> You are executing Round {{round}} of an automated {{totalRounds}}-round verification cycle.
> Focus your cognitive effort strictly on the designated depth for this round.
```

### 3.3 Safety Throttling Invariant (500ms Sleep)
- **Problem:** If multiple verification prompts are pushed to the target workspace or `.antigravity_resume_task.json` simultaneously, file system write races corrupt the task envelope. The target IDE may drop the first round or execute them out of order.
- **Rule:** The dispatch engine (`src-tauri/src/modules/repo_db/verification.rs`) MUST enforce a **minimum 500ms asynchronous sleep** (`tokio::time::sleep(Duration::from_millis(500)).await`) between sequential rounds.
- **Queue Insertion:** All verification prompts are appended to the `active_prompts` SQLite table with strictly sequential `created_at` timestamps to preserve FIFO execution order (`ORDER BY created_at ASC, id ASC`).

---

## 4. UI Component Architecture (`VerificationModal.tsx`)

### 4.1 Modal Overview & Layout
The `VerificationModal.tsx` component is a modern dark-glass modal providing complete interactive control over prompt verification.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ 🛡️ Synthesize Prompt Verification Audit                                             [✕] │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Conversation: [AGM:C001] "Fix Telegram Inbound Rerun Callbacks"                       │
│ Instance: default | Repo: Antigravity-Manager                                          │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ 1. Select Prompt Turns to Audit:                               [Select All] [Last 3]   │
│ ┌────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ [✓] [AGM:P001] Implement 4-part RCA verification template...                       │ │
│ │ [✓] [AGM:P002] Add 60-byte Telegram callback cap guard (RCA 15)...                 │ │
│ │ [ ] [AGM:P003] Update changelog attribution for @aukgit...                         │ │
│ └────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                        │
│ 2. Repetition Rounds:                                                                  │
│    [ 1x Single Audit ]   [ 2x Baseline + Stress ]   [ 3x Full Release Gate (Rec) ]     │
│                                                                                        │
│ 3. Choose Writing Style:                                                               │
│ ┌──────────────────────────┬──────────────────────────┐                                │
│ │ 🔍 Root-Cause Auditor    │ ⚡ Socratic Skeptic      │                                │
│ │ 4-part RCA & Invariants  │ Edge cases & Stress      │                                │
│ ├──────────────────────────┼──────────────────────────┤                                │
│ │ 📋 Spec Gatekeeper       │ 🛑 Release Blocker       │                                │
│ │ 1:1 Checkbox compliance  │ SemVer, Gates & @aukgit  │                                │
│ └──────────────────────────┴──────────────────────────┘                                │
│                                                                                        │
│ 4. Live Prompt Preview: [Round 1 Preview] [Round 2 Preview] [Round 3 Preview]          │
│ ┌────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ # 🛡️ VERIFICATION ROUND 1 OF 3: [Acceptance Criteria Gatekeeper]                   │ │
│ │ > You are acting as the Lead Verification Auditor...                               │ │
│ └────────────────────────────────────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ [ Cancel ]                                         [ 🚀 Synthesize & Enqueue (3 Rounds) ] │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Component Properties & State Interface
```typescript
export interface VerificationModalProps {
    isOpen: boolean;
    onClose: () => void;
    conversation: AgmConversationNode | null;
    instanceId: string;
    repoPath: string;
    onSuccess?: (enqueuedCount: number) => void;
}

export type VerificationWritingStyle =
    | 'root-cause-auditor'
    | 'socratic-skeptic'
    | 'spec-gatekeeper'
    | 'release-blocker';

export type VerificationRoundOption = 1 | 2 | 3;
```

### 4.3 Key UI Features
1. **Interactive Turn Selection:**
   - Displays all user prompt turns extracted from `conversation.sub_runs` or parsed transcript.
   - Shows dual sequence tokens (`[AGM:P001]`, `[AGM:P002]`) and word-clamped previews.
   - Quick-select action buttons: *"Select All"*, *"Last 3"*, *"Last 5"*, *"Clear"*.
2. **Segmented Round Selector:**
   - 3 segmented pill buttons (`1x`, `2x`, `3x`).
   - Tooltips explaining the focus of each repetition level.
3. **Visual Style Selector Cards:**
   - 4 dark-glass cards with distinctive Lucide icons (`Search`, `Zap`, `CheckSquare`, `ShieldAlert`).
   - Selected card displays high-contrast border and active badge.
4. **Interactive Multi-Round Preview Tabs:**
   - Tabs allowing the user to inspect the exact markdown synthesized for Round 1, Round 2, and Round 3 before enqueueing.
5. **Enqueue Action Button:**
   - Shows current target: e.g. *"Synthesize & Enqueue (3 Rounds)"*.
   - Disables with loading spinner during Tauri IPC execution.
   - Triggers desktop toast on success and closes modal automatically.

---

## 5. Backend Engine Architecture (`src-tauri/src/modules/repo_db/verification.rs`)

### 5.1 Responsibilities
The `verification.rs` backend module provides high-reliability dispatching and queuing for multi-round verification jobs:
1. Validates instance existence, repo path, and conversation identity.
2. Iterates over synthesized rounds with **500ms safety sleep** between submissions.
3. Inserts each prompt into `active_prompts` with status `'queued'`, guaranteeing strict FIFO scheduling.
4. Emits structured Audit Trail records (`AuditAction::VerificationEnqueue`).
5. Invalidates prompt tree cache so the GUI updates instantly.

### 5.2 Rust Module Implementation Design
```rust
//! src-tauri/src/modules/repo_db/verification.rs
//! Dedicated backend module for multi-round prompt verification scheduling.

use super::models::ActivePrompt;
use super::schema::connect_db;
use super::tree::invalidate_prompt_tree_cache;
use crate::error::AppError;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

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

/// Enqueue multiple verification rounds into the active_prompts FIFO queue
/// enforcing a 500ms safety sleep between sequential round insertions.
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

        // 1. Insert into active_prompts FIFO queue via blocking task
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
                    now + (idx as i64), // Ensure strictly increasing created_at for FIFO order
                ],
            ).map_err(|e| AppError::Database(format!("Failed to insert verification prompt: {e}")))?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::Process(format!("Tokio spawn failure: {e}")))??;

        enqueued += 1;
        info!("Enqueued verification round {}/{} for conv {}", round_num, total_rounds, req.conversation_id);

        // 2. Enforce 500ms safety throttling between rounds (skip after last round)
        if round_num < total_rounds {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // 3. Invalidate prompt tree cache to reflect queued state in UI
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

### 5.3 Tauri IPC Registration
In `src-tauri/src/commands/project_cmds.rs`:
```rust
#[tauri::command]
pub async fn enqueue_verification_rounds_cmd(
    req: crate::modules::repo_db::verification::VerificationEnqueueRequest,
) -> Result<crate::modules::repo_db::verification::VerificationEnqueueResult, AppError> {
    crate::modules::repo_db::verification::enqueue_verification_rounds(req).await
}
```

---

## 6. TypeScript Architecture & Generator Extension

### 6.1 Extended Types in `verificationTemplates.ts`
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
    iconName: string;
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

### 6.2 Multi-Round Generation Function
`buildMultiRoundVerificationPrompts(options: MultiRoundVerificationOptions): string[]` synthesizes an array of $N$ strings corresponding to rounds $1 \dots N$.
- **Round 1:** Focused on direct acceptance and baseline verification according to the selected writing style.
- **Round 2:** Focused on adversarial edge cases, stress testing, concurrency hazards, and regression verification.
- **Round 3:** Focused on production sign-off, quality gates, pre-flight checks (`cargo clippy`, `cargo fmt`, `npm run build`), and release readiness.

---

## 7. Architectural Invariants

| ID | Invariant Name | Scope | Description |
| :---: | :--- | :--- | :--- |
| **I1** | **Strict FIFO Queue Invariant** | Backend SQLite | All verification prompts enqueued into `active_prompts` must maintain strict FIFO ordering using `ORDER BY created_at ASC, id ASC`. |
| **I2** | **500ms Safety Throttling** | Backend Dispatch | Sequential verification rounds (2x/3x) must sleep for at least 500ms between submissions to prevent `.antigravity_resume_task.json` file collisions. |
| **I3** | **Zero Silent Errors** | Frontend & Backend | Every fallible operation must route failures to `AppError` (Rust) or `useErrorStore` (TypeScript). Bare `catch {}` is strictly forbidden. |
| **I4** | **Strict Relative Paths** | Documentation & UI | All file paths presented in verification scorecards and generated prompts must be strictly relative to the repository root (`src/...`, `src-tauri/...`). |
| **I5** | **Strict `@aukgit` Attribution** | Release Blocker | All changelogs and documentation must credit exclusively `@aukgit` (`(Thanks to @aukgit)`). No other GitHub user handles are allowed. |
| **I6** | **Verbatim User Request Preservation** | Verification Envelope | The user prompt turns selected for verification must be embedded verbatim inside `<<<AUDITED_USER_PROMPT_START>>>` delimiters. |

---

## 8. Verification & Acceptance Testing Protocol

To certify this specification upon implementation by Worker 02:
1. **Unit Test Verification:**
   - Execute unit tests for `verificationTemplates.ts` verifying that `buildMultiRoundVerificationPrompts` generates distinct, properly formatted markdown strings for all 4 writing styles across rounds 1, 2, and 3.
2. **UI Verification Modal Flow:**
   - Open `PromptTreeViewModal`, select an active conversation, click the Verification action button from `DetailHeader` or context menu.
   - Confirm that `VerificationModal` renders with the prompt turns checkbox list, 3 round selector pills, and 4 writing style cards.
   - Change selection and verify that the tabbed preview updates instantaneously.
3. **Multi-Round Enqueue Verification:**
   - Select 3 rounds and click "Synthesize & Enqueue".
   - Confirm that 3 separate prompts appear in the FIFO queue with sequential `created_at` timestamps spaced at least 500ms apart.
   - Verify that the target IDE receives and executes each verification round sequentially.
4. **Pre-flight Quality Gates:**
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
