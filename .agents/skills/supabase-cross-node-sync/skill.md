---
name: supabase-cross-node-sync
description: >-
  Orchestrate cross-node instance synchronization, distributed workspace leases, prompt queues, and free-tier auto-pruning via Supabase Split-DB architecture.
---

# Supabase Cross-Node Synchronization & Split-DB Specification

## Architectural Objectives

1. **Split-DB Partitioning on Supabase:**
   - **Root DB (Lightweight State & Metadata):** Machine/node registry, online heartbeats, instance profile status, and distributed workspace/account leases.
   - **Secondary Command DB(s) (High Frequency Queues):** Prompts, commands, execution results, and telemetry injected from Email, Telegram, or cross-node triggers.
2. **500 MB Free-Tier Guard & Auto-Pruning:**
   - Default threshold: 400 MB for Root DB, 200 MB for Secondary DB.
   - Background daemon tracks estimated table sizes / row volumes.
   - Automated FIFO pruning deletes oldest execution logs while preserving active node registries and leases.
3. **Multi-Database Federation & Cascade:**
   - Supports multiple secondary database endpoints.
   - When one database reaches capacity, fails, or is damaged, system cascades automatically to the next configured endpoint with one-click migration of active/critical records.
4. **Collision Prevention via Distributed Leases:**
   - Centralized lease registry in Root DB prevents two independent nodes from claiming or rotating to the same workspace or account concurrently.
5. **Iterative Base64 Security for Config Export/Import:**
   - Format: `<iteration_count>$<base64_encoded_payload>`.
   - Export/import via JSON and YAML without exposing plaintext tokens.
6. **Remote Control & Snapshots:**
   - Global snapshot queries ("How many machines are running?") aggregate online nodes, active instance profiles, and quota metrics.
   - Remote Fast-Forward (`FF`) actions trigger profile workspace rotation across targeted nodes.
