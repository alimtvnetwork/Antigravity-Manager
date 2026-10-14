# 02 Click and invoke sink

Status: `PENDING`

Spec: `02-spec/21-app/ui-session-recorder/01-guideline.md`

Keep `recordClickEvent` in `src/stores/error-store.ts` for the error modal. Add a session sink beside it so a recording and an error share one click shape. Wrap `invoke` in one helper. Do not add a manual log line in every command. The wrapper records the command name, start, and result. Append steps only while a recording is active. Cap the session at 500 steps or 2 MB and write one `truncated` marker.
