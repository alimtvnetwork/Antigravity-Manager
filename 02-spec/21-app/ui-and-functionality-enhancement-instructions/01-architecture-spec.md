# Architectural Specification: Multi-Instance State Isolation & Subsystems

## 1. Multi-Instance Prompt State Isolation Architecture

### 1.1 Root Cause Classification
When two instances of Antigravity or Antigravity-Manager target the same repository or workspace path:
1. `tree_collect.rs` evaluated `not_fully_idle == 0` as a gate for explicit idle status. Antigravity Cascade engine bugs frequently leave `not_fully_idle = 1` even after run completion or cancellation. Consequently, the idle status was ignored and stale running status was assigned.
2. `tree_node_conv.rs` forced `effective_is_run = item.is_running || (is_inst_alive && item.status.contains('RUNNING'))`. This unconditional override caused dead prompts in alive processes to be treated as actively running.
3. In `src/pages/instances/instanceCardData.ts`, `hasActiveTask` evaluated `(node.conversations || []).some(...)` without verifying `c.instance_id === inst.config.id`. When two instances share workspace storage, Instance 1's card mirrored Instance 2's running prompt.

### 1.2 Invariants & Fix Contract
- **Invariant I1 (Explicit Idle Supremacy):** If status contains `IDLE`, `COMPLETED`, `FAILED`, or `CANCELLED`, `is_running` MUST evaluate to `false` regardless of `not_fully_idle` or host process liveness.
- **Invariant I2 (Strict Card-Level Instance Filtering):** In the GUI instance card evaluation (`instanceCardData.ts`), a conversation node only triggers an active prompt badge on an instance card if `c.instance_id === inst.config.id` (or if instance ID is undefined and the card is the primary default instance).
- **Invariant I3 (Enqueued Turn Preservation):** In `tree_node_conv.rs`, active prompts matching an existing `conversation_id` with `status == "queued"` or `"running"` MUST NOT be discarded (`continue`); they must be appended to the target conversation's prompt turn list with `is_queued = true`.

---

## 2. SSH Cluster & Node W2 Parity Architecture

### 2.1 GitMap Parity Protocol
GitMap's Go SSH engine uses a 3-tier dial ladder:
1. Direct Key Authentication: `~/.ssh/id_ed25519`, `id_rsa`, `id_ecdsa`.
2. Windows Key Authorization: Key injection to `%ProgramData%/ssh/administrators_authorized_keys` with strict ACLs (`icacls`).
3. Password Fallback: Using vaulted credentials (`rtyrty123@` for Windows Administrator, `a` for Linux).

### 2.2 Fix Contract for AGM Cluster
- Discover node candidates from `gitmap-ssh-nodes.json`, `04-ssh-nodes.json`, and repo-secrets.
- Unblock the local mesh view in `FleetMachinesTable.tsx` and `useFleetMachines.ts` even when Supabase sync is disabled (`is_sync_enabled == false`).
- Support auto-repair and connection diagnostics against node `w2` (192.168.1.7:22).
