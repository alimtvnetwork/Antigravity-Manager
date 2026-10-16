# Specification Index: Auto-Switch and UI Functionality Issues

- **Slug**: `auto-switch-and-ui-functionality-issues`
- **Module**: `Auto-Switcher, Quota Sync, Audit Logging, Process Hygiene`
- **Specification Directory**: `02-spec/21-app/auto-switch-and-ui-functionality-issues/`
- **Related Plan**: `.ai-memory/plans/auto-switch-and-ui-functionality-issues.md`
- **Benchmark Branch**: `backup/auto-switch-and-ui-functionality-issues-benchmark`
- **Target Release**: `v4.208.0` (Minor)

---

## Document Index

1. [00-master-audit-ledger.md](file:///02-spec/21-app/auto-switch-and-ui-functionality-issues/00-master-audit-ledger.md)
   - Verbatim user prompt, screenshots evidence, and traceable discrete task tracking.
2. [01-architecture-and-quota-autoswitch-spec.md](file:///02-spec/21-app/auto-switch-and-ui-functionality-issues/01-architecture-and-quota-autoswitch-spec.md)
   - Detailed specification for live quota synchronization, 403 fallback fix without project id, 30s-50s daemon polling, auto-switcher threshold evaluation, manual Fast-Forward unblocking, and prompt backup cap (100).
3. [02-audit-log-windows-and-ui-spec.md](file:///02-spec/21-app/auto-switch-and-ui-functionality-issues/02-audit-log-windows-and-ui-spec.md)
   - Detailed specification for Interaction Audit Log clipboard copy repair (JSON/JSONL), method/target DOM climbing, stack trace capture, auto-switch decision audit logging, Windows `CREATE_NO_WINDOW` (`0x08000000`) subprocess hygiene, strict email masking, and updater polish (18pt font).

---

## Subtasks Mapping

- Subtask 01: `01-quota-sync-and-auto-switch-fix.md`
- Subtask 02: `02-process-window-suppression-create-no-window.md`
- Subtask 03: `03-audit-log-json-copy-and-callstack-enrichment.md`
- Subtask 04: `04-max-backup-prompts-cap-and-settings.md`
- Subtask 05: `05-strict-email-masking-and-updater-polish.md`
- Subtask 06: `06-e2e-verification-bump-and-release.md`
