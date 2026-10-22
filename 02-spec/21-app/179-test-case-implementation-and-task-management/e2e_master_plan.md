# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 179)

- **Slug**: `179-test-case-implementation-and-task-management`
- **Sequence**: 179
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 179. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-179-01: Proxy Pool Candidate Eligibility & Dedicated IP Isolation
- **Category**: Egress Proxy Pool Routing
- **Pre-Conditions**: Proxy pool configured with enabled, disabled, healthy, and dedicated-bound proxies.
- **Stimulus**:
  1. Evaluate shared pool selection using `is_proxy_eligible_for_shared_pool`.
  2. Evaluate dedicated proxy binding using `is_bound_proxy_eligible`.
- **Expected Outcome**:
  - Bound proxy entries are strictly excluded from shared pool rotation to protect dedicated IP integrity.
  - Disabled or unhealthy proxies with `auto_failover` evaluate to `false`.

### E2E-179-02: Email Inbound IMAP Telemetry Machine Filtering & Active Event Window
- **Category**: Email Inbound Synchronization
- **Pre-Conditions**: Telemetry messages containing machine name, IP, and lease expiration timestamps.
- **Stimulus**:
  1. Parse messages using `is_telemetry_from_current_machine` and `is_telemetry_event_active`.
- **Expected Outcome**:
  - Messages from current host machine are skipped via early continue guard clauses.
  - Active leases from sibling VMs within lookback window are extracted and excluded from account rotation.

### E2E-179-03: Multi-Instance Crash Loop Exponential Backoff Delay Enforcement
- **Category**: Process Lifecycle & Crash Prevention
- **Pre-Conditions**: Simulated child IDE instance exhibiting rapid exit codes within 10s of launch.
- **Stimulus**:
  1. Trigger successive launch and immediate termination cycles via `CrashLoopBackoffGuard`.
  2. Inspect computed delay using `required_backoff_delay`.
- **Expected Outcome**:
  - Delays escalate exponentially (2s, 4s, 8s, 16s) up to 30s ceiling.
  - Long healthy execution (>10s) resets consecutive crash counter to 0.

### E2E-179-04: Multi-Instance Rapid Crash Quarantine Gate & Auto-Expiration
- **Category**: Multi-Instance Resilience
- **Pre-Conditions**: Instance profile experiencing 3 rapid crash terminations (<5s).
- **Stimulus**:
  1. Call `CrashLoopGuard::record_termination` 3 times.
  2. Check `is_quarantined`.
- **Expected Outcome**:
  - Instance enters quarantine state immediately on 3rd failure.
  - Relaunch attempts are blocked until 120s quarantine window expires or manual reset is triggered.

### E2E-179-05: Multi-Instance Child Process Health Heartbeat Monitor & Stall Detection
- **Category**: Child Process Health Monitoring
- **Pre-Conditions**: Registered IDE instance running in background.
- **Stimulus**:
  1. Record heartbeat pulse via `ChildHealthMonitor::record_heartbeat`.
  2. Advance time past 45-second silence threshold without sending pulses.
- **Expected Outcome**:
  - Process transitions to stalled status (`is_stalled() == true`).
  - Arrival of fresh pulse immediately restores active health.

### E2E-179-06: Reverse Proxy Dynamic Route Balancer Weight Degradation & Recovery
- **Category**: Upstream Route Optimization
- **Pre-Conditions**: Multiple upstream endpoints registered in `DynamicRouteBalancer`.
- **Stimulus**:
  1. Record connection failures on route A and successes on route B.
  2. Verify weight adjustments via `get_weight`.
- **Expected Outcome**:
  - Failing route halves its weight (100 -> 50 -> 25) down to minimum floor 5.
  - Successes gradually restore weight toward base (100) via exponential recovery.

### E2E-179-07: Reverse Proxy Upstream TLS SNI Sanitization & IDN/IPv6 Literal Validation
- **Category**: TLS Protocol Security
- **Pre-Conditions**: Raw upstream target strings with mixed schemes, port suffixes, and IDN labels.
- **Stimulus**:
  1. Sanitize host strings using `sanitize_tls_sni`.
- **Expected Outcome**:
  - Schemes (`https://`) and path fragments stripped cleanly.
  - Explicit ports extracted into `Some(u16)`.
  - IPv6 literals stripped of brackets with `is_ip_literal == true`.
  - Punycode domains flagged with `is_idn == true`.

### E2E-179-08: Auto-Switcher Quota Reset Epoch Drift Compensation & Safety Buffering
- **Category**: Quota & Candidate Scoring
- **Pre-Conditions**: Upstream API providing server reset timestamps with clock skew.
- **Stimulus**:
  1. Record server timestamps in `EpochDriftCompensator`.
  2. Query `effective_reset_epoch` and `has_quota_reset_elapsed`.
- **Expected Outcome**:
  - Applies 30-second intentional safety buffer to prevent premature switching.
  - Compensates for server clock skew via smoothed drift estimation clamped to 300s.

### E2E-179-09: Split-SQLite Automatic Index Fragmentation Scoring & Maintenance Recommender
- **Category**: Split-SQLite Architecture
- **Pre-Conditions**: SQLite B-Tree indices with varying page fullness and page order.
- **Stimulus**:
  1. Evaluate indices using `IndexFragmentationScorer::score_index`.
- **Expected Outcome**:
  - Index with < 15% fragmentation returns `IndexMaintenanceRecommendation::None`.
  - Index with 15%..30% fragmentation recommends `AnalyzeOnly`.
  - Index with >= 30% fragmentation recommends `ReindexRequired`.

### E2E-179-10: End-to-End Turn Sequence Verification & Backlog Step Inventory Recalculation
- **Category**: Orchestration & Backlog Invariant
- **Pre-Conditions**: Successful completion of Steps 1 through 9.
- **Stimulus**:
  1. Verify zero build artifacts generated (`target/` untouched).
  2. Verify all subtasks marked completed in `.ai-memory/plans/subtasks/179-test-case-implementation-and-task-management/`.
  3. Verify parent plan moved to `.ai-memory/plans/completed/`.
- **Expected Outcome**:
  - Repository remains completely clean.
  - Backlog verified at 205 remaining steps across Tracks A through E.
