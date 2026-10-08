---
plan: 146-supabase-multi-machine-instances-and-remote-fleet-sync
subtask: "03"
title: Fleet Machines Table UI Component & Instances Page Mounting
domain: frontend/ui
target_files:
  - src/components/instances/FleetMachinesTable.tsx
  - src/pages/Instances.tsx
  - src/services/supabaseService.ts
status: pending
---

# 03 — Fleet Machines Table UI Component & Instances Page Mounting

## 1. Objective
Implement the `<FleetMachinesTable />` React component and mount it at the bottom of `src/pages/Instances.tsx` (below local instances in both Card mode and Table mode) to provide real-time cluster-wide visibility into remote fleet worker machines, their running instances, bound accounts, running prompt counts, and heartbeat statuses.

---

## 2. Detailed Technical Scope

### 2.1 Service Layer Extension (`src/services/supabaseService.ts`)
1. Extend `src/services/supabaseService.ts` with new interfaces:
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
       last_heartbeat_timestamp: number;
       uptime_seconds: number;
       in_flight_prompts_count: number;
       active_instances: FleetInstanceSummary[];
       bound_emails: string[];
   }
   ```
2. Add `getFleetMachines(): Promise<FleetMachineInfo[]>`:
   ```typescript
   async getFleetMachines(): Promise<FleetMachineInfo[]> {
       try {
           return await invoke<FleetMachineInfo[]>('get_fleet_machines');
       } catch (error) {
           console.warn('[SupabaseService] getFleetMachines failed:', error);
           return [];
       }
   }
   ```

### 2.2 Component Implementation (`src/components/instances/FleetMachinesTable.tsx`)
Create `src/components/instances/FleetMachinesTable.tsx` containing:
1. **State Management:**
   - `machines: FleetMachineInfo[]` (filtered list excluding local node).
   - `localNodeId: string` (from `supabaseService.getLocalNodeInfo()`).
   - `isLoading: boolean` (initial skeleton loading).
   - `isRefreshing: boolean` (subsequent background refresh).
   - `isSyncEnabled: boolean` (from `supabaseService.getConfig()`).
   - `showAllEmails: boolean` (global toggle for email masking).
   - `unmaskedRows: Record<string, boolean>` (individual row toggle).
   - `copiedIp: string | null` (tracking copied IP for feedback checkmark).
   - `countdown: number` (15s countdown timer).
2. **Auto-Polling Interval:**
   - Set up 15-second interval timer `setInterval(fetchMachines, 15000)`.
   - Clear timer on component unmount.
   - Pause timer or refresh immediately when user triggers manual refresh button.
3. **Design System & Styling (`AGENTS.md`):**
   - Container: Dark-glass card `bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 rounded-xl overflow-hidden shadow-lg`.
   - Header: Flex container with machine count badge and segmented pill capsule (`rounded-full bg-slate-800/60 dark:bg-slate-900/80 border border-slate-700/60 p-0.5 text-xs flex items-center gap-1`).
   - Table Columns:
     * **Machine & Status:** Node alias + OS subtitle + pulsing green dot (`relative flex h-2.5 w-2.5` with ping effect) if online, slate dot if offline.
     * **IP Address:** Monospace text + copy button (`Copy` / `Check` icon).
     * **Running Instance(s):** Rounded capsule badges (`rounded-full px-2 py-0.5 text-xs bg-blue-500/15 text-blue-400 border border-blue-500/30`).
     * **Bound Account(s):** Monospace masked email (e.g. `u***r@domain.com`) with `Eye` toggle icon.
     * **Prompts:** Emerald badge `⚡ N Running` if `in_flight_prompts_count > 0`, else muted badge `Idle`.
     * **Last Seen:** Monospace relative duration (`formatRelativeHeartbeat`).
4. **Empty and Disabled States:**
   - **Sync Disabled:** Dark-glass callout banner informing that Supabase sync is disabled with link to Settings.
   - **Zero Remote Nodes:** Dark-glass card informing that this local machine is currently the sole node in the cluster.

### 2.3 Mounting in `src/pages/Instances.tsx`
1. Import `FleetMachinesTable` in `src/pages/Instances.tsx`:
   ```typescript
   import { FleetMachinesTable } from '../components/instances/FleetMachinesTable';
   ```
2. Mount directly following line 1756 (closing tag of the local instances grid/table):
   ```tsx
                   </div>
               )}

               {/* Remote Fleet Machines Section (Visible in both Card & Table views) */}
               <div className="mt-8 pt-6 border-t border-slate-200/60 dark:border-slate-800/80">
                   <FleetMachinesTable />
               </div>

               {/* Create Instance Modal */}
   ```

---

## 3. Deliverables & Validation Checklist
- [ ] `src/services/supabaseService.ts` exports `FleetMachineInfo` and `getFleetMachines()`.
- [ ] `src/components/instances/FleetMachinesTable.tsx` implemented with complete dark-glass styling and 15s auto-refresh.
- [ ] Mounted in `src/pages/Instances.tsx` beneath local instances in both Card mode and Table mode.
- [ ] Masked emails toggleable individually or globally.
- [ ] IP copy button with 2-second visual feedback.
- [ ] Clean empty state when 0 remote machines are connected.
- [ ] Clean disabled state when Supabase sync is disabled.
