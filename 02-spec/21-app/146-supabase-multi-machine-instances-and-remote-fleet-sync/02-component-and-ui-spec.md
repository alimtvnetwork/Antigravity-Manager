# Component & UI Specification: Fleet Machines Table in Instances View

**Task Slug:** `146-supabase-multi-machine-instances-and-remote-fleet-sync`  
**Spec Document:** `02-component-and-ui-spec.md`  
**Status:** `APPROVED_SPEC`  
**Component Path:** `src/components/instances/FleetMachinesTable.tsx`  
**Mount Target:** `src/pages/Instances.tsx` (immediately following line 1756)  

---

## 1. Executive Summary & Design Goals

This document specifies the user interface and frontend component architecture for the **Fleet Machines Table**, mounted at the bottom of the **Instances** view in Antigravity-Manager. 

When Supabase synchronization is configured across multiple machines (e.g., development workstations, headless cloud workers, secondary laptops), users need instant visual awareness of the entire fleet:
1. Which remote machines are actively connected to the shared Supabase cluster.
2. The IP address of each worker node for connectivity troubleshooting.
3. Which profiles and instances are actively running on those remote machines.
4. What account emails are leased/bound to those remote instances to prevent duplicate usage collisions.
5. How many live prompts are executing concurrently on each remote machine.
6. The freshness of each machine's heartbeat (online status indicator).

### Core Design Invariants (from `AGENTS.md`)
- **Dual-Mode Parity:** The Fleet Machines Table must appear beneath local instances in **both** Card mode and Table mode.
- **Read-Only Inspection:** Remote fleet entries are informational only. Users cannot launch or kill remote instances or inspect remote prompts from this table.
- **Dark-Glass Styling:** Premium dark-glass aesthetic with subtle borders (`border-slate-800/80`), backdrop blur (`backdrop-blur-md`), and non-intrusive muted palettes.
- **Segmented Capsule Controls:** Contiguous segmented pill capsules (`rounded-full`, shared border, subtle dividers) for toolbar actions (e.g., auto-refresh indicator, manual refresh button, email mask toggle).
- **Masked Sensitive Data:** Account emails must be masked by default (e.g., `a***e@example.com`) with a one-click toggle to unmask for privacy during screensharing or video recordings.

---

## 2. Visual Layout & ASCII Mockups

### 2.1 Placement within `src/pages/Instances.tsx`

```
+-----------------------------------------------------------------------------------+
|  Header Bar: [Search Profiles...] [Filter] [Cards/Table Toggle] [Create Profile]  |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  LOCAL INSTANCES SECTION                                                          |
|  -----------------------                                                          |
|  [ Card Mode: 4-Column Grid ]  OR  [ Table Mode: Local Instances Compact Rows ]   |
|                                                                                   |
|  (Line 1756 in Instances.tsx closes local instances view)                         |
|                                                                                   |
+===================================================================================+
|  FLEET MACHINES / REMOTE CLUSTER SECTION (<FleetMachinesTable />)                 |
|  ----------------------------------------------------------------                 |
|  [Section Header: 🖥️ Remote Fleet Machines (3 Nodes) | ⚡ Supabase Connected]     |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  | Machine / Status | IP Address | Active Instance(s) | Bound Email(s) | Prompts| |
|  |------------------+------------+--------------------+----------------+--------| |
|  | 🟢 worker-1-mac  | 192.168.1.4| [Profile-Beta]     | a***e@work.com | 2 Run  | |
|  | 🟢 cloud-runner-2| 10.0.0.12  | [Worker-Headless]  | b***9@corp.ai  | 1 Run  | |
|  | ⚪ studio-pc-win | 192.168.1.9| [Default]          | c***1@fast.com | Idle   | |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

### 2.2 Detailed Fleet Machines Table Layout (ASCII)

```
+------------------------------------------------------------------------------------------------------------------+
|  🖥️  Remote Fleet Machines   [3 Connected]                                         (⟳ 15s)  [👁️ Reveal]  [🔄 Refresh] |
+------------------------------------------------------------------------------------------------------------------+
| Machine Name & Status | IP Address    | Running Instance(s)   | Bound Account(s)        | Prompts  | Last Seen   |
+-----------------------+---------------+-----------------------+-------------------------+----------+-------------+
| 🟢 worker-1-linux     | 192.168.1.45  | (● Primary-Agent)     | d***n@anthropic.com 👁️  | ⚡ 2 Run  | 14s ago     |
|    Ubuntu 22.04 LTS   | [📋 Copy]     |                       |                         |          |             |
|-----------------------+---------------+-----------------------+-------------------------+----------+-------------+
| 🟢 m3-max-runner      | 192.168.1.88  | (● Benchmark-Run)     | a***k@openai.com    👁️  | ⚡ 1 Run  | 28s ago     |
|    macOS 15.1.0       | [📋 Copy]     | (● Staging-Test)      |                         |          |             |
|-----------------------+---------------+-----------------------+-------------------------+----------+-------------+
| ⚪ devbox-secondary   | 10.14.0.2     | -                     | -                       | ⏸️ Idle   | 4m ago      |
|    Windows 11 Headless| [📋 Copy]     |                       |                         |          |             |
+------------------------------------------------------------------------------------------------------------------+
```

---

## 3. Table Column Specifications

| # | Column Header | Content & Semantics | Visual Styling & Badges | Interactive Behaviors |
| :--- | :--- | :--- | :--- | :--- |
| **1** | **Machine & Status** | Remote node alias (e.g. `worker-1`, `m3-runner`) and OS platform subtitle. | Emerald pulsing dot (`bg-emerald-400 animate-ping` + `bg-emerald-500`) when heartbeat < 60s.<br>Slate muted dot (`bg-slate-400`) when offline (> 60s). | Hover displays full machine ID (UUID) and agent version. |
| **2** | **IP Address** | Machine LAN/WAN IP (e.g. `192.168.1.45`). | Monospace font (`font-mono text-xs text-slate-300`). | Copy button with clipboard icon (`Copy`); switches to emerald `Check` icon for 2.0s upon click. |
| **3** | **Active Instance(s)**| Profiles currently running on that remote node. | Rounded capsule badges (`rounded-full px-2 py-0.5 text-xs bg-blue-500/15 text-blue-400 border border-blue-500/30`). | If multiple instances, flex-wrap container with subtle spacing. If none, muted dash (`-`). |
| **4** | **Bound Account(s)**  | Account emails leased by active profiles on this machine. | Monospace masked text (`a***k@domain.com`). Subtle border badge. | Click-to-unmask icon (`Eye` / `EyeOff`) toggles mask state per row. |
| **5** | **Running Prompts**   | Real-time prompt count reported by node heartbeat. | Count > 0: `⚡ N Running` with emerald pulse badge (`bg-emerald-500/15 text-emerald-400 border border-emerald-500/30`).<br>Count == 0: `Idle` muted badge (`bg-slate-800 text-slate-400`). | Tooltip: "In-flight agent requests currently executing on this machine". |
| **6** | **Last Seen**         | Relative elapsed time since last heartbeat. | Text (`text-xs text-slate-400 font-mono`). E.g. `12s ago`, `45s ago`, `3m ago`. | Tooltip displays absolute UTC timestamp (e.g. `2026-10-08 19:35:12 UTC`). |

---

## 4. TypeScript Interfaces & Data Contract

### 4.1 Interface Definitions (`src/services/supabaseService.ts`)

```typescript
export interface FleetInstanceSummary {
    profile_id: string;
    profile_name: string;
    is_running: boolean;
    bound_account_id?: string;
    bound_account_email?: string;
}

export interface FleetMachineInfo {
    node_id: string;
    node_alias: string;
    os_info?: string;
    ip_address: string;
    is_online: boolean;
    last_heartbeat_timestamp: number; // Unix seconds
    uptime_seconds: number;
    in_flight_prompts_count: number;
    active_instances: FleetInstanceSummary[];
    bound_emails: string[];
}
```

### 4.2 Helper Utilities

```typescript
/**
 * Masks an email for privacy (e.g., "developer@antigravity.ai" -> "d***r@antigravity.ai")
 */
export function maskEmail(email: string): string {
    if (!email || !email.includes('@')) return email || '-';
    const [local, domain] = email.split('@');
    if (local.length <= 2) {
        return `${local[0]}***@${domain}`;
    }
    return `${local[0]}***${local[local.length - 1]}@${domain}`;
}

/**
 * Formats a relative timestamp from a unix epoch in seconds.
 */
export function formatRelativeHeartbeat(timestampSecs: number): string {
    if (!timestampSecs) return 'Never';
    const diffSecs = Math.max(0, Math.floor(Date.now() / 1000 - timestampSecs));
    if (diffSecs < 10) return 'Just now';
    if (diffSecs < 60) return `${diffSecs}s ago`;
    const diffMins = Math.floor(diffSecs / 60);
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    return `${diffHours}h ago`;
}
```

---

## 5. Component States & Edge Cases

### 5.1 State 1: Supabase Sync Disabled / Not Configured
When Supabase is not configured or `is_sync_enabled === false`, render a graceful non-intrusive banner rather than an empty table:
- Dark-glass callout box (`bg-slate-900/40 border border-slate-800/80 rounded-xl p-4`).
- Icon: `CloudOff` in muted sky/slate.
- Title: "Multi-Machine Fleet Sync Inactive".
- Description: "Connect a Supabase database in Settings -> Supabase Sync to monitor remote fleet nodes, running instances, and account locks in real time."
- Action: "Configure Supabase" pill button linking to Settings modal.

### 5.2 State 2: Active Sync with Zero Remote Nodes (Standalone Machine)
When Supabase sync is enabled, but the current machine is the only node registered in the cluster:
- Subtle dark-glass panel with `Server` icon.
- Message: "No Remote Fleet Machines Detected".
- Subtitle: "This machine (`<local_node_alias>`) is currently the sole active node connected to the cluster. When secondary workers connect using the same Supabase database, they will appear here automatically."

### 5.3 State 3: Active Remote Nodes Present
- Renders the complete, responsive Fleet Machines Table.
- Filter out local machine node (`node_id === localNodeId`) so only remote workers are displayed in this section, preventing duplicate representation of the current machine.

### 5.4 State 4: Polling & Auto-Refresh Cycle
- Automatically queries `supabaseService.getFleetMachines()` every 15 seconds.
- Displays a subtle pulsing dot and countdown indicator in the table header capsule (`(⟳ 15s)`).
- Provides a manual `Refresh` button in the segmented pill capsule with spinning icon animation during active fetch.

---

## 6. Integration Points with `src/pages/Instances.tsx`

### 6.1 Placement
In `src/pages/Instances.tsx`:
```tsx
                </div>
            )}

            {/* Remote Fleet Machines Section */}
            <div className="mt-8 pt-6 border-t border-slate-200/60 dark:border-slate-800/80">
                <FleetMachinesTable />
            </div>

            {/* Create Instance Modal */}
            {isCreateOpen && (
```

### 6.2 Visual Cohesion
- Both Card mode and Table mode in `Instances.tsx` render `<FleetMachinesTable />` seamlessly.
- Inherits current application dark mode and theme tokens (`bg-base-100`, `dark:bg-base-200`, `border-slate-800`).
- Consistent 5–6px border-radius standard for nested badges, and `rounded-full` for segmented header capsules per `AGENTS.md`.

---

## 7. Verification & Acceptance Criteria

1. **Rendering:** `<FleetMachinesTable />` mounts cleanly below local instances in both view modes (`card` and `table`).
2. **Column Completeness:** All 6 columns (Machine/Status, IP, Running Instances, Bound Emails, Prompts, Last Seen) render cleanly without horizontal scrollbar defects.
3. **Data Masking:** Bound emails are masked by default; clicking toggle reveals or conceals them.
4. **Copy Action:** Clicking IP copy icon copies IP to system clipboard and provides 2-second visual confirmation.
5. **Prompt Badge:** Running prompts show emerald pulse (`⚡ N Running`) if > 0, otherwise muted `Idle`.
6. **Graceful Fallback:** Disabling Supabase or disconnecting network gracefully displays dark-glass notice without throwing unhandled UI errors.
