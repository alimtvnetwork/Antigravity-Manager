# Issues

This directory contains active issue tracking and bug reports.

## Open (planned, spec 85)
- [47-pro-badge-missing-when-tier-not-fetched-rca.md](./47-pro-badge-missing-when-tier-not-fetched-rca.md) (cache skip still open; raw-id half is issue 58)
- [48-stack-frame-parser-swallows-url-rca.md](./48-stack-frame-parser-swallows-url-rca.md)
- [49-update-announced-without-platform-asset-rca.md](./49-update-announced-without-platform-asset-rca.md)
- [50-instance-switch-loses-running-prompt-rca.md](./50-instance-switch-loses-running-prompt-rca.md) (2026-09-30 hypothesis; observed cause is issue 57)
- [51-auto-switch-fallback-and-dual-loop-rca.md](./51-auto-switch-fallback-and-dual-loop-rca.md)
- [52-email-recipient-duplicates-rca.md](./52-email-recipient-duplicates-rca.md)
- [53-vault-and-supabase-scripts-silent-failures-rca.md](./53-vault-and-supabase-scripts-silent-failures-rca.md)
- [54-focus-button-dual-scroll-mechanism-rca.md](./54-focus-button-dual-scroll-mechanism-rca.md)

## Fixed in v4.119.0
- [57-switch-marks-prompt-restored-before-ide-ready-rca.md](./57-switch-marks-prompt-restored-before-ide-ready-rca.md) (code fix `87135787`; live inject not watched)
- [58-pro-badge-raw-tier-id-rca.md](./58-pro-badge-raw-tier-id-rca.md) (fetch-time normalization only; `standard-tier` stays raw; issue 47 still open)

## Resolved & Historical Application Issues
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
