# Specification: Multi-Instance Period-Boundary Auto-Switching

Specification ID: `spec-task-08`  
Module: `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/instance.rs`  
Frontend Service: `src/services/instanceService.ts`  
Status: Implemented and Verified  

---

## 1. Requirement Scope

1. **Quota Period-Boundary Lookahead:**
   - Detect quota depletion before the reset period finishes (`is_depleted_before_finish`).
   - Detect period conclusion (`is_period_finished`) to avoid false-positive rotation when upstream credits have refreshed.
   - Parse RFC 3339 `reset_time` into UNIX epoch timestamp.

2. **Multi-Instance Concurrency:**
   - Monitor all running Antigravity IDE instances simultaneously.
   - Evaluate each instance's bound account quota and reset window independently.
   - Enforce collision-free candidate account allocation (`get_active_in_use_account_ids`).
   - Terminate only the target instance's process tree (`instance_processes` SQLite table) during profile rotation, leaving sibling instances completely unaffected.

3. **Multi-Factor Priority Scoring:**
   - Prioritize accounts whose reset period has finished (+15,000 points).
   - Reward accounts with healthy runway before reset time (+runway bonus up to 5,000 points).
   - Balance against idle time, minimum quota baseline, target model percentage, and subscription tier.

---

## 2. Data Structures

### 2.1 Rust Models (`src-tauri/src/modules/auto_switcher.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaPeriodStatus {
    pub quota_percent: f64,
    pub reset_time_iso: Option<String>,
    pub reset_timestamp: Option<i64>,
    pub seconds_until_reset: Option<i64>,
    pub is_period_finished: bool,
    pub is_depleted_before_finish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InstanceQuotaSummary {
    pub instance_id: String,
    pub instance_name: String,
    pub bound_email: Option<String>,
    pub quota_percent: Option<f64>,
    pub reset_time_iso: Option<String>,
    pub seconds_until_reset: Option<i64>,
    pub is_running: bool,
    pub is_depleted_before_finish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitcherStatus {
    pub is_running: bool,
    pub active_instance_id: String,
    pub active_account_email: Option<String>,
    pub current_quota_percent: Option<f64>,
    pub last_check_timestamp: i64,
    pub last_switch_timestamp: Option<i64>,
    pub last_switch_reason: Option<String>,
    #[serde(default)]
    pub monitored_instance_count: usize,
    #[serde(default)]
    pub monitored_instances: Vec<InstanceQuotaSummary>,
}
```

### 2.2 TypeScript Interface (`src/services/instanceService.ts`)

```typescript
export interface InstanceQuotaSummary {
    instance_id: string;
    instance_name: string;
    bound_email?: string;
    quota_percent?: number;
    reset_time_iso?: string;
    seconds_until_reset?: number;
    is_running: boolean;
    is_depleted_before_finish: boolean;
}

export interface AutoSwitcherStatus {
    is_running: boolean;
    active_instance_id: string;
    active_account_email?: string;
    current_quota_percent?: number;
    last_check_timestamp: number;
    last_switch_timestamp?: number;
    last_switch_reason?: string;
    monitored_instance_count?: number;
    monitored_instances?: InstanceQuotaSummary[];
}
```

---

## 3. Behavioral Invariants

1. **No Mixed Polarity:** Condition checks evaluate booleans independently without combining positive and negative checks in the same condition.
2. **No Explicit True Checks:** Boolean expressions use implicit evaluation (`if is_low { ... }`).
3. **Sandbox Independence:** Each instance data directory (`--user-data-dir`) has an isolated `state.vscdb`. Account injection targets strictly that path.
4. **Collision Invariant:** $\forall \text{ inst } \in \text{RunningInstances}, \text{ candidate.account\_id} \ne \text{inst.bound\_account\_id}$.

---

## 4. Verification Matrix

| Test Name | File | Description |
|:---|:---|:---|
| `test_calculate_next_interval_seconds_ladder` | `src-tauri/src/modules/auto_switcher.rs` | Verifies dynamic polling ladder across quota ranges. |
| `test_parse_reset_time_to_unix` | `src-tauri/src/modules/auto_switcher.rs` | Tests RFC 3339 parsing and malformed string handling. |
| `test_evaluate_account_period_status_before_finish` | `src-tauri/src/modules/auto_switcher.rs` | Tests pre-finish depletion flag when quota $\le 10\%$ and reset is in future. |
| `test_evaluate_account_period_status_after_finish` | `src-tauri/src/modules/auto_switcher.rs` | Tests period completion detection when now $\ge$ reset time. |
| `test_score_candidate_account_reset_time_priority` | `src-tauri/src/modules/auto_switcher.rs` | Tests +15,000 score priority for reset-concluded accounts. |

---

## 5. Related Documentation

- Architectural Guide: [docs/multi-instance-period-boundary-auto-switching.md](docs/multi-instance-period-boundary-auto-switching.md)
- Subtask 01: [.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/01-period-boundary-and-reset-lookahead.md](.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/01-period-boundary-and-reset-lookahead.md)
- Subtask 02: [.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/02-multi-instance-switching-engine.md](.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/02-multi-instance-switching-engine.md)
- Subtask 03: [.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/03-comprehensive-unit-tests.md](.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/03-comprehensive-unit-tests.md)
- Subtask 04: [.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/04-architecture-and-operational-docs.md](.ai-memory/plans/subtasks/60-multi-instance-period-boundary-auto-switching/04-architecture-and-operational-docs.md)
