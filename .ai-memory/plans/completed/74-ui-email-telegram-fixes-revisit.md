# Plan 74: UI, Email, and Telegram Fixes Revisit

Spec Reference: [02-spec/21-app/74-ui-email-telegram-fixes-revisit.md](../../../02-spec/21-app/74-ui-email-telegram-fixes-revisit.md)

## Status: COMPLETED

## Architectural Context
The user has reported issues regarding instance switching, UI overlap, dark-on-dark contrast, pruning safety, and notification bloat. All tasks across UI, Email, Telegram, and CLI have been resolved and verified.

## Deliverable Mapping & Verified Outcomes

1. **Task-01:** `01-instance-switching-fix.md`
   - Fixed instance resolution to exact match and removed global credential leakage in `instance.rs`.
2. **Task-02, 03:** `02-ui-modal-and-focus.md`
   - Added `Plus` button in dropdown header, modal backdrop dismissal, explicit 'X' close buttons to all dialogs, Escape key listener, auto-scroll to active card, and high-contrast light/dark styling in `InstanceSelector.tsx` and `Instances.tsx`.
   - Added state-driven `focusedAccountId` with 100ms polling retry loop across pagination and search filters in `Accounts.tsx`, guaranteeing smooth auto-scroll to focused elements.
   - Eliminated dark-on-dark contrast (`dark:bg-amber-950/40`) in `AccountCard.tsx` and `AccountRow.tsx`, replacing with luminous theme-adaptive styling (`dark:bg-slate-800/95`, `dark:text-white font-bold`, luminous amber/blue borders, and crisp non-clashing hover/selected states).
   - Added explicit 'X' close button to Preferences popup header in `NavSettings.tsx`, Escape key dismissal, and `agm:dropdown-open` event dispatch for mutual exclusivity.
3. **Task-04:** `03-prune-safety.md`
   - Conversation pruner in `agy_cleaner.rs` guards active/queued/executing prompts and guarantees retention of top 5 sessions per active workspace.
4. **Task-05, 06, 10:** `04-notification-dedupe-and-formatting.md`
   - Deduplicated workspace names using `HashSet<String>` in `render_idle_projects_email`.
   - Expanded prompt preview cards to >= 200 words with word count badge in emails and Telegram.
   - Fixed HTML breakdown in `chunk_telegram_text` by tracking unclosed tags and auto-balancing them across split boundaries.
5. **Task-07, 08, 09:** `05-cli-and-telegram-commands.md`
   - Added `/prune` and `/query` to Telegram bot menu and command processor.
   - Mapped top-level `"query" | "search" | "find"` in `agm.rs` router to `cmd_prompts_query`.
   - Updated `print_help()` and `print_help_json()` with GitMap SSH command examples and SQLite prompt queries.
