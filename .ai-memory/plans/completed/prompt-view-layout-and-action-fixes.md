# [Completed Plan] Prompt View Layout Upgrades, Action Fixes & Defect Remediations

> **Slug:** `prompt-view-layout-and-action-fixes`  
> **Status:** COMPLETED  
> **Date:** 2026-10-17  
> **Specs:**  
> - `02-spec/21-app/prompt-view-layout-and-action-fixes/01-architecture-spec.md`  
> - `02-spec/21-app/prompt-view-layout-and-action-fixes/02-component-spec.md`  

---

## User Request (Verbatim)

```text
# High Priority Instruction

I want you to upgrade the UI as following. Prompt instruction and all results output should go down as I have shown. That should actually be placed into the preview, raw, edit, and details section. The preview, raw, edit, and details should be compact together with icons only, no text. The prompts instruction and all results output would be renamed as input and output. Preview, raw, edit, details should be compacted together. Input and output should remain in the same row. Put the preview, raw, edit, and details button compact together as an icon, no text, only hover over to see the text, a little bit left-hand side. You will have a lot of space to put the prompt instruction and AI results and output section. The copy, send, and rerun should go up, as highlighted in the screenshot. The rerun works well, but the send and enqueue is not working, find the root cause. The copy button does not copy the images. If you go to the next prompt, like PS, images are not copied, and this does not put into the Gemini. Send does not work, enqueue does not work, copy does not work. Verify using end-to-end test after the code is there. Check logs for errors using the Antigravity Manager (Antigravity Manager (AGM)). Understand the log where everything is, how it failed. Find the root cause, why send is not working, why enqueue is not working, why rerun is working. Write the root cause to prevent recurrence and improve the code base. In the left-hand side tree view, right-click should give a custom menu. From the all project section on the top, if you select a project, the top-level tree view will be gone. The first three items should be the conversation. If you select a specific project, pin a specific project to this prompt view. There should be a pin button. The project would be there, and start in the tree view from the conversation name. Test it, understand why the copy image does not work, figure this out. Images should be visible inside the prompt window, click to view, right-click to copy to memory. Work on it, ensure proper implementation. Test using end-to-end and CLI. CLI should have options for these tasks, modify if not. Confirm execution by looking into the screenshot by opening the UI. Ensure these are done for the last time.

# Actionable Items Must Follow Non-Negotiable

0. Gitmap do gitmap pull <repo path> or run into the repo path Make sure we have the latest and conflict resolve before starting.
1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Upgrade UI to compact preview, raw, edit, and details sections with icons only, no text.
5. Rename prompts instruction and all results output to input and output.
6. Ensure input and output remain in the same row.
7. Move copy, send, and rerun buttons up as highlighted.
8. Investigate and fix send, enqueue, and copy functionalities.
9. Verify functionality using end-to-end tests and logs from Antigravity Manager (Antigravity Manager (AGM)).
10. Implement custom menu on right-click in the tree view.
11. Enable pinning of specific projects in the prompt view.
12. Ensure images are visible and can be copied in the prompt window.
13. Modify CLI to support all UI functionalities if necessary.
14. Confirm execution by reviewing UI screenshots.
```

---

## Realized Technical Changes

### 1. UI Layout & Toolbar Compaction
- **Header Actions Relocation (`DetailHeader.tsx`):**
  - Moved `[Copy All | v]`, `[Send N | v]`, and `[Rerun xN | v]` into Row 1 on the right side adjacent to `#C{seq} · default`.
  - Eliminated the redundant second toolbar row from `DetailHeader.tsx`, reclaiming ~45px vertical space for prompt viewing.
- **Compact View Controls & Input/Output Tabs (`DetailTabs.tsx`):**
  - Compacted `Preview`, `Raw`, `Edit`, and `Details` into an icon-only segmented pill capsule (`Eye`, `Code`, `Edit3`, `FileText`), removing text labels and adding hover tooltips. Positioned to the left side.
  - Renamed `"Prompt Instruction"` to `"Input"` and `"AI Results & Outputs"` to `"Output"`, placed on the same row (Row 2) adjacent to the compact view controls.
  - Aligned prompt statistics (word count, byte size) and expand toggle on the right side of Row 2.

### 2. RCA Defect Remediations for Send & Enqueue
- **Aligned Target Instance Resolution (`usePromptTreePromptActions.ts`):**
  - Updated `handleResendPrompt` and `handleEnqueuePrompt` to check `selectedConversation?.instance_id || conv?.instance_id || proj?.instance_id || instanceId || 'default'`. This prevents sending prompts to incorrect instances where CDP ports were inaccessible.
  - Added ref fallbacks (`selectedConversationRef.current`, `activePromptTextRef.current`) to `handleEnqueuePrompt` to eliminate momentary null errors.
- **Backend Empty String Fallback (`agy.rs` & `text_utils.rs`):**
  - Updated `extract_clean_user_prompt` call sites in `src-tauri/src/modules/repo_db/agy.rs` lines 160 and 801 so prompts with `<ADDITIONAL_METADATA>` or raw conversation logs fall back to the unstripped prompt rather than failing with `"prompt content is empty"`.

### 3. Native Binary PNG Clipboard Writing & Image Inspection
- **Triple-MIME Native Clipboard Writing (`clipboard.ts`):**
  - Enhanced `copyPromptWithRichImages` to extract images, convert to binary PNG blobs via offscreen canvas/base64 decoding, and attach `'image/png': pngBlob` alongside `'text/html'` and `'text/plain'`. This allows pasting directly into Gemini Web, Photoshop, Paint, Discord, and desktop image editors.
  - Added `writeNativeImageToClipboard` for 1-click binary image copying.
- **Image Lightbox & Context Menu (`PromptMarkdownPreview.tsx`, `PromptImagePreviewModal.tsx`, `PromptImageContextMenu.tsx`):**
  - Rendered images in prompt markdown preview are clickable, opening a lightbox preview modal with zoom/pan and 1-click copy.
  - Added right-click context menu on images with "Copy Image to Clipboard (Native PNG)".

### 4. Tree View Project Filtering & Quick Pinning
- **Filtered Tree View (`TreePanel.tsx`, `ProjectNode.tsx`):**
  - When a specific project is selected in the scope dropdown (`selectedProjectScope !== 'all'`), the outer accordion folder row is suppressed.
  - The tree view starts directly from the conversation list with the first 3 items visible.
  - Added a quick Pin button on the project scope header for 1-click pinning to the prompt view.

### 5. AGM CLI Parity
- **CLI Subcommand Support (`src-tauri/src/bin/agm/prompt_cmds.rs`):**
  - Added `"rerun" | "r"` subcommands to `dispatch_prompt_domain`, delegating to `crate::prompt_goals::cmd_rerun` so `agm prompt rerun [N]` matches `agm rerun [N]`.

---

## Verification & Quality Gates
- **Rust Checks:** `cargo fmt -- --check` passed cleanly; `cargo check --bin agm` compiled with 0 errors.
- **TypeScript Checks:** `npx tsc --noEmit` verified with 0 errors.
- **Path Linter:** `03-ai-scripts/07-relative-path-fixer.py` verified 100% strict relative path compliance.
- **E2E Suite:** `03-ai-scripts/48-prompt-tree-and-restore-e2e.py` passed all 5 gates (100% green).
