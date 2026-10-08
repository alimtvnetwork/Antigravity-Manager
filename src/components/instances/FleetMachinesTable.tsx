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
            } finally {
import React, { useState, useEffect } from 'react';
import { Server, RotateCw, Copy, Check, Eye, EyeOff, Mail, ShieldCheck, Clock } from 'lucide-react';
import { supabaseService, type FleetMachineInfo } from '../../services/supabaseService';
import { cn } from '../../utils/cn';

function maskEmailAddress(email: string): string {
    if (!email) return '';
    const atIndex = email.indexOf('@');
    if (atIndex <= 0) {
        return email.length <= 2 ? `${email}***` : `${email.slice(0, 2)}***`;
    }
    const name = email.slice(0, atIndex);
    const domain = email.slice(atIndex + 1);
    const maskedName = name.length <= 2 ? `${name.slice(0, 1)}***` : `${name.slice(0, 2)}***`;
    return `${maskedName}@${domain}`;
}

function formatUptime(seconds: number): string {
    if (seconds <= 0) return '0m';
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
}

function formatRelativeTime(timestamp: number): string {
    if (timestamp <= 0) return 'Never';
    const timestampMs = timestamp < 1e11 ? timestamp * 1000 : timestamp;
    const diffSecs = Math.max(0, Math.floor((Date.now() - timestampMs) / 1000));
    if (diffSecs < 10) return 'Just now';
    if (diffSecs < 60) return `${diffSecs}s ago`;
    const diffMins = Math.floor(diffSecs / 60);
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
}

function checkIsMachineOnline(machine: FleetMachineInfo): boolean {
    if (machine.status === 'online') {
        return true;
    }
    const timestampMs = machine.last_heartbeat_at < 1e11 ? machine.last_heartbeat_at * 1000 : machine.last_heartbeat_at;
    const diffSecs = (Date.now() - timestampMs) / 1000;
    return diffSecs >= 0 && diffSecs < 90;
}

function formatFriendlyMachineName(machine: FleetMachineInfo, index: number): string {
    if (machine.alias && machine.alias.trim().length > 0) {
        if (machine.is_local) {
            return `${machine.alias} (Local)`;
        }
        return `${machine.alias} (Remote Node-${machine.node_id ? machine.node_id.slice(0, 6) : String(index + 1)})`;
    }
    const shortNodeId = machine.node_id ? machine.node_id.slice(0, 6) : String(index + 1);
    if (machine.is_local) {
        return `Worker ${index + 1} (Local)`;
    }
    return `Worker ${index + 1} (Remote Node-${shortNodeId})`;
}

export const FleetMachinesTable: React.FC = () => {
    const [fleetMachines, setFleetMachines] = useState<FleetMachineInfo[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isSyncing, setIsSyncing] = useState<boolean>(false);
    const [copiedIp, setCopiedIp] = useState<string | null>(null);
    const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});

    useEffect(() => {
        let isMounted = true;

        const loadFleetMachines = async () => {
            try {
                const machines = await supabaseService.getFleetMachines();
                if (isMounted) {
                    setFleetMachines(machines || []);
                    setIsLoading(false);
                }
            } catch (error) {
                console.warn('[FleetMachinesTable] Failed to load fleet machines:', error);
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
        loadFleetMachines();
        const refreshInterval = setInterval(loadFleetMachines, 10000);

        return () => {
            isMounted = false;
            clearInterval(refreshInterval);
        };
    }, []);

    const handleSyncNow = async () => {
        if (isSyncing) return;
        setIsSyncing(true);
        try {
            await supabaseService.syncNow();
            const machines = await supabaseService.getFleetMachines();
            setFleetMachines(machines || []);
        } catch (error) {
            console.warn('[FleetMachinesTable] Manual sync failed:', error);
        } finally {
            setIsSyncing(false);
        }
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
        } catch (error) {
            console.warn('[FleetMachinesTable] Failed to copy IP:', error);
        }
    };

    const toggleEmailReveal = (email: string) => {
        setRevealedEmails((prev) => ({
            ...prev,
            [email]: prev[email] ? false : true,
        }));
    };

    const hasMachines = fleetMachines.length > 0;

    return (
        <section
            aria-label="Fleet Machines & Cross-Node Instance Sync"
            className="bg-white dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] rounded-[5px] p-3.5 shadow-xs mt-6"
        >
            {/* Header */}
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-3 border-b border-slate-200/70 dark:border-[#15334d]">
                <div className="flex items-center gap-2.5">
                    <div className="p-2 rounded-[5px] bg-blue-500/10 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400">
                        <Server className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-sm font-bold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                            Fleet Machines &amp; Cross-Node Instance Sync
                        </h3>
                        <p className="text-xs text-slate-500 dark:text-slate-400">
                            Real-time multi-machine instances, account leases, and running prompts synchronized via Supabase
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
                {/* Right Capsule Controls */}
                <div className="inline-flex items-center bg-slate-100/90 dark:bg-[#102a43]/90 border border-slate-200/90 dark:border-[#194066] rounded-full p-0.5 shadow-xs text-xs self-stretch sm:self-auto justify-between sm:justify-start">
                    <div className="flex items-center gap-1.5 px-3 py-1 text-emerald-600 dark:text-emerald-400 font-medium">
                        <span className="relative flex h-2 w-2">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                        </span>
                        <span>Synced</span>
                    </div>

                    <div className="h-3.5 w-px bg-slate-300 dark:bg-[#1d4b78]" />

                    <div className="px-3 py-1 font-medium text-slate-600 dark:text-slate-300">
                        {fleetMachines.length} {fleetMachines.length === 1 ? 'Machine' : 'Machines'}
                    </div>

                    <div className="h-3.5 w-px bg-slate-300 dark:bg-[#1d4b78]" />

                    <button
                        type="button"
                        onClick={handleSyncNow}
                        disabled={isSyncing}
                        className="flex items-center gap-1.5 px-3 py-1 text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 hover:bg-slate-200/60 dark:hover:bg-[#173e63] rounded-full transition-colors font-medium disabled:opacity-50 cursor-pointer"
                        title="Sync fleet machines now"
                    >
                        <RotateCw className={cn("w-3.5 h-3.5", isSyncing && "animate-spin")} />
                        <span>{isSyncing ? "Syncing..." : "Sync Now"}</span>
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
            {/* Table / Empty State */}
            {hasMachines ? (
                <div className="overflow-x-auto mt-3">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="border-b border-slate-200/80 dark:border-[#15334d]">
                                <th className="px-3 py-2 text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                                    Machine &amp; IP
                                </th>
                                <th className="px-3 py-2 text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                                    Instances / Profiles
                                </th>
                                <th className="px-3 py-2 text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                                    Connected Accounts / Emails
                                </th>
                                <th className="px-3 py-2 text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                                    Running Prompts
                                </th>
                                <th className="px-3 py-2 text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                                    Leases &amp; Heartbeat
                                </th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100 dark:divide-[#132d44]/70">
                            {fleetMachines.map((machine, index) => {
                                const isOnline = checkIsMachineOnline(machine);
                                const isLocal = machine.is_local;
                                const friendlyName = formatFriendlyMachineName(machine, index);
                                const instancesList = machine.instances || [];
                                const hasInstances = instancesList.length > 0;

                                const accountList = Array.from(
                                    new Set([
                                        ...(machine.active_accounts || []),
                                        ...instancesList.map((inst) => inst.bound_account_email).filter(Boolean),
                                    ])
                                );
                                const hasAccounts = accountList.length > 0;

                                const activeLeases = instancesList.filter((inst) => inst.is_leased);
                                const activeLeaseCount = activeLeases.length;
                                const hasRunningPrompts = machine.total_running_prompts > 0;
                                const isCurrentCopied = copiedIp === machine.ip_address;

                                return (
                                    <tr
                                        key={machine.node_id || `${index}-${machine.ip_address}`}
                                        className="hover:bg-slate-50/60 dark:hover:bg-[#0f2c45]/50 transition-colors"
                                    >
                                        {/* Column 1: Machine & IP */}
                                        <td className="px-3 py-2.5 text-xs align-middle">
                                            <div className="flex flex-col gap-1.5">
                                                <div className="flex items-center gap-2">
                                                    {isOnline ? (
                                                        <span className="relative flex h-2 w-2" title="Online">
                                                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                                            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                                                        </span>
                                                    ) : (
                                                        <span className="inline-flex rounded-full h-2 w-2 bg-slate-400 dark:bg-slate-600" title="Offline"></span>
                                                    )}

                                                    <span className="font-semibold text-slate-900 dark:text-slate-100">
                                                        {friendlyName}
                                                    </span>

                                                    {isLocal ? (
                                                        <span className="px-1.5 py-0.5 text-[10px] font-semibold tracking-wide uppercase rounded-[5px] bg-blue-500/10 text-blue-600 dark:bg-blue-500/20 dark:text-blue-400 border border-blue-500/30">
                                                            LOCAL
                                                        </span>
                                                    ) : (
                                                        <span className="px-1.5 py-0.5 text-[10px] font-semibold tracking-wide uppercase rounded-[5px] bg-purple-500/10 text-purple-600 dark:bg-purple-500/20 dark:text-purple-400 border border-purple-500/30">
                                                            REMOTE
                                                        </span>
                                                    )}
                                                </div>

                                                <div className="flex items-center gap-2">
                                                    <button
                                                        type="button"
                                                        onClick={() => handleCopyIp(machine.ip_address)}
                                                        className="inline-flex items-center gap-1.5 px-2 py-0.5 font-mono text-[11px] text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-[#102a43] hover:bg-slate-200 dark:hover:bg-[#163859] border border-slate-200 dark:border-[#1c446c] rounded-[5px] transition-colors cursor-pointer"
                                                        title="Click to copy IP address"
                                                    >
                                                        <span>{machine.ip_address || '127.0.0.1'}</span>
                                                        {isCurrentCopied ? (
                                                            <Check className="w-3 h-3 text-emerald-500" />
                                                        ) : (
                                                            <Copy className="w-3 h-3 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200" />
                                                        )}
                                                    </button>

                                                    <span className="text-[11px] text-slate-400 dark:text-slate-500">
                                                        Uptime: {formatUptime(machine.uptime_seconds)}
                                                    </span>
                                                </div>
                                            </div>
                                        </td>

                                        {/* Column 2: Instances / Profiles */}
                                        <td className="px-3 py-2.5 text-xs align-middle">
                                            <div className="flex flex-wrap gap-1.5 max-w-xs">
                                                {hasInstances ? (
                                                    instancesList.map((inst, instIdx) => {
                                                        const isInstanceRunning = inst.status === 'running' || inst.is_active;
                                                        return (
                                                            <span
                                                                key={inst.instance_id || `${instIdx}-${inst.profile_name}`}
                                                                className="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium bg-slate-100 dark:bg-[#102a43] text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-[#183d60] rounded-[5px]"
                                                            >
                                                                {isInstanceRunning && (
                                                                    <span className="relative flex h-1.5 w-1.5">
                                                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                                                        <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-emerald-500"></span>
                                                                    </span>
                                                                )}
                                                                <span>{inst.profile_name || `#${instIdx + 1}`}</span>
                                                            </span>
                                                        );
                                                    })
                                                ) : (
                                                    <span className="text-xs text-slate-400 dark:text-slate-500 italic">
                                                        0 Instances
                                                    </span>
                                                )}
                                            </div>
                                        </td>

                                        {/* Column 3: Connected Accounts / Emails */}
                                        <td className="px-3 py-2.5 text-xs align-middle">
                                            <div className="flex flex-wrap gap-1.5 max-w-xs">
                                                {hasAccounts ? (
                                                    accountList.map((email) => {
                                                        const isEmailRevealed = revealedEmails[email] || false;
                                                        return (
                                                            <span
                                                                key={email}
                                                                className="inline-flex items-center gap-1.5 px-2 py-0.5 text-xs bg-slate-100 dark:bg-[#102a43] text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-[#183d60] rounded-[5px]"
                                                            >
                                                                <Mail className="w-3 h-3 text-slate-400" />
                                                                <span className="font-mono text-[11px]">
                                                                    {isEmailRevealed ? email : maskEmailAddress(email)}
                                                                </span>
                                                                <button
                                                                    type="button"
                                                                    onClick={() => toggleEmailReveal(email)}
                                                                    className="p-0.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors cursor-pointer"
                                                                    title={isEmailRevealed ? "Mask email" : "Reveal email"}
                                                                >
                                                                    {isEmailRevealed ? (
                                                                        <EyeOff className="w-3 h-3" />
                                                                    ) : (
                                                                        <Eye className="w-3 h-3" />
                                                                    )}
                                                                </button>
                                                            </span>
                                                        );
                                                    })
                                                ) : (
                                                    <span className="text-xs text-slate-400 dark:text-slate-500 italic">
                                                        0 Accounts
                                                    </span>
                                                )}
                                            </div>
                                        </td>

                                        {/* Column 4: Running Prompts */}
                                        <td className="px-3 py-2.5 text-xs align-middle">
                                            {hasRunningPrompts ? (
                                                <div className="inline-flex items-center gap-1.5 bg-cyan-500/15 text-cyan-400 border border-cyan-500/30 font-bold px-2 py-0.5 rounded-[5px]">
                                                    <span className="relative flex h-2 w-2">
                                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75"></span>
                                                        <span className="relative inline-flex rounded-full h-2 w-2 bg-cyan-400"></span>
                                                    </span>
                                                    <span className="text-xs">{machine.total_running_prompts} Running Prompts</span>
                                                </div>
                                            ) : (
                                                <span className="text-xs text-slate-400 dark:text-slate-500 font-medium">
                                                    0 Idle
                                                </span>
                                            )}
                                        </td>

                                        {/* Column 5: Leases & Heartbeat */}
                                        <td className="px-3 py-2.5 text-xs align-middle">
                                            <div className="flex flex-col gap-1">
                                                <div className="flex items-center gap-1.5 text-xs text-slate-700 dark:text-slate-300 font-medium">
                                                    <ShieldCheck className="w-3.5 h-3.5 text-blue-500" />
                                                    <span>
                                                        {activeLeaseCount} Active {activeLeaseCount === 1 ? 'Lease' : 'Leases'}
                                                    </span>
                                                </div>
                                                <div className="flex items-center gap-1.5 text-[11px] text-slate-400 dark:text-slate-500">
                                                    <Clock className="w-3 h-3" />
                                                    <span>Heartbeat: {formatRelativeTime(machine.last_heartbeat_at)}</span>
                                                </div>
                                            </div>
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            ) : (
                <div className="flex flex-col items-center justify-center py-8 text-center text-slate-400 dark:text-slate-500">
                    <Server className="w-8 h-8 mb-2 opacity-40" />
                    <p className="text-xs font-medium text-slate-600 dark:text-slate-400">
                        {isLoading ? 'Loading fleet machines...' : 'No fleet machines detected yet'}
                    </p>
                    <p className="text-[11px] text-slate-400 dark:text-slate-500 mt-0.5">
                        Configure Supabase sync to discover multi-node machines and instances
                    </p>
                </div>
            )}
        </section>
    );
};

export default FleetMachinesTable;
