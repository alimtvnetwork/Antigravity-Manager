# 50: Comprehensive CLI Verification & Auto-Switcher Invariants

**Status:** Completed  
**Completed Date:** 2026-09-26  
**Spec Reference:** [`02-spec/21-app/48-comprehensive-cli-and-autoswitch-verification.md`](../../../02-spec/21-app/48-comprehensive-cli-and-autoswitch-verification.md)

## Summary of Accomplishments
1. **Telemetric Answers Across All Mediums**:
   - **Credit Before Switch**: Explicitly tracked and displayed as `credit_before_switch` in Email subject/cards/JSON, Telegram messages, CLI stdout, and `--json` payloads across `agm switch-if-low-credit`, `agm is-low-credit-for-switch`, `agm status`, and `agm ff`.
   - **Threshold Activated**: Explicitly tracked and displayed as `threshold_activated` across all mediums.
   - **Previous Account vs Predicted Next Account vs Selected Account**: Resolved real previous email (never instance name "default"), predicted next profile, and confirmed selected email across all outputs.
   - **Prompts Running & Resend Status**: Accurately counts running/dispatched prompts, verifies whether prompts were re-injected via `.antigravity_resume_task.json` (`prompts_resent`), and checks for Base64 image payload preservation (`has_images` / `images_attached`).
2. **Subject Standard Parity**:
   - Upgraded subject format across all switch alerts to:  
     `[Antigravity | v{VERSION} | {VM} | {IP}] [Antigravity] [JSON] Account Switched: {previous_email} -> {selected_email}`
3. **CLI Commands Exhaustive Audit & Hardening**:
   - `agm ff` (`cmd_fast_forward`): Full rotation with `--json` option outputting complete telemetry state machine.
   - `agm status` / `credits` / `status/credits`: Displays node alias, IP, account, tier, immediate/weekly credits, threshold target, predicted next profile, running prompts count, prompt resend status, and image payloads in table or `--json`.
   - `agm switch-if-low-credit` (`swlc` / `sfc`): Supports custom thresholds, `-f` export to `agm-<node_alias>-switch.json`, `--json`, and full telemetry answers.
   - `agm is-low-credit-for-switch` (`ilc`): Returns boolean (`true`/`false`) in default mode, or structured JSON with node details, IP, account, version, and next candidate profile when `--json` is supplied (and supports `-f`).
   - `agm which-prompts-running` (`wpr`): Project discovery, 1-based sequence numbering, conv ID, conv name, and prompts count in table or `--json`.
   - `agm rerun prompts N [-prefix <category>]`: Chronological rerun with template prefix/suffix wrapping and `.antigravity_resume_task.json` task generation.
   - `agm prompts ls N --json --words 100`: ASC stack list of active prompts with word preview truncation and table formatting.
   - `agm prompts-export` (`pe`) & `agm prompts-import` (`pi`): Base64 image preservation, default `agm-<repo-slug>-prompts.json` export/import, and multi-file interactive import selection.
   - `agm clear-cache` variants: Supports `--keep/k N` (default 10) across all syntax permutations.
   - `agm instances`: Supports `ls`, `ff`, `instances-all ff`, `rm`, `create --data-only(do)`, and `rm-all`.
   - `agm email status/help/add`: Dispatches HTML card with `[JSON]` block, help guide, or immediate JSON self-test verification email.
   - `agm recreate-project` & `agm recreate`: Deep workspace cleanup (removing `.antigravity_resume_task.json`, workspaceStorage, conversation DBs, and brain directories) and bootstrap prompt registration.

## Subtasks
- [x] `01-cli-commands-audit-and-verification.md` — Verified and audited all CLI commands in `agm.rs`.
- [x] `02-autoswitcher-and-email-selftest-verification.md` — Verified Auto-Switcher threshold evaluation, fallback profile selection, reactive 5s loop, and self-email dispatch on `agm email add`.
