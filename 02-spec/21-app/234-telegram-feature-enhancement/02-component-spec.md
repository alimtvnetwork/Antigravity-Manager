# Component Specification: Telegram Feature Enhancement

## Backend

| Path | Change |
| :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/instance_cards.rs` | New. Collect/format/markup per-instance cards; handle `i:` callbacks; automation enqueue; full-prompt document. |
| `src-tauri/src/modules/telegram_inbound/mod.rs` | Register `instance_cards` module. |
| `src-tauri/src/modules/telegram_inbound/daemon.rs` | `/status` `/observe` fan-out; route `i:` callbacks; export remains `sendDocument`. |
| `src-tauri/src/modules/telegram_inbound/formatting.rs` | `TELEGRAM_COPY_TEXT_MAX = 256` for native copy buttons. |
| `src-tauri/src/modules/telegram_inbound/media.rs` | CSV/JSON as `application/octet-stream`; retry without markup; downloadable filename. |
| `src-tauri/src/modules/telegram_inbound/reports_a.rs` | Instances CSV gains prompt/project columns; balance markup adds Status + Screenshot. |
| `src-tauri/src/modules/telegram_inbound/processing.rs` | `/ff [instance]`; skip HTML export preview when document path already ran. |
| `src-tauri/src/modules/telegram_inbound/open_shot.rs` | Desktop + AGM + IDE window captures. |
| `src-tauri/src/modules/telegram_inbound/tests.rs` | Card markup byte cap, copy_text 256, CSV columns, `i:` parse. |

## Frontend

| Path | Change |
| :--- | :--- |
| `src/pages/instances/InstanceCard.tsx` | Remove `hover:-translate-y-0.5`. Add `agm-hover-shade`. |
| `src/pages/Telegram.tsx` | Hover shade on cards/buttons; no position shift. |
| `src/App.css` | `@keyframes agm-hover-wash`; `.agm-hover-shade`; amber hover → black text. |

## Card HTML (compact)

```
🟢 v2 (v2-3127) · Node-… · PID 2856
👤 riseup.figma@gmail.com [PRO]
🏃 Running projects: AGM
📥 Queued projects: (none)
⚡ Running prompts (1)
  p-239ec5 · 4m · ~87 words
  <100-word preview>
⏳ Queued prompts (0)
```

Buttons (two rows typical):

`📋 Copy` `📄 Full` `↻ Rerun`  
`✅ Done?` `🚀 Released?` `🎲 Auto`  
`⏩ FF` `📸 Shot` `🔄 Refresh`

## Instances CSV columns

`id,name,is_default,running,pid,bound_email,running_prompts,queued_prompts,running_projects,queued_projects,data_dir,machine,ip`
