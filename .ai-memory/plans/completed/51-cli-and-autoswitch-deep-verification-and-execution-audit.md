# 51: CLI & Auto-Switcher Deep Verification & Execution Audit

**Status:** Completed  
**Completed Date:** 2026-09-26  
**Spec Reference:** [`02-spec/21-app/49-cli-and-autoswitch-deep-verification-and-execution-audit.md`](../../../02-spec/21-app/49-cli-and-autoswitch-deep-verification-and-execution-audit.md)

## Summary of Accomplishments & Full Audit

### 1. Complete CLI Verification
- **`agm ff` (`cmd_fast_forward`)**: Evaluates `status_before`, triggers fast-forward rotation, evaluates `status_after`, collects running prompts count, prompt resend status, and image payloads. Supports `--json` emitting a machine-readable JSON state machine with `previous_account`, `predicted_next_account`, `selected_account`, `credit_before_switch`, `current_quota_percent`, `prompts_running`, `prompts_resent`, and `has_images`.
- **`agm status` / `credits` / `status/credits` (`cmd_status`)**: Reports active account tier, immediate & weekly quota percent, threshold target, target model, predicted next account with estimated quota, total instances, running instances, running prompts, prompts resent confirmation (`Yes (Auto-Resumed via .antigravity_resume_task.json)`), and attached images (`Yes (Base64 payload preserved)`). Supports `--json`.
- **`agm switch-if-low-credit` (`swlc` / `sfc`)**: Accepts custom `-t <threshold>` or default, checks quota against threshold, triggers rotation if low or logs healthy state. Supports `-f [path]` writing to `agm-<node_alias>-switch.json` and `--json` format with all 5 telemetric answers (`credit_before_switch`, `threshold_activated`, `previous_account`, `predicted_next_account`, `selected_account`).
- **`agm is-low-credit-for-switch` (`ilc`)**: Returns pure boolean `true`/`false` in standard CLI mode. With `--json`, outputs machine telemetry with machine name, node alias, local IP, active account, and predicted next candidate. Supports `-f` file export.
- **`agm which-prompts-running` (`wpr`)**: Auto-refreshes running projects across all instances, maps conversations and prompt queues, outputs sequential index, project name, project ID, conversation ID, conversation name, and prompt queue count in ASCII table or `--json`.
- **`agm rerun prompts N [-prefix <cat>] [-suffix <cat>]`**: Executes `git pull` pre-run, reads historical prompts in current repo, takes last N, reverses to ASC chronological order, wraps with template prefix/suffix, updates `repo_prompts.db`, and injects `.antigravity_resume_task.json`.
- **`agm prompts ls N --json --words 100`**: Lists running prompts in ASC stack order with word truncation and table or JSON mode.
- **`agm prompts-export` (`pe`) & `agm prompts-import` (`pi`)**: Preserves Base64 image payloads, defaults to `agm-<repo-slug>-prompts.json`, scans for multiple JSON files with interactive choice, and reschedules prompts.
- **`agm clear-cache` variants**: Supports `--keep 10`, `-k 10`, `--keep/k 10`, `cache-clear`, and `clear cache`. Keeps 10 recent conversations, deletes old conversation DBs and brain directories, stages pruned items, and purges temporary application logs.
- **`agm instances`**: Supports `instances ls`, `instances <target> ff`, `instances-all ff`, `instances rm <target>`, `instances create "new-name" --data-only(do)` (skipping executable copy if data-only), and `instances rm-all` (preserving default instance).
- **`agm prompt <text>`**: Pulls git, wraps with prompt templates from `01-prompts/`, saves to `repo_prompts.db`, and writes `.antigravity_resume_task.json`.
- **`agm recreate-project` & `agm recreate`**: Recursively removes `.antigravity_resume_task.json`, purges `workspaceStorage`, prunes conversation DBs and `brain/` folders, seeds bootstrap prompt `"read all files and memory to understand the project"`, and spawns fresh `agy` session.
- **`agm email`**: Full suite including `status` (dispatches HTML status card + `[JSON]` block), `help` (dispatches guide email), `ls` (table or JSON), `add` (dispatches JSON self-email to newly configured address), `rm`, `mv`, `export`, and `import`.

### 2. Auto-Switcher & Invariants Verification
- **Minimum Bottleneck Quota Evaluation**: Considers non-banned models and quota group buckets to identify when any actively consumed model or bucket falls below the threshold.
- **98% Threshold Handling**: Accurately triggers rotation when active quota <= 98%.
- **Fallback Profile Selection**: Fallback pass over available accounts with `quota > 15.0` ensures rotation does not stall when testing with high thresholds.
- **5-Second Reactive Interval**: Starts with an immediate 3s check and checks every 5-second tick, reacting immediately to configuration changes.
- **Telemetry Invariants**:
  - `credit_before_switch`: Quota before switch.
  - `threshold_activated`: Threshold percentage that triggered rotation.
  - `previous_account`: Genuine email, never `"default"`.
  - `predicted_next_account`: Next candidate chosen by scoring.
  - `selected_account`: Confirmed account bound to the instance.
  - `running_prompts_count` & `prompts_resent`: Confirmation of active prompts re-injected via `.antigravity_resume_task.json`.
  - `has_images` / `images_attached`: Confirmation of Base64 image payload preservation.
- **Email Subject Header Standard**:
  `[Antigravity | v{VERSION} | {VM} | {IP}] [Antigravity] [JSON] Account Switched: {previous_email} -> {selected_email}`
