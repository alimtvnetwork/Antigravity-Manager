---
plan: 02-supabase-fleet-machines-instances-sync
subtask: "02"
title: Frontend Fleet Machines Table Component, Supabase Service API & Instances Page Integration
domain: frontend-react-typescript
depends_on:
  - 02-spec/21-app/02-supabase-fleet-machines-instances-sync/01-architecture-spec.md
  - 02-spec/21-app/02-supabase-fleet-machines-instances-sync/02-component-spec.md
citations:
  architecture_spec: 02-spec/21-app/02-supabase-fleet-machines-instances-sync/01-architecture-spec.md
  component_spec: 02-spec/21-app/02-supabase-fleet-machines-instances-sync/02-component-spec.md
  coding_guidelines: 02-spec/02-coding-guidelines/readme.md
target_files:
  - src/services/supabaseService.ts
  - src/components/instances/FleetMachinesTable.tsx
  - src/pages/Instances.tsx
  - src/locales/en.json
  - src/locales/zh.json
status: pending
---

# Subtask 02: Frontend Fleet Machines Table Component, Supabase Service API & Instances Page Integration

## 1. Context & User Requirements

### User Prompt (Verbatim):
> "In the Antigravity Manager, in the instance section, if we have the email or if we have the Superbase connected to the same one, we should be able to see all the machines using the database that what are the other machines are using as the accounts, and it should have all this information. You need to confirm that this is already implemented if the Superbase is already there. And I want you to connect to the Superbase, and you get the Superbase information in the repo secrets. So do a repo pull. I mean, get pull on the repo secrets and try to connect the Superbase credential to this, Antigravity Manager. That's the first thing. Second is that in the instance section, it should show the current instance nicely, and then afterwards, it would show the other machines in a short way. Because in the other machines, we are not going to open the instance, but it should show as a table at the end. Even in the card mode, it would show as a table by grouping like worker one, worker two with the IP and whatever the instance is running. And on those instance, what are the emails are connected and how many prompts are running, if that is possible, because prompts should be able to possible because it should be sending some of the details. We shouldn't be able to opening the prompt, at least for now. Make sure that these are the codes are already there, and that if not, then we can implement it, the UI visualization, and at the end, we can bump the minor version and make a release. Do you understand the requirements? Can you please work on it and make sure that you release it properly?"

### Core Deliverables:
1. **TypeScript Service & IPC Integration (`src/services/supabaseService.ts`)**:
   - Define `FleetInstanceItem` and `FleetMachineInfo` interfaces.
   - Implement `getFleetMachines(): Promise<FleetMachineInfo[]>` invoking `get_supabase_fleet_machines`.
   - Implement `syncNow(): Promise<void>` invoking `sync_supabase_now`.
2. **New React Component (`src/components/instances/FleetMachinesTable.tsx`)**:
   - Worker grouping: Groups machines deterministically as `Worker 1 (Local)`, `Worker 2 (Remote Node-83d705)`, etc.
   - Five dedicated columns:
     1. Machine & IP (with 1-click clipboard copy and toast).
     2. Instances / Profiles (pills with running status dots and quota badges).
     3. Connected Accounts / Emails (masked by default, unmaskable via header switch or row toggle).
     4. Running Prompts (pulsing cyan badge when > 0, else muted Idle).
     5. Leases & Heartbeat (relative heartbeat time and lease TTL).
   - Strict read-only safety: Purely observational; no remote launch, stop, restart, clone, or delete buttons.
   - Visual styling: Dark-glass container (`bg-white dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] rounded-[5px]`), 5–6px button radius, compact density.
   - 3-second auto-polling loop with page visibility listener and clean unmount cleanup.
3. **Mounting Target in `src/pages/Instances.tsx`**:
   - Mount `<FleetMachinesTable />` at line 1756 (directly after the closing bracket `)}` of the local instances view).
   - Ensures uniform bottom rendering across both Card Mode and Table Mode, preserving local instance cards/table at top.
4. **Internationalization (`src/locales/en.json` & `src/locales/zh.json`)**:
   - Complete localized strings for all table headers, actions, tooltips, and badges.

---

## 2. Target Files & Action Matrix

| Target File | Scope of Changes | Responsibility |
| :--- | :--- | :--- |
| `src/services/supabaseService.ts` | Add `FleetInstanceItem`, `FleetMachineInfo` interfaces; add `getFleetMachines()` and `syncNow()` methods. | Frontend Service API |
| `src/components/instances/FleetMachinesTable.tsx` | Create new observational table component with worker grouping, dark-glass theme, 1-click IP copy, masked emails, cyan prompt badges, and 3s polling. | Component Implementation |
| `src/pages/Instances.tsx` | Import `<FleetMachinesTable />` and mount at line 1756 below `{viewMode === 'table' ? (...) : (...)}`. | View Integration |
| `src/locales/en.json` | Add `"fleet"` key with English localization strings. | Internationalization |
| `src/locales/zh.json` | Add `"fleet"` key with Chinese localization strings. | Internationalization |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Extend `src/services/supabaseService.ts`

Add typed interfaces and service methods to `src/services/supabaseService.ts`:

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

Add methods to `export const supabaseService`:

```typescript
    /**
     * Query live multi-machine fleet telemetry across all registered Supabase nodes.
     * Gracefully falls back to local node snapshot if unseeded or disconnected.
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
```

---

### 3.2 Step 2: Implement `src/components/instances/FleetMachinesTable.tsx`

Create `src/components/instances/FleetMachinesTable.tsx` with full support for:
1. Dark-glass container: `bg-white dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] rounded-[5px]`
2. Worker grouping: `Worker 1 (Local)`, `Worker 2 (Remote Node-83d705)`, etc.
3. 1-click IP copy with `showToast` and `Check` icon feedback.
4. Masked account emails using `maskEmail(email)` with header toggle and per-row reveal.
5. Pulsing cyan badge when `running_prompts_count > 0` (`shadow-[0_0_8px_rgba(6,182,212,0.35)] animate-pulse`).
6. Leases & Heartbeat status dots (<60s online, 60-180s stale, >180s offline).
7. Strict read-only safety: Observational only; zero launch/kill buttons.
8. 3-second auto-polling loop with visibility pausing and unmount cleanup.

```tsx
import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
    Server,
    RefreshCw,
    Copy,
    Check,
    Eye,
    EyeOff,
    Clock,
    KeyRound,
    Shield,
    Activity,
    Cpu,
    ExternalLink
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { maskEmail } from '../../utils/maskEmail';
import { supabaseService, type FleetMachineInfo } from '../../services/supabaseService';
import { showToast } from '../common/ToastContainer';

export interface FleetMachinesTableProps {
    className?: string;
    pollingIntervalMs?: number;
}

function formatUptime(seconds: number): string {
    if (!seconds || seconds <= 0) return 'Just started';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) return `Up ${hours}h ${minutes}m`;
    return `Up ${minutes}m`;
}

function formatRelativeTime(timestampSecs: number): string {
    if (!timestampSecs || timestampSecs <= 0) return 'Never';
    const now = Math.floor(Date.now() / 1000);
    const diff = Math.max(0, now - timestampSecs);
    if (diff < 10) return 'Just now';
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return `${Math.floor(diff / 3600)}h ago`;
}

export const FleetMachinesTable: React.FC<FleetMachinesTableProps> = ({
    className,
    pollingIntervalMs = 3000,
}) => {
    const { t } = useTranslation();
    const [machines, setMachines] = useState<FleetMachineInfo[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isSyncing, setIsSyncing] = useState<boolean>(false);
    const [isMasked, setIsMasked] = useState<boolean>(true);
    const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});
    const [copiedIp, setCopiedIp] = useState<string | null>(null);
    const [lastSyncTime, setLastSyncTime] = useState<number | null>(null);

    const isMountedRef = useRef<boolean>(true);
    const timerRef = useRef<NodeJS.Timeout | null>(null);

    const fetchFleetData = useCallback(async (isManualTrigger: boolean = false) => {
        if (isManualTrigger) {
            setIsSyncing(true);
        }
        try {
            const data = await supabaseService.getFleetMachines();
            if (isMountedRef.current) {
                // Ensure local machine is first, then sort remaining by alias
                const sorted = [...data].sort((a, b) => {
                    if (a.is_local && !b.is_local) return -1;
                    if (!a.is_local && b.is_local) return 1;
                    return a.alias.localeCompare(b.alias);
                });
                setMachines(sorted);
                setLastSyncTime(Date.now());
            }
        } catch (error) {
            console.warn('[FleetMachinesTable] Failed to fetch fleet telemetry:', error);
        } finally {
            if (isMountedRef.current) {
                setIsLoading(false);
                if (isManualTrigger) {
                    setIsSyncing(false);
                }
            }
        }
    }, []);

    const handleSyncNow = async () => {
        setIsSyncing(true);
        try {
            await supabaseService.syncNow();
            showToast(t('fleet.syncing', 'Syncing fleet heartbeat...'), 'info');
            await fetchFleetData(true);
        } catch (error) {
            showToast('Failed to trigger fleet sync', 'error');
            setIsSyncing(false);
        }
    };

    const handleCopyIp = async (ip: string) => {
        try {
            await navigator.clipboard.writeText(ip);
            setCopiedIp(ip);
            showToast(t('fleet.ip_copied', 'IP address copied to clipboard'), 'info');
            setTimeout(() => {
                if (isMountedRef.current) setCopiedIp(null);
            }, 1500);
        } catch {
            showToast('Failed to copy IP', 'error');
        }
    };

    const toggleEmailReveal = (instanceId: string) => {
        setRevealedEmails((prev) => ({
            ...prev,
            [instanceId]: !prev[instanceId],
        }));
    };

    // Polling lifecycle with visibility optimization
    useEffect(() => {
        isMountedRef.current = true;
        fetchFleetData(false);

        const startTimer = () => {
            if (timerRef.current) clearInterval(timerRef.current);
            timerRef.current = setInterval(() => {
                if (document.visibilityState === 'visible') {
                    fetchFleetData(false);
                }
            }, pollingIntervalMs);
        };

        const handleVisibilityChange = () => {
            if (document.visibilityState === 'visible') {
                fetchFleetData(false);
                startTimer();
            } else if (timerRef.current) {
                clearInterval(timerRef.current);
            }
        };

        startTimer();
        document.addEventListener('visibilitychange', handleVisibilityChange);

        return () => {
            isMountedRef.current = false;
            if (timerRef.current) clearInterval(timerRef.current);
            document.removeEventListener('visibilitychange', handleVisibilityChange);
        };
    }, [fetchFleetData, pollingIntervalMs]);

    return (
        <div className={cn(
            "bg-white dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] rounded-[5px] shadow-sm overflow-hidden mt-6 transition-colors duration-200",
            className
        )}>
            {/* Header Toolbar */}
            <div className="px-4 py-3 bg-slate-50/70 dark:bg-[#091e30] border-b border-slate-200/80 dark:border-[#15334d] flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-2.5">
                    <Server className="w-4 h-4 text-cyan-600 dark:text-cyan-400" />
                    <div>
                        <h4 className="font-bold text-xs text-slate-800 dark:text-slate-100 uppercase tracking-wider flex items-center gap-2">
                            <span>{t('fleet.title', 'Fleet Machines & Remote Workers')}</span>
                            <span className="px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-medium bg-cyan-100/70 dark:bg-cyan-950/60 text-cyan-700 dark:text-cyan-300 border border-cyan-200 dark:border-cyan-800/60">
                                {machines.length} {machines.length === 1 ? 'Node' : 'Nodes'}
                            </span>
                        </h4>
                        <p className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
                            {t('fleet.desc', 'Observational live telemetry synchronized across fleet nodes via Supabase')}
                        </p>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    {/* Mask / Reveal Toggle */}
                    <button
                        type="button"
                        onClick={() => setIsMasked((prev) => !prev)}
                        className="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-[5px] font-medium transition-colors bg-slate-100 hover:bg-slate-200 dark:bg-[#071a27] dark:hover:bg-[#15334d] text-slate-700 dark:text-slate-300 border border-slate-200/80 dark:border-[#15334d]"
                        title={isMasked ? "Show plaintext emails" : "Mask email addresses"}
                    >
                        {isMasked ? (
                            <>
                                <Eye className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400" />
                                <span>{t('fleet.show_emails', 'Show Emails')}</span>
                            </>
                        ) : (
                            <>
                                <EyeOff className="w-3.5 h-3.5 text-blue-500" />
                                <span>{t('fleet.mask_emails', 'Mask Emails')}</span>
                            </>
                        )}
                    </button>

                    {/* Sync Now Button */}
                    <button
                        type="button"
                        disabled={isSyncing}
                        onClick={handleSyncNow}
                        className="inline-flex items-center gap-1.5 px-3 py-1 text-xs rounded-[5px] font-medium transition-all bg-cyan-600 hover:bg-cyan-500 text-white shadow-2xs disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        <RefreshCw className={cn("w-3.5 h-3.5", isSyncing && "animate-spin")} />
                        <span>{isSyncing ? t('fleet.syncing', 'Syncing...') : t('fleet.sync_now', 'Sync Now')}</span>
                    </button>
                </div>
            </div>

            {/* Table Content */}
            <div className="overflow-x-auto">
                <table className="w-full text-left border-collapse">
                    <thead>
                        <tr className="bg-slate-100/50 dark:bg-[#081a29]/80 border-b border-slate-200/80 dark:border-[#15334d] text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                            <th className="px-3 py-2 min-w-[200px]">{t('fleet.col_machine', 'Machine & IP')}</th>
                            <th className="px-3 py-2 min-w-[220px]">{t('fleet.col_instances', 'Instances / Profiles')}</th>
                            <th className="px-3 py-2 min-w-[220px]">{t('fleet.col_accounts', 'Connected Accounts')}</th>
                            <th className="px-3 py-2 min-w-[170px]">{t('fleet.col_prompts', 'Running Prompts')}</th>
                            <th className="px-3 py-2 min-w-[160px]">{t('fleet.col_status', 'Leases & Heartbeat')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-200/70 dark:divide-[#15334d]/70 text-xs">
                        {isLoading && machines.length === 0 ? (
                            <tr>
                                <td colSpan={5} className="px-4 py-8 text-center text-slate-400 dark:text-slate-500">
                                    <div className="flex items-center justify-center gap-2">
                                        <RefreshCw className="w-4 h-4 animate-spin text-cyan-500" />
                                        <span>Loading fleet telemetry from Supabase...</span>
                                    </div>
                                </td>
                            </tr>
                        ) : machines.length === 0 ? (
                            <tr>
                                <td colSpan={5} className="px-4 py-8 text-center text-slate-400 dark:text-slate-500">
                                    <p>{t('fleet.no_machines', 'No fleet machines registered yet. Ensure Supabase sync is enabled.')}</p>
                                </td>
                            </tr>
                        ) : (
                            machines.map((machine, index) => {
                                const workerLabel = machine.is_local
                                    ? t('fleet.worker_local', { num: index + 1, defaultValue: `Worker ${index + 1} (Local)` })
                                    : t('fleet.worker_remote', { num: index + 1, alias: machine.alias, defaultValue: `Worker ${index + 1} (Remote ${machine.alias})` });

                                const nowSecs = Math.floor(Date.now() / 1000);
                                const heartbeatDiff = Math.max(0, nowSecs - machine.last_heartbeat_at);
                                const isFreshHeartbeat = heartbeatDiff <= 60;
                                const isStaleHeartbeat = heartbeatDiff > 60 && heartbeatDiff <= 180;

                                return (
                                    <tr
                                        key={machine.node_id}
                                        className="hover:bg-slate-50/60 dark:hover:bg-[#0f2d47]/50 transition-colors"
                                    >
                                        {/* 1. Machine & IP */}
                                        <td className="px-3 py-2.5 align-top">
                                            <div className="flex flex-col gap-1.5">
                                                <div className="flex items-center gap-2 flex-wrap">
                                                    <span className={cn(
                                                        "px-2 py-0.5 rounded-[5px] text-[10px] font-mono font-bold border",
                                                        machine.is_local
                                                            ? "bg-blue-50 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800/60"
                                                            : "bg-slate-100 dark:bg-[#071a27] text-slate-700 dark:text-slate-300 border-slate-200 dark:border-[#15334d]"
                                                    )}>
                                                        {workerLabel}
                                                    </span>
                                                    <span className="font-semibold text-slate-800 dark:text-slate-100 truncate max-w-[130px]" title={machine.alias}>
                                                        {machine.alias}
                                                    </span>
                                                </div>

                                                <div className="flex items-center gap-1.5 flex-wrap">
                                                    {/* IP Badge with 1-click copy */}
                                                    <div className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono bg-slate-100/90 dark:bg-[#071a27] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#15334d]">
                                                        <span>{machine.ip_address || '127.0.0.1'}</span>
                                                        <button
                                                            type="button"
                                                            onClick={() => handleCopyIp(machine.ip_address || '127.0.0.1')}
                                                            className="p-0.5 rounded-[4px] hover:bg-slate-200 dark:hover:bg-[#15334d] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-colors"
                                                            title="Copy IP Address"
                                                        >
                                                            {copiedIp === (machine.ip_address || '127.0.0.1') ? (
                                                                <Check className="w-3 h-3 text-emerald-500" />
                                                            ) : (
                                                                <Copy className="w-3 h-3" />
                                                            )}
                                                        </button>
                                                    </div>

                                                    {/* Uptime */}
                                                    <span className="text-[10px] text-slate-400 dark:text-slate-500">
                                                        {formatUptime(machine.uptime_seconds)}
                                                    </span>
                                                </div>
                                            </div>
                                        </td>

                                        {/* 2. Instances / Profiles */}
                                        <td className="px-3 py-2.5 align-top">
                                            {machine.instances.length === 0 ? (
                                                <span className="text-slate-400 dark:text-slate-500 italic text-[11px]">(No profiles)</span>
                                            ) : (
                                                <div className="flex flex-col gap-1.5">
                                                    {machine.instances.map((inst) => {
                                                        const isRunning = inst.is_active || inst.status === 'running';
                                                        return (
                                                            <div
                                                                key={inst.instance_id}
                                                                className="flex items-center gap-1.5 flex-wrap"
                                                            >
                                                                <span className={cn(
                                                                    "w-2 h-2 rounded-full shrink-0",
                                                                    isRunning ? "bg-emerald-500 animate-pulse" : "bg-slate-400 dark:bg-slate-600"
                                                                )} />
                                                                <span className="font-medium text-slate-800 dark:text-slate-200 text-xs truncate max-w-[130px]" title={inst.profile_name}>
                                                                    {inst.profile_name}
                                                                </span>
                                                                {inst.quota_percent !== undefined && (
                                                                    <span className="text-[10px] font-mono px-1 py-0.2 rounded-[4px] bg-slate-100 dark:bg-[#071a27] text-slate-600 dark:text-slate-400 border border-slate-200/60 dark:border-[#15334d]/60">
                                                                        {inst.quota_percent}%
                                                                    </span>
                                                                )}
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            )}
                                        </td>

                                        {/* 3. Connected Accounts / Emails */}
                                        <td className="px-3 py-2.5 align-top">
                                            {machine.instances.length === 0 ? (
                                                <span className="text-slate-400 dark:text-slate-500 italic text-[11px]">—</span>
                                            ) : (
                                                <div className="flex flex-col gap-1.5">
                                                    {machine.instances.map((inst) => {
                                                        const email = inst.active_account_email;
                                                        const isRevealed = Boolean(revealedEmails[inst.instance_id]) || !isMasked;
                                                        const displayEmail = email
                                                            ? (isRevealed ? email : maskEmail(email))
                                                            : t('fleet.unbound', '(Unbound)');

                                                        return (
                                                            <div
                                                                key={inst.instance_id}
                                                                className="flex items-center gap-1 text-[11px] group/acc"
                                                            >
                                                                {inst.lease_expires_at && (
                                                                    <KeyRound className="w-3 h-3 text-amber-500 shrink-0" title="Active workspace lease held" />
                                                                )}
                                                                <span
                                                                    className={cn(
                                                                        "font-mono truncate max-w-[170px]",
                                                                        email ? "text-slate-700 dark:text-slate-300" : "text-slate-400 dark:text-slate-500 italic"
                                                                    )}
                                                                    title={email || undefined}
                                                                >
                                                                    {displayEmail}
                                                                </span>
                                                                {email && isMasked && (
                                                                    <button
                                                                        type="button"
                                                                        onClick={() => toggleEmailReveal(inst.instance_id)}
                                                                        className="opacity-0 group-hover/acc:opacity-100 p-0.5 rounded-[4px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-opacity"
                                                                        title={isRevealed ? "Mask email" : "Reveal email"}
                                                                    >
                                                                        {isRevealed ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
                                                                    </button>
                                                                )}
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            )}
                                        </td>

                                        {/* 4. Running Prompts */}
                                        <td className="px-3 py-2.5 align-top">
                                            {machine.instances.length === 0 ? (
                                                <span className="text-slate-400 dark:text-slate-500 italic text-[11px]">0</span>
                                            ) : (
                                                <div className="flex flex-col gap-1.5">
                                                    {machine.instances.map((inst) => {
                                                        const promptCount = inst.running_prompts_count || 0;
                                                        if (promptCount > 0) {
                                                            return (
                                                                <div key={inst.instance_id} className="flex items-center">
                                                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-cyan-50 dark:bg-cyan-950/60 text-cyan-600 dark:text-cyan-300 border border-cyan-300 dark:border-cyan-700/60 shadow-[0_0_8px_rgba(6,182,212,0.35)] animate-pulse">
                                                                        <span className="w-1.5 h-1.5 rounded-full bg-cyan-500 animate-ping" />
                                                                        <span>⚡ {promptCount} {t('fleet.prompts_running', { count: promptCount, defaultValue: `${promptCount} Running` })}</span>
                                                                    </span>
                                                                </div>
                                                            );
                                                        }
                                                        return (
                                                            <div key={inst.instance_id} className="flex items-center">
                                                                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono text-slate-400 dark:text-slate-500 bg-slate-100/60 dark:bg-[#071a27] border border-slate-200/60 dark:border-[#15334d]/60">
                                                                    {t('fleet.prompts_idle', 'Idle (0)')}
                                                                </span>
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            )}
                                        </td>

                                        {/* 5. Leases & Heartbeat */}
                                        <td className="px-3 py-2.5 align-top">
                                            <div className="flex flex-col gap-1 text-[11px]">
                                                {/* Heartbeat Status Dot & Relative Time */}
                                                <div className="flex items-center gap-1.5">
                                                    <span className={cn(
                                                        "w-2 h-2 rounded-full shrink-0",
                                                        isFreshHeartbeat
                                                            ? "bg-emerald-500"
                                                            : isStaleHeartbeat
                                                                ? "bg-amber-500"
                                                                : "bg-rose-500"
                                                    )} />
                                                    <span className={cn(
                                                        "font-medium",
                                                        isFreshHeartbeat
                                                            ? "text-emerald-700 dark:text-emerald-400"
                                                            : isStaleHeartbeat
                                                                ? "text-amber-700 dark:text-amber-400"
                                                                : "text-rose-700 dark:text-rose-400"
                                                    )}>
                                                        {isFreshHeartbeat ? t('fleet.online', 'Online') : isStaleHeartbeat ? t('fleet.stale', 'Stale') : t('fleet.offline', 'Offline')}
                                                    </span>
                                                    <span className="text-[10px] text-slate-400 dark:text-slate-500">
                                                        ({formatRelativeTime(machine.last_heartbeat_at)})
                                                    </span>
                                                </div>

                                                {/* Lease TTL if present */}
                                                {machine.instances.some((i) => Boolean(i.lease_expires_at)) && (
                                                    <div className="text-[10px] font-mono text-amber-600 dark:text-amber-400 flex items-center gap-1">
                                                        <Clock className="w-3 h-3 shrink-0" />
                                                        <span>Lease active</span>
                                                    </div>
                                                )}
                                            </div>
                                        </td>
                                    </tr>
                                );
                            })
                        )}
                    </tbody>
                </table>
            </div>

            {/* Read-Only Safety Callout Footer */}
            <div className="px-4 py-2 bg-slate-50/50 dark:bg-[#081b2a] border-t border-slate-200/60 dark:border-[#15334d]/60 flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400 flex-wrap gap-2">
                <span className="flex items-center gap-1.5">
                    <Shield className="w-3.5 h-3.5 text-blue-500/80 shrink-0" />
                    <span>{t('fleet.read_only_notice', 'Observational Only: Remote worker instances are executed on their host machines. Fleet table provides live cluster telemetry without remote process control.')}</span>
                </span>
                <span className="font-mono text-[10px]">
                    {lastSyncTime ? `Last synced: ${new Date(lastSyncTime).toLocaleTimeString()}` : 'Syncing...'}
                </span>
            </div>
        </div>
    );
};
```

---

### 3.3 Step 3: Mount in `src/pages/Instances.tsx` at Line 1756

In `src/pages/Instances.tsx`:
1. Add import:
   ```typescript
   import { FleetMachinesTable } from '../components/instances/FleetMachinesTable';
   ```
2. Mount immediately following line 1756 (below the closing bracket `)}` of the local instances conditional):
   ```tsx
               )}

               {/* Fleet Machines & Cross-Device Instances Observational Table */}
               <FleetMachinesTable />

               {/* Create Instance Modal */}
   ```

---

### 3.4 Step 4: Add Localization Strings

#### In `src/locales/en.json`:
Add under `"fleet"`:
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
    "prompts_idle": "Idle (0)",
    "unbound": "(Unbound)",
    "online": "Online",
    "stale": "Stale",
    "offline": "Offline",
    "ip_copied": "IP address copied to clipboard",
    "read_only_notice": "Observational Only: Remote worker instances are executed on their host machines. Fleet table provides live cluster telemetry without remote process control.",
    "no_machines": "No fleet machines registered yet. Ensure Supabase sync is enabled."
  }
```

#### In `src/locales/zh.json`:
Add under `"fleet"`:
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
    "prompts_idle": "空闲 (0)",
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

## 4. Non-Negotiable Invariants Checklist

- [ ] **Line 1756 Mount Location**: `<FleetMachinesTable />` mounts at line 1756 of `src/pages/Instances.tsx`, rendering below both Card Mode and Table Mode.
- [ ] **Worker Grouping**: Local node is pinned at index 0 as `Worker 1 (Local)`, remote nodes are sorted as `Worker 2 (Remote ...)`.
- [ ] **1-Click IP Copy**: Clicking IP copies to clipboard, displays checkmark feedback, and fires toast notification.
- [ ] **Email Masking**: Default state obfuscates emails; toggle switches between masked and revealed.
- [ ] **Pulsing Cyan Badge**: When `running_prompts_count > 0`, renders glowing cyan pill with `animate-pulse`; when 0, renders muted idle badge.
- [ ] **Strict Read-Only Safety**: ZERO launch, stop, restart, or delete buttons appear on remote nodes. Observational notice is rendered at table base.
- [ ] **Visual Styling**: Dark-glass container (`bg-white dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] rounded-[5px]`), 5–6px button radius, compact density.
- [ ] **Positive Boolean Conventions**: All boolean states and props use positive phrasing (`isOnline`, `isLocal`, `isMasked`, `isRunning`).
- [ ] **Polling & Performance**: 3,000ms polling loop pauses on hidden visibility and cleans up on unmount.
- [ ] **Pre-Flight Checks**: `npm run build` passes with zero type or build errors.
