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
