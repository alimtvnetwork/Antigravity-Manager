# AGM CLI Architecture & 35-40 Command Roadmap Specification

## 1. Executive Summary & GitMap Parity Philosophy

The Antigravity Manager (AGM) Command-Line Interface is the primary headless and terminal-driven automation interface for managing multi-instance profiles, rolling 4H/weekly quotas, prompt tree persistence, proxy gateways, and account rotation.

Following the design conventions of `gitmap`:
- **Dual Syntax Support**: Commands support both `agm <noun> <verb>` (e.g. `agm instance list`) and top-level verb shortcuts (e.g. `agm instances`, `agm ls`).
- **Machine & Human Output Parity**: Every command strictly supports `--json` (`-j`) for programmatic script integration, alongside ANSI-colored human tables with clean box-drawing glyphs (`┌───┬───┐`, `│   │   │`, `└───┴───┘`).
- **Quiet Mode**: `--quiet` (`-q`) suppresses informational banners and spinners, outputting only raw identifiers or machine JSON.
- **Split-DB Isolation**: Interacts with split SQLite databases (`repo_prompts.db`, `security.db`, `email_vault.db`, `task_history.db`) without cross-database SQL joins.

---

## 2. Exhaustive Command Catalog (7 Functional Groups)

### Group 1: Instance & Profile Lifecycle (Commands 1 - 6)
1. **`agm instance list` / `agm ls`**:
   - *Syntax*: `agm instance list [--running] [--json] [-q]`
   - *Description*: Tabular listing of all configured instances, profile names, process PIDs, active status, and bound accounts.
2. **`agm instance create` / `agm create`**:
   - *Syntax*: `agm instance create <name> [--from <template_id>] [-a <email>] [--launch]`
   - *Description*: Provisions a new isolated profile directory, seeds workspace storage, and optionally binds credentials.
3. **`agm instance switch` / `agm switch`**:
   - *Syntax*: `agm instance switch <instance_id> <target_email>`
   - *Description*: Updates `state.vscdb` credentials atomically for the target instance.
4. **`agm instance launch` / `agm launch`**:
   - *Syntax*: `agm instance launch <instance_id>`
   - *Description*: Launches the host Antigravity/IDE process associated with the specified instance.
5. **`agm instance stop` / `agm stop`**:
   - *Syntax*: `agm instance stop <instance_id> [--force]`
   - *Description*: Sends graceful termination signals to the target process tree.
6. **`agm instance duplicate` / `agm duplicate`**:
   - *Syntax*: `agm instance duplicate <source_id> <new_name>`
   - *Description*: Performs deep clone of extensions, settings, and workspace storage via `copy_instance_with_options`.

### Group 2: Quota & Fast-Forward Auto-Switching (Commands 7 - 12)
7. **`agm status` / `agm credits`**:
   - *Syntax*: `agm status [--json] [--sort quota|email]`
   - *Description*: Rolling 4-hour quota and weekly allowance dashboard with ANSI visual progress bars.
8. **`agm ff` / `agm fast-forward`**:
   - *Syntax*: `agm ff [instance_id] [--target <email>]`
   - *Description*: Rotates instance to the account with the highest available quota and lowest reset latency.
9. **`agm switch-if-low-credit`**:
   - *Syntax*: `agm switch-if-low-credit [--threshold 25] [--model <name>]`
   - *Description*: Evaluates current instance quota against threshold; triggers automated fast-forward switch if < 25%.
10. **`agm auto-switch status`**:
    - *Syntax*: `agm auto-switch status [--json]`
    - *Description*: Inspects background monitoring daemon state, polling interval, and last rotation audit trail.
11. **`agm auto-switch enable|disable|toggle`**:
    - *Syntax*: `agm auto-switch <action>`
    - *Description*: Controls background daemon lifecycle.
12. **`agm auto-switch config`**:
    - *Syntax*: `agm auto-switch config [--threshold <N>] [--interval <secs>] [--model <name>]`
    - *Description*: Persists auto-switch heuristics into `gui_config.json`.

### Group 3: Prompt Tree & Conversation Management (Commands 13 - 19)
13. **`agm prompts ls` / `agm prompts list`**:
    - *Syntax*: `agm prompts ls [--instance <id>] [--all] [--running-only] [--json]`
    - *Description*: Lists active running prompts, step counts, and workspace repository targets.
14. **`agm prompts tree`**:
    - *Syntax*: `agm prompts tree [--instance <id>] [--json]`
    - *Description*: Hierarchical ASCII Unicode tree view of projects and active conversations with preview snippets.
15. **`agm prompts query`**:
    - *Syntax*: `agm prompts query "<search-term>" [--project <name>] [--json]`
    - *Description*: High-speed SQLite search across conversation turns, step logs, and prompt text.
16. **`agm prompts resend`**:
    - *Syntax*: `agm prompts resend <prompt_id> [--instance <id>] [--focus]`
    - *Description*: Re-dispatches a previously executed prompt to IDE via `.antigravity_resume_task.json`.
17. **`agm prompts backup`**:
    - *Syntax*: `agm prompts backup [--instance <id>]`
    - *Description*: Snapshots all active running prompts and turns into `repo_prompts.db`.
18. **`agm prompts restore`**:
    - *Syntax*: `agm prompts restore [--instance <id>] [--keep]`
    - *Description*: Re-injects backed-up prompt tasks into target workspaces.
19. **`agm queue-scheduler`**:
    - *Syntax*: `agm queue-scheduler [--run-once] [--interval <mins>]`
    - *Description*: Runs autonomous queue scheduler dispatching pending prompts when workspaces go idle.

### Group 4: Local Proxy & Protocol Routing (Commands 20 - 25)
20. **`agm proxy status`**:
    - *Syntax*: `agm proxy status [--json]`
    - *Description*: Inspects Axum HTTP gateway port (8045), active client connections, and upstream latency.
21. **`agm proxy test`**:
    - *Syntax*: `agm proxy test [--endpoint <url>] [--model <alias>]`
    - *Description*: Executes loopback ping verifying `/health`, protocol adaptation, and payload roundtrip.
22. **`agm proxy restart`**:
    - *Syntax*: `agm proxy restart [--port <port>]`
    - *Description*: Gracefully cycles the Axum proxy listener.
23. **`agm proxy models`**:
    - *Syntax*: `agm proxy models [--json]`
    - *Description*: Catalogs supported OpenAI, Claude, and Gemini model route aliases.
24. **`agm proxy sessions`**:
    - *Syntax*: `agm proxy sessions [--json]`
    - *Description*: Reports active 64-bit FNV-1a session cache counts, token totals, and thinking budgets.
25. **`agm proxy logs`**:
    - *Syntax*: `agm proxy logs [--tail <N>] [--filter <regex>]`
    - *Description*: Live-streaming terminal view of sanitized request/response logs.

### Group 5: REST Endpoint Security & Token Management (Commands 26 - 30)
26. **`agm security status`**:
    - *Syntax*: `agm security status [--json]`
    - *Description*: Inspects current authentication mode (`Off`, `AllExceptHealth`, `StrictAll`) and LAN binding.
27. **`agm security token create`**:
    - *Syntax*: `agm security token create <label> [--expires <days>] [--admin]`
    - *Description*: Issues scoped Bearer API token saved into `security.db`.
28. **`agm security token list` / `revoke`**:
    - *Syntax*: `agm security token list [--json]`; `agm security token revoke <token_id>`
    - *Description*: Manages authorized client tokens.
29. **`agm security ip whitelist add|remove|ls`**:
    - *Syntax*: `agm security ip whitelist <action> [ip_cidr]`
    - *Description*: Manages IP whitelist entries.
30. **`agm security ip blacklist ls|unban`**:
    - *Syntax*: `agm security ip blacklist <action> [ip_cidr]`
    - *Description*: Manages rate-limiting or suspicious connection bans.

### Group 6: Supabase Fleet & SSH Delegation (Commands 31 - 35)
31. **`agm supabase status`**:
    - *Syntax*: `agm supabase status [--json]`
    - *Description*: Node ID, alias, heartbeat latency, and endpoint synchronization state.
32. **`agm supabase sync`**:
    - *Syntax*: `agm supabase sync [--force]`
    - *Description*: Performs full bi-directional sync of accounts, tokens, and instance descriptors.
33. **`agm supabase leases`**:
    - *Syntax*: `agm supabase leases [--json]`
    - *Description*: Inspects active distributed execution locks across nodes.
34. **`agm ssh nodes`**:
    - *Syntax*: `agm ssh nodes [--json]`
    - *Description*: Lists registered remote developer worker machines.
35. **`agm ssh delegate`**:
    - *Syntax*: `agm ssh delegate <node> "<command>"`
    - *Description*: Securely delegates headless CLI commands over SSH tunnels.

### Group 7: System Diagnostics, Hygiene & Self-Healing (Commands 36 - 40)
36. **`agm doctor` / `agm check`**:
    - *Syntax*: `agm doctor [--json]`
    - *Description*: 5-point subsystem diagnostic checking Account Registry, Instances, DBs, and Gateway.
37. **`agm clean` / `agm purge`**:
    - *Syntax*: `agm clean [--temp] [--logs] [--force]`
    - *Description*: Safe cleanup of orphaned temp files and logs while preserving credentials.
38. **`agm agy cache-clear`**:
    - *Syntax*: `agm agy cache-clear [--keep <N>]`
    - *Description*: Prunes stale conversation steps into OS temp storage.
39. **`agm agy undo`**:
    - *Syntax*: `agm agy undo [transaction_id]`
    - *Description*: Reverts conversation pruning transactions.
40. **`agm update` / `agm install`**:
    - *Syntax*: `agm update [--check-only] [--channel stable|beta]`
    - *Description*: Checks GitHub releases for binary updates and synchronizes PATH.

---

## 3. Implementation Status & Delivery

- **Implemented in this iteration (`src-tauri/src/modules/cli.rs`)**:
  - `agm prompts ls` / `agm prompts list` (with `--instance`, `--all`, `--force`, `--json`)
  - `agm prompts tree` (hierarchical project/conversation ASCII tree with `--json`)
  - `agm prompts backup` (split SQLite backup to `repo_prompts.db`)
  - `agm prompts restore` (with `--keep` flag)
  - `agm doctor` / `agm check` (5-subsystem health validation with `--json`)
  - Subcommand dispatcher integration and comprehensive `help` registration.
