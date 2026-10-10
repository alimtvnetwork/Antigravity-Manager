import { Mail, Folder, Copy, Check, RotateCw } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { maskEmail } from '../../../utils/maskEmail';
import type { Account } from '../../../types/account';
import { QuotaProgressBar } from '../../accounts/QuotaProgressBar';
import { formatShortPath, getActionLabel, resolveDataDir } from './instanceTableUtils';
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
                    <span className="px-1.5 py-0.5 rounded-[4px] text-[10px] font-mono font-bold bg-slate-100 dark:bg-[#071a27] text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-[#15334d]">
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
                        <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-amber-100 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300 border border-amber-300/40">
                            DEFAULT
                        </span>
                    ) : onSetDefault ? (
                        <button
                            type="button"
                            disabled={data.isBusy}
                            onClick={onSetDefault}
                            className="px-1 py-0.5 rounded-[5px] text-[9px] text-slate-400 hover:text-amber-500 hover:bg-amber-500/10 transition-colors cursor-pointer disabled:opacity-40"
                            title="Set as default profile"
                        >
                            Set Default
                        </button>
                    ) : null}
                    {data.isActive ? (
                        <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-sky-600 text-white shadow-2xs">
                            ACTIVE
                        </span>
                    ) : (
                        <button
                            type="button"
                            disabled={data.isBusy}
                            onClick={onSetActive}
                            className="px-1 py-0.5 rounded-[5px] text-[9px] text-slate-400 hover:text-sky-500 hover:bg-sky-500/10 transition-colors cursor-pointer disabled:opacity-40"
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
                            className="cursor-pointer hover:underline text-[11px] text-slate-500 dark:text-slate-400 font-mono truncate max-w-[150px]"
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
        <td className="px-2 py-1 whitespace-nowrap min-w-[150px] max-w-[190px]">
            {data.percentage !== null || data.weeklyQuota !== null ? (
                <div className="space-y-1 min-w-[150px]">
                    {data.percentage !== null && (
                        <div className="space-y-0.5">
                            <div className="flex items-center justify-between text-[10px] font-mono">
                                <span className="truncate max-w-[95px] text-slate-600 dark:text-slate-300 font-semibold">
                                    {data.modelName || 'Primary Model'} (4H)
                                </span>
                            </div>
                            <QuotaProgressBar percentage={data.percentage} resetTime={data.modelResetTime} />
                        </div>
                    )}
                    {data.weeklyQuota !== null && (
                        <div className={cn('space-y-0.5', data.percentage !== null && 'pt-1 border-t border-slate-200/50 dark:border-slate-800/50')}>
                            <div className="flex items-center justify-between text-[10px] font-mono">
                                <span className="truncate max-w-[95px] text-slate-500 dark:text-slate-400">
                                    Weekly Quota
                                </span>
                            </div>
                            <QuotaProgressBar
                                isWeekly
                                percentage={data.weeklyQuota.percentage}
                                resetTime={data.weeklyQuota.resetTime}
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

export function StatusCell({ inst, data }: { inst: InstanceStatus; data: RowCellData }) {
    return (
        <td className="px-2 py-1 whitespace-nowrap min-w-[85px] max-w-[105px]">
            {data.isBusy ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-blue-50/90 text-blue-700 dark:bg-cyan-950/60 dark:text-cyan-300 border border-blue-300/60 dark:border-cyan-500/40 backdrop-blur-xs animate-pulse shadow-2xs">
                    <RotateCw className="w-2.5 h-2.5 animate-spin text-blue-500 dark:text-cyan-400 shrink-0" />
                    <span>{getActionLabel(data.currentAction)}</span>
                </span>
            ) : inst.is_running ? (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-teal-50/90 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border border-teal-300/50 dark:border-teal-800/80 backdrop-blur-xs">
                    <span className="w-1.5 h-1.5 rounded-full bg-teal-500 animate-pulse" />
                    Running{inst.pid ? ` · ${inst.pid}` : ''}
                </span>
            ) : (
                <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium bg-slate-100/90 text-slate-600 dark:bg-[#071a27]/90 dark:text-slate-400 border border-slate-200 dark:border-[#15334d] backdrop-blur-xs">
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

export function PathCell({ inst, isCopied, onCopy }: PathCellProps) {
    const fullPath = resolveDataDir(inst);
    const shortPath = formatShortPath(fullPath);
    return (
        <td className="px-2 py-1 whitespace-nowrap min-w-[110px] max-w-[130px]" title={fullPath}>
            <div className="flex items-center justify-between gap-1 text-[11px] font-mono text-slate-500 dark:text-slate-400">
                <div className="flex items-center gap-1 min-w-0 truncate" title={fullPath}>
                    <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                    <span className="truncate">{shortPath}</span>
                </div>
                <button
                    type="button"
                    onClick={onCopy}
                    className="p-1 rounded-[5px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer shrink-0"
                    title={isCopied ? 'Copied!' : 'Copy full path'}
                >
                    {isCopied ? <Check className="w-3 h-3 text-teal-500" /> : <Copy className="w-3 h-3" />}
                </button>
            </div>
        </td>
    );
}
