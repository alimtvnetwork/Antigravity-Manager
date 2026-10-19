---
name: sequence-and-backup-testing
description: >-
  Specialized skill for verifying Prompt Tree 3-Tier sequencing (Running top, Queued middle, Completed by timestamp desc),
  standalone image Base64-to-Blob and v2.0 JSON envelope operations, prompt backup and FIFO restore with screenshot ID tokens,
  and two-machine SSH fleet instance-to-account discovery in Antigravity-Manager.
---

# Sequence and Backup Testing Skill

This skill defines the operational protocols, verification procedures, and automated test suites for four core subsystems:

## 1. Prompt Tree 3-Tier Sequencing
- **Tier 1 (Running)**: Actively running conversations are anchored at the very top of each project tree.
- **Tier 2 (Queued)**: Conversations containing enqueued prompt turns are placed immediately beneath running conversations with distinct amber badges.
- **Tier 3 (Completed)**: Completed conversations are sorted beneath queued conversations by `last_modified` timestamp descending.
- **Test Suite**: `src/components/instances/PromptTreeView/__tests__/treeSequence.test.ts`.

## 2. Standalone Image Pipeline
- **Clipboard Copying**: Taint-free Base64-to-Blob conversion directly to the system clipboard as `image/png`.
- **Export**: Packaging prompt turns into v2.0 JSON envelopes using portable content-addressed image references (`agm-asset://img_<sha256>`) and an embedded image dictionary.
- **Import**: Materializing `agm-asset://` references back into the target workspace `.antigravity/attachments/` directory without data loss.
- **Test Suite**: `src/components/instances/PromptTreeView/__tests__/imageStandalonePipeline.test.ts`.

## 3. Prompt Backup & FIFO Restore with Screenshot ID Verification
- **Enqueueing**: Registering queued prompts in `repo_prompts.db` (`active_prompts`).
- **Backup**: Storing snapshots in `backup-prompts.db` (`prompt_backups`) with `has_images = 1` and preserving screenshot ID tokens (`SS_ID_...`).
- **Restore**: Re-injecting backed-up prompts into `active_prompts` in strict First-In-First-Out (`ORDER BY created_at ASC, id ASC`) order.
- **Test Script**: `scripts/test-prompt-backup-restore-e2e.ps1`.

## 4. Two-Machine SSH Cluster & Instance-Account Mapping
- **Node Discovery**: Querying GitMap cluster nodes and `agm ssh nodes` for `w1` (192.168.1.3:22) and `w2` (192.168.1.7:22) without duplicate entries.
- **Local Instance Mapping**: Reporting active running instances on `w1` and their bound Google account emails (`riseup.team.now@gmail.com` and `zihadxshohag2@gmail.com`).
- **Remote Offline Safety**: Ensuring unreachable remote nodes (`w2`) report genuine offline status (0.0% quota) with zero false-100% quota leaks.
- **Test Script**: `scripts/test-ssh-cluster-instances.ps1`.
