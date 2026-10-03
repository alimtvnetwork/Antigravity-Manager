import { useState } from 'react';
import {
    Play,
    Square,
    RotateCcw,
    Zap,
    SlidersHorizontal,
    Copy,
    Trash2,
    Folder,
    Layers,
    Mail
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { maskEmail } from '../../utils/maskEmail';
import type { InstanceStatus } from '../../services/instanceService';
import { useAccountStore } from '../../stores/useAccountStore';

interface InstanceTableProps {
    instances: InstanceStatus[];
    activeInstanceId?: string | null;
    searchQuery?: string;
    onLaunch: (id: string) => void;
    onStop: (id: string) => void;
    onSwitch: (id: string) => void;
    onFastForward: (id: string) => void;
    onSettings: (id: string) => void;
    onClone: (id: string, name: string) => void;
    onDelete: (id: string) => void;
    onOpenPromptTree: (instanceId: string) => void;
    onSetActive: (id: string) => void;
    onSetDefault?: (id: string) => void;
}

export default function InstanceTable({
    instances,
    activeInstanceId,
    onLaunch,
    onStop,
    onSwitch,
    onFastForward,
    onSettings,
    onClone,
    onDelete,
    onOpenPromptTree,
    onSetActive,
    onSetDefault,
}: InstanceTableProps) {
    const { accounts, currentAccount } = useAccountStore();
    const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});

    const toggleEmail = (id: string) => {
        setRevealedEmails((prev) => ({ ...prev, [id]: !prev[id] }));
    };

    return (
        <div className="w-full overflow-hidden rounded-2xl border border-slate-200/80 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-xs">
            <div className="overflow-x-auto">
                <table className="w-full text-left text-xs">
                    <thead>
                        <tr className="border-b border-slate-200/70 dark:border-[#15334d] bg-slate-50/80 dark:bg-[#071a27] text-[11px] font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                            <th className="px-3.5 py-3 w-12 text-center">#</th>
                            <th className="px-3.5 py-3 min-w-[160px]">Profile Name</th>
                            <th className="px-3.5 py-3 min-w-[200px]">Bound Account</th>
                            <th className="px-3.5 py-3 min-w-[160px]">Model Quota</th>
                            <th className="px-3.5 py-3 min-w-[120px]">Status & PID</th>
                            <th className="px-3.5 py-3 min-w-[180px]">File / Data Path</th>
                            <th className="px-3.5 py-3 text-center min-w-[110px]">Prompts</th>
                            <th className="px-3.5 py-3 text-right min-w-[180px]">Actions</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-100 dark:divide-[#15334d]/60 font-medium">
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

                            return (
                                <tr
                                    key={inst.config.id}
                                    className={cn(
                                        "transition-colors hover:bg-slate-50/70 dark:hover:bg-[#102b42]",
                                        isActive && "bg-blue-50/30 dark:bg-blue-950/20"
                                    )}
                                >
                                    {/* 1. Sequence */}
                                    <td className="px-3.5 py-2.5 text-center font-mono text-[11px] text-slate-500 dark:text-slate-400">
                                        <span className="inline-flex items-center justify-center px-1.5 py-0.5 rounded-md bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d]">
                                            #{seq}
                                        </span>
                                    </td>

                                    {/* 2. Profile Name & Flags */}
                                    <td className="px-3.5 py-2.5 whitespace-nowrap">
                                        <div className="flex items-center gap-1.5 flex-wrap">
                                            <span className="font-bold text-slate-900 dark:text-slate-100 text-xs">
                                                {inst.config.name}
                                            </span>
                                            {isDefault ? (
                                                <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-amber-100 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300 border border-amber-300/40">
                                                    DEFAULT
                                                </span>
                                            ) : onSetDefault ? (
                                                <button
                                                    type="button"
                                                    onClick={() => onSetDefault(inst.config.id)}
                                                    className="text-[10px] text-slate-400 hover:text-amber-500 underline cursor-pointer"
                                                    title="Set as default profile"
                                                >
                                                    Set Default
                                                </button>
                                            ) : null}
                                            {isActive ? (
                                                <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-blue-600 text-white shadow-2xs">
                                                    ACTIVE
                                                </span>
                                            ) : (
                                                <button
                                                    type="button"
                                                    onClick={() => onSetActive(inst.config.id)}
                                                    className="text-[10px] text-slate-400 hover:text-blue-500 underline cursor-pointer"
                                                    title="Set as active target for account rotations"
                                                >
                                                    Set Active
                                                </button>
                                            )}
                                        </div>
                                        <div className="text-[10px] font-mono text-slate-400 dark:text-slate-500 mt-0.5">
                                            ID: {inst.config.id}
                                        </div>
                                    </td>

                                    {/* 3. Bound Account */}
                                    <td className="px-3.5 py-2.5 whitespace-nowrap">
                                        <div className="flex items-center gap-1.5">
                                            <Mail className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                                            {boundEmail ? (
                                                <span
                                                    onClick={() => toggleEmail(inst.config.id)}
                                                    className="font-mono text-xs text-slate-800 dark:text-slate-200 cursor-pointer hover:underline"
                                                    title={isEmailRevealed ? boundEmail : 'Click to unmask email'}
                                                >
                                                    {displayedEmail}
                                                </span>
                                            ) : (
                                                <span className="text-xs text-slate-400 italic">Unassigned</span>
                                            )}
                                            {tier.includes('ultra') && (
                                                <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 border border-purple-300/40">
                                                    ULTRA
                                                </span>
                                            )}
                                            {tier.includes('pro') && (
                                                <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-blue-100 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300 border border-blue-300/40">
                                                    PRO
                                                </span>
                                            )}
                                        </div>
                                    </td>

                                    {/* 4. Model Quota & Glow Progress */}
                                    <td className="px-3.5 py-2.5 whitespace-nowrap min-w-[160px]">
                                        {percentage !== null ? (
                                            <div className="space-y-1">
                                                <div className="flex items-center justify-between text-[10px] font-mono">
                                                    <span className="truncate max-w-[100px] text-slate-600 dark:text-slate-300">
                                                        {mainModel?.name || 'Primary Model'}
                                                    </span>
                                                    <span className="font-bold text-slate-900 dark:text-slate-100">
                                                        {percentage}%
                                                    </span>
                                                </div>
                                                <div className="h-1.5 w-full rounded-full bg-slate-100 dark:bg-[#071a27] overflow-hidden border border-slate-200/50 dark:border-[#15334d]">
                                                    <div
                                                        className={cn(
                                                            "h-full rounded-full transition-all duration-500",
                                                            percentage > 30
                                                                ? "bg-gradient-to-r from-emerald-500 via-teal-400 to-cyan-400 shadow-[0_0_8px_rgba(20,184,166,0.35)]"
                                                                : percentage > 10
                                                                ? "bg-gradient-to-r from-amber-500 to-orange-400 shadow-[0_0_8px_rgba(245,158,11,0.35)]"
                                                                : "bg-gradient-to-r from-rose-500 to-pink-500 shadow-[0_0_8px_rgba(244,63,94,0.35)]"
                                                        )}
                                                        style={{ width: `${Math.min(100, Math.max(0, percentage))}%` }}
                                                    />
                                                </div>
                                            </div>
                                        ) : (
                                            <span className="text-slate-400 italic text-[11px]">—</span>
                                        )}
                                    </td>

                                    {/* 5. Status & PID */}
                                    <td className="px-3.5 py-2.5 whitespace-nowrap">
                                        {inst.is_running ? (
                                            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold bg-emerald-50 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300 border border-emerald-300/50 dark:border-emerald-800">
                                                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                                Running {inst.pid ? `(${inst.pid})` : ''}
                                            </span>
                                        ) : (
                                            <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-slate-100 text-slate-600 dark:bg-[#071a27] dark:text-slate-400 border border-slate-200 dark:border-[#15334d]">
                                                Idle
                                            </span>
                                        )}
                                    </td>

                                    {/* 6. Executable / Data Path */}
                                    <td className="px-3.5 py-2.5 whitespace-nowrap max-w-[200px]">
                                        <div className="flex items-center gap-1.5 text-[11px] font-mono text-slate-500 dark:text-slate-400 truncate" title={inst.config.executable_path || inst.config.data_dir}>
                                            <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                                            <span className="truncate">
                                                {inst.config.executable_path || inst.config.data_dir}
                                            </span>
                                        </div>
                                    </td>

                                    {/* 7. Prompt Queue & Conversations */}
                                    <td className="px-3.5 py-2.5 text-center whitespace-nowrap">
                                        <button
                                            type="button"
                                            onClick={() => onOpenPromptTree(inst.config.id)}
                                            className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold bg-slate-100 dark:bg-[#071a27] text-slate-700 dark:text-cyan-300 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                            title="View Project and Conversation Prompt Tree"
                                        >
                                            <Layers className="w-3.5 h-3.5" />
                                            <span>Prompts</span>
                                        </button>
                                    </td>

                                    {/* 8. Actions (Segmented capsule) */}
                                    <td className="px-3.5 py-2.5 text-right whitespace-nowrap">
                                        <div className="inline-flex items-center rounded-full bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                                            {inst.is_running ? (
                                                <button
                                                    type="button"
                                                    onClick={() => onStop(inst.config.id)}
                                                    className="px-2 py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-l-full transition-colors cursor-pointer"
                                                    title="Stop Instance"
                                                >
                                                    <Square className="w-3 h-3 fill-current" />
                                                </button>
                                            ) : (
                                                <button
                                                    type="button"
                                                    onClick={() => onLaunch(inst.config.id)}
                                                    className="px-2 py-1 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40 rounded-l-full transition-colors cursor-pointer"
                                                    title="Launch Instance"
                                                >
                                                    <Play className="w-3 h-3 fill-current" />
                                                </button>
                                            )}

                                            <button
                                                type="button"
                                                onClick={() => onSwitch(inst.config.id)}
                                                className="px-2 py-1 text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/40 transition-colors cursor-pointer"
                                                title="Switch Account"
                                            >
                                                <RotateCcw className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => onFastForward(inst.config.id)}
                                                className="px-2 py-1 text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors cursor-pointer"
                                                title="Fast Forward to Best Candidate"
                                            >
                                                <Zap className="w-3 h-3 fill-current" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => onSettings(inst.config.id)}
                                                className="px-2 py-1 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                                title="Settings & Sync"
                                            >
                                                <SlidersHorizontal className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => onClone(inst.config.id, inst.config.name)}
                                                className="px-2 py-1 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 transition-colors cursor-pointer"
                                                title="Clone / Duplicate Profile"
                                            >
                                                <Copy className="w-3 h-3" />
                                            </button>

                                            {!isDefault && (
                                                <button
                                                    type="button"
                                                    onClick={() => onDelete(inst.config.id)}
                                                    disabled={inst.is_running}
                                                    className="px-2 py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-r-full transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                                    title="Delete Profile"
                                                >
                                                    <Trash2 className="w-3 h-3" />
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
