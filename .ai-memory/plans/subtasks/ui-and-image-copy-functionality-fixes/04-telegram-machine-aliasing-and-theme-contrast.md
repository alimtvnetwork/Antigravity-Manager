# Subtask 04: Telegram Machine Aliasing & Theme Contrast

**Parent Plan**: `.ai-memory/plans/ui-and-image-copy-functionality-fixes.md`  
**Target Files**:  
- `src-tauri/src/modules/telegram_inbound/node_label.rs`  
- `src-tauri/src/modules/telegram_inbound/formatting.rs`  
- `src-tauri/src/bin/agm/status_cmds.rs`  
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`  
- `src/components/instances/PromptTreeView/markdown.tsx`  

---

## 1. Objectives
1. Local Machine IP to GitMap Alias Resolution:
   - When node alias is `"none"`, empty, or auto-generated (`"Node-XXXXXX"`), load `~/.gitmap/gitmap-ssh-nodes.json`.
   - Match local IP `192.168.1.3` to find alias `w1`.
   - Annotate in Telegram and diagnostics with `(collected from gitmap)`.
2. Telegram Formatting:
   - Separate machine and instance into distinct lines:
     ```
     <b>Machine:</b> w1 (192.168.1.3) [collected from gitmap]
     <b>Instance:</b> default
     ```
3. Detail Pane Theme Contrast:
   - Increase dark mode background richness and contrast in `PromptInstructionView.tsx` and `markdown.tsx` (`dark:bg-[#0c0514]` with crisp text and subtle glass borders).
