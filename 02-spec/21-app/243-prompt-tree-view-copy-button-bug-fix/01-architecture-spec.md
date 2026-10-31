# Architecture Specification: Prompt Tree View Copy Button Bug Fix & Attachment Normalization

**Slug**: `prompt-tree-view-copy-button-bug-fix`  
**Sequence**: `243`  
**Spec Path**: `02-spec/21-app/243-prompt-tree-view-copy-button-bug-fix/01-architecture-spec.md`  
**Status**: APPROVED_FOR_EXECUTION  
**Target Branch**: `main`  
**Attribution**: strictly `@aukgit`  

---

## 1. Executive Summary & Root Cause Analysis

Two critical functional and UX defects have been identified in the Prompt Tree View modal:

### 1.1 Copy Button Omission of Screenshot Attachments (Root Cause)
- **Defect**: When clicking the primary "Copy" button in `src/components/instances/PromptTreeView/DetailHeader.tsx`, screenshot attachments associated with the conversation or active prompt turn were completely omitted from the clipboard payload.
- **Root Cause**: `handleCopyAll()` in `DetailHeader.tsx` directly copied raw `source` text from `resolvePromptBody(selectedConversation, activePromptText)` using `copyToClipboard(source)`, without calling `formatPromptWithAttachments()` or consolidating `selectedConversation.media_paths` / `selectedTurn.media_paths`. Furthermore, `buildAttachmentsBlock()` in `src/utils/promptAttachment.ts` previously emitted placeholder token `1. @[file_name_sequence](file;${p})` instead of the canonical 3-tier sequence, description, and filename format.
- **Impact**: Pasting copied prompts into Antigravity IDE, OneNote, or external chats lost multimodal context and failed to invoke IDE file attachment recognition.

### 1.2 Dropdown Popovers Failing to Dismiss on Outside Click (Root Cause)
- **Defect**: Opening the Copy options dropdown (`isCopyMenuOpen`), the Send / Dispatch configuration popover (`isMenuOpen` / `isDispatchMenuOpen`), or the Next Breakdown configuration dropdown (`isBreakdownDropdownOpen`) failed to dismiss when clicking anywhere outside the menus.
- **Root Cause**: All three menus relied solely on `onMouseLeave` handlers on the floating popover `<div>` containers. Zero `mousedown` or `touchstart` event listeners were mounted to detect outside clicks.
- **Impact**: Popovers remained permanently open over prompt text areas, inspector views, and action buttons until manually hovered out or re-clicked.

---

## 2. Canonical Attachment Format Specification

### 2.1 Grammar & Structure
Multimodal attachments appended to prompt payloads across Copy, Send (Direct Preempt), Enqueue (FIFO), and Rerun (Multi-Round) must adhere strictly to the canonical 3-tier markdown link syntax:

```markdown
Attachments:
1. [Attachment 01 | <Description> | <Filename>](file;<exact_file_path>)
2. [Attachment 02 | <Description> | <Filename>](file;<exact_file_path>)
```

Where:
1. **Third Bracket (`[...]`)**: Contains display metadata:
   - **Sequence Number**: 2-digit zero-padded index (`Attachment 01`, `Attachment 02`, etc.).
   - **Description**: Semantic description of the image content (e.g., `Screenshot Attachment`, `Billing Error Screenshot`, or user-specified alt text).
   - **Filename**: Base filename extracted from the target path (e.g., `media_1793460288056_37113f30.png`).
2. **Third Bracket Close (`]`) & Parenthesis Open (`(`)**: The closing third bracket `]` is immediately followed by opening round parenthesis `(`.
3. **Keyword Prefix**: Exact lowercase string `file` followed by semicolon `;`.
4. **Exact Path**: Normalized absolute or workspace-relative path.
5. **Parenthesis Close (`)`)**: Terminates the attachment link.

### 2.2 Backward Compatibility & Extractor Invariant
Parsers in TypeScript (`promptAttachment.ts`, `clipboard.ts`) and Rust (`detection.rs`) must robustly extract clean file targets from:
- `[Attachment XX | Description | Filename](file;<path>)`
- `@[label](file;<path>)`
- `[sequence number] description (file;path)`
- Legacy `@[file;<path>]` and bare `@[<path>]`
- Standard markdown `![alt](<path>)`

---

## 3. UI/UX Click-Outside Dismissal Architecture

### 3.1 Hook Integration
`DetailHeader.tsx` mounts `useClickOutside` from `src/hooks/useClickOutside.ts` targeting separate container refs:
1. `copyMenuRef`: Encompasses Capsule A (`[ Copy | ▾ ]`) and Flyout 1.
2. `dispatchMenuRef`: Encompasses Capsule B (`[ Send | Enqueue | Rerun | ▾ ]`) and Flyout 2.
3. `breakdownMenuRef`: Encompasses Capsule C (`[ ✨ Next Breakdown | ▾ ]`) and Flyout 3.

When a click occurs outside the active menu container, the corresponding state (`isCopyMenuOpen`, `isDispatchMenuOpen`, `isBreakdownDropdownOpen`) is set to `false` immediately.
