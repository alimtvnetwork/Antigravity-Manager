# Master Plan: Sequence and Backup Testing

## Plan Information
- **Title**: Sequence and Backup Testing (Prompt Tree Sorting, Standalone Image Pipeline, Backup/Restore Verification, 2-Machine SSH Discovery)
- **Slug**: sequence-and-backup-testing
- **Author**: @aukgit
- **Created**: 2026-10-19
- **Status**: Completed (All 5 Subtasks Verified 100% Green)
- **Specification**: `02-spec/21-app/sequence-and-backup-testing/spec.md`

---

## Strategic Goals & Deliverables

1. **Prompt Tree View Hierarchy**:
   - Verify strict 3-tier sequence: `RUNNING` (Tier 1, top) -> `QUEUED` (Tier 2, middle, amber badge) -> Completed/Historical (Tier 3, timestamp descending).
   - Confirm active conversation and project sorting order.
2. **Standalone Image Copy, Import, and Export**:
   - Test binary PNG Blob generation from Base64 for clipboard copying without canvas tainting.
   - Verify v2.0 JSON envelope export containing standalone embedded base64 image data.
   - Verify clean re-import of prompt bundles with standalone image assets.
3. **Prompt Backup and Restore Subsystem**:
   - Seed test prompts into the prompt queue with screenshot IDs.
   - Run prompt backup (`agm prompt backup` / UI Export) to create immutable snapshots.
   - Run prompt restoration (`agm prompt restore` / UI Import) to reconstitute prompts into the active database in strict FIFO order.
   - Verify that screenshot IDs and prompt text are fully preserved post-restore.
4. **Two-Machine SSH Cluster Connectivity & Account-Instance Mapping**:
   - Verify SSH connectivity to local machine (`w1`) and remote machine (`w2`).
   - Query running instances on both machines.
   - Verify the exact bound Google account email for every instance.

---

## Subtasks Breakdown

- [x] **Subtask 1**: Verify Prompt Tree View 3-Tier Sequence (`02-spec/21-app/sequence-and-backup-testing/subtask-1.md`)
  - Subtask Plan: `.ai-memory/plans/subtasks/sequence-and-backup-testing/subtask-1-tree-sequence.md`
- [x] **Subtask 2**: Verify Standalone Image Pipeline: Copy, Export, Import (`02-spec/21-app/sequence-and-backup-testing/subtask-2.md`)
  - Subtask Plan: `.ai-memory/plans/subtasks/sequence-and-backup-testing/subtask-2-image-pipeline.md`
- [x] **Subtask 3**: End-to-End Prompt Backup & Restore Testing with Screenshot IDs (`02-spec/21-app/sequence-and-backup-testing/subtask-3.md`)
  - Subtask Plan: `.ai-memory/plans/subtasks/sequence-and-backup-testing/subtask-3-backup-restore.md`
- [x] **Subtask 4**: Two-Machine SSH Connectivity & Instance-Account Binding Verification (`02-spec/21-app/sequence-and-backup-testing/subtask-4.md`)
  - Subtask Plan: `.ai-memory/plans/subtasks/sequence-and-backup-testing/subtask-4-ssh-instances.md`
- [x] **Subtask 5**: Pre-flight Quality Gates & Final Evidence Audit (`02-spec/21-app/sequence-and-backup-testing/subtask-5.md`)
  - Subtask Plan: `.ai-memory/plans/subtasks/sequence-and-backup-testing/subtask-5-quality-gates.md`

---

## Non-Negotiable Invariants & Guardrails
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Git Paths**: Zero absolute filesystem paths (`D:\...`, `C:\...`).
- **Developer Session Immunity**: Never terminate or disrupt PID 10076 (`default-copy-5597`).
- **Strictly Attribute @aukgit**: All release notes and changelogs attribute only `@aukgit`.
