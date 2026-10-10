import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
    Server,
    RotateCw,
    Copy,
    Check,
    Eye,
    EyeOff,
    Zap,
    Clock,
    CloudOff,
    ExternalLink,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { supabaseService } from '../../services/supabaseService';
import { cn } from '../../utils/cn';
import { useErrorStore } from '../../stores/error-store';

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

export function maskEmailAddress(email: string): string {
    if (!email || !email.includes('@')) {
        return email || '—';
    }
    const [local, domain] = email.split('@');
    if (!local || local.length <= 2) {
        return `${local ? local[0] : '*'}***@${domain || ''}`;
    }
    const firstChar = local[0];
    const lastChar = local[local.length - 1];
    return `${firstChar}***${lastChar}@${domain}`;
}

export function formatRelativeHeartbeat(timestamp?: number | null): string {
    if (!timestamp || timestamp <= 0) {
        return 'Never';
    }
    const ms = timestamp < 10000000000 ? timestamp * 1000 : timestamp;
    const diffSec = Math.max(0, Math.floor((Date.now() - ms) / 1000));
    if (diffSec < 5) return 'Just now';
    if (diffSec < 60) return `${diffSec}s ago`;
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHour = Math.floor(diffMin / 60);
    if (diffHour < 24) return `${diffHour}h ago`;
    const diffDay = Math.floor(diffHour / 24);
    return `${diffDay}d ago`;
}

export function FleetMachinesTable() {
    const navigate = useNavigate();
    const [machines, setMachines] = useState<FleetMachineInfo[]>([]);
    const [localNodeId, setLocalNodeId] = useState<string>('');
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
    const [isSyncEnabled, setIsSyncEnabled] = useState<boolean>(true);
    const [showAllEmails, setShowAllEmails] = useState<boolean>(false);
    const [unmaskedRows, setUnmaskedRows] = useState<Record<string, boolean>>({});
    const [copiedIp, setCopiedIp] = useState<string | null>(null);
    const [countdown, setCountdown] = useState<number>(15);

    const fetchFleetMachinesApi = useCallback(async (): Promise<FleetMachineInfo[]> => {
        if (typeof (supabaseService as any).getFleetMachines === 'function') {
            return await (supabaseService as any).getFleetMachines();
        }
        try {
            return await invoke<FleetMachineInfo[]>('get_fleet_machines');
        } catch (err) {
            console.warn('[FleetMachinesTable] get_fleet_machines invoke failed:', err);
            // Tracked in the error module; empty list shown, retried on next refresh.
            useErrorStore.getState().trackWarning(err, {
              source: 'FleetMachinesTable.fetchFleetMachinesApi',
              endpoint: 'get_fleet_machines',
              triggerAction: 'fetch_fleet_machines',
            });
            return [];
        }
    }, []);

    const fetchMachines = useCallback(
        async (showLoadingSpinner: boolean = false) => {
            if (showLoadingSpinner) {
                setIsRefreshing(true);
            }
            try {
                const config = await supabaseService.getConfig().catch(() => null);
                const syncActive = Boolean(config?.is_sync_enabled);
                setIsSyncEnabled(syncActive);

                if (!syncActive) {
                    setMachines([]);
                    return;
                }

                let effectiveLocalId = localNodeId;
                if (!effectiveLocalId) {
                    const localInfo = await supabaseService.getLocalNodeInfo().catch(() => null);
                    if (localInfo?.node_id) {
                        effectiveLocalId = localInfo.node_id;
                        setLocalNodeId(effectiveLocalId);
                    }
                }

                const data = await fetchFleetMachinesApi();
                const filtered = data.filter((m) => m.node_id !== effectiveLocalId);
                setMachines(filtered);
            } catch (err) {
                console.warn('[FleetMachinesTable] fetchMachines error:', err);
                // Tracked in the error module; stale list kept, retried on next refresh.
                useErrorStore.getState().trackWarning(err, {
                  source: 'FleetMachinesTable.fetchMachines',
                  triggerAction: 'fetch_machines',
                });
            } finally {
                setIsRefreshing(false);
            }
        },
        [fetchFleetMachinesApi, localNodeId]
    );

    // Initial load
    useEffect(() => {
        let isMounted = true;
        const init = async () => {
            setIsLoading(true);
            try {
                const [configRes, localNodeRes] = await Promise.allSettled([
                    supabaseService.getConfig(),
                    supabaseService.getLocalNodeInfo(),
                ]);

                let syncActive = false;
                if (configRes.status === 'fulfilled' && configRes.value) {
                    syncActive = Boolean(configRes.value.is_sync_enabled);
                    if (isMounted) setIsSyncEnabled(syncActive);
                }

                let localId = '';
                if (localNodeRes.status === 'fulfilled' && localNodeRes.value) {
                    localId = localNodeRes.value.node_id;
                    if (isMounted) setLocalNodeId(localId);
                }

                if (syncActive) {
                    const data = await fetchFleetMachinesApi();
                    if (isMounted) {
                        const filtered = data.filter((m) => m.node_id !== localId);
                        setMachines(filtered);
                    }
                }
            } catch (err) {
                console.warn('[FleetMachinesTable] init error:', err);
                // Tracked in the error module; loading spinner cleared, empty state shown.
                useErrorStore.getState().trackWarning(err, {
                  source: 'FleetMachinesTable.init',
                  triggerAction: 'init_machines',
                });
            } finally {
                if (isMounted) {
                    setIsLoading(false);
                }
            }
        };

        init();
        return () => {
            isMounted = false;
        };
    }, [fetchFleetMachinesApi]);

    // 15-second polling interval
    useEffect(() => {
        if (!isSyncEnabled) return;

        const timer = setInterval(() => {
            setCountdown((prev) => {
                if (prev <= 1) {
                    fetchMachines(false);
                    return 15;
                }
                return prev - 1;
            });
        }, 1000);

        return () => clearInterval(timer);
    }, [isSyncEnabled, fetchMachines]);

    const handleManualRefresh = () => {
        setCountdown(15);
        fetchMachines(true);
    };

    const handleCopyIp = async (ip: string) => {
        if (!ip) return;
        try {
            await navigator.clipboard.writeText(ip);
            setCopiedIp(ip);
            setTimeout(() => {
                setCopiedIp(null);
            }, 2000);
        } catch (err) {
            console.warn('[FleetMachinesTable] Failed to copy IP:', err);
            // Tracked in the error module; copy feedback simply won't show, user can copy manually.
            useErrorStore.getState().trackWarning(err, {
              source: 'FleetMachinesTable.copyIp',
              triggerAction: 'copy_ip',
            });
        }
    };

    const toggleRowMask = (nodeId: string) => {
        setUnmaskedRows((prev) => ({
            ...prev,
            [nodeId]: !prev[nodeId],
        }));
    };

    // Disabled State: Supabase sync disabled
    if (!isLoading && !isSyncEnabled) {
        return (
            <div className="rounded-xl bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 p-5 shadow-lg">
                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                    <div className="flex items-start gap-3">
                        <div className="p-2.5 rounded-lg bg-amber-500/10 text-amber-400 border border-amber-500/20 shrink-0 mt-0.5 sm:mt-0">
                            <CloudOff className="w-5 h-5" />
                        </div>
                        <div>
                            <h4 className="text-sm font-semibold text-slate-200">
                                Multi-Machine Fleet Sync is Disabled
                            </h4>
                            <p className="text-xs text-slate-400 mt-0.5 leading-relaxed">
                                Fleet monitoring and cross-machine instance leases are inactive. Enable Supabase Sync in configuration to coordinate instances and track remote fleet nodes across your network.
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={() => navigate('/supabase')}
                        className="shrink-0 px-3 py-1.5 rounded-lg text-xs font-medium bg-blue-600/20 text-blue-400 border border-blue-500/30 hover:bg-blue-600/30 hover:text-blue-300 transition-colors flex items-center gap-1.5 cursor-pointer"
                    >
                        <span>Supabase Settings</span>
                        <ExternalLink className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>
        );
    }

    // Loading State
    if (isLoading) {
        return (
            <div className="rounded-xl bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 p-6 shadow-lg space-y-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-slate-800/60 animate-pulse" />
                        <div className="space-y-1.5">
                            <div className="w-36 h-4 rounded bg-slate-800/60 animate-pulse" />
                            <div className="w-56 h-3 rounded bg-slate-800/40 animate-pulse" />
                        </div>
                    </div>
                    <div className="w-24 h-7 rounded-full bg-slate-800/60 animate-pulse" />
                </div>
                <div className="space-y-2">
                    {[1, 2, 3].map((i) => (
                        <div key={i} className="w-full h-10 rounded-lg bg-slate-800/40 animate-pulse" />
                    ))}
                </div>
            </div>
        );
    }

    // Empty State: 0 remote nodes detected
    if (machines.length === 0) {
        return (
            <div className="rounded-xl bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 p-8 text-center shadow-lg">
                <div className="w-12 h-12 rounded-xl bg-slate-800/60 border border-slate-700/60 flex items-center justify-center mx-auto text-slate-400 mb-3 shadow-inner">
                    <Server className="w-6 h-6 text-slate-400" />
                </div>
                <h4 className="text-sm font-semibold text-slate-200">
                    No Remote Fleet Machines Detected
                </h4>
                <p className="text-xs text-slate-400 mt-1 max-w-md mx-auto leading-relaxed">
                    This local machine is currently the sole active node registered in the Supabase cluster. Once other worker machines or fleet nodes join with the shared Supabase credentials, their status, instances, and active prompts will appear here automatically.
                </p>
                <div className="mt-4 flex items-center justify-center gap-2">
                    <button
                        type="button"
                        onClick={handleManualRefresh}
                        disabled={isRefreshing}
                        className="px-3 py-1.5 rounded-lg text-xs font-medium bg-slate-800/80 hover:bg-slate-700/80 text-slate-300 border border-slate-700/60 transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                    >
                        <RotateCw className={cn("w-3.5 h-3.5 text-cyan-400", isRefreshing && "animate-spin")} />
                        <span>Check Again</span>
                    </button>
                </div>
            </div>
        );
    }

    // Full Table View
    return (
        <div className="rounded-xl bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 shadow-lg overflow-hidden">
            {/* Table Header toolbar */}
            <div className="px-4 py-3.5 border-b border-slate-700/40 dark:border-slate-800/60 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-slate-900/30">
                <div className="flex items-center gap-2.5">
                    <div className="p-1.5 rounded-lg bg-blue-500/10 text-blue-400 border border-blue-500/20">
                        <Server className="w-4 h-4" />
                    </div>
                    <div>
                        <div className="flex items-center gap-2">
                            <h3 className="font-semibold text-sm text-slate-100">
                                Other Machines / Fleet Nodes
                            </h3>
                            <span className="px-2 py-0.5 rounded-full text-[11px] font-bold bg-blue-500/20 text-blue-300 border border-blue-500/30">
                                {machines.length}
                            </span>
                        </div>
                        <p className="text-[11px] text-slate-400">
                            Real-time multi-machine cluster state synced via Supabase
                        </p>
                    </div>
                </div>

                {/* Segmented Pill Capsule Toolbar */}
                <div className="flex items-center rounded-full bg-slate-800/60 dark:bg-slate-900/80 border border-slate-700/60 p-0.5 text-xs divide-x divide-slate-700/60 shadow-xs self-stretch sm:self-auto justify-end">
                    {/* Countdown / Poll status pill */}
                    <div className="px-2.5 py-1 text-slate-400 flex items-center gap-1.5 font-mono text-[11px]">
                        <span className="inline-block w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse" />
                        <span>{countdown}s</span>
                    </div>

                    {/* Reveal / Mask all emails toggle */}
                    <button
                        type="button"
                        onClick={() => setShowAllEmails((prev) => !prev)}
                        className="px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors flex items-center gap-1.5 cursor-pointer"
                        title={showAllEmails ? "Mask all account emails" : "Reveal all account emails"}
                    >
                        {showAllEmails ? (
                            <EyeOff className="w-3.5 h-3.5 text-blue-400" />
                        ) : (
                            <Eye className="w-3.5 h-3.5 text-slate-400" />
                        )}
                        <span>{showAllEmails ? "Mask" : "Reveal"}</span>
                    </button>

                    {/* Manual refresh button */}
                    <button
                        type="button"
                        onClick={handleManualRefresh}
                        disabled={isRefreshing}
                        className="px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                        title="Refresh Fleet Status Now"
                    >
                        <RotateCw className={cn("w-3.5 h-3.5 text-cyan-400", isRefreshing && "animate-spin")} />
                        <span>Refresh</span>
                    </button>
                </div>
            </div>

            {/* Table */}
            <div className="overflow-x-auto">
                <table className="w-full text-left text-xs">
                    <thead>
                        <tr className="border-b border-slate-700/40 dark:border-slate-800/60 bg-slate-950/40 text-[11px] font-semibold uppercase tracking-wider text-slate-400">
                            <th className="px-3.5 py-2.5 min-w-[170px]">Machine & Status</th>
                            <th className="px-3.5 py-2.5 min-w-[140px]">IP Address</th>
                            <th className="px-3.5 py-2.5 min-w-[180px]">Running Instance(s)</th>
                            <th className="px-3.5 py-2.5 min-w-[190px]">Bound Account Email(s)</th>
                            <th className="px-3.5 py-2.5 min-w-[130px]">Running Prompts</th>
                            <th className="px-3.5 py-2.5 text-right min-w-[110px]">Last Seen</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-800/40 font-medium">
                        {machines.map((machine) => {
                            const isRowUnmasked = showAllEmails || Boolean(unmaskedRows[machine.node_id]);
                            const emailList = Array.from(
                                new Set([
                                    ...(machine.bound_emails || []),
                                    ...(machine.active_instances || [])
                                        .map((i) => i.bound_account_email)
                                        .filter(Boolean) as string[],
                                ])
                            );

                            return (
                                <tr
                                    key={machine.node_id}
                                    className="hover:bg-slate-800/30 transition-colors duration-150"
                                >
                                    {/* 1. Machine & Status */}
                                    <td className="px-3.5 py-2.5 align-middle">
                                        <div className="flex items-center gap-2.5">
                                            <div className="relative flex items-center justify-center">
                                                {machine.is_online ? (
                                                    <span className="relative flex h-2.5 w-2.5" title="Online">
                                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                                                        <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.8)]" />
                                                    </span>
                                                ) : (
                                                    <span
                                                        className="inline-flex rounded-full h-2.5 w-2.5 bg-slate-500/80"
                                                        title="Offline"
                                                    />
                                                )}
                                            </div>
                                            <div className="flex flex-col min-w-0">
                                                <div className="flex items-center gap-1.5 flex-wrap">
                                                    <span
                                                        className="font-semibold text-slate-100 text-xs truncate max-w-[140px]"
                                                        title={machine.node_alias}
                                                    >
                                                        {machine.node_alias || 'Unnamed Worker'}
                                                    </span>
                                                    {machine.is_online ? (
                                                        <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-semibold bg-emerald-500/15 text-emerald-400 border border-emerald-500/25">
                                                            ONLINE
                                                        </span>
                                                    ) : (
                                                        <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-medium bg-slate-700/40 text-slate-400 border border-slate-600/30">
                                                            OFFLINE
                                                        </span>
                                                    )}
                                                </div>
                                                <div className="flex items-center gap-1.5 text-[10px] text-slate-400 mt-0.5 font-mono">
                                                    {machine.os_info && (
                                                        <span className="truncate max-w-[130px]" title={machine.os_info}>
                                                            {machine.os_info}
                                                        </span>
                                                    )}
                                                    <span
                                                        className="text-slate-500 truncate max-w-[90px]"
                                                        title={`Node ID: ${machine.node_id}`}
                                                    >
                                                        #{machine.node_id.slice(0, 8)}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </td>

                                    {/* 2. IP Address */}
                                    <td className="px-3.5 py-2.5 align-middle">
                                        <div className="inline-flex items-center gap-1.5 font-mono text-xs text-slate-300 bg-slate-800/40 px-2 py-0.5 rounded border border-slate-700/40">
                                            <span>{machine.ip_address || '—'}</span>
                                            {machine.ip_address && (
                                                <button
                                                    type="button"
                                                    onClick={() => handleCopyIp(machine.ip_address)}
                                                    className="p-0.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
                                                    title="Copy IP Address"
                                                >
                                                    {copiedIp === machine.ip_address ? (
                                                        <Check className="w-3 h-3 text-emerald-400" />
                                                    ) : (
                                                        <Copy className="w-3 h-3" />
                                                    )}
                                                </button>
                                            )}
                                        </div>
                                    </td>

                                    {/* 3. Running Instance(s) */}
                                    <td className="px-3.5 py-2.5 align-middle">
                                        {machine.active_instances && machine.active_instances.length > 0 ? (
                                            <div className="flex items-center gap-1.5 flex-wrap max-w-[260px]">
                                                {machine.active_instances.map((inst, i) => (
                                                    <span
                                                        key={inst.profile_id || `${inst.profile_name}-${i}`}
                                                        className="rounded-full px-2 py-0.5 text-xs bg-blue-500/15 text-blue-400 border border-blue-500/30 flex items-center gap-1 font-medium shadow-2xs"
                                                        title={`Profile: ${inst.profile_name}${inst.bound_account_email ? ` (${inst.bound_account_email})` : ''}`}
                                                    >
                                                        <span className="w-1.5 h-1.5 rounded-full bg-blue-400" />
                                                        <span className="truncate max-w-[120px]">{inst.profile_name}</span>
                                                    </span>
                                                ))}
                                            </div>
                                        ) : (
                                            <span className="text-slate-500 italic text-xs">No active profiles</span>
                                        )}
                                    </td>

                                    {/* 4. Bound Account Email(s) */}
                                    <td className="px-3.5 py-2.5 align-middle">
                                        {emailList.length > 0 ? (
                                            <div className="flex items-center gap-2 flex-wrap">
                                                <div className="flex flex-col gap-1">
                                                    {emailList.map((email, i) => (
                                                        <span
                                                            key={`${email}-${i}`}
                                                            className="font-mono text-xs text-slate-300"
                                                            title={isRowUnmasked ? email : 'Masked email (click eye icon to toggle)'}
                                                        >
                                                            {isRowUnmasked ? email : maskEmailAddress(email)}
                                                        </span>
                                                    ))}
                                                </div>
                                                <button
                                                    type="button"
                                                    onClick={() => toggleRowMask(machine.node_id)}
                                                    className="p-1 rounded-full hover:bg-slate-700/50 text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
                                                    title={isRowUnmasked ? 'Mask email for this node' : 'Reveal email for this node'}
                                                >
                                                    {isRowUnmasked ? (
                                                        <EyeOff className="w-3.5 h-3.5 text-blue-400" />
                                                    ) : (
                                                        <Eye className="w-3.5 h-3.5" />
                                                    )}
                                                </button>
                                            </div>
                                        ) : (
                                            <span className="text-slate-500 italic text-xs">Unassigned</span>
                                        )}
                                    </td>

                                    {/* 5. Running Prompts */}
                                    <td className="px-3.5 py-2.5 align-middle">
                                        {machine.in_flight_prompts_count > 0 ? (
                                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-semibold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 shadow-2xs">
                                                <Zap className="w-3 h-3 text-emerald-400 fill-emerald-400 animate-pulse" />
                                                <span>{machine.in_flight_prompts_count} Running</span>
                                            </span>
                                        ) : (
                                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-slate-700/30 text-slate-400 border border-slate-700/50">
                                                <span>Idle</span>
                                            </span>
                                        )}
                                    </td>

                                    {/* 6. Last Heartbeat / Seen */}
                                    <td className="px-3.5 py-2.5 align-middle text-right">
                                        <span
                                            className="font-mono text-xs text-slate-400 inline-flex items-center gap-1"
                                            title={
                                                machine.last_heartbeat_timestamp > 0
                                                    ? new Date(
                                                          machine.last_heartbeat_timestamp < 10000000000
                                                              ? machine.last_heartbeat_timestamp * 1000
                                                              : machine.last_heartbeat_timestamp
                                                      ).toLocaleString()
                                                    : 'No heartbeat received'
                                            }
                                        >
                                            <Clock className="w-3 h-3 text-slate-500" />
                                            <span>{formatRelativeHeartbeat(machine.last_heartbeat_timestamp)}</span>
                                        </span>
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}

export default FleetMachinesTable;
