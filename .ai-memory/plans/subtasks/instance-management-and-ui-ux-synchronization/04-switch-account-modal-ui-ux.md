# Subtask 04: Switch Account Modal Modern UI/UX

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `04-switch-account-modal-ui-ux`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Elevate the Switch Account modal to conform with the application's modern dark-glass UI/UX guidelines: eliminate mismatched controls, implement segmented capsule controls, clear account cards, accessible keyboard escape/focus management, and responsive dialog layouts.

## Scope of Changes
- Audit current switch account modal structure and identify outdated styling.
- Align backdrop with `backdrop-blur-md bg-zinc-950/80` and dark-glass border treatment.
- Render accounts as interactive segmented cards showing email, quota gauge, and status pill.
- Provide smooth feedback during the switch sequence (stopping old instance -> updating credentials -> ready).
- Ensure modal dismisses properly via Escape key, Backdrop click, and explicit close button.

## Verification
- Visually cohesive with main Antigravity-Manager theme.
- Responsive, accessible, and clean interaction states.
