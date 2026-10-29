# Issues

This directory contains active issue tracking and bug reports.

## Fixed in v4.271.0
- [64-switch-restore-gate-parameter-mismatch-and-status-disconnect-rca.md](./64-switch-restore-gate-parameter-mismatch-and-status-disconnect-rca.md) (solved in `c36538824`: `switch.rs:676` passed `target_inst_id` into `list_prompt_backups` expecting `batch_id`; `'in_progress'` status omitted in queue dispatch; FIFO `pick_first_queued_prompt` enforced; reactive `notify_prompt_queue()` signal wired; unit test suites added)

## Fixed in v4.209.0
- [../rca/17-instance-pid-stop-switch-and-audit-trail.md](../rca/17-instance-pid-stop-switch-and-audit-trail.md) (solved in `52aaf4cb`: the bare number is the PID; stop returned success after dropping that PID; the audit dialog read `recent_switches` while the query returned `records`)

## Fixed in v4.278.0
- [66-instances-switch-json-fake-success.md](./66-instances-switch-json-fake-success.md) (`agm instances switch --json` no longer prints `"switched"` before `switch_account_to_instance`; `bound_email` must match or `success: false`)

## Open (planned, spec 85)
- [47-pro-badge-missing-when-tier-not-fetched-rca.md](./47-pro-badge-missing-when-tier-not-fetched-rca.md) (cache skip still open; raw-id half is issue 58)
- [48-stack-frame-parser-swallows-url-rca.md](./48-stack-frame-parser-swallows-url-rca.md)
- [49-update-announced-without-platform-asset-rca.md](./49-update-announced-without-platform-asset-rca.md)
- [50-instance-switch-loses-running-prompt-rca.md](./50-instance-switch-loses-running-prompt-rca.md) (2026-09-30 hypothesis; observed cause is issue 57)
- [51-auto-switch-fallback-and-dual-loop-rca.md](./51-auto-switch-fallback-and-dual-loop-rca.md)
- [52-email-recipient-duplicates-rca.md](./52-email-recipient-duplicates-rca.md)
- [53-vault-and-supabase-scripts-silent-failures-rca.md](./53-vault-and-supabase-scripts-silent-failures-rca.md)
- [54-focus-button-dual-scroll-mechanism-rca.md](./54-focus-button-dual-scroll-mechanism-rca.md)

## Fixed in v4.124.0
- [62-switch-audit-wrong-key-and-macos-wrapper-pid-rca.md](./62-switch-audit-wrong-key-and-macos-wrapper-pid-rca.md) (solved in `366d84e5`, shipped v4.126.0: default switch audit reads prompts by instance id `"default"`; `is_instance_running` recovers the real PID when the saved macOS `open` wrapper PID is gone)
- [61-running-prompt-resumption-and-conversation-continuity-rca.md](./61-running-prompt-resumption-and-conversation-continuity-rca.md) (preview fallback, schema fallback, `session_id` kept, 120s quota floor)
- [60-switch-drops-live-conversation-rca.md](./60-switch-drops-live-conversation-rca.md) (preview and conversation id kept; resume file no longer drops `session_id`; quota floor 2 minutes; saved PID trusted)

## Fixed in v4.122.0
- [59-switch-blocks-on-quota-and-mailbox-rca.md](./59-switch-blocks-on-quota-and-mailbox-rca.md) (quota fetch, mailbox poll, and email no longer block the switch command)

## Fixed in v4.119.0
- [57-switch-marks-prompt-restored-before-ide-ready-rca.md](./57-switch-marks-prompt-restored-before-ide-ready-rca.md) (code fix `87135787`; live inject not watched)
- [58-pro-badge-raw-tier-id-rca.md](./58-pro-badge-raw-tier-id-rca.md) (fetch-time normalization only; `standard-tier` stays raw; issue 47 still open)

## Resolved & Historical Application Issues
- [65-anti-gravity-rerun-and-ui-fixes.md](./65-anti-gravity-rerun-and-ui-fixes.md) (resolved 2026-10-27; rerun target ID UUID, sequential enqueue retention, and filter compaction)
- [56-release-fmt-gate-blocked-by-token-workflow-scope.md](./56-release-fmt-gate-blocked-by-token-workflow-scope.md) (resolved 2026-10-01, `6b18ddc4`)
- [55-update-settings-missing-auto-check-rca.md](./55-update-settings-missing-auto-check-rca.md) (fixed 2026-09-30, `c4983761`)
- [01-instance-launch-and-user-isolation-rca.md](./01-instance-launch-and-user-isolation-rca.md)
- [02-instance-launch-and-profile-isolation-rca.md](./02-instance-launch-and-profile-isolation-rca.md)
- [03-installer-update-failure-rca.md](./03-installer-update-failure-rca.md)
- [45-instance-switching-rca.md](./45-instance-switching-rca.md)
- [46-cicd-release-bottlenecks.md](./46-cicd-release-bottlenecks.md)
- [46-email-html-replies-node-identity-smart-rotator-rca.md](./46-email-html-replies-node-identity-smart-rotator-rca.md)

## CI/CD Pipeline Issues & RCAs
For all CI/CD pipeline issues and RCAs (01 through 42), see [.ai-memory/cicd-issues/](../cicd-issues/readme.md) and the comprehensive index in [.ai-memory/cicd-index.md](../cicd-index.md).
