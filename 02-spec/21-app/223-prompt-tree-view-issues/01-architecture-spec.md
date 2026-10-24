# 223 Prompt Tree View Issues

Status: active

## Root cause

Current Conv already passes `conversationTargetMode=current` and the selected `conversation_id`. `agy_conversation_id` used to return `None` for every `conv-` id, so `spawn_or_replace_prompt_via_agy` omitted `--conversation` and `agy` opened a new chat. `conv-` ids are real conversation keys. `new`, `p-`, `proj-`, `queued-`, and path-like values stay rejected. Image clicks in the detail pane still open `PromptImagePreviewModal` instead of an IDE file; dispatch markdown is built by `workspaceImageMarkdown` in `usePromptTreePromptActions.ts`. Image markdown wrapped absolute paths in angle brackets, which the IDE cannot open as a workspace file and which Windows rejects as a path (os error 123). The tree timer dropped to 2 seconds whenever any project was running, and conversation hover added a border that shifted the row.

## Rules

1. The tree reloads on the configured Sync interval (default 30 seconds) or the Refresh button. A running project does not shorten that interval.
2. Current Conv passes the conversation id to `agy --conversation` and does not use the focused-composer injector.
3. New Conv passes no conversation id.
4. Attached images in dispatched markdown are workspace-relative (`![Attached Image N](relative/path)`), never `file:///` and never angle-bracket absolute paths.
5. Hovering a conversation does not add a border.
6. Header actions Import/Export and Hide Tree are icons with tooltips. Reset and Refresh show those words. Send, Enqueue, and Rerun share one control plus a dropdown.
7. Instance card actions are one full-width segmented capsule. Yellow hover text is black.

## Error history

`agm errors list` reads `errors.db`. The dashboard drawer reads the in-memory store and is not cleared by `agm errors clear`. Codes without an `[E####]` tag are stored as E9001. See `.agents/skills/agm-error-manager-cli/skill.md`.

`NowRunningUpNext: prompt prompt-<uuid> not found in tree` is `trackWarning` from `PromptTreeViewModal.tsx` when `findStripConversation` misses. Historical rows that say `failed_to_resolve_parent` plus os error 123 were written before `src-tauri/src/commands/cmd_files.rs` switched that failure to `invalid_path: parent directory cannot be opened`. Do not reintroduce the old prefix.

Fast-forward rules to revalidate live in `02-spec/21-app/222-fast-forward-algorithm-issues-and-confirmation/spec.md`. Do not open sequence 224 for this task.
