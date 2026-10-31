# Migration & Verification Specification: Prompt Tree View Copy Button Bug Fix

**Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Sequence**: `243`  
**Spec Path**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/03-migration-and-verification-spec.md`  
**Status**: APPROVED_FOR_EXECUTION  
**Target Branch**: `main`  
**Attribution**: strictly `@aukgit`  

---

## 1. Acceptance Criteria Checklist

- [ ] **AC-01 (Primary Copy with Attachments)**: Clicking the primary "Copy" button in `DetailHeader.tsx` generates clipboard text containing `Attachments:\n1. [Attachment 01 | Description | filename](file;<path>)` whenever screenshots/media are present.
- [ ] **AC-02 (Canonical 3-Tier Format)**: Formatted attachment lines match pattern `^\d+\.\s*\[Attachment\s+\d{2}\s*\|\s*[^|]+\|\s*[^\]]+\]\(file;[^\)]+\)$`.
- [ ] **AC-03 (Parity Across Dispatch Modes)**: Send, Enqueue, Rerun, and Copy all emit identical canonical attachment blocks.
- [ ] **AC-04 (Copy Menu Click-Outside Dismissal)**: Clicking anywhere outside the Copy options menu immediately dismisses Flyout 1.
- [ ] **AC-05 (Dispatch Popover Click-Outside Dismissal)**: Clicking anywhere outside the Unified Dispatch popover immediately dismisses Flyout 2.
- [ ] **AC-06 (Next Breakdown Click-Outside Dismissal)**: Clicking anywhere outside the Next Breakdown dropdown immediately dismisses Flyout 3.
- [ ] **AC-07 (Pre-Flight Checks Passed)**: `cargo fmt -- --check` and `npm run build` succeed with exit code 0.
- [ ] **AC-08 (Unit Test Verification)**: `npm test -- promptAttachment` passes 100%.

---

## 2. Backward Compatibility Matrix

| Input Format | Extractor Target | Normalized Output on Format |
| :--- | :--- | :--- |
| `[Attachment 01 \| Screenshot \| shot.png](file;path/to/shot.png)` | `path/to/shot.png` | `1. [Attachment 01 \| Screenshot \| shot.png](file;path/to/shot.png)` |
| `@[file_name_sequence](file;assets/shot.png)` | `assets/shot.png` | `1. [Attachment 01 \| Screenshot Attachment \| shot.png](file;assets/shot.png)` |
| `@[assets/shot.png]` | `assets/shot.png` | `1. [Attachment 01 \| Screenshot Attachment \| shot.png](file;assets/shot.png)` |
| `![Alt text](assets/shot.png)` | `assets/shot.png` | `1. [Attachment 01 \| Alt Text Screenshot \| shot.png](file;assets/shot.png)` |
