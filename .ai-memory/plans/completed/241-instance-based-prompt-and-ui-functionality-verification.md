# Master Plan: Instance-Based Prompt and UI Functionality Verification

- **Task**: Instance-Based Prompt and UI Functionality Verification
- **Slug**: `instance-based-prompt-and-ui-functionality-verification`
- **Plan ID**: `241`
- **Spec Path**: `02-spec/21-app/241-instance-based-prompt-and-ui-functionality-verification/01-architecture-spec.md`
- **Status**: Completed (Audit & RCA Only per Rule 13)
- **Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)

---

## 1. Context & Objectives

Exhaustive verification of 8 requested capabilities across AGM multi-instance architecture:
1. Instance-based prompt sending and queued prompts visibility.
2. Action buttons: Copy prompt, send prompt, send button UI, enqueue button, and rerun button.
3. Canonical image attachment pattern formatting.
4. Candidate account synchronization without selecting an existing account.
5. Application startup check of `gitmap` machine information.
6. Current machine added to cluster list and confirmed fixed.
7. Auto-switching of instance account and restoring prompt on current conversation.
8. Addition of Next Breakdown / Next 100 Improvements breakdown in instance prompt tree view.
9. **Negative Constraint**: For any functionality verified as missing or incomplete, author 4-part RCA (Why, How, How to complete) and DO NOT modify code.

---

## 2. Completed Subtasks

- [x] [01-subtask-prompt-sending-and-queue-visibility.md](../subtasks/241-instance-based-prompt-and-ui-functionality-verification/01-subtask-prompt-sending-and-queue-visibility.md): Verified instance-based prompt dispatching and queued prompts visibility in Prompt Tree.
- [x] [02-subtask-prompt-action-controls-and-attachments.md](../subtasks/241-instance-based-prompt-and-ui-functionality-verification/02-subtask-prompt-action-controls-and-attachments.md): Confirmed copy prompt, send prompt, send button UI, enqueue button, rerun button, and canonical image attachment pattern.
- [x] [03-subtask-account-selection-synchronization.md](../subtasks/241-instance-based-prompt-and-ui-functionality-verification/03-subtask-account-selection-synchronization.md): Audited synchronization of finding candidate accounts without selecting existing accounts and cluster active lease exclusions.
- [x] [04-subtask-gitmap-machine-and-cluster-fleet.md](../subtasks/241-instance-based-prompt-and-ui-functionality-verification/04-subtask-gitmap-machine-and-cluster-fleet.md): Verified GitMap machine info startup inspection and local machine registration in cluster fleet list.
- [x] [05-subtask-auto-switch-and-prompt-restoration.md](../subtasks/241-instance-based-prompt-and-ui-functionality-verification/05-subtask-auto-switch-and-prompt-restoration.md): Tested and logically confirmed prompt backup, auto-switching, 10s settling delay, and prompt restoration onto current conversation.
- [x] [06-subtask-next-100-breakdown-and-rca-synthesis.md](../subtasks/241-instance-based-prompt-and-ui-functionality-verification/06-subtask-next-100-breakdown-and-rca-synthesis.md): Confirmed Next Breakdown / Next 100 Improvements inside prompt tree view option is MISSING; synthesized complete audit findings and authored 4-part RCA in `02-spec/21-app/241-instance-based-prompt-and-ui-functionality-verification/03-rca-and-gap-analysis.md` and `02-spec/22-app-issues/38-missing-improvements-breakdown-in-prompt-tree-view-rca.md`.
