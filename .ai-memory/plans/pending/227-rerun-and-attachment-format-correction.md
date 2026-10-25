# Master Execution Plan: Rerun and Attachment Format Correction

- **Slug**: `rerun-and-attachment-format-correction`
- **Sequence**: `227`
- **Spec Directory**: `02-spec/21-app/227-rerun-and-attachment-format-correction/`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/227-rerun-and-attachment-format-correction/`
- **Status**: `COMPLETED`
- **Protocol**: `execute-parent-task-with-n-steps-v6` (`A = 3`, `H = 2`, Capacity = 6)
- **Completed At**: 2026-10-25
- **Verified Gates**:
  - Attachment syntax updated to `@[file;<path>]` and regex parsers normalized across `promptAttachment.ts`, `clipboard.ts`, `PromptInstructionView.tsx`, and `detection.rs`.
  - Backend rerun routes in `prompt_history.rs` and `send_flow.rs` converted from immediate injection to `repo_db::enqueue_prompt_for_instance`.
  - Frontend multi-prompt resend in `usePromptTreePromptActions.ts` converted to `enqueuePrompt`.
  - 100% green unit tests: `promptAttachment.test.ts` (12/12 pass), `rerunActions.test.ts` (21/21 pass), `detection.rs` tests (7/7 pass), `prompt_history` & `send_flow` tests (23/23 pass).
  - Pre-flight checks passed: `cargo fmt`, `cargo clippy`, `npm run build`.

---

## 1. User Request (Verbatim)

```text
# Rerun and Attachment Format Correction: high priority instruction, non-negotiable task

There is a small serious problem with the rerun. The rerun needs to enqueue. It does not run send directly. That's the first thing. Second is that when you put the attachment, you did not put the actual format that I have given you. In the third bracket, you need to put the file, then semicolon, and then the file path. You didn't do that. You just used third bracket and file path, which is very wrong. You need to fix it for the image path.orrection
## slug: rerun-and-attachment-format-correction

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work t first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/XX-rerun-and-attachment-format-correction/ and enqueue plan task in .ai-memory/plans/pending/XX-rerun-and-attachment-format-correction.md (subtasks in .ai-memory/plans/subtasks/XX-rerun-and-attachment-format-correction/) first, (xx should be figured out by the folder structure and running AI tools, xx is sequence 01, 02)
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Fix the rerun process to ensure it enqueues and does not run send directly.
6. Correct the attachment format by ensuring the third bracket contains the file, a semicolon, and then the file path.
7. Add a section to confirm if the end-to-end test is completed and released with build test, and ensure it is production tested.
8. Confirm that all tasks are end-to-end tested before release.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]
```

---

## 2. Synthesized Architectural Findings & Decisions

### 2.1 Attachment Syntax Correction (`@[file;<file_path>]`)
1. **Third Bracket Canonical Form**:
   South Asian English convention designates square brackets `[...]` as "third brackets". The requirement mandates:
   `@[file;<file_path>]` (e.g. `1. @[file;assets/screenshots/image.png]`).
2. **Formatter Touchpoints**:
   - `src/utils/promptAttachment.ts:262`: Update `buildAttachmentsBlock` to map paths to `${idx + 1}. @[file;${p}]`.
   - `src/components/instances/PromptTreeView/clipboard.ts:1028`: Update `copyPromptWithRichImages` replacement to preserve `file;` prefix syntax.
3. **Regex Extraction & Backward Compatibility**:
   - `src/utils/promptAttachment.ts`: Update `collectPromptAttachmentRawTargets`, `extractAttachmentsFromPrompt`, `extractRawTarget`, and `stripProtocolPrefix` to match `/@\[(?:file;\s*)?(.*?)\]/gi` and strip `file;` prefix so that clean local paths are passed to filesystem loaders and WebView renderers.
   - `src/components/instances/PromptTreeView/clipboard.ts`: Update `extractAssets` and `extractLocalPathFromUri` to strip `file;` prefix.
   - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`: Update `extractImagesFromPrompt` and `normalizeMediaUri` to defensively strip `file;`.
   - `src-tauri/src/modules/repo_db/detection.rs`: Update `extract_image_uris_or_paths` to strip `file;` prefix from bracketed tokens.

### 2.2 Rerun Enqueue Invariant (`status: 'queued'`)
1. **Backend Telegram Inbound Reruns**:
   - `src-tauri/src/modules/telegram_inbound/prompt_history.rs:137`: Replace `execute_prompt_injection(&args)` in `rerun_prompt_by_id` with `crate::modules::repo_db::enqueue_prompt_for_instance`.
   - `src-tauri/src/modules/telegram_inbound/send_flow.rs:1177`: In `dispatch_turn_rerun_execution`, replace `execute_prompt_injection(&args)` and remove `submit_prompt_via_cdp_for_instance(..., true)`. Enqueue via `enqueue_prompt_for_instance`.
   - Trigger `crate::modules::scheduler::notify_prompt_queue()` and `crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(instance))`.
2. **Frontend Rerun Actions**:
   - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts:1545`: `dispatchRerunMessages` is already compliant (calls `enqueuePrompt`).
   - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts:1818`: `dispatchMultiPromptResend` called `sendPromptNow`. Fix to call `enqueuePrompt` to preserve FIFO queue ordering.

---

## 3. Work Breakdown Structure & Disjoint Boundaries

| Task ID | Phase | Component | Assigned Worker | Disjoint Owned Files |
|---|---|---|---|---|
| **Spec-01** | Phase 2 | Architecture & Queue Spec | Spec Author 01 | `02-spec/21-app/227-rerun-and-attachment-format-correction/01-architecture-spec.md`, `.ai-memory/plans/subtasks/227-rerun-and-attachment-format-correction/01-attachment-format.md` |
| **Spec-02** | Phase 2 | Component & Logic Spec | Spec Author 02 | `02-spec/21-app/227-rerun-and-attachment-format-correction/02-component-spec.md`, `.ai-memory/plans/subtasks/227-rerun-and-attachment-format-correction/02-backend-rerun-enqueue.md` |
| **Spec-03** | Phase 2 | Migration & Verification Spec | Spec Author 03 | `02-spec/21-app/227-rerun-and-attachment-format-correction/03-migration-and-verification-spec.md`, `.ai-memory/plans/subtasks/227-rerun-and-attachment-format-correction/03-frontend-rerun-and-tests.md` |
| **Exec-01** | Phase 3 | Canonical Attachment Formatting & Parsers | Worker 01 | `src/utils/promptAttachment.ts`, `src/utils/__tests__/promptAttachment.test.ts`, `src/components/instances/PromptTreeView/clipboard.ts`, `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src-tauri/src/modules/repo_db/detection.rs` |
| **Exec-02** | Phase 3 | Backend Telegram Rerun Enqueue | Worker 02 | `src-tauri/src/modules/telegram_inbound/prompt_history.rs`, `src-tauri/src/modules/telegram_inbound/send_flow.rs` |
| **Exec-03** | Phase 3 | Frontend Resend Enqueue & E2E Report | Worker 03 | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`, `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`, `src/tests/rerunActions.test.ts`, `02-spec/21-app/227-rerun-and-attachment-format-correction/04-e2e-report.md` |

---

## 4. Verification Evidence & Quality Gates
- **Rust Quality Gate**: `cd src-tauri && cargo fmt -- --check` & `cargo check --lib --bin agm` & `cargo clippy --lib --bin agm`.
- **Frontend Quality Gate**: `npm test -- promptAttachment` & `npm test -- rerunActions` & `npm run build`.
- **Path Linter**: `python .agents/scripts/07-relative-path-fixer.py 02-spec/21-app/227-rerun-and-attachment-format-correction`.
- **Final Atomic Commit**: `gitmap cpb "(RERUN): enforce fifo enqueue for all reruns and canonical file semicolon attachment format"`.
