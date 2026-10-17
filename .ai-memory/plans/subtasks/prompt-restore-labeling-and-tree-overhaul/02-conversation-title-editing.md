---
plan: prompt-restore-labeling-and-tree-overhaul
subtask: "02"
title: Conversation Title Storage Synchronization & Multi-Entry UI Editing (Double-Click & Right-Click)
domain: full-stack
depends_on: 01-prompt-restore-labeling-and-running-detection.md
citations:
  architecture_spec: ../../../../02-spec/21-app/prompt-restore-labeling-and-tree-overhaul/02-conversation-title-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib/run.rs
  - src-tauri/src/modules/repo_db/mod.rs
  - src-tauri/src/modules/repo_db/sequences.rs
  - src-tauri/src/modules/repo_db/tree.rs
  - src-tauri/src/modules/cli/prompts.rs
  - src/components/instances/PromptTreeView/ConversationNode.tsx
  - src/components/instances/PromptTreeView/ConversationContextMenu.tsx
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/usePromptTreeData.ts
status: pending
---

# Subtask 02: Conversation Title Storage Synchronization & Multi-Entry UI Editing (Double-Click & Right-Click)

## 1. Context & Objective
In Antigravity-Manager, users need the ability to rename conversations directly from the Prompt Tree View. Currently, conversation nodes only show auto-generated titles or truncated short IDs with no in-place renaming capabilities.

This subtask implements:
1. **Dual-Storage Parity Backend Routine:** Updates `conversation_summaries.db` (`conversation_summaries.title`) in the instance's `.gemini` home directory and `repo_prompts.db` (`agm_conversation_sequences.title` and `updated_at`), enforcing SQLite WAL mode and a 5000ms `busy_timeout`.
2. **Backend Tauri IPC Command & Registration:** `rename_conversation(instance_id, conversation_id, new_title)` registered in `src-tauri/src/commands/instance.rs`, `commands/mod.rs`, and `src-tauri/src/lib/run.rs`.
3. **Headless CLI Parity:** `agm prompts rename <conversation_id> "<new_title>" [--instance <id>] [--json]` in `src-tauri/src/modules/cli/prompts.rs`.
4. **Three Frontend Entry Points:**
   - **Entry Point 1:** Double-clicking title text in `ConversationNode.tsx` activates an inline `<input>` with `e.stopPropagation()` (preventing inspector modal trigger), committing on Enter/blur and cancelling on Escape.
   - **Entry Point 2:** Right-clicking the conversation row opens custom `ConversationContextMenu.tsx` with `✏️ Rename Conversation`, `🔍 Inspect Transcript`, and `📋 Copy Conversation ID`.
   - **Entry Point 3:** Double-clicking the top-level conversation title in `DetailHeader.tsx` activates inline editing with Enter/Escape/blur handling.
5. **Bidirectional Cache Invalidation:** Purges both SQLite `prompt_tree_cache` and frontend in-memory `promptTreeSwrCache`.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Modules | Planned Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/repo_db/sequences.rs` | `rename_conversation_dual_store` | New core function: locates instance `conversation_summaries.db` via `gemini_dirs_for_instance`, executes WAL update with 5000ms `busy_timeout`, updates `agm_conversation_sequences` in `repo_prompts.db`, and invalidates tree cache. |
| `src-tauri/src/modules/repo_db/mod.rs` | Re-exports | Export `rename_conversation_dual_store`. |
| `src-tauri/src/commands/instance.rs` | `rename_conversation` | New `#[tauri::command]` invoking blocking task for `rename_conversation_dual_store`, returning `Result<(), AppError>`. |
| `src-tauri/src/commands/mod.rs` | Re-exports | Export `rename_conversation` from `instance`. |
| `src-tauri/src/lib/run.rs` | `generate_handler!` | Register `commands::rename_conversation`. |
| `src-tauri/src/modules/cli/prompts.rs` | `execute_prompts_rename`, `handle_prompts_subcommand` | Add `rename` subcommand handling positional args `<conversation_id>` and `<new_title>`, `--instance`, and `--json`. |
| `src/components/instances/PromptTreeView/ConversationContextMenu.tsx` | `ConversationContextMenu` (NEW) | Custom dark-glass context menu via `createPortal`, viewport edge clamping, Escape/click-outside dismissal, 3 action items. |
| `src/components/instances/PromptTreeView/ConversationNode.tsx` | `ConversationNode`, `ConversationNodeProps` | Add inline double-click title edit state (`isEditingTitle`), suppress `onDoubleClick` bubbling (`e.stopPropagation()`), mount `ConversationContextMenu` on `onContextMenu`. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader`, `DetailHeaderProps` | Add inline double-click editing on top-level title `<h3>`, Enter/blur commit, Escape cancel. |
| `src/components/instances/PromptTreeView/usePromptTreeData.ts` | `invalidatePromptTreeCache` | Verify export and ensure local in-memory cache and re-fetch synchronization. |

---

## 3. Step-by-Step Implementation Guide

### Phase A: Backend Dual-Storage Routine & Commands

#### Step 1: Implement `rename_conversation_dual_store` in `src-tauri/src/modules/repo_db/sequences.rs`
1. Define function:
   ```rust
   pub fn rename_conversation_dual_store(
       instance_id: &str,
       conversation_id: &str,
       new_title: &str,
   ) -> Result<(), crate::error::AppError> {
       let clean_title = new_title.trim();
       if clean_title.is_empty() {
           return Err(crate::error::AppError::Config("Title cannot be empty".into()));
       }
       let clean_cid = conversation_id.trim();
       if clean_cid.is_empty() {
           return Err(crate::error::AppError::Config("Conversation ID cannot be empty".into()));
       }

       // 1. Update instance conversation_summaries.db
       let candidate_dirs = super::gemini_dirs::gemini_dirs_for_instance(instance_id);
       let mut updated_summaries = false;
       for dir in candidate_dirs {
           let db_path = dir.join("conversation_summaries.db");
           if db_path.exists() {
               if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                   crate::error::record_ignored(
                       conn.pragma_update(None, "busy_timeout", 5000),
                       "set busy_timeout on conversation_summaries.db",
                   );
                   crate::error::record_ignored(
                       conn.pragma_update(None, "journal_mode", "WAL"),
                       "set journal_mode to WAL on conversation_summaries.db",
                   );
                   let mut count = conn.execute(
                       "UPDATE conversation_summaries SET title = ?1 WHERE conversation_id = ?2",
                       rusqlite::params![clean_title, clean_cid],
                   ).unwrap_or(0);
                   if count == 0 {
                       let pattern = format!("%{}%", clean_cid);
                       count = conn.execute(
                           "UPDATE conversation_summaries SET title = ?1 WHERE conversation_id LIKE ?2",
                           rusqlite::params![clean_title, pattern],
                       ).unwrap_or(0);
                   }
                   if count > 0 {
                       updated_summaries = true;
                   }
               }
           }
       }

       // 2. Update central repo_prompts.db (agm_conversation_sequences)
       let now = chrono::Utc::now().timestamp();
       if let Ok(repo_conn) = super::connection::get_connection() {
           crate::error::record_ignored(
               repo_conn.pragma_update(None, "busy_timeout", 5000),
               "set busy_timeout on repo_prompts.db",
           );
           crate::error::record_ignored(
               repo_conn.execute(
                   "UPDATE agm_conversation_sequences SET title = ?1, updated_at = ?2 WHERE conversation_id = ?3",
                   rusqlite::params![clean_title, now, clean_cid],
               ),
               "update title in agm_conversation_sequences",
           );
           // Also update active_prompts preview if untitled
           crate::error::record_ignored(
               repo_conn.execute(
                   "UPDATE active_prompts SET preview_raw = ?1, preview_clean = ?1 WHERE session_id = ?2 AND (preview_raw IS NULL OR preview_raw = '' OR preview_raw LIKE 'Untitled%')",
                   rusqlite::params![clean_title, clean_cid],
               ),
               "update active_prompts title if untitled",
           );
       }

       // 3. Invalidate prompt tree cache
       super::tree::invalidate_prompt_tree_cache(Some(instance_id));
       super::tree::invalidate_prompt_tree_cache(Some("all"));

       Ok(())
   }
   ```
2. Re-export in `src-tauri/src/modules/repo_db/mod.rs`.

#### Step 2: Implement Tauri IPC Command in `src-tauri/src/commands/instance.rs`
1. Add command:
   ```rust
   #[tauri::command]
   pub async fn rename_conversation(
       instance_id: Option<String>,
       conversation_id: String,
       new_title: String,
   ) -> Result<(), crate::error::AppError> {
       let resolved = instance_id.filter(|s| !s.is_empty()).unwrap_or_else(|| "default".to_string());
       tokio::task::spawn_blocking(move || {
           crate::modules::repo_db::rename_conversation_dual_store(&resolved, &conversation_id, &new_title)
       })
       .await
       .map_err(|e| crate::error::AppError::Process(format!("Task execution failed: {e}")))?
   }
   ```
2. Re-export from `src-tauri/src/commands/mod.rs` (already re-exported via `pub use instance::*;`).
3. Register in `src-tauri/src/lib/run.rs` under `generate_handler!`:
   ```rust
   commands::rename_conversation,
   ```

#### Step 3: Implement CLI Subcommand in `src-tauri/src/modules/cli/prompts.rs`
1. In `handle_prompts_subcommand`:
   ```rust
   "rename" => execute_prompts_rename(sub_args),
   ```
2. Add `execute_prompts_rename(sub_args: &[String])` handling:
   - Positional extraction for `conversation_id` and `new_title`.
   - `--instance` / `-i` flag extraction.
   - `--json` / `-j` envelope output.
   - Calling `repo_db::rename_conversation_dual_store`.

---

### Phase B: Frontend Context Menu & Multi-Entry UI Editing

#### Step 4: Create `ConversationContextMenu.tsx`
Create `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`:
- Render via `createPortal(..., document.body)`.
- Clamp position:
  ```ts
  const left = Math.max(8, Math.min(x, window.innerWidth - 200));
  const top = Math.max(8, Math.min(y, window.innerHeight - 150));
  ```
- Listeners for `Escape`, click-outside (`mousedown`), and `scroll`.
- Actions:
  - `✏️ Rename Conversation` -> calls `onRename()` and closes.
  - `🔍 Inspect Transcript` -> calls `onInspect()` and closes.
  - `📋 Copy Conversation ID` -> copies ID to clipboard, flashes check icon, and closes.

#### Step 5: Update `ConversationNode.tsx`
1. Props extension:
   - Accept optional `onRename?: (convId: string, newTitle: string) => Promise<void>` or handle locally via Tauri invoke.
2. State:
   - `const [isEditingTitle, setIsEditingTitle] = useState(false);`
   - `const [titleDraft, setTitleDraft] = useState(conv.title || '');`
   - `const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);`
3. Entry Point 1 (Double-Click on Title):
   - On the title `<span>`:
     ```tsx
     onDoubleClick={(e) => {
         e.stopPropagation(); // Invariant: Prevent parent row double-click (inspector modal)
         setTitleDraft(conv.title || conv.short_id || '');
         setIsEditingTitle(true);
     }}
     ```
   - When `isEditingTitle`:
     - Render `<input autoFocus ... />` with Enter/Escape/blur.
     - On commit: invoke `rename_conversation`, update local title, invalidate SWR cache.
4. Entry Point 2 (Right-Click Context Menu):
   - On the root `<div>`:
     ```tsx
     onContextMenu={(e) => {
         e.preventDefault();
         setContextMenuPos({ x: e.clientX, y: e.clientY });
     }}
     ```
   - Render `<ConversationContextMenu ... />` when `contextMenuPos !== null`.

#### Step 6: Update `DetailHeader.tsx`
1. In Row 1 (line 241):
   - Replace static `<h3>` with conditional inline editor:
     ```tsx
     {isEditingHeaderTitle ? (
         <input
             type="text"
             autoFocus
             value={headerTitleDraft}
             onChange={(e) => setHeaderTitleDraft(e.target.value)}
             onKeyDown={async (e) => {
                 if (e.key === 'Enter') await handleSaveHeaderTitle();
                 else if (e.key === 'Escape') setIsEditingHeaderTitle(false);
             }}
             onBlur={handleSaveHeaderTitle}
             className="text-xs font-bold text-slate-900 dark:text-white bg-white dark:bg-[#1f1535] border border-blue-500 rounded px-1.5 py-0.5 outline-none max-w-[260px] md:max-w-md shadow-inner"
         />
     ) : (
         <h3
             onDoubleClick={() => {
                 setHeaderTitleDraft(selectedConversation.title || selectedConversation.short_id || '');
                 setIsEditingHeaderTitle(true);
             }}
             className="text-xs font-bold text-slate-900 dark:text-white truncate max-w-[200px] md:max-w-xs cursor-pointer hover:underline decoration-blue-500/50 underline-offset-2"
             title={`${selectedConversation.title || selectedConversation.short_id} (Double-click to rename)`}
         >
             {selectedConversation.title || selectedConversation.short_id}
         </h3>
     )}
     ```
2. On commit:
   - Call `invoke('rename_conversation', { instanceId, conversationId, newTitle })`.
   - Invalidate in-memory cache: `invalidatePromptTreeCache(instanceId)`.
   - Update `selectedConversation.title`.

---

## 4. Verification & Pre-flight Checklist

- [ ] **Dual-Storage Update Check:** Renaming conversation `cid-test` modifies both `conversation_summaries.db` and `agm_conversation_sequences`.
- [ ] **Cache Invalidation:** Backend `prompt_tree_cache` row deleted; frontend `promptTreeSwrCache` invalidated.
- [ ] **Double-Click Propagation Isolation:** Double-clicking title text DOES NOT trigger `onOpenInspector`.
- [ ] **Escape Reversion:** Pressing Escape in inline input restores previous title without database mutation.
- [ ] **Context Menu Viewport Clamping:** Right-clicking near the bottom/right edges never overflows the window.
- [ ] **CLI Parity:** `agm prompts rename <cid> "Title"` succeeds and returns code 0.
- [ ] **Pre-flight Checks:**
  - `cd src-tauri && cargo fmt -- --check`
  - `cd src-tauri && cargo clippy --all-targets --all-features`
  - `npm run build`
