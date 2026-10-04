import { useState } from 'react';
import {
    Play,
    Square,
    RotateCcw,
    RotateCw,
    History,
    Zap,
    SlidersHorizontal,
    Copy,
    Check,
    Trash2,
    Folder,
    Layers,
    Mail
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { maskEmail } from '../../utils/maskEmail';
import type { InstanceStatus } from '../../services/instanceService';
import { useAccountStore } from '../../stores/useAccountStore';
import { WaterDrainProgressBar } from '../common/WaterDrainProgressBar';

export type InstanceActionType = 'launch' | 'stop' | 'switch' | 'fast-forward' | 'wipe' | 'delete' | 'sync' | null;

interface InstanceTableProps {
    instances: InstanceStatus[];
    activeInstanceId?: string | null;
    searchQuery?: string;
    actionState?: Record<string, InstanceActionType>;
    onLaunch: (id: string) => void;
    onStop: (id: string) => void;
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
    if (parts.length <= 2) return fullPath;
    return `...${sep}${parts.slice(-2).join(sep)}`;
}


export default function InstanceTable({
    instances,
    activeInstanceId,
    actionState,
    onLaunch,
    onStop,
    onSwitch,
    onFastForward,
    onAudit,
    onSync,
    syncingInstanceIds,
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

    const toggleEmail = (id: string) => {
        setRevealedEmails((prev) => ({ ...prev, [id]: !prev[id] }));
    };

    const handleCopyPath = (id: string, path: string) => {
        navigator.clipboard.writeText(path);
        setCopiedPathId(id);
        setTimeout(() => setCopiedPathId(null), 1500);
    };

    return (
        <div className="w-full overflow-hidden rounded-xl border border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-[#0c2438] shadow-xs">
            <div className="overflow-x-auto">
                <table className="w-full text-left text-xs">
                    <thead>
                        <tr className="border-b border-slate-200/80 dark:border-slate-800/80 bg-slate-50/80 dark:bg-[#071a27] text-[11px] font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                            <th className="px-2.5 py-2.5 w-10 text-center">#</th>
                            <th className="px-3 py-2.5 min-w-[170px]">Profile & Account</th>
                            <th className="px-3 py-2.5 min-w-[130px]">Model Quota</th>
                            <th className="px-2.5 py-2.5 min-w-[100px]">Status & PID</th>
                            <th className="px-3 py-2.5 min-w-[150px]">File / Data Path</th>
                            <th className="px-3 py-2.5 text-right min-w-[210px]">Actions</th>
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

                            const currentAction = actionState?.[inst.config.id] || null;
                            const isBusy = Boolean(currentAction);

                            return (
                                <tr
                                    key={inst.config.id}
                                    className={cn(
                                        "transition-colors duration-150 border-b border-slate-200/80 dark:border-slate-800/80 last:border-b-0",
                                        isBusy && "opacity-75",
                                        isActive
                                            ? "bg-sky-50/40 dark:bg-[#0a2338] hover:bg-sky-50/70 dark:hover:bg-[#0d2c46]"
                                            : "hover:bg-slate-50/80 dark:hover:bg-[#0d253a]/70"
                                    )}
                                >
                                    {/* 1. Sequence */}
                                    <td className="px-2.5 py-2 text-center font-mono text-[11px] text-slate-500 dark:text-slate-400">
                                        <span className="inline-flex items-center justify-center px-1.5 py-0.5 rounded-[5px] bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d]">
                                            #{seq}
                                        </span>
                                    </td>

                                    {/* 2. Merged Profile & Account */}
                                    <td className="px-3 py-2 whitespace-nowrap min-w-[170px]">
                                        <div className="flex items-center gap-1.5 flex-wrap">
                                            <span className="font-semibold text-slate-800 dark:text-slate-100 text-xs">
                                                {inst.config.name}
                                            </span>
                                            {isDefault ? (
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-amber-100 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300 border border-amber-300/40">
                                                    DEFAULT
                                                </span>
                                            ) : onSetDefault ? (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onSetDefault(inst.config.id)}
                                                    className="px-1.5 py-0.5 rounded-[5px] text-[10px] text-slate-400 hover:text-amber-500 hover:bg-amber-500/10 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
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
                                                    className="px-1.5 py-0.5 rounded-[5px] text-[10px] text-slate-400 hover:text-sky-500 hover:bg-sky-500/10 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                                                    title="Set as active target for account rotations"
                                                >
                                                    Set Active
                                                </button>
                                            )}
                                            {tier.includes('ultra') && (
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 border border-purple-300/40">
                                                    ULTRA
                                                </span>
                                            )}
                                            {tier.includes('pro') && (
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-sky-100 dark:bg-sky-950/60 text-sky-700 dark:text-sky-300 border border-sky-300/40">
                                                    PRO
                                                </span>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 font-mono mt-0.5">
                                            <span
                                                className={cn(
                                                    "w-1.5 h-1.5 rounded-full shrink-0",
                                                    !boundEmail
                                                        ? "bg-slate-300 dark:bg-slate-600"
                                                        : boundAccount?.disabled
                                                        ? "bg-rose-500"
                                                        : boundAccount?.proxy_disabled
                                                        ? "bg-amber-500"
                                                        : "bg-teal-500 dark:bg-cyan-400"
                                                )}
                                                title={
                                                    !boundEmail
                                                        ? "Unassigned"
                                                        : boundAccount?.disabled
                                                        ? "Account disabled"
                                                        : boundAccount?.proxy_disabled
                                                        ? "Proxy disabled"
                                                        : "Account active"
                                                }
                                            />
                                            <Mail className="w-3 h-3 text-slate-400 shrink-0" />
                                            {boundEmail ? (
                                                <span
                                                    onClick={() => toggleEmail(inst.config.id)}
                                                    className="cursor-pointer hover:underline text-slate-600 dark:text-slate-300 truncate max-w-[170px]"
                                                    title={isEmailRevealed ? boundEmail : 'Click to unmask email'}
                                                >
                                                    {displayedEmail}
                                                </span>
                                            ) : (
                                                <span className="text-slate-400 dark:text-slate-500 italic text-[11px]" title={`ID: ${inst.config.id}`}>
                                                    Unassigned
                                                </span>
                                            )}
                                            {boundAccount?.disabled ? (
                                                <span className="px-1 py-0.2 rounded-[5px] text-[9px] font-semibold bg-rose-100 text-rose-700 dark:bg-rose-950/60 dark:text-rose-300 border border-rose-300/40">
                                                    Disabled
                                                </span>
                                            ) : null}
                                        </div>
                                    </td>

                                    {/* 3. Model Quota & Glow Progress */}
                                    <td className="px-3 py-2 whitespace-nowrap min-w-[130px]">
                                        {percentage !== null ? (
                                            <div className="space-y-1">
                                                <div className="flex items-center justify-between text-[10px] font-mono">
                                                    <span className="truncate max-w-[90px] text-slate-600 dark:text-slate-300">
                                                        {mainModel?.name || 'Primary Model'}
                                                    </span>
                                                    <span className="font-bold text-slate-900 dark:text-slate-100">
                                                        {percentage}%
                                                    </span>
                                                </div>
                                                <WaterDrainProgressBar percentage={percentage} />
                                            </div>
                                        ) : (
                                            <span className="text-slate-400 italic text-[11px]">—</span>
                                        )}
                                    </td>

                                    {/* 4. Status & PID */}
                                    <td className="px-2.5 py-2 whitespace-nowrap min-w-[100px]">
                                        {inst.is_running ? (
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

                                    {/* 5. Executable / Data Path (Truncated ending path with copy button) */}
                                    <td
                                        className="px-3 py-2 whitespace-nowrap min-w-[150px] max-w-[190px]"
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

                                    {/* 6. Actions (Segmented capsule with 5-6px rounded buttons) */}
                                    <td className="px-3 py-2 text-right whitespace-nowrap min-w-[210px]">
                                        <div className="inline-flex items-center rounded-[5px] overflow-hidden bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                                            {inst.is_running ? (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onStop(inst.config.id)}
                                                    className="px-2 py-1 text-rose-600 dark:text-rose-400 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-l-[5px] transition-colors cursor-pointer disabled:opacity-50"
                                                    title="Stop Instance"
                                                >
                                                    {currentAction === 'stop' ? (
                                                        <RotateCw className="w-3 h-3 animate-spin text-rose-500" />
                                                    ) : (
                                                        <Square className="w-3 h-3 fill-current" />
                                                    )}
                                                </button>
                                            ) : (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onLaunch(inst.config.id)}
                                                    className="px-2 py-1 text-teal-600 dark:text-teal-400 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-l-[5px] transition-colors cursor-pointer disabled:opacity-50"
                                                    title="Launch Instance"
                                                >
                                                    {currentAction === 'launch' ? (
                                                        <RotateCw className="w-3 h-3 animate-spin text-emerald-500" />
                                                    ) : (
                                                        <Play className="w-3 h-3 fill-current" />
                                                    )}
                                                </button>
                                            )}

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
                                                    <RotateCcw className="w-3 h-3" />
                                                )}
                                            </button>

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
                                                    <Zap className="w-3 h-3 fill-current" />
                                                )}
                                            </button>

                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => onOpenPromptTree(inst.config.id)}
                                                className="px-2 py-1 text-slate-600 dark:text-slate-300 hover:text-cyan-600 dark:hover:text-cyan-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                title="Prompts & Conversations"
                                            >
                                                <Layers className="w-3 h-3" />
                                            </button>

                                            {onAudit && (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => onAudit(inst.config.id, inst.config.name)}
                                                    className="px-2 py-1 text-slate-500 hover:text-amber-500 dark:text-slate-400 dark:hover:text-amber-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                    title="Audit Trail"
                                                >
                                                    <History className="w-3 h-3" />
                                                </button>
                                            )}

                                            {onSync && (
                                                <button
                                                    type="button"
                                                    onClick={() => onSync(inst.config.id)}
                                                    disabled={isBusy || Boolean(syncingInstanceIds?.[inst.config.id])}
                                                    className="px-2 py-1 text-teal-600 dark:text-teal-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                    title="Sync PID and Quota"
                                                >
                                                    <RotateCw className={cn("w-3 h-3 text-teal-500", (currentAction === 'sync' || syncingInstanceIds?.[inst.config.id]) && "animate-spin")} />
                                                </button>
                                            )}

                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => onSettings(inst.config.id)}
                                                className="px-2 py-1 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                title="Settings & Sync"
                                            >
                                                <SlidersHorizontal className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => onClone(inst.config.id, inst.config.name)}
                                                className="px-2 py-1 text-indigo-600 dark:text-indigo-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50"
                                                title="Clone / Duplicate Profile"
                                            >
                                                <Copy className="w-3 h-3" />
                                            </button>

                                            {!isDefault && (
                                                <button
                                                    type="button"
                                                    onClick={() => onDelete(inst.config.id)}
                                                    disabled={inst.is_running || isBusy}
                                                    className="px-2 py-1 text-rose-600 dark:text-rose-400 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-r-[5px] transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                                    title="Delete Profile"
                                                >
                                                    {currentAction === 'delete' ? (
                                                        <RotateCw className="w-3 h-3 animate-spin text-rose-500" />
                                                    ) : (
                                                        <Trash2 className="w-3 h-3" />
                                                    )}
                                                </button>
                                            )}
                                        </div>
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
