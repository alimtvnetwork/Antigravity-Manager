# Plan: Prompt Tree View Copy Button Bug Fix & Attachment Normalization

**Task**: `Prompt Tree View Copy Button Bug Fix`  
**Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Sequence**: `243`  
**Status**: COMPLETED  
**Branch**: `main`  
**Attribution**: strictly `@aukgit`  
**Spec Path**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/01-architecture-spec.md`  

---

## Subtask Breakdown

1. **Subtask 01: Canonical Attachment Normalization & Tests** [COMPLETED]
   - File: `.ai-memory/plans/subtasks/243-prompt-tree-view-copy-button-bug-fix/01-subtask-canonical-attachment-normalization.md`
   - Target: `src/utils/promptAttachment.ts`, `src/utils/__tests__/promptAttachment.test.ts`, `src-tauri/src/modules/repo_db/detection.rs`.
   - Update `buildAttachmentsBlock` to generate `1. [Attachment 01 | Description | filename](file;<path>)`.
   - Ensure parsers extract paths from both `[...](file;...)` and legacy patterns.

2. **Subtask 02: Prompt Tree View Copy Button Fix** [COMPLETED]
   - File: `.ai-memory/plans/subtasks/243-prompt-tree-view-copy-button-bug-fix/02-subtask-copy-button-attachments.md`
   - Target: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `usePromptTreePromptActions.ts`.
   - Ensure `handleCopyAll()` resolves conversation and turn media paths and applies `formatPromptWithAttachments`.

3. **Subtask 03: Dropdown Popover Click-Outside Dismissal** [COMPLETED]
   - File: `.ai-memory/plans/subtasks/243-prompt-tree-view-copy-button-bug-fix/03-subtask-dropdown-click-outside.md`
   - Target: `src/components/instances/PromptTreeView/DetailHeader.tsx`.
   - Integrate `useClickOutside` on Copy Menu, Dispatch Popover, and Next Breakdown Dropdown.

4. **Subtask 04: Full Verification & Release Pre-Flight** [COMPLETED]
   - File: `.ai-memory/plans/subtasks/243-prompt-tree-view-copy-button-bug-fix/04-subtask-verification-and-release.md`
   - Run unit tests, verify `npm test -- promptAttachment`, `cargo fmt -- --check`, `npm run build`, and release ceremony.
