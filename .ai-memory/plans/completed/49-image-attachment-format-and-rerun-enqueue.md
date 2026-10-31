# Execution Plan: 49-image-attachment-format-and-rerun-enqueue

## Strategic Intent & Systems Reasoning
Normalize prompt image attachment formatting across Antigravity Manager so all image attachment representations are converted from legacy Markdown image syntax (`![Attached Image N](<uri>)`) into canonical bracketed file references:
```text
Attachments:
1. @[C:\path\to\image.png]
2. @[C:\path\to\image2.png]
```
This applies uniformly across all prompt operations:
- Copy: Copy AI Payload, Copy Prompt, Copy Context, Copy Rich Images
- Send: Send Prompt Now
- Enqueue: Enqueue to SQLite prompt queue
- Rerun: Rerun prompt

Additionally, enforce the **Rerun Enqueue Invariant**:
- The Rerun action must **always enqueue** into the SQLite prompt queue (`enqueuePrompt` / `enqueue_prompt_to_repo_db`), and **never send directly** (`sendPromptNow`).

## User Request (Verbatim)
```text
When copying prompt or send or enque

the image format needs to be corrected

FROM

![Attached Image 1](file:///C:/Users/Administrator/.gemini/antigravity/brain/747effe5-75ae-48de-b5e8-574467ff27bb/.user_uploaded/media_1792894009084_45b4893f.png)

TO

Attachments:
1. @[C:\Users\Administrator\.gemini\antigravity\brain\747effe5-75ae-48de-b5e8-574467ff27bb\.user_uploaded\media_1792894009084_45b4893f.png]

Output like this for all the send, enque, copy, rerrun everything 

also the rerun should always enque not send directly fix it please 

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing work for improved work or less friction, faster execution for AI and coding without errors and following Coding Guidelines.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Track in-flight file touches and prevent write collisions using GitMap RC Heatmap (`gitmap rc heatmap status` / `gitmap rc heatmap tree` / `gitmap rc heatmap live` / `gitmap rc heatmap sync`).
- Only add relative paths, never add absolute paths during work; this must be respected on release page and in release notes as well.
```

## Architecture Boundaries & File Ownership
- Spec Folder: `02-spec/21-app/49-image-attachment-format-and-rerun-enqueue/`
  - `01-architecture-spec.md` (Spec Author 01)
  - `02-component-spec.md` (Spec Author 02)
  - `03-migration-and-verification-spec.md` (Spec Author 03)
- Plan Folder: `.ai-memory/plans/`
  - `.ai-memory/plans/pending/49-image-attachment-format-and-rerun-enqueue.md` (Lead Orchestrator)
  - `.ai-memory/plans/subtasks/49-image-attachment-format-and-rerun-enqueue/01-subtask-attachment-formatter.md` (Spec Author 01)
  - `.ai-memory/plans/subtasks/49-image-attachment-format-and-rerun-enqueue/02-subtask-prompt-actions-and-rerun.md` (Spec Author 02)
  - `.ai-memory/plans/subtasks/49-image-attachment-format-and-rerun-enqueue/03-subtask-backend-and-tests.md` (Spec Author 03)
- Source Implementation Files:
  - `src/utils/promptAttachment.ts` (Worker 01): Canonical pure helper for attachment formatting and path normalization
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` (Worker 02): Prompt actions (send, enqueue, rerun dispatch)
  - `src/components/instances/PromptTreeView/DetailHeader.tsx` (Worker 02): Detail header actions, rerun enqueue handler
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx` (Worker 02): Instruction view direct rerun & copy
  - `src/components/instances/PromptTreeView/clipboard.ts` (Worker 01): Clipboard formatting utilities
  - `src-tauri/src/modules/repo_db/detection.rs` (Worker 03): Rust backend image extraction supporting `@[...]` tokens
  - `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` (Worker 03): Updated rerun test suite asserting 100% enqueue calls

## Verification Evidence Contract
1. Frontend test suite passes: `rerunActions.test.ts` passes with 0 `send_prompt_now` calls.
2. Formatter tests pass: Legacy `![Attached Image N](<uri>)` and `file:///` URIs correctly normalize to `Attachments:\n1. @[C:\path\to\image.png]`.
3. Pre-flight checks pass: `npm run build` and `cd src-tauri && cargo fmt -- --check && cargo check --lib`.
4. GitMap atomic conventional commit: `gitmap cpb "(PROMPT): normalize image attachments to bracketed list and enforce rerun enqueue"`.
