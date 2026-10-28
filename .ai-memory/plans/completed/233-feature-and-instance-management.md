# Master Plan 233: Feature and Instance Management, Idle Notification Hub, UI Step/Click Trace Logging & CLI Parity

## Status: COMPLETED
**Slug**: `feature-and-instance-management`
**Spec Reference**: `02-spec/21-app/233-feature-and-instance-management/`

## Executive Summary
This task resolved the poor color and contrast of the instance section and switch account buttons, implemented automated idle notifications across both Telegram and Email when the IDE enters idle mode, introduced centralized UI step transition and button trace logging, expanded the AGM CLI to trigger UI clicks on instance elements, and conducted end-to-end testing of instance switching and prompt restoration prior to a minor version bump (`v4.274.0`).

## Modular Subtasks
1. [x] **Subtask 01**: Instance Section UI Polish & Switch Button Styling (`02-spec/21-app/233-feature-and-instance-management/02-component-spec.md`)
   - File: `.ai-memory/plans/subtasks/233-feature-and-instance-management/01-instance-section-colors-and-switch-button.md`
2. [x] **Subtask 02**: Idle Mode Auto-Notification Hub for Telegram and Email (`02-spec/21-app/233-feature-and-instance-management/01-architecture-spec.md`)
   - File: `.ai-memory/plans/subtasks/233-feature-and-instance-management/02-idle-mode-telegram-and-email-auto-notifications.md`
3. [x] **Subtask 03**: UI Step & Click Trace Logging & CLI Click Expansion (`02-spec/21-app/233-feature-and-instance-management/02-component-spec.md`)
   - File: `.ai-memory/plans/subtasks/233-feature-and-instance-management/03-step-and-click-trace-logging-and-cli-click-enhancement.md`
4. [x] **Subtask 04**: Instance Switching & Prompt Restoring E2E Verification (`02-spec/21-app/233-feature-and-instance-management/03-migration-and-verification-spec.md`)
   - File: `.ai-memory/plans/subtasks/233-feature-and-instance-management/04-instance-switching-and-prompt-restoring-verification.md`
5. [x] **Subtask 05**: E2E Quality Gates, Minor Version Bump & Release Ceremony
   - File: `.ai-memory/plans/subtasks/233-feature-and-instance-management/05-e2e-testing-and-release-ceremony.md`

## Invariants
- Caller IDE `v3-7391` preserved.
- Sole attribution to `@aukgit` (`(Thanks to @aukgit)`).
- Strict relative Git paths across all docs and commits.
- Zero errors in `errors.db`.
