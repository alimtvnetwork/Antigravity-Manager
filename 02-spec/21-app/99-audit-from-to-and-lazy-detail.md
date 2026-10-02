# 99 — Audit from-to and lazy detail

The audit list shows a masked from → to. The domain is never shown. The first characters of the name stay so two accounts can be told apart.

The list query does not return `payload_json`. The Detail button calls `get_task_history_detail` for that one row. The panel is a table: from, to, when, reason, how, the prompt that was running, and whether it was injected again.

Older rows that already stored `from_email` and `to_email` inside `payload_json` are copied into the list columns when the split file opens. The prompt text stays in the detail call.
