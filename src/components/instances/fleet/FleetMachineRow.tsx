import { Copy, Check, Eye, EyeOff, Zap, Clock, GitBranch } from 'lucide-react';
import { maskEmailAddress, formatRelativeHeartbeat, toDisplayTimestamp } from './fleetFormatUtils';
import type { FleetMachineInfo } from './fleetTypes';

interface FleetMachineRowProps {
    machine: FleetMachineInfo;
    isRowUnmasked: boolean;
    copiedIp: string | null;
    onCopyIp: (ip: string) => void;
    onToggleMask: (nodeId: string) => void;
}

function isAdminIdentifier(val?: string | null): boolean {
    if (!val) return false;
    const lower = val.toLowerCase().trim();
    return (
        lower.includes('administrator') ||
        lower.includes('root') ||
        lower.includes('${adminuser}') ||
        lower.includes('${linuxuser}') ||
        lower.includes(':') ||
        lower.startsWith('password')
    );
}

function sanitizeAdminText(text?: string | null): string {
    if (!text) return '';
    return text
        .replace(/\$\{adminUser\}|\$\{linuxUser\}/gi, '')
        .replace(/\b(administrator|root)\b/gi, '')
        .trim();
}

export function FleetMachineRow({
    machine,
    isRowUnmasked,
    copiedIp,
    onCopyIp,
    onToggleMask,
}: FleetMachineRowProps) {
    const emailList = Array.from(
        new Set([
            ...(machine.bound_emails || []),
            ...(machine.active_instances || [])
                .map((i) => i.bound_account_email)
                .filter(Boolean) as string[],
        ])
    ).filter((email) => !isAdminIdentifier(email));

    const cleanAlias = sanitizeAdminText(machine.node_alias) || 'Unnamed Worker';
    const cleanOsInfo = sanitizeAdminText(machine.os_info);
    const mExt = machine as {
        is_local?: boolean;
        alias?: string;
        source?: string;
        is_lead?: boolean;
        git_repo?: string;
        git_link?: string;
    };
    const aliasLower = (machine.node_alias || mExt.alias || '').trim().toLowerCase();
    const isLeadNode = Boolean(mExt.is_lead || aliasLower === 'main');
    const isLocalNode = Boolean(
        mExt.is_local ||
        aliasLower === 'w1' ||
        aliasLower.startsWith('local') ||
        mExt.source === 'local'
    );

    return (
        <tr key={machine.node_id} className="hover:bg-slate-800/30 transition-colors duration-150">
            {/* 1. Machine & Status with inlined demoted IP badge */}
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
                                title={cleanAlias}
                            >
                                {cleanAlias}
                            </span>
                            {isLeadNode && (
                                <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-semibold bg-purple-500/15 text-purple-300 border border-purple-500/30 shadow-[0_0_8px_rgba(168,85,247,0.15)]">
                                    Lead
                                </span>
                            )}
                            {isLocalNode && !isLeadNode && (
                                <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-semibold bg-cyan-500/15 text-cyan-400 border border-cyan-500/30 shadow-[0_0_8px_rgba(6,182,212,0.15)]">
                                    Local
                                </span>
                            )}
                            {machine.is_online ? (
                                <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-semibold bg-emerald-500/15 text-emerald-400 border border-emerald-500/25">
                                    ONLINE
                                </span>
                            ) : (
                                <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-medium bg-slate-700/40 text-slate-400 border border-slate-600/30">
                                    OFFLINE
                                </span>
                            )}
                            {(machine as { source?: string }).source === 'ssh-mesh' && (
                                <span className="text-[9px] font-mono px-1.5 py-0.2 rounded-md bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30">
                                    Mesh
                                </span>
                            )}
                            {/* Demoted IP address compact pill with 1-click copy */}
                            {machine.ip_address && (
                                <div
                                    className="inline-flex items-center gap-1 font-mono text-[10px] text-slate-400 bg-slate-800/40 px-1.5 py-0.2 rounded border border-slate-700/50"
                                    title={`Host IP: ${machine.ip_address} (Click copy icon)`}
                                >
                                    <span>{machine.ip_address}</span>
                                    <button
                                        type="button"
                                        onClick={() => onCopyIp(machine.ip_address)}
                                        className="p-0.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
                                        title="Copy IP Address"
                                    >
                                        {copiedIp === machine.ip_address ? (
                                            <Check className="w-2.5 h-2.5 text-emerald-400" />
                                        ) : (
                                            <Copy className="w-2.5 h-2.5" />
                                        )}
                                    </button>
                                </div>
                            )}
                        </div>
                        <div className="flex items-center gap-1.5 text-[10px] text-slate-400 mt-0.5 font-mono flex-wrap">
                            {cleanOsInfo && (
                                <span className="truncate max-w-[130px]" title={cleanOsInfo}>
                                    {cleanOsInfo}
                                </span>
                            )}
                            <span
                                className="text-slate-500 truncate max-w-[90px]"
                                title={`Node ID: ${machine.node_id}`}
                            >
                                #{machine.node_id.slice(0, 8)}
                            </span>
                            {(mExt.git_repo || mExt.git_link || isLeadNode || isLocalNode) && (
                                <span
                                    className="inline-flex items-center gap-1 font-mono text-[9px] text-cyan-400 bg-cyan-950/40 px-1.5 py-0.2 rounded border border-cyan-800/40"
                                    title={`Git Link: ${mExt.git_link || 'alimtvnetwork / Antigravity-Manager'}`}
                                >
                                    <GitBranch className="w-2.5 h-2.5 text-cyan-400 shrink-0" />
                                    <span className="truncate max-w-[120px]">
                                        {mExt.git_repo || 'Antigravity-Manager'}
                                    </span>
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </td>

            {/* 2. Connected Google Account Email(s) - Elevated to Primary */}
            <td className="px-3.5 py-2.5 align-middle">
                {emailList.length > 0 ? (
                    <div className="flex items-center gap-2 flex-wrap">
                        <div className="flex flex-col gap-1">
                            {emailList.map((email, i) => (
                                <span
                                    key={`${email}-${i}`}
                                    className="font-mono text-xs text-slate-200 font-medium"
                                    title={isRowUnmasked ? email : 'Masked email (click eye icon to toggle)'}
                                >
                                    {isRowUnmasked ? email : maskEmailAddress(email)}
                                </span>
                            ))}
                        </div>
                        <button
                            type="button"
                            onClick={() => onToggleMask(machine.node_id)}
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
                ) : isLeadNode ? (
                    <span className="text-amber-400/80 italic text-xs" title="Lead machine awaiting account sync">
                        Awaiting Account Sync
                    </span>
                ) : (
                    <span className="text-slate-500 italic text-xs">Unassigned</span>
                )}
            </td>

            {/* 3. Running Instance(s) */}
            <td className="px-3.5 py-2.5 align-middle">
                {machine.active_instances && machine.active_instances.length > 0 ? (
                    <div className="flex items-center gap-1.5 flex-wrap max-w-[260px]">
                        <span className="px-1.5 py-0.5 rounded text-[10px] font-bold bg-blue-500/20 text-blue-300 border border-blue-500/30">
                            {machine.active_instances.length} {machine.active_instances.length === 1 ? 'inst' : 'insts'}
                        </span>
                        {machine.active_instances.map((inst, i) => {
                            const cleanProfileName = sanitizeAdminText(inst.profile_name) || `Profile-${i + 1}`;
                            const boundEmail = inst.bound_account_email && !isAdminIdentifier(inst.bound_account_email)
                                ? (isRowUnmasked ? inst.bound_account_email : maskEmailAddress(inst.bound_account_email))
                                : null;
                            const tooltip = `Profile: ${cleanProfileName}${boundEmail ? ` (${boundEmail})` : ''}`;
                            return (
                                <span
                                    key={inst.profile_id || `${cleanProfileName}-${i}`}
                                    className="rounded-full px-2 py-0.5 text-xs bg-blue-500/15 text-blue-400 border border-blue-500/30 flex items-center gap-1 font-medium shadow-2xs"
                                    title={tooltip}
                                >
                                    <span className={`w-1.5 h-1.5 rounded-full ${inst.is_running ? 'bg-emerald-400 animate-pulse' : 'bg-blue-400'}`} />
                                    <span className="truncate max-w-[120px]">{cleanProfileName}</span>
                                </span>
                            );
                        })}
                    </div>
                ) : (
                    <span className="text-slate-500 italic text-xs">No active profiles</span>
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
                            ? new Date(toDisplayTimestamp(machine.last_heartbeat_timestamp)).toLocaleString()
                            : 'No heartbeat received'
                    }
                >
                    <Clock className="w-3 h-3 text-slate-500" />
                    <span>{formatRelativeHeartbeat(machine.last_heartbeat_timestamp)}</span>
                </span>
            </td>
        </tr>
    );
}
