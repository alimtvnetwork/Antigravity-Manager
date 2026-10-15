# 07 — UI & functionality fixes: send/enqueue, copy-all, import/export, Telegram buttons

A later AI runs this file without prior chat. This case is a **code-path
validation record** for the `antigravity-manager-agm-ui-and-functionality-fixes`
program (spec: `02-spec/21-app/antigravity-manager-agm-ui-and-functionality-fixes/01-spec.md`).

**Environment constraint (read first):** this machine is headless Linux with no
`agm` binary and no built Tauri target (`which agm` empty, no
`src-tauri/target/debug/agm`). GUI clicks, `agm ui ...`, `agm prompt ...`, and
live Telegram bot taps cannot run here. Every flow below was traced
**frontend → IPC → backend** with file:line evidence, checked against the
workers' claimed deliverables, and marked `VALIDATED` only where the full code
path exists and is coherent. Anything requiring a live window, a live bot, or
a built binary is `NEEDS-MACHINE` with exact steps.

Per `00-decisions.md`: do not score the installed `agm` if it is older than
the commit under test; a green reading of source is not a pass — the
NEEDS-MACHINE sections exist because a live pass still needs a machine.

## Flow 1 — Send dropdown with "Enqueue to FIFO Queue" — VALIDATED (code path)

Worker Task-02 claimed the Send dropdown gained an "Enqueue to FIFO Queue" entry.

- [ ] `src/components/instances/PromptTreeView/DetailHeader.tsx:404` — the Send
      dropdown ("Send Options" menu) contains an "Enqueue to FIFO Queue" entry,
      wired to the same `onEnqueue()` handler as the standalone Enqueue split
      button (comment at `:402` says so explicitly). Errors route to
      `reportPromptActionError(err, 'DetailHeader.onEnqueue')` — no silent no-op.
- [ ] `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts:407`
      `handleEnqueuePrompt`: guards on no-conversation with a **visible** error
      (`setError('Queue failed: no conversation selected — select a prompt first.')`
      — the old silent return was the "broken enqueue" symptom), then calls
      `enqueuePrompt(targetInstId, repoPath, promptContent, targetConvId)`.
- [ ] `src/services/instanceService/lifecycle.ts:70` `enqueuePrompt` → IPC
      `enqueue_prompt` → `src-tauri/src/commands/instance.rs:429` →
      `enqueue_prompt_for_instance` (`src-tauri/src/modules/repo_db/agy.rs:753`)
      via `enqueue_prompt_for_instance_full` (`src-tauri/src/modules/repo_db/queue.rs:280`).
- [ ] Enqueue is **distinct** from Send: Send calls
      `sendPromptNow(targetInstId, repoPath, promptContent, targetConvId)`
      (`usePromptTreePromptActions.ts:299`) → IPC `send_prompt_now` →
      `send_prompt_now_for_instance` (`src-tauri/src/modules/repo_db/agy.rs:612`),
      which ensures the instance is running, records the prompt with
      `status: "running"`, and dispatches immediately via
      `spawn_or_replace_prompt_via_agy(&active_prompt, true)` (`:694`), with a
      direct-CDP insertion fallback (`:699-708`) and audit records. No enqueue
      on the Send path — the "Send secretly enqueues" bug shape is gone.
- [ ] Send takes `repoPath` directly (`agy.rs:612` param) and resolves the
      instance via `resolve_instance_id` — it does **not** require the project
      to be the active one. Frontend passes the selected project's
      `repo_path` (`usePromptTreePromptActions.ts:289`) and the project-level
      `instance_id` (`:293`), any project, not just the active tab.
- [ ] FIFO invariant holds on every queue read:
      `src-tauri/src/modules/repo_db/dispatch.rs:35`
      `ORDER BY created_at ASC, id ASC`;
      `src-tauri/src/modules/repo_db/agy.rs:861` (drain oldest queued prompt);
      `src-tauri/src/modules/repo_db/queue.rs:316,320,324` (requeue reads).
      No `ORDER BY updated_at DESC` in dispatch logic.
- [ ] NEEDS-MACHINE: click Send-dropdown → "Enqueue to FIFO Queue" with the
      prompt tree open; then verify `active_prompts` gained a `queued` row for
      that conversation, and that a second enqueue lands **after** it
      (FIFO: `created_at ASC, id ASC` with ≥2 queued prompts). Then press Send
      and confirm `agy` spawns immediately (process list shows the agy worker)
      rather than a queued row sitting idle.

## Flow 2 — Copy all tree prompts — VALIDATED (code path)

Worker Task-05 claimed a "Copy All Tree Prompts" action on the unified menu.

- [ ] `src/components/instances/PromptTreeView/ModalHeader.tsx:105`
      `<ImportExportMenu onCopyAllTree={onCopyAllTreePrompts} ... />` —
      the menu button lives in the tree modal header (right-hand side), one
      entry point with the four import/export operations.
- [ ] `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts:380`
      `handleCopyAllTreePrompts`: iterates **all** projects and conversations
      (skips empty ones), builds markdown sections
      `# <project>` / `## <title> (<seq_code>)`, counts projects and
      conversations, then calls `copyPromptWithRichImages`. Empty tree shows
      "No prompts in the tree to copy." — never a silent no-op.
- [ ] Copy mechanism (`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts:15`
      `copyPromptWithRichImages`): prompt text → rich HTML by rewriting
      `![alt](url)` to
      `<img src="url" alt="alt" style="max-width:100%; border-radius: 8px; margin: 8px 0;" /><br />`
      and `\n` to `<br />`; writes a single `ClipboardItem` with **both**
      `text/html` and `text/plain` flavors. Fallback ladder: rich write fails
      → `trackWarning` to the error store → plain-text `copyToClipboard` →
      `onError` with the exception. "Compact" means: text stays as markdown
      (no re-encoding), images are embedded **by reference** (their URL or
      `data:` URI is preserved, not expanded/re-downloaded), one clipboard
      item — small enough to paste into a chat box or the IDE composer.
- [ ] NEEDS-MACHINE: open the tree modal with ≥2 projects, Import/Export →
      "Copy All Tree Prompts", confirm the action message
      "Copied N prompt(s) from M project(s)", then paste into a scratch doc
      and verify the `#`/`##` sections, the conversation titles/seq codes,
      and that images render (rich paste) — or fall back to plain text.

## Flow 3 — SQLite export/import round-trip — VALIDATED (code path)

Worker Task-05 claimed single-file SQLite export/import via
`sqlite_export.rs` + `cmd_files.rs` IPC.

- [ ] Backend module `src-tauri/src/modules/repo_db/sqlite_export.rs` (440 lines):
      `write_prompt_tree_db` (`:127`) snapshots `running_projects`,
      `active_prompts`, `agm_project_sequences`, `agm_conversation_sequences`
      into one portable `.db` (delete journal mode, `meta` table with kind
      `agm/prompt-tree-export` and schema version `1.0`);
      `import_prompt_tree_db` (`:289`) validates (SQLite magic, required
      tables, export kind — rejects bogus files and wrong-kind DBs with real
      errors) and merges back into `repo_prompts.db`; byte variants for the
      web path `export_prompt_tree_db_bytes` (`:355`) /
      `import_prompt_tree_db_bytes` (`:381`).
- [ ] IPC surface `src-tauri/src/commands/cmd_files.rs`:
      `export_prompt_tree_sqlite` (`:214`), `export_prompt_tree_sqlite_bytes`
      (`:224`), `import_prompt_tree_sqlite` (`:233`),
      `import_prompt_tree_sqlite_bytes` (`:241`) — all `#[tauri::command]`.
- [ ] Frontend `usePromptTreeCoreActions.ts:440` `handleExportSqlite`: native
      save dialog (Tauri) or blob download (web via byte payload), then
      reports **real counts** —
      "Exported P project(s), Q prompt(s) to `<path>`" from `TreeExportStats`
      (`project_count`, `prompt_count`, `sequence_count`), never a bare path.
- [ ] Frontend `:496` `handleImportSqlite`: native open dialog (Tauri) or
      hidden file input (web → base64 bytes → validated byte import), then
      "Imported N prompt(s) from SQLite export." and `loadTree(true, true)`
      refresh. All failures go through `captureError` + `setError`.
- [ ] Unit tests in the module (`:407`, `:420`): a non-database file is
      rejected ("not a SQLite" / "too small"); a DB with the wrong export
      kind is rejected ("not an AGM prompt-tree export"). Validation path has
      test coverage; there is no full export→import→compare round-trip test.
- [ ] NEEDS-MACHINE (exact steps): with the tree modal open on a tree with
      ≥2 projects and queued prompts, Import/Export → "Export Tree as SQLite",
      save the `.db`; note the reported project/prompt counts; then
      "Import Tree from SQLite", pick the file; confirm the reported imported
      count equals the exported prompt count and the tree still lists every
      project and conversation. Also try importing a text file renamed to
      `.db` and confirm a real error surfaces (no silent acceptance).

## Flow 4 — JSON export/import round-trip — VALIDATED (code path)

The unified menu reuses the existing backup/restore handlers for JSON.

- [ ] `src/components/instances/PromptTreeView/ModalHeader.tsx:106,108` —
      `onExportJson={onBackup}`, `onImportJson={onRestore}`: one
      Import/Export section hosts all four operations, as spec 3.6 requires.
- [ ] Export: `handleBackup` → native save dialog → IPC
      `write_prompt_backup_file` (`src-tauri/src/commands/cmd_files.rs:161`).
- [ ] Import: `handleRestore` (`usePromptTreeCoreActions.ts:225`) first
      `JSON.parse(content)` (invalid JSON → real error, never silent), then
      IPC `restore_prompts_backup`, then `dispatch_backed_up_prompts`, and
      reports "Restored N prompt(s) and sent M to the instance." — or the
      import-failed dispatch note with the real error. Web path: hidden file
      input → same validation.
- [ ] The backup/restore path itself was previously covered by case
      `03-cdp-onboarding-and-prompt.md` ("Prompt Tree Backup wrote a file and
      Restore dispatched it").
- [ ] NEEDS-MACHINE: Import/Export → "Export Tree as JSON", save; then
      "Import Tree from JSON", pick the file; confirm the restored prompt
      count matches the backup and the tree repaints. Feed a malformed JSON
      file and confirm a real error surfaces.

## Flow 5 — New conversation option — VALIDATED (code path)

Worker Task-04 claimed a "new conversation" option in the action area.

- [ ] State: `usePromptTreeState.ts:46` type `'current' | 'new'`,
      default `'current'` (`:138`).
- [ ] UI: "Destination" select (Current Conv / New Conv) inside the **Send**
      dropdown (`DetailHeader.tsx:368-377`) and inside the **Enqueue**
      dropdown (`:481-490`) — both actions offer it.
- [ ] Semantics: Send passes
      `targetConvId = conversationTargetMode === 'new' ? undefined : conv?.conversation_id`
      (`usePromptTreePromptActions.ts:297`); Enqueue does the same
      (`:424`). The backend `send_prompt_now_for_instance` takes
      `conversation_id: Option<&str>` (`agy.rs:612-618`) and maps it to
      `session_id` (`:650`) — `None` means a fresh conversation on dispatch.
- [ ] NEEDS-MACHINE: with Destination = "New Conv", press Send; confirm a new
      conversation appears in the tree for that project (new `seq_code` /
      conversation id) and the prompt is dispatched to it, not appended to
      the selected one. Repeat via the Enqueue dropdown entry.

## Flow 6 — Telegram send / find-instance / credits buttons — VALIDATED (code path)

Worker Task-07 claimed three fixes with RCAs 14/15/16. All three fixes are
in the working tree.

- [ ] RCA 14 — `cmd:` double-slash: `.ai-memory/rca/14-telegram-cmd-callback-double-slash.md`.
      The live daemon formatted `cmd:/send` as `//send` (slash not stripped),
      matching no command arm → tap died silently. Fix:
      `normalize_cmd_callback_payload` (`src-tauri/src/modules/telegram_inbound/daemon.rs:11-20`)
      strips every leading slash after `cmd:`, so `cmd:/send` and `cmd:fleet`
      both normalize to `/send`, `/fleet`. Both dispatch sites (one-shot
      poller and the live background loop) now call it.
- [ ] RCA 15 — 64-byte `callback_data` cap: `.ai-memory/rca/15-telegram-picker-callback-64byte-cap.md`.
      Oversized payloads made the Bot API reject the whole message
      (`BUTTON_DATA_INVALID`), logged only via `record_ignored`. Fix:
      `MAX_CALLBACK_BYTES = 60` (`src-tauri/src/modules/telegram_inbound/send_flow.rs:22-23`)
      with `callback_fits` (`:82-84`) and `picker_safe` (rejects `|` and `:`
      in labels, `:78-80`); every picker button (`send:m:`, `send:i:`,
      `send:c:`, `send:cman:`) is now guarded, and overlong machine/instance
      labels are excluded from pickers (`machine_entry_usable`, `:86-88`).
- [ ] RCA 16 — unknown-action silence: `.ai-memory/rca/16-telegram-unknown-cmd-callback-silence.md`.
      Unrecognized `cmd:` taps produced zero feedback. Fix:
      `src-tauri/src/modules/telegram_inbound/daemon.rs:700-718` — when
      delivery returns `None` for a `cmd:` payload, the bot sends
      "⚠️ <b>Unknown button action:</b> ..." and suggests refreshing the
      menu, then still answers the callback query. The send-flow's own
      `send_unknown()` (`send_flow.rs:67`) covers unknown `send:` actions
      with "⚠️ <b>Unknown send action.</b> Send <code>/send</code> to start over."
- [ ] Buttons exist with `cmd:` payloads under the cards:
      `daemon.rs:303-362` — "🚀 Prompt Wizard (/send)" → `cmd:/send`,
      "🏷️ Instances" → `cmd:/instances`, "💰 Account Balances" → `cmd:/balance`,
      plus fleet/CSV/SSH variants. Find-instance is the `/send` two-step
      picker (`send_flow.rs`): pick machine → pick instance → pick/type
      conversation → await prompt text, with the 64-byte guards above.
- [ ] NEEDS-MACHINE (bot must be configured; cannot be simulated from this
      sandbox): in the bot chat, tap "🚀 Prompt Wizard (/send)" — the
      instance picker must arrive (no silent spinner). Pick machine →
      instance → conversation, send a short line, confirm the prompt reaches
      that instance's conversation. Tap "🏷️ Instances" — instance list
      arrives. Tap "💰 Account Balances" — the real credit table returns.
      Tap a button from a stale/old menu — the "Unknown button action"
      notice must arrive, not silence. If no bot is configured, mark this
      section NOT RUN.

## Master checklist copy (07)

```text
Date: 2026-10-16
Binary version: none on this machine (headless Linux; code-path validation only)
Commit: working tree of program antigravity-manager-agm-ui-and-functionality-fixes
Instance used: none

Decisions
[x] Binary version matches the commit — N/A: no binary built; code paths traced on the working tree
[ ] Builds were one at a time, -j 1 — N/A: no build run (docs-only task)
[ ] Switch and send used the copy instance unless the user named Default — N/A: no live send run
[ ] Threshold ended at 15.0 — N/A: no auto-switch touched
[x] No token, password, or prompt body in the reply

07 UI and functionality fixes (code-path validation)
[x] Send dropdown has Enqueue to FIFO Queue entry (DetailHeader.tsx:404)
[x] Enqueue distinct from Send; enqueue = FIFO (queue.rs:280, dispatch.rs:35)
[x] New conversation option present, Send + Enqueue (DetailHeader.tsx:368,481)
[x] Copy all tree prompts present (usePromptTreeCoreActions.ts:380)
[x] Copy embeds images as rich HTML + plain-text fallback (usePromptTreePromptActions.ts:15)
[x] SQLite export/import IPC wired (cmd_files.rs:214,224,233,241; sqlite_export.rs:127,289,355,381)
[x] Import validates the file and reports real errors (sqlite_export.rs tests :407,:420)
[x] JSON export/import unified in the same menu (ModalHeader.tsx:105-112)
[x] Telegram cmd: double-slash fixed (daemon.rs:11-20; RCA 14)
[x] Telegram 64-byte cap guarded (send_flow.rs:22-23,82-88; RCA 15)
[x] Telegram unknown-action fallback sends a notice (daemon.rs:700-718; RCA 16)

Verdict
[x] PASS on code-path validation for all seven flows
[ ] LIVE PASS still needs the user's machine for: Send click + agy spawn, FIFO order with ≥2 queued prompts, new-conversation tree entry, clipboard paste content, SQLite/JSON import round-trip counts, Telegram bot taps
```
