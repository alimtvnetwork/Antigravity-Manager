# 223 Prompt Tree View Issues

Status: active

## Root cause

Send and Enqueue dropped the selected conversation id whenever that conversation was already running, so Current Conv fell through to the focused-composer injector and opened a new chat. Image markdown wrapped absolute paths in angle brackets, which the IDE cannot open as a workspace file and which Windows rejects as a path (os error 123). The tree timer dropped to 2 seconds whenever any project was running, and conversation hover added a border that shifted the row.

## Rules

1. The tree reloads on the configured Sync interval (default 30 seconds) or the Refresh button. A running project does not shorten that interval.
2. Current Conv passes the conversation id to `agy --conversation` and does not use the focused-composer injector.
3. New Conv passes no conversation id.
4. Attached images in dispatched markdown are workspace-relative (`![Attached Image N](relative/path)`), never `file:///` and never angle-bracket absolute paths.
5. Hovering a conversation does not add a border.
6. Header actions Import/Export and Hide Tree are icons with tooltips. Reset and Refresh show those words. Send, Enqueue, and Rerun share one control plus a dropdown.
7. Instance card actions are one full-width segmented capsule. Yellow hover text is black.

## Error history

`agm errors list` reads `errors.db`. The dashboard drawer reads the in-memory store. Codes without an `[E####]` tag are stored as E9001. See `.agents/skills/agm-error-manager-cli/skill.md`.
