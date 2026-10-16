# Telegram project picker and node-name cache

Read `.ai-memory/rca/18-telegram-project-listing-and-node-cache.md` before editing `/send`, machine labels, or `list_all_prompts`. That RCA is the full root cause, the file map, the mistakes, and the checklist. This page is the pointer.

Shipped in v4.211.0 (`92e8caf5`).

The picker used to show `project_id · session_id[:8]` in FIFO order and label this computer `Node-<digits>`. It now shows eight projects per page, newest conversation first, project name then conversation title, selected by number. The current machine button says This machine and uses Telegram `style: success`. Generated node names are cached in `node_display_cache.db` for `node_name_refresh_hours` (default 24).

Do not reorder `list_all_prompts`. Do not put the conversation id in the label or in `callback_data`. Do not call GitMap on every message. Do not hold the send-flow mutex across `.await`.
