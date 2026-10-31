# Subtask 02: Confirm Prompt Action Controls & Canonical Image Attachments

- **Task**: Instance-Based Prompt and UI Functionality Verification
- **Subtask**: 02 of 06
- **Status**: Completed
- **Path**: `.ai-memory/plans/subtasks/241-instance-based-prompt-and-ui-functionality-verification/02-subtask-prompt-action-controls-and-attachments.md`

## Audit Outcomes
- Verified `DetailHeader.tsx` and `usePromptTreePromptActions.ts`:
  - Copy prompt: Dual-MIME rich image and pure text copying working seamlessly.
  - Send prompt: Immediate send bypass via `onResend` straight to CDP/DB.
  - Send button UI: Segmented split dropdown (`handleToggleDispatchMenu`) with transparent borderless styling and smooth mode transitions.
  - Enqueue button: Persists prompt as `enqueued_ide` without stripping task text.
  - Rerun button: Preserves actual task prompt text via `resolveRerunTemplateBody` instead of internal nudge strings.
- Verified attachment pattern: Canonical 3-tier reference `[Title | Slug | Seq](file;<path>)` and `Attachments:\n1. @[<path>]` enforced and released in v4.282.0 / v4.283.0.
