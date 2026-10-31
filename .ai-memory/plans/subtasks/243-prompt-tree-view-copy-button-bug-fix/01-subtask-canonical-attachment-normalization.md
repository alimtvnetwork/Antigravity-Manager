# Subtask 01: Canonical Attachment Normalization & Tests

**Task Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Subtask**: `01-subtask-canonical-attachment-normalization`  
**Sequence**: `243-01`  
**Spec Reference**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/02-component-and-attachment-spec.md`  

## Objectives
1. Update `buildAttachmentsBlock()` in `src/utils/promptAttachment.ts` to output:
   `1. [Attachment 01 | Description | filename.png](file;path/to/file.png)`
2. Ensure `extractAttachmentsFromPrompt()` and `stripAttachmentsFromPrompt()` support all bracket variants.
3. Update `src-tauri/src/modules/repo_db/detection.rs` to detect both `[label](file;...)` and `@[label](file;...)`.
4. Update unit tests in `src/utils/__tests__/promptAttachment.test.ts` and `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`.
