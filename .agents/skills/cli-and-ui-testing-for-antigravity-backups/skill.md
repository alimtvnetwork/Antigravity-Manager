---
name: cli-and-ui-testing-for-antigravity-backups
description: >-
  Specialized skill for testing and operating Antigravity CLI and UI backup, restore, fast-forwarding, sequence discovery, and multimodal image tracing across AGM and GitMap.
---

# CLI and UI Testing for Antigravity Backups

## Overview

This skill guides automated and interactive operation and verification of running prompt inspection, multimodal backup, and restoration across the Antigravity Manager (`agm`) and GitMap (`gitmap agy`) CLI ecosystems:

1. **Running Prompts & Instances Listing**:
   - `agm running [--json]`: Lists instances, assigned sequence IDs (`[AGM:P001 | GM:#1]`), active and queued prompts under each instance, a bounded 100–200 word preview (configurable via settings), and attached image count.
   - `gitmap agy running [--json]`: GitMap wrapper presenting an aligned, readable summary with sequence IDs and image indicators.
2. **Fast-Forward Delegation (`gitmap agy ff`)**:
   - Delegates instance account rotation and fast-forward switching directly to AGM.
3. **Multimodal Running Prompt Backup**:
   - `agm backup running [instance]`: Backs up running prompts, Base64 images, and disk-referenced image attachments with real-time step tracing.
   - `gitmap agy backup rp` / `gitmap agy backup running-prompts`: GitMap command for running prompt snapshot.
4. **Multimodal Restoration**:
   - `agm restore running [instance]`: Restores prompts into SQLite FIFO queues and seeds `.antigravity_resume_task.<inst>.json` with step-by-step progress logging.
   - `gitmap ag restore rp [instance]`: GitMap restore wrapper.
5. **Configurable Word Preview Limit**:
   - Configurable between 100 and 200 words in GUI settings (`promptPreviewWordLimit`). Defaults to 100 words.
6. **End-to-End Verification**:
   - PowerShell E2E test suite: `scripts/test-agm-gitmap-running-backup-e2e.ps1`.
   - Complete isolation from caller developer session (`default-copy-5597`).
