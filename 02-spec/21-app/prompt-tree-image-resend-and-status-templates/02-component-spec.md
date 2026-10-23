# Component Specification: Rich Multi-Chapter Verification & Rerun Prompt Template Architecture

**Specification ID:** `02-spec/21-app/prompt-tree-image-resend-and-status-templates/02-component-spec.md`  
**Parent Plan:** `.ai-memory/plans/prompt-tree-image-resend-and-status-templates.md`  
**Associated Subtask:** `.ai-memory/plans/subtasks/prompt-tree-image-resend-and-status-templates/02-status-templates.md`  
**Status:** `DRAFT`  
**Target Modules:**
- `src/components/instances/PromptTreeView/verificationTemplates.ts` (New Module)
- `src/components/instances/PromptTreeView/DetailHeader.tsx` (Component Modification)
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` (Hook Modification)

---

## 1. Executive Summary & Problem Classification

### 1.1 The "Bare 3-Word Suffix" Anti-Pattern (Root Cause)
In previous versions of the Antigravity-Manager Prompt Tree View, when a user selected a confirmation suffix (e.g. `Done?`, `Released?`, `Sure?`) or triggered a rerun action from `DetailHeader.tsx`, the system merely appended an isolated 3-word string:
```text
Is it done?
```
or
```text
Is it released?
```
When dispatched to active IDE instances or enqueued into the background scheduler, this terse string caused severe failure modes:
1. **Context Loss & Hallucination**: AI models receiving only `"Is it done?"` lacked the original task context, acceptance criteria, file references, and specific verification guidelines. Models frequently responded with superficial affirmations ("Yes, it is done!") without performing actual code checks or tests.
2. **Ambiguity on Release Standards**: The `"Is it released?"` suffix failed to enforce repository-mandated release rules, such as 14-location SemVer synchronization, strict `@aukgit` attribution, README updates, or pre-flight gate executions (`cargo clippy`, `cargo fmt`, `npm run build`).
3. **Loss of Original Prompt during Rerun**: Clicking "Rerun" dispatched bare follow-up nudges (e.g. `"Quick status check — is it done yet?"`), completely stripping the original prompt and conversation goals.

### 1.2 Target Architecture: 5-Chapter Evidence-Driven Verification Prompt Engine
This specification introduces a centralized, modular verification and rerun prompt generator:
`src/components/instances/PromptTreeView/verificationTemplates.ts`

Instead of unanchored suffix strings, the generator synthesizes a structured, **5-Chapter Multi-Tier Audit Prompt** that wraps the original task prompt and enforces rigorous verification contracts:
- **Chapter 1: 🚨 High Priority Audit Mandate** — Non-negotiable top-instruction priority mandate, role designation, and anti-hallucination directive.
- **Chapter 2: 📊 Domain-Specific Verification Checklists** — Comprehensive, binary verification matrices for `"Is it done?"`, `"Is it released?"`, `"Are you confident?"`, `"Double check all edge cases"`, and `"Verify build and tests"`.
- **Chapter 3: 🔍 Git & Telemetry Gates** — Mandatory environment checks (`git status --porcelain`, branch validation, `gitmap pe -t` pipeline health).
- **Chapter 4: 📋 Verbatim Original Prompt Embedding** — Faithful encapsulation of the original user prompt, attachments, and relative asset links.
- **Chapter 5: 📝 Evidence Scorecard Contract** — Mandatory structured output format (Markdown table or structured JSON) with line-numbered proof and a binary completion verdict.

```
┌────────────────────────────────────────────────────────────────────────┐
│                   5-Chapter Verification Prompt Envelope               │
├────────────────────────────────────────────────────────────────────────┤
│ Chapter 1: 🚨 High Priority Audit Mandate                              │
│            • Top-priority directive & anti-hallucination mandate       │
├────────────────────────────────────────────────────────────────────────┤
│ Chapter 2: 📊 Domain-Specific Verification Checklists                  │
│            • 'Is it done?' / 'Is it released?' / 'Are you confident?'  │
├────────────────────────────────────────────────────────────────────────┤
│ Chapter 3: 🔍 Environment, Git & Telemetry Gates                      │
│            • git status, branch checks, gitmap pe -t pipeline status   │
├────────────────────────────────────────────────────────────────────────┤
│ Chapter 4: 📋 Verbatim Original Prompt Embedding                       │
│            • Full user prompt preserved verbatim with image links      │
├────────────────────────────────────────────────────────────────────────┤
│ Chapter 5: 📝 Concluding Evidence Scorecard Contract                   │
│            • Strict Markdown table scorecard: Item | Status | Evidence │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. The 5-Chapter Verification Architecture

### 2.1 Chapter 1: 🚨 High Priority Audit Mandate
Chapter 1 establishes the operational authority and mental model of the responding AI model.

#### Content & Directives:
- **Priority Declaration**: Any instruction within this audit block supersedes vague conversational defaults.
- **Auditor Role**: The model acts as the **Lead Verification Auditor**.
- **Anti-Hallucination & Evidence Invariant**: Assumptions, unverified claims, and subjective statements (e.g. "I believe the code is correct") are strictly forbidden. Every claim must be backed by exact file paths, line ranges, or command outputs.
- **Zero Silent Failures**: Swallowing errors, ignoring failing tests, or omitting pending items is classified as a critical defect.

```markdown
# 🚨 CHAPTER 1: HIGH PRIORITY AUDIT MANDATE

> [!IMPORTANT]
> **Audit Authority & Non-Negotiable Precedence:**
> You are acting as the Lead Verification Auditor. Your objective is to conduct a strict,
> evidence-based verification of the task described below.
> 
> - **Anti-Hallucination Policy:** Do NOT assume, extrapolate, or claim completion without inspecting the exact files and running the necessary checks.
> - **Zero Silent Failures:** Every check must yield explicit verifiable proof. If any step is unfinished, broken, or untested, explicitly report it as FAILED with actionable diagnostics.
```

---

### 2.2 Chapter 2: 📊 Domain-Specific Verification Checklists
Chapter 2 supplies concrete, domain-specific checklists mapped to the user's selected verification intent.

#### 2.2.1 Checklist: "Is it done?" (`is-it-done`)
MANDATORY verification points for task completion:
1. **Codebase Inspection & Citation**: Read and cite exact touched files and line numbers (`src/...#L10-L25` or `src-tauri/src/...#L50-L75`).
2. **1-Sentence Root Cause Diagnosis**: State clearly the root cause of the initial problem that was resolved.
3. **Minimum Surgical Change**: Verify that all modifications represent the minimal, robust change required, avoiding unrelated refactoring or code churn.
4. **Verifiable Execution Proof**: Validate that unit tests compile and run, UI elements render properly, or CLI commands execute without error.
5. **Specification & Plan Synchronization**: Ensure that `.ai-memory/plans/` and subtask files have been updated, all completed action items are checked (`- [x]`), and no orphan TODOs remain.

```markdown
### Chapter 2.1: 'Is It Done?' Verification Matrix
- [ ] **1. Exact Code Inspection:** Have you read and cited all modified source files with line-numbered links?
- [ ] **2. Root Cause Articulation:** Have you clearly stated the 1-sentence root cause of the issue?
- [ ] **3. Surgical Implementation:** Are all modifications strictly scoped to the problem without unnecessary code churn?
- [ ] **4. Concrete Verification:** Have you run targeted tests or verified UI/CLI behavior with real command output?
- [ ] **5. Memory & Plan Sync:** Are all relevant plans under `.ai-memory/plans/` updated and marked `COMPLETED`?
```

#### 2.2.2 Checklist: "Is it released?" (`is-it-released`)
MANDATORY release engineering and governance checks derived from `AGENTS.md` and repository release guidelines:
1. **Branch Hygiene**: Verify the current branch matches the target release channel (`main` for official stable releases, `beta` for preview pre-releases).
2. **14-Location SemVer Synchronization**: Validate that versions match across all required configuration manifests:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
   - `version.json`
   - Other project manifests managed by `npm run bump`.
3. **Strict `@aukgit` Attribution**: Audit `changelog.md` and `changelog_en.md` to ensure attribution strictly credits `@aukgit` (`(Thanks to @aukgit)`). No other GitHub handles (`@...`) may appear in release notes.
4. **README Changelog Synchronization**: Ensure that release notes in both `README.md` and `README_EN.md` (under "## 📝 Changelog" / "## 📝 Changelog") are fully synchronized with the latest version.
5. **Pre-Flight Quality Gates**:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
6. **Tag & Release Conventions**: Tag format must be `vX.Y.Z` on `main` or `vX.Y.Z-beta.N` on `beta`.
7. **Installer Parity**: Confirm installer scripts (`install.ps1`, `install.sh`) reference the correct release paths.
8. **Thinking Store Invalidation Toggle**: Check `src/components/common/SuggestionDeleteThinkingModal.tsx` (`SUGGESTION_DELETE_THINKING_STORE` set to `false` for standard releases or `true` with target version for schema upgrades).

```markdown
### Chapter 2.2: 'Is It Released?' Governance & Release Gate Matrix
- [ ] **1. Release Branch Separation:** Is the current branch strictly `main` (for stable) or `beta` (for preview)?
- [ ] **2. 14-Location SemVer Sync:** Are all 14 project manifest version fields atomically updated?
- [ ] **3. Strict Attribution Invariant:** Are changelogs attributed strictly to `@aukgit` (`(Thanks to @aukgit)`) with zero other handles?
- [ ] **4. README Synchronization:** Are both `README.md` and `README_EN.md` update logs synchronized with the release notes?
- [ ] **5. Pre-flight Quality Gates:** Have `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, and `npm run build` all passed?
- [ ] **6. Git Tag Precision:** Does the git tag strictly match `CHANGELOG.md` heading (e.g. `vX.Y.Z`)?
- [ ] **7. Distribution Scripts:** Are `install.ps1` and `install.sh` verified for the target release?
```

#### 2.2.3 Checklist: "Are you confident?" (`are-you-confident`)
Architectural quality and coding guideline conformance checks:
1. **Function & File Decomposition**: Functions must be <= 15 lines of core logic; oversized components must be decomposed into sub-components or custom hooks.
2. **Branch Immutability & Clean Construction**: Clean return helpers, guard clauses with early returns, discrete condition checks.
3. **Boolean Hygiene**: Implicit positive booleans only (`if (isReady)` vs `if (isReady === true)`). Zero double negatives or mixed polarity (`if (!isDisabled)` -> `if (isEnabled)`).
4. **No Silent Errors (Mandatory House Rule)**:
   - Rust: Errors must propagate via `AppError` or log via `tracing`. No bare `let _ =`, `.ok()`, or `.unwrap_or_default()` without error routing.
   - TypeScript: Errors must route via `useErrorStore.getState().captureException` or `trackWarning`. No bare `catch {}` blocks.
5. **Vertical Line Spacing**: Blank lines before `if`, blank lines after `}`, blank lines before `return`.
6. **Relative Path Discipline**: Strict relative path usage (`02-spec/...`, `src/...`); zero absolute machine paths (`C:\...`, `d:\...`).

```markdown
### Chapter 2.3: 'Are You Confident?' Architectural & Code Quality Matrix
- [ ] **1. Compact Functions:** Are functions bounded to <= 15 lines with single responsibility?
- [ ] **2. Guard Clauses & Early Returns:** Are nested `if/else` structures flattened with guard returns?
- [ ] **3. Positive Boolean Polarity:** Are all boolean flags positive, implicit, and free of double negatives?
- [ ] **4. Zero Silent Errors:** Does every catch block and error result route to the error store or AppError?
- [ ] **5. Vertical Rhythm:** Is vertical line spacing enforced (blank lines around blocks and before returns)?
- [ ] **6. Relative Paths:** Are all referenced file paths strictly repository-relative?
```

#### 2.2.4 Checklist: "Double check all edge cases" (`edge-cases`)
Robustness and boundary condition checks:
- Empty data states, null/undefined inputs, and collection bounds.
- Disconnected networks, offline mode, and API timeout scenarios.
- Multi-instance concurrency, race conditions, and PID recycling protection.
- Unicode, emoji, and multi-byte character preservation in prompts.

#### 2.2.5 Checklist: "Verify build and tests" (`verify-tests`)
Compilation and execution gate:
- Frontend build: `npm run build` succeeds with zero errors.
- Rust Clippy: `cd src-tauri && cargo clippy --all-targets --all-features` produces zero warnings/errors.
- Rust Formatter: `cd src-tauri && cargo fmt -- --check` passes cleanly.
- Unit Tests: Targeted unit tests pass with zero regressions.

---

### 2.3 Chapter 3: 🔍 Environment, Git & Telemetry Gates
Chapter 3 requires the model to verify environment health and repository status before making any assertions.

#### Required Verification Actions:
1. **Working Tree Cleanliness**: Run or check `git status --porcelain`. Identify any uncommitted or untracked changes.
2. **Current Branch Check**: Confirm current active branch via `git branch --show-current`.
3. **CI/CD Telemetry Check**: If available, check pipeline telemetry via `gitmap pe -t` or local runner to ensure prior runs are green.

```markdown
# 🔍 CHAPTER 3: ENVIRONMENT, GIT & TELEMETRY GATES

Run or verify the following diagnostic commands before completing your report:
1. **Working Tree Status:** Check `git status --porcelain` to identify modified, added, or untracked files.
2. **Active Branch:** Check `git branch --show-current` to confirm alignment with your task scope.
3. **Pipeline Health:** Ensure recent CI/CD pipeline steps are not failing (`gitmap pe -t` or local pre-flight runner).
```

---

### 2.4 Chapter 4: 📋 Verbatim Original Prompt Embedding
Chapter 4 solves the critical **Prompt Amnesia** defect. The original user instruction, including any markdown formatting, parameters, and referenced image attachment links, is preserved in its entirety inside a dedicated inspection block.

#### Formatting Rules:
- The original prompt is fenced inside a distinct markdown container.
- Relative image links (`![alt](./.antigravity/attachments/...)`) are preserved.
- The original prompt is NOT truncated, summarized, or paraphrased.

```markdown
# 📋 CHAPTER 4: VERBATIM ORIGINAL PROMPT & TASK CONTEXT

The following is the complete, un-truncated original task specification that must be verified:

---
<<<ORIGINAL_TASK_PROMPT_START>>>
${originalPrompt}
<<<ORIGINAL_TASK_PROMPT_END>>>
---
```

---

### 2.5 Chapter 5: 📝 Concluding Evidence Scorecard Contract
Chapter 5 imposes a strict output contract on the model. Rather than providing conversational text, the model must conclude with a structured Markdown table summarizing each gate.

```markdown
# 📝 CHAPTER 5: EVIDENCE SCORECARD CONTRACT

You MUST conclude your audit with the following Evidence Scorecard table:

| Verification Gate | Status | File / Line Reference | Verifiable Proof / Output |
| :--- | :---: | :--- | :--- |
| Codebase Citation & Reading | PASS / FAIL | `src/...#L12-L45` | File inspected and verified |
| 1-Sentence Root Cause | PASS / FAIL | N/A | Diagnostic statement provided |
| Surgical Code Change | PASS / FAIL | `src/...` | Zero extraneous edits |
| Unit Test / Build Gate | PASS / FAIL | Terminal / Output | Build passes with 0 warnings |
| Coding Guidelines (Booleans/Errors) | PASS / FAIL | `src/...` | Positive booleans, AppError handled |
| Memory & Plan Synchronization | PASS / FAIL | `.ai-memory/...` | Subtask marked COMPLETED |

### Concluding Verdict:
- **ALL_CRITERIA_MET:** `[TRUE | FALSE]`
- **BLOCKING_ISSUES:** `[None | Describe any unresolved issues]`
- **RECOMMENDED_NEXT_STEP:** `[Proceed to next subtask | Fix failing items | Release]`
```

---

## 3. Sequential Rerun Nudge Architecture

### 3.1 Nudge Mechanics & Iteration Counter
When the user clicks "Rerun" (e.g. `Rerun ×2 Now` or `Rerun ×3 Now`), the system executes sequential follow-up dispatches. Previously, each rerun was a detached message like `"Quick status check — is it done yet?"`.

Under the new architecture:
1. Every rerun dispatch includes an iteration header:
   `[Rerun Execution Nudge — Round {round} of {total}]`
2. Every rerun dispatch embeds the **full original prompt** in Chapter 4, preventing context drift across multiple cycles.
3. Every rerun dispatch explicitly instructs the model to resume incomplete steps rather than restarting completed work from scratch.

### 3.2 Context Preservation & Anti-Duplication Directives
The prompt template explicitly instructs the model:
> "Do NOT restart tasks that have already been verified. Check the plan ledger, inspect what files have already been modified, and immediately execute the remaining pending subtasks."

```markdown
# 🔄 RERUN EXECUTION NUDGE — ROUND ${round} OF ${total}

> [!NOTE]
> This is automated execution nudge **Round ${round} of ${total}** for the task below.
> 
> - **State Preservation:** Review files already modified and steps already completed. Do NOT duplicate work.
> - **Forward Progress:** Focus immediately on remaining incomplete or unverified action items.
> - **Self-Correction:** If an earlier step resulted in an error, apply the 4-part Root Cause Analysis (RCA) and correct it now.

### Original Task Prompt:
---
${originalPrompt}
---

Please provide an updated status of your progress in this round and complete any remaining work.
```

---

## 4. Module Interface Contract (`verificationTemplates.ts`)

### 4.1 Type Definitions
```typescript
/**
 * Supported verification template types.
 */
export type VerificationTemplateType =
    | 'is-it-done'
    | 'is-it-released'
    | 'are-you-confident'
    | 'edge-cases'
    | 'verify-tests'
    | 'rerun-nudge'
    | 'none';

/**
 * Metadata descriptor for verification options rendered in UI dropdowns.
 */
export interface VerificationOptionMeta {
    value: VerificationTemplateType;
    label: string;
    dropdownLabel: string;
    badgeLabel: string;
    description: string;
    iconName: 'check-circle' | 'rocket' | 'shield-check' | 'alert-triangle' | 'terminal' | 'repeat' | 'slash';
}

/**
 * Context supplied when generating a rich verification prompt.
 */
export interface VerificationTemplateContext {
    /** The complete original prompt text to embed into Chapter 4. */
    originalPrompt: string;
    /** Target instance identifier (e.g. 'default' or instance UUID). */
    instanceId?: string;
    /** Repository path for relative path anchoring. */
    repoPath?: string;
    /** Target conversation ID. */
    convId?: string;
    /** Current Git branch if known. */
    branch?: string;
    /** Current round for rerun nudges (1-indexed). */
    currentRound?: number;
    /** Total scheduled rounds for rerun nudges. */
    totalRounds?: number;
}

/**
 * Context supplied specifically for rerun nudges.
 */
export interface RerunTemplateContext {
    originalPrompt: string;
    round: number;
    total: number;
    instanceId?: string;
    repoPath?: string;
    convId?: string;
    customNudgeMessage?: string;
}
```

### 4.2 Module Export Interface
```typescript
/**
 * All available verification options for UI selection.
 */
export const VERIFICATION_OPTIONS: ReadonlyArray<VerificationOptionMeta>;

/**
 * Default option when none is explicitly selected.
 */
export const DEFAULT_VERIFICATION_TYPE: VerificationTemplateType = 'none';

/**
 * Maps legacy 3-word suffix strings to structured VerificationTemplateType.
 */
export function parseVerificationTypeFromSuffix(rawSuffix: string): VerificationTemplateType;

/**
 * Builds the complete 5-chapter verification prompt string.
 */
export function buildRichVerificationPrompt(
    type: VerificationTemplateType,
    context: VerificationTemplateContext
): string;

/**
 * Builds a structured rerun nudge prompt embedding iteration counters and original prompt.
 */
export function buildRerunPrompt(context: RerunTemplateContext): string;

/**
 * Retrieves the verification option metadata by type.
 */
export function getVerificationOption(type: VerificationTemplateType): VerificationOptionMeta | undefined;
```

---

## 5. UI Integration Contract

### 5.1 `DetailHeader.tsx` Suffix Dropdown Overhaul
In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
1. Suffix dropdown `<select>` options update from raw strings to `VerificationTemplateType` values:
   - `<option value="none">None (Send as is)</option>`
   - `<option value="is-it-done">Is it done? (Audit & Scorecard)</option>`
   - `<option value="is-it-released">Is it released? (Release Gates)</option>`
   - `<option value="are-you-confident">Are you confident? (Code Guidelines)</option>`
   - `<option value="edge-cases">Double check all edge cases</option>`
   - `<option value="verify-tests">Verify build and tests</option>`
2. In `handleRerun`, `activePromptText` is passed into `dispatchRerunMessages`:
   ```typescript
   const result = await dispatchRerunMessages({
       targetInstId,
       repoPath: rerunRepoPath ?? '',
       convId: targetConvId,
       count: rerunCount,
       messageOption: rerunMessageOption,
       originalPrompt: activePromptText, // Full prompt preservation
       source: 'DetailHeader.rerunSection',
   });
   ```

### 5.2 `usePromptTreePromptActions.ts` Dispatch Updates
In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
1. **Prompt Resolution**:
   Instead of:
   ```typescript
   if (confirmationSuffix && confirmationSuffix !== 'None (Send as is)') {
       promptContent = `${promptContent}\n\n${confirmationSuffix}`;
   }
   ```
   The action hook executes:
   ```typescript
   const templateType = parseVerificationTypeFromSuffix(confirmationSuffix);
   if (templateType !== 'none') {
       promptContent = buildRichVerificationPrompt(templateType, {
           originalPrompt: promptContent,
           instanceId: targetInstId,
           repoPath,
           convId: targetConvId,
       });
   }
   ```
2. **`dispatchRerunMessages` Iteration Loop**:
   ```typescript
   for (let i = 0; i < requested; i += 1) {
       const richRerunMessage = buildRerunPrompt({
           originalPrompt: ctx.originalPrompt || '',
           round: i + 1,
           total: requested,
           instanceId: ctx.targetInstId,
           repoPath: ctx.repoPath,
           convId: ctx.convId,
           customNudgeMessage: ctx.messageOption === 'randomized' 
               ? RERUN_RANDOM_MESSAGE_SET[Math.floor(Math.random() * RERUN_RANDOM_MESSAGE_SET.length)]
               : undefined,
       });

       await sendPromptNow(ctx.targetInstId, ctx.repoPath, richRerunMessage, ctx.convId);
       // Audit record with rich metadata...
   }
   ```
3. **`handleEnqueuePrompt` Alignment**:
   `handleEnqueuePrompt` also applies `buildRichVerificationPrompt` if a confirmation suffix is active, ensuring that enqueued prompts also enjoy rich verification chapters.

---

## 6. Acceptance Criteria

- [ ] **AC-1 (Module Creation):** `src/components/instances/PromptTreeView/verificationTemplates.ts` is implemented and exports `buildRichVerificationPrompt`, `buildRerunPrompt`, `parseVerificationTypeFromSuffix`, and `VERIFICATION_OPTIONS`.
- [ ] **AC-2 (Chapter 1 & 2):** Verification templates generate Chapter 1 (High Priority Audit Mandate) and Chapter 2 with domain-specific checklists for `is-it-done`, `is-it-released`, and `are-you-confident`.
- [ ] **AC-3 (Chapter 3, 4 & 5):** Verification templates generate Chapter 3 (Git & telemetry gates), Chapter 4 (verbatim original prompt embedding), and Chapter 5 (evidence scorecard contract table).
- [ ] **AC-4 (Rerun Preservation):** `buildRerunPrompt` outputs the sequential iteration header (`Round {i} of {total}`) and preserves the verbatim original prompt text.
- [ ] **AC-5 (DetailHeader Integration):** `DetailHeader.tsx` passes `activePromptText` to `dispatchRerunMessages`.
- [ ] **AC-6 (Prompt Actions Integration):** `usePromptTreePromptActions.ts` applies `buildRichVerificationPrompt` in `handleResendPrompt` and `handleEnqueuePrompt`.
- [ ] **AC-7 (Backward Compatibility):** Legacy suffix strings (e.g. `"Is it done?"`) map cleanly to new template types via `parseVerificationTypeFromSuffix`.
- [ ] **AC-8 (No Silent Errors):** All rerun failures and generation errors route through `useErrorStore` and `persistClientCrash`.
- [ ] **AC-9 (Zero Compile/Lint Errors):** `npm run build` passes with zero TypeScript errors.
