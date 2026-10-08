# Component & User Interface Specification: Multi-Machine Fleet Observational Table & Instances Page Integration

- **Specification Slug:** `02-supabase-fleet-machines-instances-sync`
- **Specification Path:** `02-spec/21-app/02-supabase-fleet-machines-instances-sync/02-component-spec.md`
- **Target Release Version:** `v4.155.0` (Minor Bump from `v4.154.0`)
- **Author:** Spec Author 02 (Frontend Component Architecture Working Group)
- **Status:** APPROVED & MANDATED

---

## 1. Executive Summary & Component Hierarchy

### 1.1 Purpose & User Requirements
Antigravity Manager manages isolated desktop IDE instances across local and remote nodes. When connected to a shared Supabase database (configured via `repo-secrets` or user settings), operators need comprehensive visibility into the entire multi-machine cluster:
1. Which machines are active in the fleet (`Worker 1 (Local)`, `Worker 2 (Remote Node-83d705)`, etc.).
2. What instances and profiles are hosted on each machine.
3. What accounts and email addresses are bound to those instances (with privacy masking by default and 1-click unmasking).
4. How many agent prompts are actively running on each instance (with high-visibility pulsing cyan badges).
5. Active workspace lease expiration, IP addresses (with 1-click copy), and node heartbeat health.
6. **Strict Read-Only Safety**: The fleet view is purely observational; remote instance launch, termination, or modification controls are strictly omitted to eliminate cross-machine race conditions and accidental process termination.

### 1.2 UI Component Placement in `src/pages/Instances.tsx`
To preserve the primary local workflow while granting continuous fleet awareness, the new `<FleetMachinesTable />` component is mounted directly at **line 1756** of `src/pages/Instances.tsx`:

```text
src/pages/Instances.tsx
├── Header & Toolbar (Search, Auto-Switch Capsule, View Mode Toggle, New Instance Button)
├── Local Instances Section
│   ├── If viewMode === 'table': <InstancesTable />
│   └── Else (viewMode === 'card'): Card Grid (div.grid > filteredInstances.map(...))
│       └── Line 1756: Closing conditional brace `)}`
│
├── [MOUNT TARGET: Line 1756] <FleetMachinesTable />
│   └── Observational cluster table rendered uniformly below both Card and Table views
│
└── Modals & Overlays (CreateModal, CloneModal, SettingsModal, SwitchModal, AuditModal)
```

By mounting `<FleetMachinesTable />` immediately after line 1756 (below the local instances conditional block):
- Local instance cards and tables remain prominently positioned at the top of the viewport.
- The fleet table renders uniformly at the bottom regardless of whether the user selects Card Mode or Table Mode.
- Operators can inspect both local and remote nodes without toggling distinct tabs or leaving the Instances workspace.

---

## 2. Design System & Visual Styling Specifications

Conforming to Specification 141 and AGM UI Guidelines, all elements in `<FleetMachinesTable />` follow strict visual constraints:

### 2.1 Dark-Glass Container & Palette
- **Outer Container**:
  ```tsx
  className="bg-white dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] rounded-[5px] shadow-sm overflow-hidden mt-6 transition-colors duration-200"
  ```
- **Table Header Banner**:
  ```tsx
  className="px-4 py-3 bg-slate-50/70 dark:bg-[#091e30] border-b border-slate-200/80 dark:border-[#15334d] flex items-center justify-between gap-3 flex-wrap"
  ```
- **Table Rows**:
  ```tsx
  className="border-b border-slate-200/70 dark:border-[#15334d]/70 last:border-b-0 hover:bg-slate-50/60 dark:hover:bg-[#0f2d47]/50 transition-colors"
  ```
- **Card / Worker Sub-header**:
  ```tsx
  className="bg-slate-100/50 dark:bg-[#071a27]/60 border-b border-slate-200/60 dark:border-[#15334d]/60 px-4 py-2 flex items-center justify-between text-xs"
  ```

### 2.2 Strict Rounding Standards (5–6px Radius)
In alignment with Specification 141:
- Buttons, input controls, pills, and outer badges MUST use `rounded-[5px]` or `rounded-[6px]`.
- Circular buttons (`rounded-full`) are strictly forbidden for action buttons and containers.
- Circular rounding (`rounded-full`) is strictly reserved for miniature status indicator dots (e.g. `w-2 h-2 rounded-full`).
- Segmented pill capsules use `rounded-[5px] overflow-hidden divide-x divide-slate-200 dark:divide-[#15334d]`.

### 2.3 Compact Density & Monospace Accents
- **Cell Padding**: Compact `px-3 py-2` to maximize informational density.
- **Font Scale**:
  - Primary text: `text-xs` (12px), `font-medium`
  - Secondary badges: `text-[11px]` (11px)
  - Monospace tags (IP, Node ID, Prompt counts): `font-mono text-[10px]`
- **Interactive Micro-Interactions**: Smooth hover effects with `transition-all duration-150 ease-out active:scale-[0.98]`.

---

## 3. Detailed Component Architecture: `<FleetMachinesTable />`

### 3.1 Component Props & State Interface
`src/components/instances/FleetMachinesTable.tsx` is a self-contained, high-performance React component:

```typescript
export interface FleetMachinesTableProps {
    className?: string;
    pollingIntervalMs?: number; // Defaults to 3000ms
}
```

#### Internal Component State:
```typescript
// 1. Telemetry State
const [machines, setMachines] = useState<FleetMachineInfo[]>([]);
const [isLoading, setIsLoading] = useState<boolean>(true);
const [isSyncing, setIsSyncing] = useState<boolean>(false);
const [lastSyncTime, setLastSyncTime] = useState<number | null>(null);

// 2. Privacy & Masking State (Positive Boolean)
const [isMasked, setIsMasked] = useState<boolean>(true); // Default true: mask emails
const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});

// 3. 1-Click Clipboard Feedback
const [copiedIp, setCopiedIp] = useState<string | null>(null);

// 4. Polling & Lifecycle Refs
const isMountedRef = useRef<boolean>(true);
const pollingTimerRef = useRef<NodeJS.Timeout | null>(null);
```

### 3.2 3-Second Polling & Lifecycle Management
1. **Initial Mount**:
   - Executes immediate fetch: `fetchFleetMachines()`.
   - Starts interval timer: `setInterval(fetchFleetMachines, pollingIntervalMs || 3000)`.
2. **Page Visibility Optimization**:
   - Subscribes to `document.addEventListener('visibilitychange', ...)`.
   - When tab/window is hidden (`document.visibilityState === 'hidden'`), polling pauses to prevent unnecessary network and IPC traffic.
   - When tab becomes visible again, triggers an immediate refresh and resumes the 3,000ms timer.
3. **Clean Unmount**:
   - On component teardown, clears `pollingTimerRef.current` and removes the visibility listener to guarantee zero memory leaks.

---

## 4. Worker Grouping & Table Columns Specification

### 4.1 Worker Grouping Order & Naming Rules
Machines are ordered deterministically:
1. **Local Node First**:
   - The local host machine (`is_local === true`) is ALWAYS pinned as `Worker 1 (Local)`.
   - Badge styling: `bg-blue-50 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-blue-800/60 font-semibold px-2 py-0.5 rounded-[5px]`.
2. **Remote Nodes**:
   - Remote machines are sorted deterministically (by `alias` ascending, then `node_id`).
   - Grouping label: `Worker {index + 1} (Remote {alias})` (e.g. `Worker 2 (Remote Node-83d705)`).
   - Badge styling: `bg-slate-100 dark:bg-[#071a27] text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-[#15334d] px-2 py-0.5 rounded-[5px]`.

```mermaid
flowchart TD
    DATA[FleetMachineInfo[] from get_supabase_fleet_machines] --> SORT[Sort Machines Engine]
    SORT --> LOCAL[Worker 1 Local Node pinned at index 0]
    SORT --> REMOTE[Remote Workers sorted deterministically Worker 2, Worker 3...]
    LOCAL --> RENDER[Render Fleet Table Rows / Worker Groups]
    REMOTE --> RENDER
```

### 4.2 Five Column Table Specifications

| Col # | Column Header | Data Content | Visual Indicators & Controls |
| :--- | :--- | :--- | :--- |
| **Col 1** | **Machine & IP** | - Worker Sequence Tag (`Worker 1 (Local)`)<br/>- Node Alias (`Node-83d705`)<br/>- IP Address (`192.168.1.100`)<br/>- Uptime (`Up 4h 12m`) | - Monospace IP badge with 1-click copy action.<br/>- On copy: icon toggles `Copy` -> `Check`, displays `showToast('IP copied', 'info')`.<br/>- Local vs Remote badge indicator. |
| **Col 2** | **Instances / Profiles** | - Instance Profile Names (e.g. `Default`, `Profile-2`)<br/>- Instance Active State (`is_active: bool`)<br/>- Quota Percent (`quota_percent: number`) | - Pill capsules per profile (`rounded-[5px]`).<br/>- Emerald dot `bg-emerald-500` if active/running (`is_active`).<br/>- Slate dot `bg-slate-400` if idle/stopped.<br/>- Quota badge (e.g. `98%`). |
| **Col 3** | **Connected Accounts** | - Bound Account Email (`active_account_email`)<br/>- Active Account ID (`active_account_id`)<br/>- Workspace Lease Status | - Default state: Masked (`maskEmail(email)` -> `au***@gmail.com`).<br/>- Unmasked toggle: Header toggle or per-row eye icon.<br/>- Fallback: `(Unbound)` in muted slate text.<br/>- Lease Lock indicator `KeyRound` when lease held. |
| **Col 4** | **Running Prompts** | - Active Prompt Count per instance (`running_prompts_count`)<br/>- Total Prompts on Machine (`total_running_prompts`) | - If count > 0: **Pulsing Cyan Badge** (`animate-pulse shadow-[0_0_8px_rgba(6,182,212,0.4)] text-cyan-600 dark:text-cyan-300`).<br/>- Displays `⚡ X Prompts Running`.<br/>- If count === 0: Muted badge `Idle (0)`. |
| **Col 5** | **Leases & Heartbeat** | - Last Heartbeat relative time (e.g. `4s ago`)<br/>- Node status (`Online` vs `Stale` vs `Offline`)<br/>- Lease TTL expiration (`lease_expires_at`) | - Emerald dot: Heartbeat < 60s (`Online`).<br/>- Amber dot: Heartbeat 60s–180s (`Stale`).<br/>- Red/Slate dot: Heartbeat > 180s (`Offline`).<br/>- Lease countdown badge (e.g. `TTL: 65s`). |

---

## 5. Visual Specifications for Key Micro-Interactions

### 5.1 Pulsing Cyan Prompt Badge (`running_prompts_count > 0`)
When an instance or machine has active prompts executing:
```tsx
<span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-cyan-50 dark:bg-cyan-950/60 text-cyan-600 dark:text-cyan-300 border border-cyan-300 dark:border-cyan-700/60 shadow-[0_0_8px_rgba(6,182,212,0.35)] animate-pulse">
    <span className="w-1.5 h-1.5 rounded-full bg-cyan-500 animate-ping" />
    <span className="truncate">⚡ {promptCount} {promptCount === 1 ? 'Prompt' : 'Prompts'} Running</span>
</span>
```
When idle (`running_prompts_count === 0`):
```tsx
<span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-mono text-slate-400 dark:text-slate-500 bg-slate-100/60 dark:bg-[#071a27] border border-slate-200/60 dark:border-[#15334d]">
    <span>Idle (0)</span>
</span>
```

### 5.2 1-Click IP Address Copy Action
```tsx
<div className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-mono bg-slate-100/80 dark:bg-[#071a27] text-slate-700 dark:text-slate-300 border border-slate-200/70 dark:border-[#15334d]">
    <span>{machine.ip_address || '127.0.0.1'}</span>
    <button
        type="button"
        onClick={async () => {
            await navigator.clipboard.writeText(machine.ip_address);
            setCopiedIp(machine.ip_address);
            showToast('IP address copied to clipboard', 'info');
            setTimeout(() => setCopiedIp(null), 1500);
        }}
        className="p-0.5 rounded-[4px] hover:bg-slate-200 dark:hover:bg-[#15334d] text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 transition-colors"
        title="Copy IP Address"
    >
        {copiedIp === machine.ip_address ? (
            <Check className="w-3 h-3 text-emerald-500" />
        ) : (
            <Copy className="w-3 h-3" />
        )}
    </button>
</div>
```

### 5.3 Email Masking & Unmasking Toggle
- Privacy by default: Obfuscates emails using `maskEmail(email)` (e.g. `joh***@company.com`).
- Header toolbar switch: Global toggle controlling `isMasked`.
- Row-level toggle: Allows clicking any masked email badge or row eye icon to reveal the specific email without unmasking the entire fleet.

---

## 6. Strict Read-Only Safety Enforcements

> [!CAUTION]
> **Strict Read-Only Constraint**:
> Under NO circumstances should `<FleetMachinesTable />` render launch, terminate, kill, restart, or delete buttons for remote instances. 

### Rationale:
1. **Host-Level Process Ownership**: Antigravity instances require local GUI window hooks, local process tokens, and operating-system-specific filesystem locks. Triggering remote launches or terminations from an observational table would lead to orphan processes and database desynchronization.
2. **Cluster Safety**: In multi-machine setups, remote workers may be actively running critical agent tasks. Accidental clicks on a remote machine row must NEVER interrupt in-flight operations.
3. **Observational Clarification Notice**:
   Render a subtle footer banner at the base of the table:
   ```tsx
   <div className="px-4 py-2 bg-slate-50/50 dark:bg-[#081b2a] border-t border-slate-200/60 dark:border-[#15334d]/60 flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400">
       <span className="flex items-center gap-1.5">
           <Shield className="w-3.5 h-3.5 text-blue-500/80" />
           <span>Observational Only: Remote worker instances are executed on their host machines. Fleet table provides live cluster telemetry without remote process control.</span>
       </span>
       <span className="font-mono text-[10px]">
           {lastSyncTime ? `Last synced: ${new Date(lastSyncTime).toLocaleTimeString()}` : 'Syncing...'}
       </span>
   </div>
   ```

---

## 7. TypeScript Service & API Specification (`src/services/supabaseService.ts`)

### 7.1 Interface Definitions
In `src/services/supabaseService.ts`, declare the exact contract emitted by Tauri IPC:

```typescript
export interface FleetInstanceItem {
    instance_id: string;
    profile_name: string;
    active_account_id: string;
    active_account_email: string;
    is_active: boolean;
    quota_percent: number;
    status: string;
    running_prompts_count: number;
    updated_at: number;
    lease_expires_at?: number | null;
}

export interface FleetMachineInfo {
    node_id: string;
    alias: string;
    ip_address: string;
    uptime_seconds: number;
    last_heartbeat_at: number;
    is_online: boolean;
    is_local: boolean;
    instances: FleetInstanceItem[];
    total_running_prompts: number;
}
```

### 7.2 Service Methods Implementation
Extend `supabaseService` with typed invoke calls:

```typescript
export const supabaseService = {
    // ... existing methods ...

    /**
     * Query live multi-machine fleet telemetry across all registered Supabase nodes.
     * Guaranteed fallback to local machine snapshot if disconnected or unseeded.
     */
    async getFleetMachines(): Promise<FleetMachineInfo[]> {
        try {
            return await invoke<FleetMachineInfo[]>('get_supabase_fleet_machines');
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService:getFleetMachines' });
            return [];
        }
    },

    /**
     * Trigger an immediate manual heartbeat and fleet synchronization push to Supabase.
     */
    async syncNow(): Promise<void> {
        try {
            await invoke('sync_supabase_now');
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService:syncNow' });
            throw error;
        }
    },
};
```

---

## 8. Internationalization (i18n) Specification

Add the following keys to both `src/locales/en.json` and `src/locales/zh.json`:

### 8.1 English (`src/locales/en.json`)
```json
"fleet": {
  "title": "Fleet Machines & Remote Workers",
  "desc": "Observational live telemetry synchronized across fleet nodes via Supabase",
  "sync_now": "Sync Now",
  "syncing": "Syncing Fleet...",
  "mask_emails": "Mask Emails",
  "show_emails": "Show Emails",
  "worker_local": "Worker {{num}} (Local)",
  "worker_remote": "Worker {{num}} (Remote {{alias}})",
  "col_machine": "Machine & IP",
  "col_instances": "Instances / Profiles",
  "col_accounts": "Connected Accounts",
  "col_prompts": "Running Prompts",
  "col_status": "Leases & Heartbeat",
  "prompts_running": "{{count}} Running",
  "prompts_idle": "Idle",
  "unbound": "(Unbound)",
  "online": "Online",
  "stale": "Stale",
  "offline": "Offline",
  "ip_copied": "IP address copied to clipboard",
  "read_only_notice": "Observational Only: Remote worker instances are executed on their host machines. Fleet table provides live cluster telemetry without remote process control.",
  "no_machines": "No fleet machines registered yet. Ensure Supabase sync is enabled."
}
```

### 8.2 Chinese (`src/locales/zh.json`)
```json
"fleet": {
  "title": "集群机器与远程工作节点",
  "desc": "通过 Supabase 实时同步的多机集群观测数据与运行状态",
  "sync_now": "立即同步",
  "syncing": "正在同步集群...",
  "mask_emails": "隐藏邮箱",
  "show_emails": "显示邮箱",
  "worker_local": "工作节点 {{num}} (本机)",
  "worker_remote": "工作节点 {{num}} (远程 {{alias}})",
  "col_machine": "机器与 IP",
  "col_instances": "实例 / 配置文件",
  "col_accounts": "关联账号",
  "col_prompts": "正在执行的 Prompt",
  "col_status": "租约与心跳",
  "prompts_running": "{{count}} 个正在执行",
  "prompts_idle": "空闲",
  "unbound": "(未绑定账号)",
  "online": "在线",
  "stale": "延迟",
  "offline": "离线",
  "ip_copied": "IP 地址已复制到剪贴板",
  "read_only_notice": "仅供观测：远程工作节点的实例由对应物理机调度执行。集群表格仅提供集群监控，不开放远程控制。",
  "no_machines": "暂无已注册的集群机器。请确认 Supabase 同步已开启。"
}
```

---

## 9. Verification Gates & Acceptance Criteria Matrix

| Verification ID | Verification Scope | Expected Behavior |
| :--- | :--- | :--- |
| **VG-COMP-01** | Line 1756 Mount | `<FleetMachinesTable />` is mounted at line 1756 of `src/pages/Instances.tsx`, rendering below both Card Mode and Table Mode. |
| **VG-COMP-02** | Local Worker Pinning | `is_local === true` is always positioned as `Worker 1 (Local)` at index 0. |
| **VG-COMP-03** | 1-Click IP Copy | Clicking the IP copy icon copies the IP address to clipboard, toggles `Check` icon, and displays toast `showToast`. |
| **VG-COMP-04** | Email Masking Toggle | Emails are masked by default (`maskEmail(email)`), unmasked upon toggling the header switch or row toggle. |
| **VG-COMP-05** | Pulsing Cyan Badge | When `running_prompts_count > 0`, the prompt badge displays a glowing, pulsing cyan badge with active count. When 0, displays muted `Idle (0)`. |
| **VG-COMP-06** | Read-Only Invariance | Zero launch, stop, restart, clone, or delete buttons appear on remote machine rows. Observational disclaimer is rendered at table base. |
| **VG-COMP-07** | Auto-Polling Lifecycle | 3-second auto-polling refreshes fleet data, pauses when tab is hidden, and cleans up timers on component unmount. |
| **VG-COMP-08** | TypeScript & Build Gate | `npm run build` passes with zero type errors, lint violations, or compilation warnings. |
