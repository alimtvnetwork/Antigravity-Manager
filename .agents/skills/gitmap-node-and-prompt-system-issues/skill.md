---
name: gitmap-node-and-prompt-system-issues
description: >-
  Specialized skill for diagnosing, repairing, and optimizing GitMap node clustering, SSH machine identification and account/instance mapping, prompt tree actions (send, backup, restore, reset, refresh, sorting, preview deduplication, z-index layering), Supabase error circuit breaker with 5-minute backoff, high-frequency transaction dampening, and Telegram prompt machine resolution.
---

# GitMap Node and Prompt System Issues

## Overview
This skill guides the end-to-end resolution of issues across:
1. **GitMap Node & SSH Cluster Management**:
   - GitHub/cluster node naming and aliasing.
   - Visibility of node account mappings, active instances count, and target machine discovery in SSH section.
   - Reliable SSH connection testing and execution.
2. **Prompt Tree & UX System**:
   - Prompt Tree header and action bar: Send, Backup, Restore, Reset, and Refresh.
   - Screen glitch elimination caused by rapid state updates / re-render loops.
   - Running prompts and Enqueued prompts prioritized at the top of the tree view, badged with `enqueued` or `queued`.
   - Prompt Preview header decluttering: moving redundant info (`User Prompt`, instance IDs) into a dedicated `Details` dialog.
   - Dropdown z-index / stacking context remediation: preventing underlying banners and buttons from bleeding through the Import/Export menu.
3. **Supabase Rate Limiting & Transaction Dampening**:
   - 5-minute error backoff circuit breaker on Supabase connection failures.
   - Reduction of high-frequency polling/transactions across account credit synchronizations and prompt liveness audits (damping interval to >=30s or ~5 minutes).
4. **Telegram Prompt Transmission**:
   - Accurate machine identification and prompt forwarding.
   - Robust send button action without silent drops.
