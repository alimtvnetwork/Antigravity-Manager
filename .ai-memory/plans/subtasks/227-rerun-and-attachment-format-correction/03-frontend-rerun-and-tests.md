---
plan: 227-rerun-and-attachment-format-correction
subtask: "03"
title: Frontend Multi-Prompt Resend Enqueue Fix, Rerun Action Tests & E2E Verification Report
domain: frontend-react-typescript
assigned_worker: Exec-03
depends_on:
  - 02-spec/21-app/227-rerun-and-attachment-format-correction/01-architecture-spec.md
  - 02-spec/21-app/227-rerun-and-attachment-format-correction/02-component-spec.md
  - 02-spec/21-app/227-rerun-and-attachment-format-correction/03-migration-and-verification-spec.md
citations:
  architecture_spec: ../../../../02-spec/21-app/227-rerun-and-attachment-format-correction/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/227-rerun-and-attachment-format-correction/02-component-spec.md
  migration_spec: ../../../../02-spec/21-app/227-rerun-and-attachment-format-correction/03-migration-and-verification-spec.md
target_files:
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
  - src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts
  - src/tests/rerunActions.test.ts
  - 02-spec/21-app/227-rerun-and-attachment-format-correction/04-e2e-report.md
status: pending
---

# Subtask 03: Frontend Multi-Prompt Resend Enqueue Fix, Rerun Action Tests & E2E Verification Report

## 1. Context & Motivation

In task 227 (`rerun-and-attachment-format-correction`), the user established a non-negotiable rule:
> "There is a small serious problem with the rerun. The rerun needs to enqueue. It does not run send directly. That's the first thing. Second is that when you put the attachment, you did not put the actual format that I have given you. In the third bracket, you need to put the file, then semicolon, and then the file path... Confirm that all tasks are end-to-end tested before release."

An audit of the frontend code revealed:
1. `dispatchRerunMessages` in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts:1545` was previously fixed to call `enqueuePrompt` across all rounds (1..N).
2. However, `dispatchMultiPromptResend` in `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts:1818` (triggered by the context menu "Resend Last N" actions) still invoked `sendPromptNow`. Because `sendPromptNow` immediately cancels running IDE executions and executes out-of-order CDP injections, resending multiple historical turns aborted ongoing work and bypassed the SQLite FIFO queue (`active_prompts`).
3. Unit test coverage in `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` covers `dispatchRerunMessages`, but does not test `dispatchMultiPromptResend`'s queue compliance or verify that 0 calls are made to `sendPromptNow`.
4. Production release gates require authoring a comprehensive end-to-end verification report at `02-spec/21-app/227-rerun-and-attachment-format-correction/04-e2e-report.md` capturing test execution logs, dual-format parser verifications, FIFO queue invariants, and preflight checks.

---

## 2. Target Files & Disjoint Ownership

| Target File | Owned Symbols / Areas | Planned Modifications |
|---|---|---|
| `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | `dispatchMultiPromptResend` (lines ~1799–1854) | Replace `sendPromptNow` with `enqueuePrompt` to guarantee FIFO queueing. Retain error tracking and delay pacing. |
| `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` | Unit test cases for resend & rerun actions | Add tests verifying `dispatchMultiPromptResend` enqueues all items via `enqueue_prompt`, calls `send_prompt_now` 0 times, forwards media, and isolates errors. |
| `src/tests/rerunActions.test.ts` | Test entrypoint | Verify re-export execution matches Node test runner conventions. |
| `02-spec/21-app/227-rerun-and-attachment-format-correction/04-e2e-report.md` | Formal E2E verification report | Author full verification report capturing test outputs, acceptance standards, and quality gate results. |

---

## 3. Step-by-Step Implementation Plan

### Step 1: Audit and Verify Frontend Dispatch Touchpoints
- Inspect `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`.
- Confirm that `dispatchRerunMessages` (lines 1545–1670) strictly calls `enqueuePrompt`.
- Confirm that `handleVerifyLastPromptsAndEnqueue` (lines 1924–1975) strictly calls `enqueuePrompt`.
- Identify line 1818 in `dispatchMultiPromptResend`:
  ```typescript
  await sendPromptNow(
      options.instanceId,
      options.repoPath,
      rewrittenPrompt,
      options.conversationId,
      promptMedia.length > 0 ? promptMedia : undefined
  );
  ```

### Step 2: Surgically Fix `dispatchMultiPromptResend`
In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
- Replace the `sendPromptNow` call with `enqueuePrompt`:
  ```typescript
  // CRITICAL INVARIANT: Multi-prompt resends must strictly enqueue into the FIFO queue.
  await enqueuePrompt(
      options.instanceId,
      options.repoPath,
      rewrittenPrompt,
      options.conversationId,
      undefined,
      promptMedia.length > 0 ? promptMedia : undefined
  );
  ```
- Ensure error tracking via `persistClientCrash` and `useErrorStore.getState().trackWarning` remains intact.
- Preserve inter-prompt delay loop (`options.delayMs ?? 500`).

### Step 3: Expand Unit Tests in `rerunActions.test.ts`
In `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`:
- Import `dispatchMultiPromptResend` from `../usePromptTreePromptActions`.
- Add test: `verifies dispatchMultiPromptResend enqueues all selected prompts via enqueue_prompt and 0 via send_prompt_now`:
  - Reset `mockInvokeCalls = []`.
  - Invoke `dispatchMultiPromptResend` with 3 prompts and sample media.
  - Assert `result.sent === 3` and `result.failed === 0`.
  - Assert `mockInvokeCalls.filter(c => c.cmd === 'enqueue_prompt').length === 3`.
  - Assert `mockInvokeCalls.filter(c => c.cmd === 'send_prompt_now').length === 0`.
  - Check that parameters (`promptContent`, `conversationId`, `mediaPaths`) are preserved in each call.
- Add test: `verifies dispatchMultiPromptResend isolates individual failures without aborting remaining batch items`:
  - Set `mockInvokeFailCmd = 'enqueue_prompt'` and `mockInvokeFailAfterCount = 1`.
  - Verify `result.sent === 1` and `result.failed >= 1`.
  - Verify error was tracked without throwing an unhandled exception.

### Step 4: Run Targeted Verification Suites
Execute the fast Node frontend test runner:
```powershell
node scripts/run-frontend-tests.mjs rerunActions
node scripts/run-frontend-tests.mjs promptAttachment
```
Confirm 100% of tests pass with 0 failures.

### Step 5: Author the Formal End-to-End Verification Report
Create `02-spec/21-app/227-rerun-and-attachment-format-correction/04-e2e-report.md`:
- Summarize task objectives, problem context, and architectural solutions.
- Include unit test execution logs from `promptAttachment.test.ts` and `rerunActions.test.ts`.
- Document verified FIFO database queue ingestion.
- Document dual-format compatibility (`@[file;<path>]` and legacy `@[<path>]`).
- Confirm non-crashing UI behavior and error store integration.
- Document Preflight Quality Gate status (fmt, clippy, build, relative paths).

### Step 6: Execute Preflight Quality Gates
- Rust formatting & clippy check:
  ```powershell
  cd src-tauri
  cargo fmt -- --check
  cargo clippy --lib --bin agm -- -D warnings
  cd ..
  ```
- Frontend build check:
  ```powershell
  npm run build
  ```
- Relative path verification:
  Ensure zero absolute paths exist in the touched files and specs.

---

## 4. Acceptance Criteria & Verification Gates

1. **Strict FIFO Queue Route**: Every resend / rerun operation triggered from context menus or detail headers routes through `enqueue_prompt`. Zero calls are made to `send_prompt_now` in batch operations.
2. **Third Bracket Attachment Syntax**: All generated attachments use `@[file;<path>]`.
3. **Legacy Attachment Parsing**: Legacy `@[<path>]` tokens in historical transcripts or databases are parsed identically without errors.
4. **Unit Test Pass Rate**: 100% of unit tests in `rerunActions.test.ts` and `promptAttachment.test.ts` pass cleanly.
5. **Report Complete**: `04-e2e-report.md` is authored, comprehensive, and contains zero absolute paths.
