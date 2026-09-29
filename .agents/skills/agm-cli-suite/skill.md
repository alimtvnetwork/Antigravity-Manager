---
name: agm-cli-suite
description: Specialized skill for operating, extending, and testing the 11,000-line native agm terminal CLI binary (src-tauri/src/bin/agm.rs), command dispatching, dual sequence formatting ([AGM:P001 | GM:#1]), machine-readable JSON flags, and GitMap AGY parity in Antigravity-Manager.
---

# AGM Native Terminal CLI Suite

Manages the autonomous native terminal companion (`agm`) for Antigravity-Manager implemented in `src-tauri/src/bin/agm.rs` and `src-tauri/src/modules/cli.rs`.

## Architectural Overview

The `agm` CLI operates completely decoupled from the Tauri GUI and WebView2 display server, functioning as a standalone headless binary for local terminals, remote SSH execution, and automated CI/CD pipelines.

```mermaid
flowchart TD
    CLI["agm &lt;subcommand&gt; [args]"] --> NORM["Normalize subcommand (strip '/', '-', lowercase)"]
    NORM --> ROUTE{"Subcommand Router"}
    ROUTE --> STATUS["agm status / credits / ilc / swlc"]
    ROUTE --> INST["agm instances / instances-all / create / rm / assign"]
    ROUTE --> TREE["agm tree / active / queues ([AGM:P001 | GM:#1])"]
    ROUTE --> PROMPTS["agm prompts / prompt / rerun / brp / rrp / wpr"]
    ROUTE --> AGY["agm agy (fpug / sug / active / backup / restore)"]
    ROUTE --> FORMAT["agm which-format / format (2-Tier Envelope Inspector)"]
    ROUTE --> DELEGATE["agm delegate-update / open-ui / update"]
```

## Core Subcommand Categories

### 1. Status, Quotas & Account Switching
- `agm status` / `agm credits`: Displays node identity, local IPv4, active account, 5h rolling quota, 7d weekly quota, and running prompt counts. Supports `--json` (`-j`).
- `agm is-low-credit-for-switch` (`ilc`): Non-mutating probe returning exit code 0/1 or machine telemetry in `--json` mode.
- `agm switch-if-low-credit` (`swlc` / `sfc`): Evaluates threshold percentage, switching to the highest-scoring candidate if active quota is depleted. Supports `-f <file>` export.
- `agm switch <account_email_or_id>`: Direct credential switch for the active profile.
- `agm ff` / `smart-switch`: Immediate fast-forward rotation to the freshest standby account (>100% 4h quota).

### 2. Multi-Instance Management
- `agm instances` (alias `ls`): Lists all configured sandbox instances with status, PID, data directory, and bound accounts.
- `agm instances create <name> [--data-only]`: Creates new profile sandbox. `--data-only` (`do`) creates lightweight sandboxes sharing the base binary.
- `agm instances assign <target> <repo_path...>`: Binds workspace project folders to an instance profile.
- `agm instances all ff`: Broadcasts fast-forward rotation across every registered instance sequentially.
- `agm instances rm-all`: Removes all secondary instances, strictly protecting the default instance.

### 3. Dual-Sequence Tree View & Prompt Dispatch
- `agm tree` (or `agm active`): Emits deterministic dual-sequence tree view:
  ```
  [AGM:P001 | GM:#1] [ProjID: <id>] <repo_path> [Instance: #<seq> <name> (<email>)]
  ├─ [AGM:C001 | GM:<cid>] "<Title>" (<status> · <steps> steps)
  │  └─ [Prompt ≤200w]: "<truncated prompt text>"
  ```
- `agm prompt <target> "<prompt_text>" [--instance <id>] [--node <node>]`: Injects instruction prompts by resolving target sequence (`P001`, `C001`, `GM:#1`, or repo slug).

### 4. Prompt Backup, Recovery & Rerun
- `agm which-prompts-running` (`wpr`): Scans `workspaceStorage` and `repo_prompts.db` for active tasks.
- `agm backup-running-prompts` (`brp`): Backs up active prompts and multimodal images before account switches.
- `agm restore-running-prompts` (`rrp`): Restores backed-up prompts to active state.
- `agm rerun prompts <N>`: Reruns last N prompts in ascending order, pulling git upstream first.
- `agm finish-prompts-until-green` (`fpug`): Waits for running tasks to complete green.
- `agm shutdown-until-green` (`sug`): Waits for tasks to complete green, then initiates system shutdown.

### 5. Format Inspection & Bulk Operations
- `agm which-format <files...>`: Two-tier JSON envelope inspector classifying files into canonical schemas (`agm/accounts-export`, `agm/supabase-endpoints`, etc.) and generating single-line copy-pasteable import commands.

## Key Invariants & Rules

1. **Headless Early Interception**: Administrative commands (`delegate-update`, `open-ui`, `update`, `which-format`) must execute before initializing any graphical or windowing subsystem.
2. **Deterministic Dual Bracket Notation**: Tree output must strictly conform to `[AGM:P001 | GM:#1]` and `[AGM:C001 | GM:<cid>]`.
3. **Machine-Readable Contract**: Any command supporting `--json` must emit valid JSON matching its respective serde DTO to stdout.
4. **Default Instance Immunity**: The default profile (`id == "default"`) can never be removed or unbound.

## Verification Checklist

- [ ] Command compiles cleanly in `src-tauri/src/bin/agm.rs`.
- [ ] Added CLI flags support both long form (`--json`) and short form (`-j`).
- [ ] Fast-path subcommands bypass GUI initialization.
- [ ] Error messages return non-zero exit codes (`std::process::exit(1)`).
