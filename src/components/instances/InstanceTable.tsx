import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import {
    Play,
    Square,
    RotateCcw,
    RotateCw,
    History,
    FastForward,
    SlidersHorizontal,
    Copy,
    Check,
    Trash2,
    Folder,
    Layers,
    Mail,
    MoreHorizontal,
    Cpu,
    ArrowLeftRight
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { maskEmail } from '../../utils/maskEmail';
import type { InstanceStatus } from '../../services/instanceService';
import { useAccountStore } from '../../stores/useAccountStore';
import { QuotaProgressBar } from '../accounts/QuotaProgressBar';

export type InstanceActionType = 'launch' | 'stop' | 'restart' | 'switch' | 'fast-forward' | 'wipe' | 'delete' | 'sync' | null;

interface InstanceTableProps {
    instances: InstanceStatus[];
    activeInstanceId?: string | null;
    searchQuery?: string;
    actionState?: Record<string, InstanceActionType>;
    onLaunch: (id: string) => void;
    onStop: (id: string) => void;
    onRestart: (id: string) => void;
    onSwitch: (id: string) => void;
    onFastForward: (id: string) => void;
    onAudit?: (id: string, name: string) => void;
    onSync?: (id: string) => void;
    syncingInstanceIds?: Record<string, boolean>;
    onSettings: (id: string) => void;
    onClone: (id: string, name: string) => void;
    onDelete: (id: string) => void;
    onOpenPromptTree: (instanceId: string) => void;
    onSetActive: (id: string) => void;
    onSetDefault?: (id: string) => void;
}

function formatShortPath(fullPath: string): string {
    if (!fullPath) return '';
    const isWindows = fullPath.includes('\\') || /^[a-zA-Z]:/.test(fullPath);
    const sep = isWindows ? '\\' : '/';
    const parts = fullPath.split(/[\\/]/).filter(Boolean);
    if (parts.length === 0) return fullPath;
    return `...${sep}${parts[parts.length - 1]}`;
}

function getActionLabel(action: InstanceActionType): string {
    switch (action) {
        case 'launch':
            return 'Launching...';
        case 'stop':
            return 'Stopping...';
        case 'restart':
            return 'Restarting...';
        case 'switch':
            return 'Switching...';
        case 'fast-forward':
            return 'Rotating...';
        case 'sync':
            return 'Syncing...';
        case 'wipe':
            return 'Wiping...';
        case 'delete':
            return 'Deleting...';
        default:
            return 'Processing...';
    }
}

export default function InstanceTable({
    instances,
    activeInstanceId,
    actionState,
    onLaunch,
    onStop,
    onRestart,
    onSwitch,
    onFastForward,
    onAudit,
    onSync,
    onSettings,
    onClone,
    onDelete,
    onOpenPromptTree,
    onSetActive,
    onSetDefault,
}: InstanceTableProps) {
    const { accounts, currentAccount } = useAccountStore();
    const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});
    const [copiedPathId, setCopiedPathId] = useState<string | null>(null);
    const [openMoreId, setOpenMoreId] = useState<string | null>(null);
    const [menuPos, setMenuPos] = useState<{ top: number; left: number } | null>(null);
    const moreMenuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (moreMenuRef.current && !moreMenuRef.current.contains(e.target as Node)) {
                setOpenMoreId(null);
            }
        };
        const handleScroll = () => {
            setOpenMoreId(null);
        };
        document.addEventListener('mousedown', handleClickOutside);
        window.addEventListener('scroll', handleScroll, true);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            window.removeEventListener('scroll', handleScroll, true);
        };
    }, []);

    const toggleEmail = (id: string) => {
        setRevealedEmails((prev) => ({ ...prev, [id]: !prev[id] }));
    };

    const handleCopyPath = (id: string, path: string) => {
        navigator.clipboard.writeText(path);
        setCopiedPathId(id);
        setTimeout(() => setCopiedPathId(null), 1500);
    };

    return (
        <div className="w-full overflow-hidden rounded-[5px] border border-slate-200/90 dark:border-slate-800/90 bg-white dark:bg-[#0c2438] shadow-xs">
            <div className="overflow-x-auto scrollbar-thin scrollbar-thumb-slate-300 dark:scrollbar-thumb-[#15334d]">
                <table className="w-full text-left text-xs">
                    <thead>
                        <tr className="border-b border-slate-200/90 dark:border-slate-800/90 bg-slate-50/80 dark:bg-[#071a27] text-[11px] font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                            <th className="px-2 py-1.5 min-w-[150px] max-w-[190px]">Profile & Account</th>
                            <th className="px-2 py-1.5 min-w-[160px] max-w-[200px]">Model & Weekly Quota</th>
                            <th className="px-2 py-1.5 min-w-[90px] max-w-[110px]">Status & PID</th>
                            <th className="px-2 py-1.5 min-w-[110px] max-w-[130px]">File / Data Path</th>
                            <th className="px-2 py-1.5 text-right w-[130px]">Actions</th>
                        </tr>
                    </thead>
                    <tbody className="font-medium">
                        {instances.map((inst, idx) => {
                            const seq = inst.config.seq_num ?? idx + 1;
                            const isActive = activeInstanceId === inst.config.id;
                            const isDefault = inst.config.is_default || inst.config.id === 'default';
                            const boundAccount = accounts.find((a) => {
                                if (inst.config.bound_account_id) return a.id === inst.config.bound_account_id;
                                if (inst.config.bound_email) return a.email.toLowerCase() === inst.config.bound_email.toLowerCase();
                                if (isActive && currentAccount) return a.email.toLowerCase() === currentAccount.email.toLowerCase();
                                return false;
                            });
                            const boundEmail = inst.config.bound_email || boundAccount?.email || '';
                            const isEmailRevealed = Boolean(revealedEmails[inst.config.id]);
                            const displayedEmail = boundEmail
                                ? (isEmailRevealed ? boundEmail : maskEmail(boundEmail))
                                : 'Unassigned';

                            // Quota extraction
                            const quota = boundAccount?.quota;
                            const tier = quota?.subscription_tier?.toLowerCase() || '';
                            const models = quota?.models || [];
                            const mainModel = models.find((m) => m.name.toLowerCase().includes('pro')) || models.find((m) => m.name.toLowerCase().includes('flash')) || models[0] || null;
                            const percentage = mainModel ? Math.round(mainModel.percentage) : null;
                            const weeklyQuota = (() => {
                                if (!quota) return null;
                                const qAny = quota as any;
                                if (qAny.weekly) {
                                    return {
                                        percentage: Math.round(qAny.weekly.percentage ?? (qAny.weekly.remaining_fraction ? qAny.weekly.remaining_fraction * 100 : 0)),
                                        resetTime: qAny.weekly.reset_time || qAny.weekly.resetTime,
                                    };
                                }
                                const groups = quota.quota_groups || [];
                                const group = groups.find((item) => {
                                    const name = (item.display_name || '').toLowerCase();
                                    return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
                                }) || groups[0];
                                const bucket = (group?.buckets || []).find((item) =>
                                    (item.window || '').toLowerCase().includes('week')
                                    || (item.bucket_id || '').toLowerCase().includes('week')
                                );
                                if (!bucket) return null;
                                return {
                                    percentage: Math.round((bucket.remaining_fraction || 0) * 100),
                                    resetTime: bucket.reset_time,
                                };
                            })();

                            const currentAction = actionState?.[inst.config.id] || null;
                            const isBusy = Boolean(currentAction);

                            return (
                                <tr
                                    key={inst.config.id}
                                    className={cn(
                                        "transition-colors duration-150 border-b border-slate-200/90 dark:border-slate-800/90 last:border-b-0",
                                        isBusy && "pointer-events-none select-none opacity-60",
                                        isActive
                                            ? "bg-sky-50/40 dark:bg-[#0a2338] hover:bg-sky-50/70 dark:hover:bg-[#0d2c46]"
                                            : "hover:bg-slate-50/80 dark:hover:bg-[#0d253a]/70"
                                    )}
                                >
                                    {/* 1. Merged Profile & Account */}
                                    <td className="px-2 py-1 min-w-[150px] max-w-[190px]">
                                        <div className="flex items-center gap-1.5 flex-wrap">
                                            <span className="px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-slate-100 dark:bg-[#071a27] text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-[#15334d]">
                                                #{seq}
                                            </span>
                                            <span className="font-bold text-slate-800 dark:text-slate-100 text-xs truncate max-w-[120px]" title={inst.config.name}>
                                                {inst.config.name}
                                            </span>
                                            <span
                                                className={cn(
                                                    "w-2 h-2 rounded-full shrink-0",
                                                    inst.is_running ? "bg-teal-500 animate-pulse" : "bg-slate-300 dark:bg-slate-600"
                                                )}
                                                title={inst.is_running ? "Running" : "Idle"}
                                            />
                                            {isDefault ? (
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-amber-100 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300 border border-amber-300/40">
                                                    DEFAULT
                                                </span>
                                            ) : onSetDefault ? (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onSetDefault(inst.config.id)}
                                                    className="px-1 py-0.5 rounded-[5px] text-[9px] text-slate-400 hover:text-amber-500 hover:bg-amber-500/10 transition-colors cursor-pointer disabled:opacity-40"
                                                    title="Set as default profile"
                                                >
                                                    Set Default
                                                </button>
                                            ) : null}
                                            {isActive ? (
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-sky-600 text-white shadow-2xs">
                                                    ACTIVE
                                                </span>
                                            ) : (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onSetActive(inst.config.id)}
                                                    className="px-1 py-0.5 rounded-[5px] text-[9px] text-slate-400 hover:text-sky-500 hover:bg-sky-500/10 transition-colors cursor-pointer disabled:opacity-40"
                                                    title="Set as active target for account rotations"
                                                >
                                                    Set Active
                                                </button>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 font-mono mt-0.5 flex-wrap">
                                            <Mail className="w-3 h-3 text-slate-400 shrink-0" />
                                            {boundEmail ? (
                                                <span
                                                    onClick={() => toggleEmail(inst.config.id)}
                                                    className="cursor-pointer hover:underline text-[11px] text-slate-500 dark:text-slate-400 font-mono truncate max-w-[150px]"
                                                    title={isEmailRevealed ? boundEmail : 'Click to unmask email'}
                                                >
                                                    {displayedEmail}
                                                </span>
                                            ) : (
                                                <span className="text-slate-400 dark:text-slate-500 italic text-[11px]" title={`ID: ${inst.config.id}`}>
                                                    Unassigned
                                                </span>
                                            )}
                                            {tier.includes('ultra') && (
                                                <span className="px-1.5 py-0.2 rounded-[5px] text-[9px] font-bold bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 border border-purple-300/40">
                                                    ULTRA
                                                </span>
                                            )}
                                            {tier.includes('pro') && (
                                                <span className="px-1.5 py-0.2 rounded-[5px] text-[9px] font-bold bg-sky-100 dark:bg-sky-950/60 text-sky-700 dark:text-sky-300 border border-sky-300/40">
                                                    PRO
                                                </span>
                                            )}
                                            {boundAccount?.disabled ? (
                                                <span className="px-1 py-0.2 rounded-[5px] text-[9px] font-semibold bg-rose-100 text-rose-700 dark:bg-rose-950/60 dark:text-rose-300 border border-rose-300/40">
                                                    Disabled
                                                </span>
                                            ) : null}
                                        </div>
                                    </td>

                                    {/* 2. Model Quota & Weekly Quota */}
                                    <td className="px-2 py-1 whitespace-nowrap min-w-[150px] max-w-[190px]">
                                        {(percentage !== null || weeklyQuota !== null) ? (
                                            <div className="space-y-1 min-w-[150px]">
                                                {percentage !== null && (
                                                    <div className="space-y-0.5">
                                                        <div className="flex items-center justify-between text-[10px] font-mono">
                                                            <span className="truncate max-w-[95px] text-slate-600 dark:text-slate-300 font-semibold">
                                                                {mainModel?.name || 'Primary Model'} (4H)
                                                            </span>
                                                        </div>
                                                        <QuotaProgressBar
                                                            percentage={percentage}
                                                            resetTime={mainModel?.reset_time}
                                                        />
                                                    </div>
                                                )}
                                                {weeklyQuota !== null && (
                                                    <div className={cn("space-y-0.5", percentage !== null && "pt-1 border-t border-slate-200/50 dark:border-slate-800/50")}>
                                                        <div className="flex items-center justify-between text-[10px] font-mono">
                                                            <span className="truncate max-w-[95px] text-slate-500 dark:text-slate-400">
                                                                Weekly Quota
                                                            </span>
                                                        </div>
                                                        <QuotaProgressBar
                                                            isWeekly
                                                            percentage={weeklyQuota.percentage}
                                                            resetTime={weeklyQuota.resetTime}
                                                        />
                                                    </div>
                                                )}
                                            </div>
                                        ) : (
                                            <span className="text-slate-400 italic text-[11px]">—</span>
                                        )}
                                    </td>

                                    {/* 3. Status & PID */}
                                    <td className="px-2 py-1 whitespace-nowrap min-w-[90px] max-w-[110px]">
                                        {isBusy ? (
                                            <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[11px] font-bold bg-blue-50 text-blue-700 dark:bg-cyan-950/60 dark:text-cyan-300 border border-blue-300/60 dark:border-cyan-500/40 animate-pulse shadow-2xs">
                                                <RotateCw className="w-3 h-3 animate-spin text-blue-500 dark:text-cyan-400 shrink-0" />
                                                <span>{getActionLabel(currentAction)}</span>
                                            </span>
                                        ) : inst.is_running ? (
                                            <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[11px] font-semibold bg-teal-50 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border border-teal-300/50 dark:border-teal-800/80">
                                                <span className="w-1.5 h-1.5 rounded-full bg-teal-500 animate-pulse" />
                                                Running {inst.pid ? `(${inst.pid})` : ''}
                                            </span>
                                        ) : (
                                            <span className="inline-flex items-center px-2 py-0.5 rounded-[5px] text-[11px] font-medium bg-slate-100 text-slate-600 dark:bg-[#071a27] dark:text-slate-400 border border-slate-200 dark:border-[#15334d]">
                                                Idle
                                            </span>
                                        )}
                                    </td>

                                    {/* 4. Executable / Data Path */}
                                    <td
                                        className="px-2 py-1 whitespace-nowrap min-w-[110px] max-w-[130px]"
                                        title={(inst as any).data_dir || inst.config.data_dir || inst.config.executable_path}
                                    >
                                        {(() => {
                                            const fullPath = (inst as any).data_dir || inst.config.data_dir || inst.config.executable_path || '';
                                            const shortPath = formatShortPath(fullPath);
                                            const isCopied = copiedPathId === inst.config.id;
                                            return (
                                                <div className="flex items-center justify-between gap-1 text-[11px] font-mono text-slate-500 dark:text-slate-400">
                                                    <div
                                                        className="flex items-center gap-1 min-w-0 truncate"
                                                        title={(inst as any).data_dir || inst.config.data_dir || fullPath}
                                                    >
                                                        <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                                                        <span className="truncate">{shortPath}</span>
                                                    </div>
                                                    <button
                                                        type="button"
                                                        onClick={() => handleCopyPath(inst.config.id, (inst as any).data_dir || inst.config.data_dir || fullPath)}
                                                        className="p-1 rounded-[5px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer shrink-0"
                                                        title={isCopied ? "Copied!" : "Copy full path"}
                                                    >
                                                        {isCopied ? <Check className="w-3 h-3 text-teal-500" /> : <Copy className="w-3 h-3" />}
                                                    </button>
                                                </div>
                                            );
                                        })()}
                                    </td>

                                    {/* 5. Actions: 3 Primary + More Dropdown */}
                                    <td className="px-2 py-1 text-right whitespace-nowrap w-[130px]">
                                        <div className="inline-flex items-center rounded-[5px] overflow-hidden bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                                            {/* Primary 1: Launch / Stop */}
                                            {/* Primary 1: Split Stop / Restart when running, or Launch when stopped */}
                                            {inst.is_running ? (
                                                <div className="inline-flex items-center rounded-l-[5px] overflow-hidden">
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => onStop(inst.config.id)}
                                                        className="px-2 py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer disabled:opacity-50"
                                                        title="Stop Instance"
                                                    >
                                                        {currentAction === 'stop' ? (
                                                            <RotateCw className="w-3 h-3 animate-spin text-rose-500" />
                                                        ) : (
                                                            <Square className="w-3 h-3 fill-current" />
                                                        )}
                                                    </button>
                                                    <div className="w-px h-3.5 bg-slate-300 dark:bg-slate-700/80 my-auto" />
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => onRestart(inst.config.id)}
                                                        className="px-2 py-1 text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors cursor-pointer disabled:opacity-50"
                                                        title="Restart Instance on Current Account"
                                                    >
                                                        {currentAction === 'restart' ? (
                                                            <RotateCw className="w-3 h-3 animate-spin text-amber-500" />
                                                        ) : (
                                                            <RotateCcw className="w-3 h-3" />
                                                        )}
                                                    </button>
                                                </div>
                                            ) : (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onLaunch(inst.config.id)}
                                                    className="px-2 py-1 text-teal-600 dark:text-teal-400 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-l-[5px] transition-colors cursor-pointer disabled:opacity-50"
                                                    title="Launch Instance"
                                                >
                                                    {currentAction === 'launch' ? (
                                                        <RotateCw className="w-3 h-3 animate-spin text-teal-500" />
                                                    ) : (
                                                        <Play className="w-3 h-3 fill-current" />
                                                    )}
                                                </button>
                                            )}

                                            {/* Primary 2: Switch Account */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => onSwitch(inst.config.id)}
                                                className="px-2 py-1 text-sky-600 dark:text-sky-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                title="Switch Account"
                                            >
                                                {currentAction === 'switch' ? (
                                                    <RotateCw className="w-3 h-3 animate-spin text-sky-500" />
                                                ) : (
                                                    <ArrowLeftRight className="w-3 h-3" />
                                                )}
                                            </button>

                                            {/* Primary 3: Fast Forward */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => onFastForward(inst.config.id)}
                                                className="px-2 py-1 text-amber-600 dark:text-amber-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                title="Fast Forward to Best Candidate"
                                            >
                                                {currentAction === 'fast-forward' ? (
                                                    <RotateCw className="w-3 h-3 animate-spin text-amber-500" />
                                                ) : (
                                                    <FastForward className="w-3 h-3 fill-current" />
                                                )}
                                            </button>

                                            {/* Primary 4: More Dropdown Button */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    if (openMoreId === inst.config.id) {
                                                        setOpenMoreId(null);
                                                    } else {
                                                        const rect = e.currentTarget.getBoundingClientRect();
                                                        setMenuPos({ top: rect.bottom + 4, left: rect.right });
                                                        setOpenMoreId(inst.config.id);
                                                    }
                                                }}
                                                className={cn(
                                                    "px-2 py-1 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-r-[5px] transition-colors cursor-pointer",
                                                    openMoreId === inst.config.id && "bg-slate-200 dark:bg-[#15334d]"
                                                )}
                                                title="More Actions (Prompts, Audit, Sync, Settings, Clone, Delete)"
                                            >
                                                <MoreHorizontal className="w-3 h-3" />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>

            {/* More Menu Dropdown Portal */}
            {openMoreId && menuPos && createPortal(
                (() => {
                    const inst = instances.find((i) => i.config.id === openMoreId);
                    if (!inst) return null;
                    const isDefault = inst.config.is_default || inst.config.id === 'default';
                    return (
                        <div
                            ref={moreMenuRef}
                            className="fixed z-[10000] min-w-[180px] rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] py-1 text-slate-800 dark:text-slate-200 shadow-xl text-xs"
                            style={{ top: menuPos.top, left: menuPos.left, transform: 'translateX(-100%)' }}
                            onClick={(e) => e.stopPropagation()}
                        >
                            <button
                                type="button"
                                onClick={() => { setOpenMoreId(null); onOpenPromptTree(inst.config.id); }}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-slate-700 dark:text-slate-200 cursor-pointer"
                            >
                                <Layers className="w-3.5 h-3.5 text-cyan-600 dark:text-cyan-400" />
                                <span>Prompts & History</span>
                            </button>
                            {onAudit && (
                                <button
                                    type="button"
                                    onClick={() => { setOpenMoreId(null); onAudit(inst.config.id, inst.config.name); }}
                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-slate-700 dark:text-slate-200 cursor-pointer"
                                >
                                    <History className="w-3.5 h-3.5 text-amber-500" />
                                    <span>Audit Trail</span>
                                </button>
                            )}
                            {onSync && (
                                <button
                                    type="button"
                                    onClick={() => { setOpenMoreId(null); onSync(inst.config.id); }}
                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-teal-600 dark:text-teal-400 cursor-pointer"
                                >
                                    <Cpu className="w-3.5 h-3.5 text-teal-500" />
                                    <span>Sync PID & Quota</span>
                                </button>
                            )}
                            {inst.is_running && (
                                <button
                                    type="button"
                                    onClick={() => { setOpenMoreId(null); onRestart(inst.config.id); }}
                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-amber-600 dark:text-amber-400 cursor-pointer"
                                >
                                    <RotateCcw className="w-3.5 h-3.5 text-amber-500" />
                                    <span>Restart Instance</span>
                                </button>
                            )}
                            <button
                                type="button"
                                onClick={() => { setOpenMoreId(null); onSettings(inst.config.id); }}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-slate-700 dark:text-slate-200 cursor-pointer"
                            >
                                <SlidersHorizontal className="w-3.5 h-3.5 text-blue-500" />
                                <span>Settings & Sync</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => { setOpenMoreId(null); onClone(inst.config.id, inst.config.name); }}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-indigo-600 dark:text-indigo-400 cursor-pointer"
                            >
                                <Copy className="w-3.5 h-3.5 text-indigo-500" />
                                <span>Clone Profile</span>
                            </button>
                            {!isDefault && (
                                <button
                                    type="button"
                                    disabled={inst.is_running}
                                    onClick={() => { setOpenMoreId(null); onDelete(inst.config.id); }}
                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-rose-50 dark:hover:bg-rose-950/40 text-rose-600 dark:text-rose-400 disabled:opacity-40 cursor-pointer"
                                >
                                    <Trash2 className="w-3.5 h-3.5" />
                                    <span>Delete Profile</span>
                                </button>
                            )}
                        </div>
                    );
                })(),
                document.body
            )}
        </div>
    );
}
