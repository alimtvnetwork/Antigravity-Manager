# Subtask 01: Batch 1 (Imp 121–126: Prompt Tree Advanced Forensics & Rich Media Attachment Handling)

- **Parent Plan**: `.ai-memory/plans/pending/161-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/161-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-121: Multimodal Image Attachment Hash Resolver** (`src-tauri/src/modules/repo_db/attachment_hasher.rs`): In-memory bytes and file SHA-256 fingerprinting with MIME auto-detection for prompt attachment deduplication.
- **Imp-122: Conversation Turn Token Counter Estimator** (`src/utils/turnTokenEstimator.ts`): Client-side token estimation for text, code blocks (3.2 chars/token), vision tiles (258 tokens/image), and cost estimates.
- **Imp-123: Prompt Tree Branch ASCII Tree Visualizer** (`src-tauri/src/modules/repo_db/ascii_tree.rs`): Monospace branch tree renderer (`├──`, `└──`, `│`) for terminal logs and CLI diagnostics.
- **Imp-124: Subagent Hierarchy Nesting Depth Guard** (`src-tauri/src/modules/repo_db/depth_guard.rs`): Capping recursive subagent hierarchy to maximum depth (default 10) to prevent runaway nesting loops.
- **Imp-125: Rich Prompt Template Macro Engine** (`src/utils/promptTemplateMacro.ts`): Replaces contextual tokens (`{{workspace}}`, `{{account}}`, `{{branch}}`, `{{timestamp}}`, `{{prompt_id}}`) in templates.
- **Imp-126: Attachment Filetype Detection & Sanitizer** (`src-tauri/src/modules/repo_db/attachment_mime.rs`): Inspects magic header bytes for PNG, JPEG, WebP, and GIF to prevent corrupt or spoofed image injections.

## Targeted Improvements
- **Imp-121: Multimodal Image Attachment Hash Resolver** (`src-tauri/src/modules/repo_db/attachment_hasher.rs`)
- **Imp-122: Conversation Turn Token Counter Estimator** (`src/utils/turnTokenEstimator.ts`)
- **Imp-123: Prompt Tree Branch ASCII Tree Visualizer** (`src-tauri/src/modules/repo_db/ascii_tree.rs`)
- **Imp-124: Subagent Hierarchy Nesting Depth Guard** (`src-tauri/src/modules/repo_db/depth_guard.rs`)
- **Imp-125: Rich Prompt Template Macro Engine** (`src/utils/promptTemplateMacro.ts`)
- **Imp-126: Attachment Filetype Detection & Sanitizer** (`src-tauri/src/modules/repo_db/attachment_mime.rs`)
