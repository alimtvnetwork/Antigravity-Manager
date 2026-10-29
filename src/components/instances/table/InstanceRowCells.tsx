import { copyToClipboard } from '../../../utils/clipboard';
import { useState } from 'react';
import { Mail, Folder, Copy, Check, RotateCw, Terminal } from 'lucide-react';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';
import { maskEmail } from '../../../utils/maskEmail';
import type { Account } from '../../../types/account';
import { QuotaProgressBar } from '../../accounts/QuotaProgressBar';
import { formatPathEnding, formatShortPath, getActionLabel, resolveDataDir } from './instanceTableUtils';
import type { InstanceActionType } from './instanceTableTypes';
import type { InstanceStatus } from '../../../services/instanceService';

export interface RowCellData {
    seq: number;
    isActive: boolean;
    isDefault: boolean;
    boundAccount?: Account;
    boundEmail: string;
    isEmailRevealed: boolean;
    displayedEmail: string;
    tier: string;
    percentage: number | null;
    modelName: string | null;
    modelResetTime?: string;
    weeklyQuota: { percentage: number; resetTime?: string } | null;
    currentAction: InstanceActionType;
    isBusy: boolean;
}

export function buildRowCellData(
    inst: InstanceStatus,
    idx: number,
    activeInstanceId: string | null | undefined,
    accounts: Account[],
    currentAccount: Account | null,
    revealedEmails: Record<string, boolean>,
    actionState: Record<string, InstanceActionType> | undefined
): RowCellData {
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
    const displayedEmail = boundEmail ? (isEmailRevealed ? boundEmail : maskEmail(boundEmail)) : 'Unassigned';

    const quota = boundAccount?.quota;
    const tier = quota?.subscription_tier?.toLowerCase() || '';
    const models = quota?.models || [];
    const mainModel =
        models.find((m) => m.name.toLowerCase().includes('pro')) ||
        models.find((m) => m.name.toLowerCase().includes('flash')) ||
        models[0] ||
        null;
    const percentage = mainModel ? Math.round(mainModel.percentage) : null;
    const weeklyQuota = extractWeeklyQuota(quota);

    const currentAction = actionState?.[inst.config.id] || null;
    return {
        seq,
        isActive,
        isDefault,
        boundAccount,
        boundEmail,
        isEmailRevealed,
        displayedEmail,
        tier,
        percentage,
        modelName: mainModel?.name || null,
        modelResetTime: mainModel?.reset_time,
        weeklyQuota,
        currentAction,
        isBusy: Boolean(currentAction),
    };
}

interface WeeklyQuotaShape {
    percentage?: number;
    remaining_fraction?: number;
    reset_time?: string;
    resetTime?: string;
}

function extractWeeklyQuota(quota: Account['quota']): { percentage: number; resetTime?: string } | null {
    if (!quota) return null;
    const q = quota as unknown as {
        weekly?: WeeklyQuotaShape;
        quota_groups?: Array<{
            display_name?: string;
            buckets?: Array<{ window?: string; bucket_id?: string; remaining_fraction?: number; reset_time?: string }>;
        }>;
    };
    if (q.weekly) {
        const fraction = q.weekly.percentage ?? (q.weekly.remaining_fraction ? q.weekly.remaining_fraction * 100 : 0);
        return {
            percentage: Math.round(fraction),
            resetTime: q.weekly.reset_time || q.weekly.resetTime,
        };
    }
    const groups = q.quota_groups || [];
    const group =
        groups.find((item) => {
            const name = (item.display_name || '').toLowerCase();
            return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
        }) || groups[0];
    const bucket = (group?.buckets || []).find(
        (item) =>
            (item.window || '').toLowerCase().includes('week') ||
            (item.bucket_id || '').toLowerCase().includes('week')
    );
    if (!bucket) return null;
    return {
        percentage: Math.round((bucket.remaining_fraction || 0) * 100),
        resetTime: bucket.reset_time,
    };
}

interface ProfileCellProps {
    inst: InstanceStatus;
    data: RowCellData;
    onToggleEmail: () => void;
    onSetActive: () => void;
    onSetDefault?: () => void;
}

export function ProfileCell({ inst, data, onToggleEmail, onSetActive, onSetDefault }: ProfileCellProps) {
    return (
        <>
            <td className="px-2 py-1 min-w-[140px] max-w-[180px]">
                <div className="flex items-center gap-1.5 flex-wrap">
                    <span className="px-1.5 py-0.5 rounded-[4px] text-[10px] font-mono font-bold bg-slate-100 dark:bg-[#090611] text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-[#40285e]">
                        #{data.seq}
                    </span>
                    <span className="font-bold text-slate-800 dark:text-slate-100 text-xs truncate max-w-[110px]" title={inst.config.name}>
                        {inst.config.name}
                    </span>
                    <span
                        className={cn(
                            'w-2 h-2 rounded-full shrink-0',
                            inst.is_running ? 'bg-teal-500 animate-pulse' : 'bg-slate-300 dark:bg-slate-600'
                        )}
                        title={inst.is_running ? 'Running' : 'Idle'}
                    />
                    {data.isDefault ? (
                        <span className="px-2 py-0.5 rounded-[5px] text-[9.5px] font-extrabold bg-amber-200 text-amber-900 dark:bg-amber-950 dark:text-amber-200 border border-amber-400/90 dark:border-amber-500/70 shadow-xs tracking-wider">
                            DEFAULT
                        </span>
                    ) : onSetDefault ? (
                        <button
                            type="button"
                            disabled={data.isBusy}
                            onClick={onSetDefault}
                            className="px-1 py-0.5 rounded-[5px] text-[9px] text-slate-400 hover:text-amber-500 hover:bg-amber-500/10 transition-all duration-300 ease-out cursor-pointer disabled:opacity-40"
                            title="Set as default profile"
                        >
                            Set Default
                        </button>
                    ) : null}
                    {data.isActive ? (
                        <span className="px-2 py-0.5 rounded-[5px] text-[9.5px] font-extrabold bg-sky-700 dark:bg-sky-500 text-white border border-sky-800 dark:border-sky-400 shadow-xs tracking-wider">
                            ACTIVE
                        </span>
                    ) : (
                        <button
                            type="button"
                            disabled={data.isBusy}
                            onClick={onSetActive}
                            className="px-1 py-0.5 rounded-[5px] text-[9px] text-slate-400 hover:text-sky-500 hover:bg-sky-500/10 transition-all duration-300 ease-out cursor-pointer disabled:opacity-40"
                            title="Set as active target for account rotations"
                        >
                            Set Active
                        </button>
                    )}
                </div>
                <div className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 font-mono mt-0.5 flex-wrap">
                    <Mail className="w-3 h-3 text-slate-400 shrink-0" />
                    {data.boundEmail ? (
                        <span
                            onClick={onToggleEmail}
                            className="cursor-pointer hover:underline text-[11px] text-slate-600 dark:text-slate-300 font-mono truncate max-w-[150px] transition-colors duration-300 ease-out group-hover:text-amber-600 hover:text-amber-700 dark:group-hover:text-amber-300 dark:hover:text-amber-200 font-medium"
                            title={data.isEmailRevealed ? data.boundEmail : 'Click to unmask email'}
                        >
                            {data.displayedEmail}
                        </span>
                    ) : (
                        <span className="text-slate-400 dark:text-slate-500 italic text-[11px]" title={`ID: ${inst.config.id}`}>
                            Unassigned
                        </span>
                    )}
                    {data.tier.includes('ultra') && (
                        <span className="px-1.5 py-0.2 rounded-[5px] text-[9px] font-bold bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 border border-purple-300/40">
                            ULTRA
                        </span>
                    )}
                    {data.tier.includes('pro') && (
                        <span className="px-1.5 py-0.2 rounded-[5px] text-[9px] font-bold bg-sky-100 dark:bg-sky-950/60 text-sky-700 dark:text-sky-300 border border-sky-300/40">
                            PRO
                        </span>
                    )}
                    {data.boundAccount?.disabled ? (
                        <span className="px-1 py-0.2 rounded-[5px] text-[9px] font-semibold bg-rose-100 text-rose-700 dark:bg-rose-950/60 dark:text-rose-300 border border-rose-300/40">
                            Disabled
                        </span>
                    ) : null}
                </div>
            </td>
        </>
    );
}

export function QuotaCell({ data }: { data: RowCellData }) {
    return (
        <td className="px-2 py-1 whitespace-nowrap min-w-[190px] w-[26%] group-hover:[&_span.font-mono]:text-yellow-400 dark:group-hover:[&_span.font-mono]:text-yellow-200">
            {data.percentage !== null || data.weeklyQuota !== null ? (
                <div className="space-y-1.5 w-full min-w-[180px]">
                    {data.percentage !== null && (
                        <div className="space-y-0.5 w-full">
                            <div className="flex items-center justify-between text-[10px] font-mono">
                                <span className="truncate text-slate-600 dark:text-slate-300 font-semibold transition-colors group-hover:text-yellow-500 dark:group-hover:text-yellow-200">
                                    {data.modelName || 'Primary Model'} (4H)
                                </span>
                            </div>
                            <QuotaProgressBar className="w-full" percentage={data.percentage} resetTime={data.modelResetTime} compact />
                        </div>
                    )}
                    {data.weeklyQuota !== null && (
                        <div className={cn('space-y-0.5 w-full', data.percentage !== null && 'pt-1 border-t border-slate-200/50 dark:border-slate-800/50')}>
                            <div className="flex items-center justify-between text-[10px] font-mono">
                                <span className="truncate text-slate-500 dark:text-slate-400 transition-colors group-hover:text-yellow-500 dark:group-hover:text-yellow-200">
                                    Weekly Quota
                                </span>
                            </div>
                            <QuotaProgressBar
                                className="w-full"
                                isWeekly
                                percentage={data.weeklyQuota.percentage}
                                resetTime={data.weeklyQuota.resetTime}
                                compact
                            />
                        </div>
                    )}
                </div>
            ) : (
                <span className="text-slate-400 italic text-[11px]">—</span>
            )}
        </td>
    );
}



export interface StatusProcessEnvCellProps {
    inst: InstanceStatus;
    data: RowCellData;
    isCopied: boolean;
    onCopy: () => void;
}

export function StatusProcessEnvCell({ inst, data, isCopied, onCopy }: StatusProcessEnvCellProps) {
    const [isExeCopied, setIsExeCopied] = useState(false);
    const fullPath = resolveDataDir(inst);
    const exeFull = inst.config.executable_path || '';
    const exeTail = exeFull
        ? (formatShortPath(exeFull) || formatPathEnding(exeFull))
        : (inst.config.is_default || inst.config.id === 'default' ? '.../Antigravity.exe' : 'Default IDE');
    const exeEnding = exeTail;
    const dataEnding = fullPath ? (formatShortPath(fullPath) || formatPathEnding(fullPath)) : '—';

    const handleCopyExe = async (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            await copyToClipboard(exeFull || exeEnding);
            setIsExeCopied(true);
            setTimeout(() => setIsExeCopied(false), 1500);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceRowCells.StatusProcessEnvCell',
                triggerAction: 'copy_exe_path',
            });
        }
    };

    return (
        <td
            className="px-2 py-1 whitespace-nowrap min-w-[340px] w-[40%]"
            title={`Status: ${data.isBusy ? getActionLabel(data.currentAction) : inst.is_running ? (inst.pid ? `Running (PID: ${inst.pid})` : 'Running') : 'Idle'}\nIDE Executable: ${exeFull || exeEnding}\nData Directory: ${fullPath}`}
        >
            <div className="flex items-center justify-between gap-1.5 text-[11px] font-mono whitespace-nowrap flex-nowrap w-full">
                <div className="flex items-center gap-1.5 min-w-0 font-mono text-[11px] truncate flex-1 whitespace-nowrap flex-nowrap">
                    {/* Status Pill */}
                    {data.isBusy ? (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-blue-100 text-blue-900 dark:bg-cyan-950 dark:text-cyan-200 border border-blue-400/80 dark:border-cyan-400/80 backdrop-blur-xs animate-pulse shadow-xs shrink-0 whitespace-nowrap">
                            <RotateCw className="w-2.5 h-2.5 animate-spin text-blue-600 dark:text-cyan-300 shrink-0" />
                            <span>{getActionLabel(data.currentAction)}</span>
                        </span>
                    ) : inst.is_running ? (
                        <span
                            className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-100 text-emerald-900 dark:bg-emerald-950/80 dark:text-[#1af18d] border border-emerald-400/80 dark:border-emerald-500/60 backdrop-blur-xs shadow-xs font-mono shrink-0 whitespace-nowrap"
                            title={inst.pid ? `Process ID (PID): ${inst.pid} — Active and Running` : 'Process ID (PID): Active and Running'}
                        >
                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-600 dark:bg-[#1af18d] animate-pulse shrink-0" />
                            <span>{inst.pid ? `PID: ${inst.pid}` : 'Running'}</span>
                        </span>
                    ) : (
                        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] font-semibold bg-slate-200 text-slate-800 dark:bg-[#1c132b] dark:text-slate-200 border border-slate-300 dark:border-[#523478] backdrop-blur-xs shrink-0 whitespace-nowrap">
                            Idle
                        </span>
                    )}

                    {/* Path Endings together on the exact same line */}
                    <div className="flex items-center gap-1.5 min-w-0 flex-1 text-[11px] text-slate-700 dark:text-slate-200 font-mono truncate">
                        <div
                            className="flex items-center gap-1 min-w-0 shrink truncate font-semibold"
                            title={`IDE: ${exeFull || exeEnding}`}
                        >
                            <Terminal className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                            <span className="truncate whitespace-nowrap">{exeEnding}</span>
                        </div>
                        <span className="text-slate-400 dark:text-slate-500 shrink-0 select-none text-[10px]">·</span>
                        <div
                            className="flex items-center gap-1 min-w-0 text-[10px] text-slate-500 dark:text-slate-400 truncate"
                            title={`Data: ${fullPath}`}
                        >
                            <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                            <span className="truncate whitespace-nowrap">{dataEnding}</span>
                        </div>
                    </div>
                </div>

                {/* Right side: Segmented pill capsule with copy buttons */}
                <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 backdrop-blur-xs p-0.5 shrink-0 ml-1 shadow-2xs whitespace-nowrap flex-nowrap">
                    <button
                        type="button"
                        onClick={handleCopyExe}
                        data-ui-name="copy-exe-btn"
                        data-ui-id={`copy-exe-${inst.config.id}`}
                        className="px-1.5 py-0.5 rounded-l-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-all duration-300 ease-out cursor-pointer"
                        title={isExeCopied ? 'Copied executable path!' : 'Copy executable path'}
                    >
                        {isExeCopied ? <Check className="w-3 h-3 text-emerald-500" /> : <Terminal className="w-3 h-3" />}
                    </button>
                    <span className="w-px h-2.5 bg-slate-200 dark:bg-[#40285e] shrink-0" />
                    <button
                        type="button"
                        onClick={onCopy}
                        data-ui-name="copy-path-btn"
                        data-ui-id={`copy-path-${inst.config.id}`}
                        className="px-1.5 py-0.5 rounded-r-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-all duration-300 ease-out cursor-pointer"
                        title={isCopied ? 'Copied data path!' : 'Copy full data path'}
                    >
                        {isCopied ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                    </button>
                </div>
            </div>
        </td>
    );
}

export function StatusCell({ inst, data }: { inst: InstanceStatus; data: RowCellData }) {
    return (
        <td className="px-2 py-1 whitespace-nowrap min-w-[130px] max-w-[170px]">
            {data.isBusy ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-blue-50/90 text-blue-700 dark:bg-cyan-950/60 dark:text-cyan-300 border border-blue-300/60 dark:border-cyan-500/40 backdrop-blur-xs animate-pulse shadow-2xs">
                    <RotateCw className="w-2.5 h-2.5 animate-spin text-blue-500 dark:text-cyan-400 shrink-0" />
                    <span>{getActionLabel(data.currentAction)}</span>
                </span>
            ) : inst.is_running ? (
                <span
                    className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/30 backdrop-blur-xs shadow-2xs font-mono"
                    title={inst.pid ? `Process ID (PID): ${inst.pid} — Active and Running` : 'Process ID (PID): Active and Running'}
                >
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 dark:bg-[#1af18d] animate-pulse shrink-0" />
                    <span>{inst.pid ? `PID: ${inst.pid}` : 'Running'}</span>
                </span>
            ) : (
                <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium bg-slate-100/90 text-slate-600 dark:bg-[#090611]/90 dark:text-slate-400 border border-slate-200 dark:border-[#40285e] backdrop-blur-xs">
                    Idle
                </span>
            )}
        </td>
    );
}

interface PathCellProps {
    inst: InstanceStatus;
    isCopied: boolean;
    onCopy: () => void;
}

export interface IdeExeAndDataCellProps {
    inst: InstanceStatus;
    isCopied: boolean;
    onCopy: () => void;
}

export function IdeExeAndDataCell({ inst, isCopied, onCopy }: IdeExeAndDataCellProps) {
    const [isExeCopied, setIsExeCopied] = useState(false);
    const fullPath = resolveDataDir(inst);
    const exeFull = inst.config.executable_path || '';
    const exeTail = exeFull
        ? (exeFull.split(/[/\\]/).filter(Boolean).pop() || formatPathEnding(exeFull))
        : (inst.config.is_default || inst.config.id === 'default' ? 'Antigravity.exe' : 'Default IDE');
    const exeEnding = exeTail;
    const dataEnding = formatPathEnding(fullPath) || '—';

    const handleCopyExe = async (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            await copyToClipboard(exeFull || exeEnding);
            setIsExeCopied(true);
            setTimeout(() => setIsExeCopied(false), 1500);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceRowCells.IdeExeAndDataCell',
                triggerAction: 'copy_exe_path',
            });
        }
    };

    return (
        <td
            className="px-2 py-1 whitespace-nowrap min-w-[220px] max-w-[340px]"
            title={`IDE Executable: ${exeFull || exeEnding}\nData Directory: ${fullPath}`}
        >
            <div className="flex items-center justify-between gap-1.5 text-[11px] font-mono whitespace-nowrap flex-nowrap">
                <div className="flex items-center gap-1.5 min-w-0 font-mono text-[11px] truncate flex-1 whitespace-nowrap flex-nowrap">
                    <div
                        className="flex items-center gap-1 min-w-0 shrink-0 text-slate-700 dark:text-slate-200 font-semibold truncate max-w-[140px] whitespace-nowrap flex-nowrap"
                        title={`IDE: ${exeFull || exeEnding}`}
                    >
                        <Terminal className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                        <span className="truncate whitespace-nowrap">{exeEnding}</span>
                    </div>
                    <span className="text-slate-400 dark:text-slate-500 shrink-0 select-none text-[10px]">·</span>
                    <div
                        className="flex items-center gap-1 min-w-0 text-[10px] text-slate-500 dark:text-slate-400 truncate max-w-[150px] whitespace-nowrap flex-nowrap"
                        title={`Data: ${fullPath}`}
                    >
                        <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                        <span className="truncate whitespace-nowrap">{dataEnding}</span>
                    </div>
                </div>
                <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 backdrop-blur-xs p-0.5 shrink-0 ml-1 shadow-2xs">
                    <button
                        type="button"
                        onClick={handleCopyExe}
                        data-ui-name="copy-exe-btn"
                        data-ui-id={`copy-exe-${inst.config.id}`}
                        className="px-1.5 py-0.5 rounded-l-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                        title={isExeCopied ? 'Copied executable path!' : 'Copy executable path'}
                    >
                        {isExeCopied ? <Check className="w-3 h-3 text-emerald-500" /> : <Terminal className="w-3 h-3" />}
                    </button>
                    <span className="w-px h-2.5 bg-slate-200 dark:bg-[#40285e] shrink-0" />
                    <button
                        type="button"
                        onClick={onCopy}
                        data-ui-name="copy-path-btn"
                        data-ui-id={`copy-path-${inst.config.id}`}
                        className="px-1.5 py-0.5 rounded-r-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                        title={isCopied ? 'Copied data path!' : 'Copy full data path'}
                    >
                        {isCopied ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                    </button>
                </div>
            </div>
        </td>
    );
}

export function PathCell({ inst, isCopied, onCopy }: PathCellProps) {
    const fullPath = resolveDataDir(inst);
    const exeFull = inst.config.executable_path || '';
    const dataEnding = formatPathEnding(fullPath);
    const exeEnding = exeFull ? formatPathEnding(exeFull) : '';
    const sameLine = exeEnding ? `${dataEnding}  ·  ${exeEnding}` : dataEnding;
    return (
        <td className="px-2 py-1 whitespace-nowrap min-w-[220px] max-w-[280px]" title={exeFull ? `${fullPath}\n${exeFull}` : fullPath}>
            <div className="flex items-center justify-between gap-1 text-[11px] font-mono text-slate-500 dark:text-slate-400">
                <div className="flex items-center gap-1 min-w-0" title={fullPath}>
                    <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                    <span className="whitespace-nowrap">{sameLine}</span>
                </div>
                <button
                    type="button"
                    onClick={onCopy}
                    className="p-1 rounded-[5px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer shrink-0"
                    title={isCopied ? 'Copied!' : 'Copy full path'}
                >
                    {isCopied ? <Check className="w-3 h-3 text-teal-500" /> : <Copy className="w-3 h-3" />}
                </button>
            </div>
        </td>
    );
}

export type InstanceInfo = InstanceStatus;

export function ExePathCell({ inst }: { inst: InstanceInfo }) {
    const [isCopied, setIsCopied] = useState(false);
    const effectivePath =
        inst.config.executable_path ||
        (inst.config.is_default || inst.config.id === 'default'
            ? 'Antigravity.exe (Default)'
            : 'Default IDE');
    const shortPath = inst.config.executable_path
        ? formatPathEnding(inst.config.executable_path)
        : effectivePath;

    const handleCopy = async () => {
        try {
            await copyToClipboard(effectivePath);
            setIsCopied(true);
            setTimeout(() => setIsCopied(false), 1500);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceRowCells.ExePathCell',
                triggerAction: 'copy_exe_path',
            });
        }
    };

    return (
        <td className="px-2 py-1 whitespace-nowrap min-w-[180px] max-w-[220px]" title={effectivePath}>
            <div className="flex items-center justify-between gap-1 text-[11px] font-mono text-slate-500 dark:text-slate-400">
                <div className="flex items-center gap-1 min-w-0" title={effectivePath}>
                    <Terminal className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                    <span className="whitespace-nowrap">{shortPath}</span>
                </div>
                <button
                    type="button"
                    onClick={handleCopy}
                    className="p-1 rounded-[5px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer shrink-0"
                    title={isCopied ? 'Copied!' : 'Copy executable path'}
                >
                    {isCopied ? <Check className="w-3 h-3 text-teal-500" /> : <Copy className="w-3 h-3" />}
                </button>
            </div>
        </td>
    );
}
