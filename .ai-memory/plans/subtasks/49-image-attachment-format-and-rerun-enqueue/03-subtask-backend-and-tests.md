# Subtask 03: Backend Token Extraction, Clipboard Integration, and Rerun Test Verification

**Plan ID:** `.ai-memory/plans/subtasks/49-image-attachment-format-and-rerun-enqueue/03-subtask-backend-and-tests.md`  
**Parent Task:** `49-image-attachment-format-and-rerun-enqueue`  
**Worker Role:** Worker 03 (Backend, Clipboard & Test Specialist)  
**Status:** Ready for Execution  

---

## 1. Context & Objectives

This subtask implements the backend token extraction enhancements, rich media protection guards, clipboard utilities formatting, and test suite refactoring required to complete task `49-image-attachment-format-and-rerun-enqueue`.

### Core Targets:
- **Rust Backend Token Extraction**: `src-tauri/src/modules/repo_db/detection.rs`
- **Rust Backend Rich Media Guard**: `src-tauri/src/modules/repo_db/agy.rs`
- **Frontend Clipboard Utilities**: `src/components/instances/PromptTreeView/clipboard.ts`
- **Frontend Rerun Test Suite**: `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`
- **Rust Backend Unit Tests**: `src-tauri/src/modules/repo_db/detection.rs` (or associated test modules)

---

## 2. Technical Invariants & Guardrails

1. **Rerun Enqueue Invariant**:
   - `dispatchRerunMessages` must strictly dispatch 100% of rerun rounds via `enqueue_prompt`.
   - Direct execution via `send_prompt_now` is strictly forbidden during rerun operations.
   - Unit tests must assert `mockInvokeCalls.filter(c => c.cmd === 'send_prompt_now').length === 0`.
2. **Bracketed Token Extraction Invariant**:
   - Rust function `extract_image_uris_or_paths` must trim `@`, `[`, and `]` delimiters alongside quotes and angles.
   - Tokens such as `@[path/to/image.png]` or numbered items `1. @[path/to/image.png]` must resolve to `path/to/image.png` in `found_paths`.
3. **Rich Media Protection Invariant**:
   - In `agy.rs`, `has_rich_media` must evaluate to `true` whenever prompt content contains `Attachments:` or `@[`, preventing the backend from overwriting the rich clipboard.
4. **Strict Relative Git Paths**:
   - Never reference absolute filesystem paths in code comments, test fixtures, or docs.
5. **Coding Guidelines**:
   - Positive boolean naming (`has_rich_media`, `is_enqueued`).
   - Zero `any` in TypeScript implementations.
   - No silent errors: propagate via `AppError` or route to `useErrorStore.getState().trackWarning`.

---

## 3. Step-by-Step Implementation Tasks

### Step 1: Rust Token Extraction Enhancement (`src-tauri/src/modules/repo_db/detection.rs`)
In `extract_image_uris_or_paths`:
- Update `clean = token.trim_matches(...)` to include `c == '@'`:
  ```rust
  let clean = token.trim_matches(|c| {
      c == '"'
          || c == '\''
          || c == '('
          || c == ')'
          || c == '['
          || c == ']'
          || c == '<'
          || c == '>'
          || c == '@'
  });
  ```
- Ensure file scheme stripping (`file:///` and `file://`) continues to normalize disk paths correctly.

### Step 2: Rust Rich Media Protection Guard (`src-tauri/src/modules/repo_db/agy.rs`)
In `run_active_prompt_with_details` (line ~826) and `requeue_prompt_with_details` (line ~1028):
- Extend `has_rich_media` expression:
  ```rust
  let has_rich_media = initial_img.is_some()
      || !img_paths.is_empty()
      || prompt_content.contains("![")
      || prompt_content.contains("<img")
      || prompt_content.contains("data:image/")
      || prompt_content.contains("Attached Image")
      || prompt_content.contains("Attachments:")
      || prompt_content.contains("@[");
  ```

### Step 3: Clipboard Utilities Formatting (`src/components/instances/PromptTreeView/clipboard.ts`)
- Import `formatPromptWithAttachments` from `src/utils/promptAttachment.ts`.
- In `copyPromptWithRichImages`:
  - Replace legacy `![Turn Image ...]` string concatenation with `formatPromptWithAttachments(markdownText, mediaPaths)`.
  - Use the normalized text for plain text blob creation and rich HTML generation.
- In `copyPromptAiPayload` (or where prompt text is prepared for AI copy):
  - Pass the prompt text through `formatPromptWithAttachments`.
- Ensure `copyFullPrompt` (if exported) formats prompt text with attachments.

### Step 4: Rerun Unit Test Suite Refactoring (`src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`)
- **Multi-Round Rerun Test (lines 280-321)**:
  - Update description: `"verifies dispatchRerunMessages enqueues all rounds (1..N) via enqueue_prompt and 0 via send_prompt_now"`.
  - Assert `mockInvokeCalls.filter(c => c.cmd === 'send_prompt_now').length === 0`.
  - Assert `mockInvokeCalls.filter(c => c.cmd === 'enqueue_prompt').length === 3`.
  - Verify all 3 rounds dispatch `enqueue_prompt` with the correct `instanceId`, `conversationId`, and distinct messages.
- **Graceful Error Handling Test (lines 323-343)**:
  - Update mock invoker failure logic to test graceful error handling when `enqueue_prompt` encounters an error during multi-round dispatch.
  - Assert `result.dispatched` and `result.failed` reflect the simulated error without unhandled promise rejections.
- **Anti-Repeat History Consecutive Runs Test (lines 403-458)**:
  - Assert all 5 prompt calls across Run 1 (3 calls) and Run 2 (2 calls) use `enqueue_prompt`.
  - Assert 0 calls use `send_prompt_now`.

### Step 5: Rust Unit Test Additions (`src-tauri/src/modules/repo_db/detection.rs`)
- Add unit tests verifying:
  - Input: `"Ref task\nAttachments:\n1. @[assets/shot.png]"` $\rightarrow$ `found_paths` contains `"assets/shot.png"`.
  - Input: `"1. @[file:///path/to/test.webp]"` $\rightarrow$ `found_paths` contains `"path/to/test.webp"`.
  - Input: multiple bracketed images $\rightarrow$ deduplicated `found_paths`.

---

## 4. Verification & Evidence Plan

| Order | Command | Expected Outcome |
|---|---|---|
| 1 | `npx tsx src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` | 14+ tests passing, 0 failures, 0 `send_prompt_now` calls |
| 2 | `cd src-tauri && cargo test --lib extract_image_uris_or_paths` | All Rust token extraction tests pass |
| 3 | `cd src-tauri && cargo fmt -- --check` | Clean exit (code 0) |
| 4 | `cd src-tauri && cargo check --lib` | Clean exit (code 0), zero compile errors |
| 5 | `npm run build` | Clean production frontend bundle build |
