# Message catalog for Task-04 rewrite (from Research 01 audit)

> Source: read-only audit of `src-tauri/src/modules/telegram_inbound/`.
> Task-04 worker: rewrite each string per the "suggested rewrite" column.
> Preserve the `ide_exe_line` IDE lines added by Task-02 (formatting.rs:576,
> used in send_flow.rs:375, send_flow.rs:668, injection.rs:273, injection.rs:376).

## A. `/send` guided flow (send_flow.rs)

| # | file:line | function | trigger | current text | rewrite to |
|---|-----------|----------|---------|--------------|------------|
| 1 | send_flow.rs:63 | `send_unknown` | stale/invalid `send:*` callback | `⚠️ <b>Unknown send action.</b> Send <code>/send</code> to start over.` | `⚠️ That send button has expired or isn't recognized. Send /send to start fresh.` |
| 2 | send_flow.rs:395 | `start_send_flow` | `/send cancel` or Cancel button | `🧹 <b>Send flow cancelled.</b> Nothing was dispatched.` | `Send cancelled — no prompt was dispatched.` |
| 3 | send_flow.rs:399 | `start_send_flow` | local label not picker-safe | `⚠️ <b>Send unavailable:</b> this machine's label cannot be shown in the destination picker (unsafe characters or too long for a button). Rename the node alias and try <code>/send</code> again.` | `Sending is unavailable: this machine's name can't be shown as a button. Rename the node alias and try /send again.` |
| 4 | send_flow.rs:405 | `start_send_flow` | step 1 of 3 | `📮 <b>Send a prompt — step 1 of 3: pick the destination machine</b>…👆 Tap a machine below, then pick the instance and the conversation. The destination locks <b>before</b> any prompt text is accepted.` | `Where should your prompt go? (Step 1 of 3 — pick the machine)` … `The destination is confirmed before any prompt text is accepted.` |
| 5 | send_flow.rs:441 | `on_machine_picked` | unresolvable alias | `⚠️ <b>Unknown machine:</b> <code>{}</code>. Send <code>/send</code> to pick again.` | `That machine isn't available. Send /send to pick again.` |
| 6 | send_flow.rs:463 | `on_machine_picked` | step 2 of 3 | `📮 <b>Send a prompt — step 2 of 3: pick the instance on {machine}</b>…👆 Tap an instance. Instance ids resolve on the target machine.` | `(Step 2 of 3 — pick the instance on {machine})` … `Tap an instance; its id is resolved on the target machine.` |
| 7 | send_flow.rs:490 | `on_instance_picked` | unknown machine/instance | `⚠️ <b>Unknown machine:</b> <code>{}</code>. Send <code>/send</code> to start over.` / `⚠️ <b>Unknown instance:</b> <code>{}</code>. Send <code>/send</code> to start over.` | `That machine/instance is no longer available. Send /send to start over.` |
| 8 | send_flow.rs:308 | `render_project_page` | step 3 of 3 | `📮 <b>Send a prompt — step 3 of 3: pick the project</b>…👆 Tap a number, or type that number. <b>Next</b> shows the next eight projects.` | `(Step 3 of 3 — pick the project)` … `Tap a number, or type it. Tap Next for the next 8.` |
| 9 | send_flow.rs:331 | `render_project_page` | zero projects | `No projects were found for this instance. Tap <b>✏️ Type it manually</b> and reply with the conversation Seq ID (e.g. <code>C001</code>), project name, or conversation id.` | `No projects found for this instance. Tap ✏️ Type it manually, then reply with the conversation ID (e.g. C001), project name, or conversation id.` |
| 10 | send_flow.rs:388 | `on_project_number` | bad number | `⚠️ <b>That number is not on this page.</b> Tap a number that is listed, or send <code>/send</code> to start over.` | `That number isn't on this page. Tap one of the listed numbers, or send /send to start over.` |
| 11 | send_flow.rs:529 | `on_manual_conversation` | type-it-manually | `✏️ <b>Type the conversation</b>…The destination locks before any prompt text is accepted.` | `Type the conversation. Reply with the conversation ID (e.g. C001), project name, or conversation id. Your destination is confirmed before any prompt text is accepted.` |
| 12 | send_flow.rs:374 | `destination_locked_reply` | conversation chosen | `🎯 <b>Destination locked</b>\n• <b>Machine:</b> {}\n• <b>Instance:</b> <code>{}</code>\n• <b>Conversation:</b> <code>{}</code>\n\n✍️ Now send the prompt text as a plain message. It will go to exactly this destination — nothing is guessed.` | `🎯 <b>Destination confirmed</b> — {machine} · {instance} · {conversation}\n\nNow send your prompt as a plain message. It will go to exactly this destination.` (KEEP the IDE exe line Task-02 inserted between Instance and Conversation) |
| 13 | send_flow.rs:666 | `dispatch_send_prompt` | send receipt | `✅ <b>Prompt sent</b>\n• <b>Machine:</b> <code>{}</code>\n• <b>Instance:</b> <code>{}</code>\n• <b>Conversation:</b> <code>{}</code>\n\n{injected}\n\n{proof}` | `✅ <b>Prompt delivered</b> to {machine} · {instance} · {conversation}` + IDE exe line (KEEP Task-02's) + injection result + proof line |
| 14 | send_flow.rs:585 | `handle_send_callback` | send:cancel | same as #2 | same as #2 (dedupe) |

## B. `/target` aim flow (ui_remote.rs)

| # | file:line | function | trigger | current text | rewrite to |
|---|-----------|----------|---------|--------------|------------|
| 16 | ui_remote.rs:719 | `UiVerb::SetAim` | `/target <instance> <C001>` | `🎯 <b>Next prompt goes here</b>…Send the prompt as the next plain message. It stays aimed for 10 minutes. <code>/target clear</code> forgets it.` | `🎯 Your next message will go to {instance} · {conversation}. This stays active for 10 minutes. /target clear cancels it.` |
| 17 | ui_remote.rs:837 | `aim_status_html` | `/where` | `🎯 <b>Current aim</b>…The next plain message is the prompt.` | `🎯 Current target: {instance} · {conversation}. Your next plain message will be sent there as a prompt.` |
| 18 | ui_remote.rs:842 | `aim_status_html` | no aim | `🎯 <b>No aim set.</b> Send <code>/target default C001</code> first, then the prompt text.` | `No target set. Send /target default C001 first, then your prompt.` |
| 19 | ui_remote.rs:692 | `UiVerb::ClearAim` | `/target clear` | `🧹 <b>Aim cleared.</b> The next plain message is no longer sent as a prompt.` | `Target cleared — your next message won't be sent as a prompt.` |
| 20 | ui_remote.rs:766 | `UiVerb::AimedPrompt` | aimed prompt receipt | `🎯 <b>Prompt sent to</b> <code>{}</code> on <code>{}</code>\n{note}\n\n{injected}` | `✅ Prompt delivered to {conversation} on {instance}` + result |
| 21 | ui_remote.rs:762 | note line | screenshot note | `📸 The manager window on this machine is attached. This does not click a conversation inside the IDE.` / `Screenshot was not attached: <code>{}</code>` | `A screenshot of the manager window is attached above (the conversation itself wasn't clicked).` / `Screenshot unavailable: {err}` |
| 22 | ui_remote.rs:494 | `window_command_section` | `/help` footer | `🟢 Send <code>/target &lt;instance&gt; &lt;C001&gt;</code> first — the next plain message is the prompt and goes to that conversation. <code>/target clear</code> forgets the aim. For a guided machine → instance → conversation picker with screenshot proof of delivery, use <code>/send</code>.` | `Send /target <instance> <C001> first — your next plain message becomes the prompt for that conversation. /target clear cancels. For a guided picker with delivery proof, use /send.` |

## C. Prompt injection receipts (injection.rs)

| # | file:line | function | trigger | current text | rewrite to |
|---|-----------|----------|---------|--------------|------------|
| 23 | injection.rs:67 | `execute_prompt_injection` | no target/text | `⚠️ <b>Missing Prompt Content:</b> Please specify both the target…` | `Please include both a target and your prompt, e.g. /prompt default Write unit tests` |
| 24 | injection.rs:106 | same | target only | (near-duplicate of #23) | same as #23 (dedupe) |
| 25 | injection.rs:160 | same | empty text | `⚠️ <b>Empty Prompt:</b> Prompt text cannot be empty.` | `Your prompt text is empty — nothing was sent.` |
| 26 | injection.rs:238 | same | DB save failure | `⚠️ <b>Failed to Save Prompt:</b> <code>{}</code>` | `Couldn't save your prompt: {err}` |
| 27 | injection.rs:275 | local receipt | THE local receipt | `🚀 <b>Prompt Injected Locally!</b>\n\n• <b>Prompt ID:</b> <code>{id}</code>\n• <b>Dual Seq ID:</b> <code>[{}]</code>\n• <b>Conversation ID:</b> <code>{}</code>\n• <b>Instance:</b> <code>{}</code>\n• <b>Target Workspace:</b> <code>{}</code>\n• <b>Path:</b> <code>{}</code>\n• <b>Status:</b> {badge}\n• <b>Prompt Content:</b>\n<pre>{800 chars}</pre>` (badges `🟢 Executing via agy` / `⚪ Enqueued in Split DB`) | `✅ <b>Prompt sent</b> to {instance} · {workspace}\n<b>Status:</b> Executing now / Queued\n<b>Reference:</b> <code>{id8}</code>\n<b>Your prompt:</b>\n<pre>{content}</pre>` — hide Dual Seq ID, Conversation ID, Path unless debugging. KEEP Task-02's IDE exe line. |
| 28 | injection.rs:364 | remote status | remote path | `🟢 Dispatched immediately via GitMap Cluster SSH` / `📥 Enqueued to Supabase Secondary DB (Node will execute on poll)` / `⚠️ Dispatched instruction (Node recorded in queue)` | `Sent via GitMap Cluster SSH` / `Queued on the secondary database — the node will pick it up` / `Queued (the node recorded your instruction)` |
| 29 | injection.rs:372 | remote receipt | THE remote receipt | `🌐 <b>Remote Prompt Dispatched!</b>\n\n• <b>Target Node:</b> <code>{}</code>\n• <b>Target Seq / Project:</b> <code>{}</code>\n• <b>Instance:</b> <code>{}</code>\n• <b>Status:</b> {}\n• <b>Prompt Content:</b>\n<pre>{800 chars}</pre>` | `✅ <b>Prompt sent</b> to {node} · {project} ({instance})\n<b>Status:</b> {plain status}\n<b>Reference:</b> <code>{id8}</code>\n<b>Your prompt:</b>\n<pre>{content}</pre>` — KEEP Task-02's IDE exe line. |

## D. Delivery proof (open_shot.rs)

| # | file:line | function | trigger | current text | rewrite to |
|---|-----------|----------|---------|--------------|------------|
| 30 | open_shot.rs:624 | `capture_and_send_proof_shot` | proof OK | `📸 <b>Proof:</b> screenshot from <code>{}</code> attached above.` | `Screenshot from {machine} attached above as delivery confirmation.` |
| 31 | open_shot.rs:604 | same | no target | `📸 <b>Proof screenshot skipped:</b> <code>{}</code>` | `Delivery screenshot skipped: {reason}` |
| 32 | open_shot.rs:627 | same | rejected | `📸 <b>Proof screenshot failed:</b> captured but Telegram rejected the photo: <code>{}</code>` | `Delivery screenshot couldn't be sent: {reason}` |
| 33 | open_shot.rs:634 | same | capture failed | `📸 <b>Proof screenshot failed on <code>{}</code>:</b> <code>{}</code>` | `Couldn't capture a delivery screenshot on {machine}: {reason}` |

## E. Status/history/reports — de-jargon pass

- reports_a.rs:248 (`/status` rows): `• <b>{project}</b> 🟢 {duration}\n   {preview} <i>({words} words)</i>\n   <i>Expand: <code>/expand {id8}</code></i>` → `• {project} — running for {duration}\n   {preview}\n   View full text: /expand {id8}`
- reports_a.rs:269 (empty): `• None (All workspaces idle)` → `Nothing is running — all workspaces are idle`
- reports_a.rs:332 (hint): `💡 Send <code>/expand &lt;id&gt;</code> to view full prompt text…` → `Send /expand <id> to read a full prompt, or /active for the live view.`
- reports_cluster_a.rs:233 (`/projects`): de-jargon header → `📂 Workspaces & projects ({n})`; simplify sample invocations.
- reports_cluster_a.rs:252 (`/active`): `⚡ <b>Antigravity Active Running Prompts (GitMap)</b>` → `⚡ Active prompts`
- reports_cluster_a.rs:129 (`/snapshot`): `• Active Queued Prompts: <b>{}</b> pending in Supabase Secondary DB` → `• {n} prompts queued (waiting in the secondary database)`
- reports_b.rs:195 (`/query` no match): `🔍 <b>Prompt Query:</b> No prompts matched <code>{}</code>.…` → `No prompts matched "{term}". Send /query for recent prompts or /observe for the live view.`
- reports_b.rs:203 (`/query` results): `🔍 <b>Prompts Query Results:</b> (Found {} matches)…` → `🔍 {n} prompts found…` (ref {id8}, drop "Details:" jargon)
- reports_b.rs:94 (`/expand`): keep structure; headline → `Prompt {id8} — {project}\nMachine: {name} · Instance: {id}\nStatus: {status}\nFull text:\n{content}` — hide full UUID and internal IP, use 8-char ref.
- reports_b.rs:124 (`/expand` bad id): `⚠️ <b>Prompt Not Found:</b>…` → `No prompt matched "{q}". Send /active for running prompts or /prompts all for the full list.`
- reports_cluster_a.rs:293/308/319/335 (`/queues`): `📥 <b>Antigravity Workspace Prompt Queues</b>` → `📥 Prompt queues`; `All workspace queues are currently empty.` → `The prompt queue is empty.`; rows `{i}. [⏳ QUEUED] <code>{id12}</code>` → `{i}. ⏳ {project} — "{summary}" (ref {id8})`; `⚠️ <b>Queue Check Failed:</b>` → `Couldn't read the prompt queue: {err}`
- reports_cluster_b.rs:84 (`/prompts all`): `📝 <b>State Database Prompt Queue ({} Prompts, Showing Top {})</b>` → `📝 Prompts ({total}, showing {n})`; rows: hide raw full id (use id8), drop "State Database"/"split SQLite" jargon.
- reports_cluster_b.rs:111 (empty): `📝 <b>Prompt Queue:</b> No prompts currently registered in split SQLite database.` → `No prompts on record.`
- reports_cluster_b.rs:68 (`/templates`): `📋 <b>Available Reusable Prompt Templates ({} Templates)</b>` → `📋 Prompt templates ({n})`; simplify how-to lines.
- commands.rs:192/206/210 (`/backup ls`): `🎒 <b>Split SQLite Prompt Backups ({} total):</b>` → `📦 Prompt backups ({n})`; `⚠️ <b>Backup Query Failed:</b>` → `Couldn't read prompt backups: {err}`; no-batch → `No prompt backups stored.`
- commands.rs:274/285 (`/backup`): `🎒 <b>Running Prompts Backed Up Successfully!</b>…• <b>Vault Path:</b>…` → `✅ Backed up {n} running prompts (backup {id8}). Send /restore anytime to resume them.` (hide vault path); failure → `Backup failed: {err}`
- commands.rs:244 (`/restore`): `♻️ <b>Prompt Restoration Complete:</b>…from split SQLite backup.` → `♻️ Restored {n} prompts from backup; {m} re-sent to their workspaces.`; failure → `Restore failed: {err}`
- processing.rs:276/343 (`/ff`): `⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>…` → `⏩ Switched to the highest-quota account. {n} prompts backed up before the switch; {m} re-sent ({d} dispatched). Everything continues without interruption.`
- reports_b.rs:245 (`/help` prompt section): verb changes — "Prompt local X instance directly" → "Send a prompt to…"; "forgets" → "cancels"; "Inject" → "Send to".
- reports_b.rs:295 (`/help` footer): `/send` "locks" → "confirms"; `/target` "aims" → "points … at".
- config.rs:110 (BotCommand menu): add missing `/send` and `/target`; de-jargon descriptions ("Inject" → "Send", drop "split SQLite DB"/"AGM Seq IDs").
- daemon.rs:701 (unknown cmd:): `⚠️ <b>Unknown button action:</b> <code>{}</code>\nThe menu may be stale — send a fresh command to get new buttons.` → `That button is no longer valid — send a fresh command to get new buttons.`

## Global rules
- Jargon purge: "Inject/Injection", "dispatched", "Seq ID", "Split DB", "split SQLite database", "State Database", "Dual Seq ID", "Re-injection", "Vault Path", "Prompt aim", "Instance ids resolve on the target machine". Use plain words.
- IDs: 8-char refs (`id8`) in user-facing text; never full UUIDs or internal IPs.
- **Test assertions**: `send_flow.rs` `mod tests` (line ~801+) and `tests.rs` assert on old strings — update them to match rewrites.
- **KEEP Task-02's IDE exe lines** (`formatting.rs:576 ide_exe_line`, used at send_flow.rs:375, send_flow.rs:668, injection.rs:273, injection.rs:376) — rewrite around them, don't delete them.
