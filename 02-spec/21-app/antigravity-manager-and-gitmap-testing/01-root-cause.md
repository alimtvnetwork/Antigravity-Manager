# Prompt tree load, crash, and GitMap CLI failure

**Status:** Active  
**Slug:** `antigravity-manager-and-gitmap-testing`

## What the user sees

Opening Instances, then Prompts and History, stalls the first time and can close the window. A second open in the same session is faster. There is often no bug log the user can open from the tree itself.

## Root cause — slow first load

`get_project_conversation_tree` in `src-tauri/src/modules/repo_db/tree.rs` builds every project and conversation by scanning instance registries, workspace folders, and conversation databases (`tree_phases.rs`, `tree_collect.rs`). That scan is the slow part.

The result is stored in SQLite table `prompt_tree_cache`. Three things made that cache useless:

1. `purge_corrupted_running_projects` in `src-tauri/src/modules/repo_db/schema.rs` ran `DELETE FROM prompt_tree_cache` on every database open. A CLI tree run on this machine logged `2 prompt_tree_cache rows wiped` before it scanned 48 workspaces. The first open of every process was always a full scan.
2. The TTL written with a cache row was **5 seconds**, so even a surviving row expired before the next look.
3. `get_project_conversation_tree_for_instance` called the cache with `force: true`, so the CLI tree command never read the row it had just written.

The window also keeps an in-memory copy in `promptTreeSwrCache` inside `src/components/instances/PromptTreeView/usePromptTreeData.ts`. That is why a second open in the same session feels fast: the UI paints the memory copy while the backend may still be scanning.

The list request asks for `maxWords: 2000`, so the JSON sent back to the window is large. A large payload on top of a full disk scan is what makes the first open feel stuck.

## Root cause — the window closes with no log

The scan runs on a blocking thread in `src-tauri/src/commands/instance.rs` (`get_project_conversation_tree`). A panic in that scan used to unwind out of the command. On Windows that takes the webview down. The React error store never runs, so there is no on-screen stack.

A panic hook already writes `crash_stacktrace.log` when the process itself panics. A webview death from an unwound command does not always leave that file, which matches “there is no bug log.”

The command now catches the panic and returns the backtrace as an error string so the window stays up and the error modal can show it.

## What changed

- Startup no longer deletes `prompt_tree_cache`.
- Cache TTL is 120 seconds, not 5.
- The CLI tree read uses the cache (`force: false`).
- Tree computation is wrapped in `catch_unwind`.
- The tree header has a **Reset** button. It clears `prompt_tree_cache` for that instance and reloads. Running and queued prompts are not deleted.
- CLI: `agm prompts reset-tree` and `agm prompts reset-db`.

## What was run on this machine

`agm prompts tree --instance default --json` on the installed binary finished in about 2.5 seconds and listed the default instance, including Antigravity-Manager, gitmap, wp-exam, and white-presentation-v1. The log line before the scan was `2 prompt_tree_cache rows wiped`. A second run was about 2.0 seconds because that binary still deletes the cache and forces a rescan. The default instance process was not running (`alive=false`). Send and enqueue were not issued against a dead instance. The window was not clicked. The new Reset command is not in the installed binary until the next build.

## What was not proven on this machine

The desktop window was not clicked through. `gitmap` itself failed to start here (`paging file is too small`, then `panic: nil TLS` inside `modernc.org/libc`). That is a GitMap process failure on this host, not an Antigravity Manager tree bug. GitMap login, CLI color, and auto-commit live in the GitMap repository, not in this tree. They were not changed here.

Ubuntu and Linux were not run. The tree scan uses the same Rust path on those systems; the webview crash mode above is the Windows case.
