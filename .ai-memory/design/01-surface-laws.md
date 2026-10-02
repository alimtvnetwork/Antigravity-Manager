# Surface laws

Follow these on every new or edited screen in Antigravity-Manager. The Accounts table is the reference.

## Contrast

- A selected or hovered row uses a white background and `text-slate-950`. Do not put dark text on a navy row, and do not put blue links on a blue panel.
- Dark mode email and code text uses `#f8fafc` when the panel behind it is dark.
- A focused account stays on that white row and scrolls into view. A 4-second poll must not clear the focus.

## Menus

- A menu inside a sticky table cell is clipped. Render it with a portal on `document.body`, `position: fixed`, and `z-index` above the sticky cell (`z-10`).
- Place the menu from the button's `getBoundingClientRect`, and translate it so it stays inside the viewport.
- Keep the primary actions as icons: switch, IDE switch, CLI switch, proxy, delete. Put secondary actions in one More menu: refresh, details, fingerprint, export.

## Quota

- The header offers Gemini or Claude. There is no All column pair.
- The left cell is that model's 4-hour percentage and reset. The right cell is that model's weekly bucket (`window` contains `week`).
- The page-level 5H / Weekly toggle must not replace those two cells.

## Priority and tier

- If every visible account has the same priority, hide the priority chip. Missing priority counts as 50.
- Double-click the email or the chip to edit in place. Enter saves. Escape cancels.
- Store subscription tier as `PRO`, `ULTRA`, or `FREE` after `normalize_subscription_tier`. Log an unknown raw id. Do not relabel `standard-tier` as Pro until a captured payload shows that id on a known Pro account.

## History

- New account-switch and account-add work writes a row in the task-history split database. The screen for that list is Audit, beside Settings. See `.ai-memory/learned/24-task-history-split-db.md`.
