# Subtask 03: Multi-Tier Credit Threshold Polling Ladder & Cluster Auto-Switch

Traceability ID: Task-03
Target Files:
- src-tauri/src/modules/auto_switcher.rs
- src-tauri/src/commands/auto_switcher.rs
- src/components/settings/AutoSwitcherSettings.tsx
- src/services/autoSwitcherService.ts

Action:
- Implement multi-tier dynamic polling interval in `auto_switcher.rs`:
  - Default: Configurable standard interval (e.g. 15–30m).
  - Caution Tier (credits < 20%): Accelerate checks to 3 minutes (configurable via UI slider).
  - Critical Tier (credits <= 12%): Accelerate checks to 1 minute.
- Auto-Switch Trigger: Once remaining quota reaches <= 12%, immediately trigger auto-switching to the best candidate:
  - If Supabase cross-node sync is active: query cluster instances across all nodes, filter out actively leased instances, and choose the node/instance with highest credits.
  - If standalone/local: switch to the local instance with highest credit balance.
- Add UI slider and number input in `AutoSwitcherSettings.tsx` to configure Caution Tier (<20%) check interval (default: 3 minutes) and Critical Tier (<=12%) check interval (default: 1 minute).

Acceptance Criteria:
- Quota drops below 20% dynamically step polling down to 3 minutes.
- Quota drops below or equal to 12% dynamically step polling down to 1 minute and automatically trigger auto fast-forward.
- Cluster nodes with active leases are respected and not overwritten during failover.

Targeted Verification:
- python 03-ai-scripts/05-guideline-autofixer.py src-tauri/src/modules/auto_switcher.rs

Status: COMPLETED
- Implemented multi-tier dynamic polling ladder (`calculate_next_interval_seconds`) in `auto_switcher.rs`.
- Added automatic fast-forward failover rotation when model credits drop <= 12%.
- Configured Caution interval (180s default) and Critical interval (60s default) in `AutoProfileSwitcherConfig`.
- Added UI sliders and auto fast-forward toggle in `AutoSwitcherSettings.tsx`.
