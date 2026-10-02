# 98 — Fast account switch and compact pinned models

## Switch

The switch command returns after credentials are written and the IDE is relaunched.

Quota refresh, the mailbox poll inside `select_candidate_profiles`, and email/Telegram run after that return. They must not be awaited by `switch_account` or `switch_account_to_instance`.

`wait_for_instance_prompt_channel` runs only when `count_backed_up_prompts(instance_id) > 0`. With a backed-up prompt, the wait and the single re-push stay. With none, the 1.5s sleep is skipped.

Root cause: `.ai-memory/issues/59-switch-blocks-on-quota-and-mailbox-rca.md`.

## Pinned quota models

The Settings block is a filter plus compact chips. Pinned models sort first and show a check. The model id is the label. The long name is the tooltip. At least one model stays pinned.
