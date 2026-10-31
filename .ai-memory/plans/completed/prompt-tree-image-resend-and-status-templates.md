# Completed Plan: Prompt Tree Image Resend & Status Verification Templates

## Task Metadata
- **Slug**: `prompt-tree-image-resend-and-status-templates`
- **Specification**: [`02-spec/21-app/prompt-tree-image-resend-and-status-templates/01-architecture-spec.md`](file:///d:/work/Antigravity-Manager/02-spec/21-app/prompt-tree-image-resend-and-status-templates/01-architecture-spec.md) and [`02-spec/21-app/prompt-tree-image-resend-and-status-templates/02-component-spec.md`](file:///d:/work/Antigravity-Manager/02-spec/21-app/prompt-tree-image-resend-and-status-templates/02-component-spec.md)
- **Status**: `COMPLETED`
- **Execution Strategy**: 2 Autonomous Subagents (`invoke_subagent`), Disjoint File Ownership, Zero Git Lock Collisions

---

## 1. User Request (Verbatim)
> If we use in the prompt tree, is it done or rerun? Or send it using the is it done? Is it released? That has a little bit of template you can find in the coding guideline for is it released or things like that. That should have a template. It has more than few, and that has a detailed instruction, like how to check that. You should include that, and then it should actually contain the full prompt. Currently sending the prompt from the work tree of the prompt work tree from the instances, it does not copy the image. It does not send the image when we resend them or enqueue them. Try to find the root cause and fix it. Image sending is high priority, and you can copy the image in many different ways. One is that you convert the image to a URL. Another could be you embed the image to a file, or you could use any one of the free services to upload the image with the URL. Another one you could use is the Lightshot. I do have the Lightshot in the machine. You could open those images and use the Lightshot to take the screenshot and create a URL. Another could be converting the image to Base64, but that would create the prompts a bit bigger. Try to find the best way to deal with this. It is not fixed, so your job would be finding the root cause and fix the prompt tree section, sending the prompt, resending it with different versions. Rerunning needs to be fixed. Rerun needs to send with different version. Is it released? Is it done? Are you confident? Things like that. You'd have a big chapter on top of it and then put the same prompt. Currently, it's just sending the is it done? That's it. It's an issue you need to fix.

---

## 2. Root Cause Analysis Summary
1. **Frontend IPC Payload Omission**: The frontend only transmitted raw text strings to `send_prompt_now` / `enqueue_prompt`, discarding image metadata and attachments.
2. **CLI 24KB Truncation**: `src-tauri/src/modules/repo_db/agy.rs:212` applied `truncate_utf8(&clean_prompt, 24000)`, corrupting Base64 images larger than 24 KB.
3. **CDP Input.insertText Typing Limits**: Attempting to type multi-megabyte Base64 strings into the IDE React DOM crashed or froze the IDE composer.
4. **Clipboard Degradation**: Fallbacks called plain-text `copyToClipboard` instead of dual-MIME `copyPromptWithRichImages`.
5. **Asset Extractor Regex Bug**: In `usePromptTreePromptActions.ts:188`, `extractAssets` pushed `match[1]` (the alt text) instead of `match[2]` (the actual image data URI / URL).
6. **Weak Status Verification & Rerun Nudge**: Clicking "Rerun" or selecting suffixes like "Is it done?" or "Is it released?" dispatched bare 3-word strings without task context, leading to model hallucination.

---

## 3. Implemented Architecture & Solutions

### 3.1 Dual-MIME Rich Image Transmission & Extraction Ladder
- **`src/components/instances/PromptTreeView/types.ts`**:
  - Defined `ExtractedPromptImage` and `PromptRewriteResult` interfaces.
- **`src/components/instances/PromptTreeView/clipboard.ts`**:
  - Implemented `extractPromptImages` to extract Base64 data URIs, calculate SHA-256 fingerprints, and rewrite massive inline data URIs to compact relative attachment markdown links (`![alt](./.antigravity/attachments/<hash>.<ext>)`), shrinking payloads from 2MB to under 80 bytes.
  - Fixed `extractAssets` regex to capture group 2 (`match[2]`), resolving the broken image downloader bug.
  - Enhanced `copyPromptWithRichImages` with rich HTML `<img>` rendering and dual-MIME (`text/html` + `text/plain`) `ClipboardItem` handling.
- **`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`**:
  - Deduplicated `copyPromptWithRichImages` by importing from `./clipboard`.
  - In `handleResendPrompt` and `handleEnqueuePrompt`:
    - Materialized/rewrote images before backend IPC dispatch.
    - Primed system clipboard with rich dual-MIME HTML images so users can paste directly into IDE.
    - Wrapped prompts with `buildRichVerificationPrompt` when confirmation suffixes are chosen.

### 3.2 5-Chapter Verification & Rerun Prompt Engine
- **`src/components/instances/PromptTreeView/verificationTemplates.ts`**:
  - **Chapter 1: 🚨 High Priority Verification & Completion Audit Mandate**: Strict auditor instructions, anti-hallucination mandate, no-silent-errors rule.
  - **Chapter 2: 📊 Grounded Verification Matrix & Actionable Checklist**:
    - `is-it-done`: Read & cited exact files/lines, 1-sentence root cause diagnosis, minimum surgical change verification, verifiable execution proof, plan/memory synchronization.
    - `is-it-released`: Channel branch validation (`main` vs `beta`), 14-location SemVer sync via `npm run bump`, strict `@aukgit` attribution invariant (`(Thanks to @aukgit)` - zero other handles), README changelog synchronization (`README.md` & `README_EN.md`), pre-flight quality gates (`cargo fmt`, `cargo clippy`, `npm run build`), tag naming (`vX.Y.Z`), distribution installers (`install.ps1`, `install.sh`), thinking store invalidation toggle check.
    - `are-you-confident`: Function decomposition (<= 15 lines), early returns, clean constructor returns, implicit positive booleans only, zero silent errors, relative paths only.
    - `edge-cases-and-tests`: Boundary value checks, error recovery ladders, test coverage.
  - **Chapter 3: 🔍 Environment, Git & Release Telemetry Gates**: Enforces `git status --porcelain`, branch check (`git branch --show-current`), and CI/CD telemetry checks (`gitmap pe -t`).
  - **Chapter 4: 📋 Verbatim Original Prompt Embedding**: Faithfully embeds un-truncated user prompt, attachments, and relative markdown links inside `<<<ORIGINAL_TASK_PROMPT_START>>>`.
  - **Chapter 5: 📝 Concluding Evidence Scorecard Contract**: Enforces structured Markdown table with Gate, Status (PASS/FAIL), File/Line Reference, Verifiable Proof/Output, and a binary `ALL_CRITERIA_MET` verdict.
  - **Rerun Nudge Engine**: Added `buildRerunPrompt(originalPrompt, round, total)` tracking iterations (`[Rerun Execution Nudge — Round {round} of {total}]`) while fully embedding the original task context in Chapter 4.
- **`src/components/instances/PromptTreeView/DetailHeader.tsx`**:
  - Connected `VERIFICATION_OPTIONS` to the confirmation suffix select dropdown.
  - Passed `activePromptText` to `dispatchRerunMessages` to guarantee full prompt context preservation across rerun loops.

---

## 4. Verification Evidence
- `npm run build`: Passed cleanly (`tsc && vite build` in 17.87s with 0 errors).
- `cargo fmt -- --check`: Passed cleanly with zero diffs.
- `cargo clippy --all-targets --all-features`: Passed with 0 errors.
- SQLite Subtask DB (`agent-task.db`): Both `Task-01` and `Task-02` marked `COMPLETED` with logged evidence.
