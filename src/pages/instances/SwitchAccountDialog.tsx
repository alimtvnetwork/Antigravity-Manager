import { useState, useEffect, useCallback } from 'react';
import { ArrowRightLeft, Search, RotateCw, X, Check, Eye, EyeOff } from 'lucide-react';
import { cn } from '../../utils/cn';
import { maskEmail } from '../../utils/maskEmail';
import { showToast } from '../../components/common/ToastContainer';
import { findQuotaModel } from '../../config/modelConfig';
import { QuotaProgressBar } from '../../components/accounts/QuotaProgressBar';
import type { Account, QuotaBucket, QuotaGroup } from '../../types/account';
import type { InstancePageApi } from './instancePageTypes';

function readWeeklyQuota(groups: QuotaGroup[] | undefined): { percentage: number | null; resetTime?: string } {
    if (!groups?.length) return { percentage: null };
    const group =
        groups.find((item) => {
            const name = (item.display_name || '').toLowerCase();
            return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
        }) || groups[0];
    const bucket = (group?.buckets || []).find((item: QuotaBucket) => {
        const windowName = (item.window || '').toLowerCase();
        const bucketId = (item.bucket_id || '').toLowerCase();
        return windowName.includes('week') || bucketId.includes('week');
    });
    if (!bucket || typeof bucket.remaining_fraction !== 'number' || !Number.isFinite(bucket.remaining_fraction)) {
        return { percentage: null, resetTime: bucket?.reset_time };
    }
    return {
        percentage: Math.min(100, Math.max(0, bucket.remaining_fraction * 100)),
        resetTime: bucket.reset_time,
    };
}

function accountQuotaScores(acc: Account): {
    fourH: number | null;
    fourHResetTime?: string;
    weekly: number | null;
    weeklyResetTime?: string;
    modelName: string | null;
} {
    if (acc.disabled || acc.quota?.is_forbidden) {
        return { fourH: null, weekly: null, modelName: null };
    }
    const proModel = findQuotaModel(acc.quota?.models, 'gemini-pro');
    const flashModel = findQuotaModel(acc.quota?.models, 'gemini-flash');
    const model = proModel || flashModel || acc.quota?.models?.[0];
    const raw4h = model && Number.isFinite(model.percentage) ? model.percentage : null;
    const fourH = raw4h === null ? null : Math.min(100, Math.max(0, raw4h));
    const weeklyData = readWeeklyQuota(acc.quota?.quota_groups);
    return {
        fourH,
        fourHResetTime: model?.reset_time,
        weekly: weeklyData.percentage,
        weeklyResetTime: weeklyData.resetTime,
        modelName: model?.name || null,
    };
}

function sortScore(value: number | null): number {
    return value === null || !Number.isFinite(value) ? 0 : value;
}

export function SwitchAccountDialog({ api }: { api: InstancePageApi }) {
    const {
        switchTargetInstance,
        setSwitchTargetInstance,
        accountSearchQuery,
        setAccountSearchQuery,
        accounts,
        actionState,
        isLoading,
        setActionState,
        setActionError,
        switchAccountToInstance,
    } = api;

    const [switchingAccountId, setSwitchingAccountId] = useState<string | null>(null);
    const [showAllEmails, setShowAllEmails] = useState(false);
    const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});

    const isEmailVisible = (id: string) => showAllEmails || Boolean(revealedEmails[id]);
    const toggleEmail = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setRevealedEmails((prev) => ({ ...prev, [id]: !prev[id] }));
    };

    if (!switchTargetInstance) return null;

    const targetId = switchTargetInstance.config.id;
    const isSwitching = actionState[targetId] === 'switch' || switchingAccountId !== null;
    const isAnySwitching = isSwitching || isLoading;

    const boundAccount = accounts.find((a) => {
        if (switchTargetInstance.config.bound_account_id) return a.id === switchTargetInstance.config.bound_account_id;
        if (switchTargetInstance.config.bound_email) return a.email.toLowerCase() === switchTargetInstance.config.bound_email.toLowerCase();
        return false;
    });
    const boundQuota = boundAccount ? accountQuotaScores(boundAccount) : null;

    const close = useCallback(() => {
        if (isAnySwitching) return;
        setSwitchTargetInstance(null);
        setAccountSearchQuery('');
        setSwitchingAccountId(null);
    }, [isAnySwitching, setAccountSearchQuery, setSwitchTargetInstance]);

    const filtered = accounts
        .filter((a) => {
            const q = accountSearchQuery.toLowerCase().trim();
            if (!q) return true;
            return a.email.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
        })
        .sort((a, b) => {
            const aQuota = accountQuotaScores(a);
            const bQuota = accountQuotaScores(b);
            const aTotal = sortScore(aQuota.fourH) + sortScore(aQuota.weekly);
            const bTotal = sortScore(bQuota.fourH) + sortScore(bQuota.weekly);
            return bTotal - aTotal;
        });

    const handleSelect = async (acc: Account) => {
        if (isAnySwitching) return;
        setSwitchingAccountId(acc.id);
        setActionState((prev) => ({ ...prev, [targetId]: 'switch' }));
        setActionError(null);
        try {
            await switchAccountToInstance(acc.id, targetId);
            showToast(`Switched ${switchTargetInstance.config.name} to ${acc.email}`, 'success');
            close();
        } catch (e: unknown) {
            const msg = (e as Error)?.toString?.() || 'Failed to switch account';
            setActionError(msg);
            showToast(msg, 'error');
        } finally {
            setActionState((prev) => ({ ...prev, [targetId]: null }));
            setSwitchingAccountId(null);
        }
    };

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !isAnySwitching) {
                close();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [close, isAnySwitching]);

    return (
        <div
            className="fixed inset-0 bg-black/70 backdrop-blur-md flex items-center justify-center z-[99999] p-4 overflow-y-auto animate-in fade-in duration-200"
            role="dialog"
            aria-modal="true"
            aria-labelledby="switch-account-modal-title"
            onClick={(e) => {
                if (e.target === e.currentTarget && !isAnySwitching) {
                    close();
                }
            }}
        >
            <div
                className="backdrop-blur-md bg-white/95 dark:bg-[var(--ui-surface-1)]/95 border border-slate-200/90 dark:border-[var(--ui-border-subtle)] shadow-2xl rounded-2xl p-6 w-full max-w-lg max-h-[85vh] flex flex-col"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between pb-3.5 border-b border-slate-200/90 dark:border-[var(--ui-border-subtle)]">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-purple-500/10 text-purple-600 dark:text-[var(--ui-interactive)] border border-purple-500/20 shadow-2xs">
                            <ArrowRightLeft className="w-5 h-5" />
                        </div>
                        <div>
                            <h3 id="switch-account-modal-title" className="font-bold text-base text-slate-900 dark:text-white">
                                Switch Account for {switchTargetInstance.config.name}
                            </h3>
                            <div className="flex items-center gap-2 mt-0.5 flex-wrap">
                                <p className="text-xs text-slate-500 dark:text-slate-400">
                                    Currently bound:{' '}
                                    <span className="font-mono font-medium text-slate-700 dark:text-slate-200">
                                        {switchTargetInstance.config.bound_email
                                            ? (showAllEmails
                                                ? switchTargetInstance.config.bound_email
                                                : maskEmail(switchTargetInstance.config.bound_email))
                                            : 'None'}
                                    </span>
                                </p>
                                {boundQuota && (boundQuota.fourH !== null || boundQuota.weekly !== null) && (
                                    <div className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)]/80 backdrop-blur-xs px-2 py-0.5 text-[9px] font-mono shadow-2xs text-slate-600 dark:text-[var(--ui-text-secondary)]">
                                        <span className="flex items-center gap-1 font-semibold text-slate-700 dark:text-[var(--ui-text-primary)]">
                                            <span className="text-slate-400 dark:text-[var(--ui-text-muted)] font-normal">4H:</span>
                                            <span>{boundQuota.fourH !== null ? `${Math.round(boundQuota.fourH)}%` : '—'}</span>
                                        </span>
                                        <span className="w-px h-2 bg-slate-200 dark:bg-[var(--ui-border-subtle)] mx-1 shrink-0" />
                                        <span className="flex items-center gap-1 font-semibold text-slate-700 dark:text-[var(--ui-text-primary)]">
                                            <span className="text-slate-400 dark:text-[var(--ui-text-muted)] font-normal">Weekly:</span>
                                            <span>{boundQuota.weekly !== null ? `${Math.round(boundQuota.weekly)}%` : '—'}</span>
                                        </span>
                                        <span className="w-px h-2 bg-slate-200 dark:bg-[var(--ui-border-subtle)] mx-1 shrink-0" />
                                        <span className="flex items-center gap-1 font-semibold text-purple-700 dark:text-[var(--ui-interactive)]">
                                            <span className="text-slate-400 dark:text-[var(--ui-text-muted)] font-normal">Total:</span>
                                            <span className="font-bold">
                                                {(boundQuota.fourH !== null && boundQuota.weekly !== null)
                                                    ? `${Math.round((boundQuota.fourH + boundQuota.weekly) / 2)}%`
                                                    : (boundQuota.fourH !== null ? `${Math.round(boundQuota.fourH)}%` : (boundQuota.weekly !== null ? `${Math.round(boundQuota.weekly)}%` : '—'))}
                                            </span>
                                        </span>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Header Action Segmented Capsule */}
                    <div className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)] p-0.5 backdrop-blur-xs shadow-2xs">
                        <button
                            type="button"
                            onClick={() => setShowAllEmails((prev) => !prev)}
                            className="p-1.5 rounded-l-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-white dark:hover:bg-[var(--ui-surface-hover)] transition-colors duration-200 cursor-pointer"
                            title={showAllEmails ? 'Mask emails' : 'Reveal emails'}
                            aria-label={showAllEmails ? 'Mask emails' : 'Reveal emails'}
                        >
                            {showAllEmails ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                        </button>
                        <span className="w-px h-3.5 bg-slate-200 dark:bg-[var(--ui-border-subtle)] shrink-0" />
                        <button
                            type="button"
                            onClick={close}
                            disabled={isAnySwitching}
                            className="p-1.5 rounded-r-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-white dark:hover:bg-[var(--ui-surface-hover)] transition-colors duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                            title="Close dialog (Esc)"
                            aria-label="Close dialog"
                        >
                            <X className="w-4 h-4" />
                        </button>
                    </div>
                </div>

                {/* Filter Search Input & Counter */}
                <div className="pt-3.5 pb-1 flex items-center justify-between gap-2">
                    <div className="relative flex-1">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400" />
                        <input
                            type="text"
                            value={accountSearchQuery}
                            onChange={(e) => setAccountSearchQuery(e.target.value)}
                            disabled={isAnySwitching}
                            placeholder="Filter accounts by email or ID..."
                            className="w-full pl-9 pr-8 py-1.5 text-xs bg-slate-50/80 dark:bg-[var(--ui-canvas)] border border-slate-200/90 dark:border-[var(--ui-border-subtle)] rounded-full focus:outline-none focus:ring-1 focus:ring-[var(--ui-focus-ring)] text-slate-900 dark:text-slate-100 disabled:opacity-50 transition-colors"
                        />
                        {accountSearchQuery && (
                            <button
                                type="button"
                                onClick={() => setAccountSearchQuery('')}
                                disabled={isAnySwitching}
                                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-0.5 rounded-full cursor-pointer"
                                title="Clear filter"
                                aria-label="Clear filter"
                            >
                                <X className="w-3.5 h-3.5" />
                            </button>
                        )}
                    </div>
                    <span className="text-[11px] text-slate-400 shrink-0 font-medium px-1 select-none">
                        {filtered.length} {filtered.length === 1 ? 'account' : 'accounts'}
                    </span>
                </div>

                {/* Account List */}
                <div className="overflow-y-auto py-2 space-y-2.5 flex-1 my-1 pr-1">
                    {filtered.length === 0 ? (
                        <p className="text-xs text-slate-400 text-center py-6">No accounts matching filter.</p>
                    ) : (
                        filtered.map((acc) => {
                            const isCurrent =
                                switchTargetInstance.config.bound_account_id === acc.id ||
                                (switchTargetInstance.config.bound_email &&
                                    switchTargetInstance.config.bound_email.toLowerCase() === acc.email.toLowerCase());
                            const { fourH, fourHResetTime, weekly, weeklyResetTime, modelName } = accountQuotaScores(acc);
                            const tier = (acc.quota?.subscription_tier || 'FREE').toUpperCase();
                            const isThisSwitching = switchingAccountId === acc.id;

                            const fourHVal = fourH !== null ? `${Math.round(fourH)}%` : '—';
                            const weeklyVal = weekly !== null ? `${Math.round(weekly)}%` : '—';
                            const totalVal = (fourH !== null && weekly !== null)
                                ? `${Math.round((fourH + weekly) / 2)}%`
                                : (fourH !== null ? `${Math.round(fourH)}%` : (weekly !== null ? `${Math.round(weekly)}%` : '—'));

                            return (
                                <div
                                    key={acc.id}
                                    role="button"
                                    tabIndex={isCurrent || isAnySwitching ? -1 : 0}
                                    onClick={() => {
                                        if (!isCurrent && !isAnySwitching) {
                                            handleSelect(acc);
                                        }
                                    }}
                                    onKeyDown={(e) => {
                                        if ((e.key === 'Enter' || e.key === ' ') && !isCurrent && !isAnySwitching) {
                                            e.preventDefault();
                                            handleSelect(acc);
                                        }
                                    }}
                                    className={cn(
                                        'ui-interactive-row p-3.5 rounded-xl border flex items-center justify-between gap-3 transition-all duration-200 group',
                                        isCurrent
                                            ? 'bg-emerald-500/10 dark:bg-emerald-950/20 border-emerald-400/50 dark:border-emerald-600/40 shadow-2xs cursor-default'
                                            : isAnySwitching
                                              ? 'opacity-60 cursor-not-allowed bg-white/80 dark:bg-[var(--ui-surface-1)]/80 border-slate-200/90 dark:border-[var(--ui-border-subtle)]'
                                              : 'bg-white/80 dark:bg-[var(--ui-surface-1)]/80 border-slate-200/90 dark:border-[var(--ui-border-subtle)] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] hover:border-purple-400/50 dark:hover:border-[var(--ui-border-emphasis)] hover:shadow-2xs cursor-pointer focus:outline-none focus:ring-1 focus:ring-[var(--ui-focus-ring)]'
                                    )}
                                >
                                    <div className="flex items-start gap-3 flex-1 min-w-0">
                                        {/* Circular Radio Indicator */}
                                        <div
                                            className={cn(
                                                'w-4 h-4 rounded-full border flex items-center justify-center shrink-0 mt-0.5 transition-all',
                                                isCurrent
                                                    ? 'border-emerald-500 bg-emerald-500/20 text-[#1af18d]'
                                                    : 'border-slate-300 dark:border-[var(--ui-border-subtle)] group-hover:border-purple-400 dark:group-hover:border-[var(--ui-interactive)]'
                                            )}
                                        >
                                            {isCurrent && <div className="w-1.5 h-1.5 rounded-full bg-[#1af18d]" />}
                                        </div>

                                        <div className="min-w-0 flex-1">
                                            <div className="flex items-center gap-2 mb-1.5 flex-wrap">
                                                <span
                                                    onClick={(e) => toggleEmail(acc.id, e)}
                                                    className="cursor-pointer hover:underline font-mono text-xs font-semibold text-slate-900 dark:text-slate-100 truncate"
                                                    title={isEmailVisible(acc.id) ? 'Click to mask email' : 'Click to reveal email'}
                                                >
                                                    {isEmailVisible(acc.id) ? acc.email : maskEmail(acc.email)}
                                                </span>
                                                <span
                                                    className={cn(
                                                        'text-[9px] font-bold px-1.5 py-0.5 rounded shadow-2xs',
                                                        tier.includes('ULTRA')
                                                            ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300'
                                                            : tier.includes('PRO')
                                                              ? 'bg-sky-100 text-sky-700 dark:bg-sky-900/40 dark:text-sky-300'
                                                              : 'bg-gray-100 text-gray-600 dark:bg-[var(--ui-border-subtle)] dark:text-slate-300'
                                                    )}
                                                >
                                                    {tier}
                                                </span>
                                                {isCurrent && (
                                                    <span className="inline-flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-500/10 dark:bg-emerald-950/40 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/30 shadow-2xs">
                                                        <Check className="w-3 h-3 text-[#1af18d]" />
                                                        <span>Current</span>
                                                    </span>
                                                )}
                                            </div>

                                            {/* Quota visualization */}
                                            {(fourH !== null || weekly !== null) ? (
                                                <div className="space-y-2 w-full mt-2">
                                                    {/* Contiguous segmented pill capsule header displaying quota metrics */}
                                                    <div className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)]/80 backdrop-blur-xs px-2.5 py-0.5 text-[10px] font-mono shadow-2xs text-slate-600 dark:text-[var(--ui-text-secondary)]">
                                                        <span className="flex items-center gap-1 font-semibold text-slate-700 dark:text-[var(--ui-text-primary)]">
                                                            <span className="text-slate-400 dark:text-[var(--ui-text-muted)] font-normal">4H:</span>
                                                            <span>{fourHVal}</span>
                                                        </span>
                                                        <span className="w-px h-2.5 bg-slate-200 dark:bg-[var(--ui-border-subtle)] mx-1.5 shrink-0" />
                                                        <span className="flex items-center gap-1 font-semibold text-slate-700 dark:text-[var(--ui-text-primary)]">
                                                            <span className="text-slate-400 dark:text-[var(--ui-text-muted)] font-normal">Weekly:</span>
                                                            <span>{weeklyVal}</span>
                                                        </span>
                                                        <span className="w-px h-2.5 bg-slate-200 dark:bg-[var(--ui-border-subtle)] mx-1.5 shrink-0" />
                                                        <span className="flex items-center gap-1 font-semibold text-purple-700 dark:text-[var(--ui-interactive)]">
                                                            <span className="text-slate-400 dark:text-[var(--ui-text-muted)] font-normal">Total:</span>
                                                            <span className="font-bold">{totalVal}</span>
                                                        </span>
                                                    </div>

                                                    {fourH !== null && (
                                                        <div className="space-y-0.5 w-full">
                                                            <div className="flex items-center justify-between text-[10px] font-mono">
                                                                <span className="truncate text-slate-600 dark:text-slate-300 font-semibold">
                                                                    {modelName ? `${modelName} (4h)` : 'Primary Model (4h)'}
                                                                </span>
                                                            </div>
                                                            <QuotaProgressBar
                                                                className="w-full"
                                                                percentage={fourH}
                                                                resetTime={fourHResetTime}
                                                                compact
                                                            />
                                                        </div>
                                                    )}
                                                    {weekly !== null && (
                                                        <div className={cn('space-y-0.5 w-full', fourH !== null && 'pt-1 border-t border-slate-200/50 dark:border-[var(--ui-border-subtle)]/50')}>
                                                            <div className="flex items-center justify-between text-[10px] font-mono">
                                                                <span className="truncate text-slate-500 dark:text-slate-400">
                                                                    Weekly Quota
                                                                </span>
                                                            </div>
                                                            <QuotaProgressBar
                                                                className="w-full"
                                                                isWeekly
                                                                percentage={weekly}
                                                                resetTime={weeklyResetTime}
                                                                compact
                                                            />
                                                        </div>
                                                    )}
                                                </div>
                                            ) : (
                                                <p className="text-[11px] font-mono text-slate-400 italic mt-1.5">
                                                    No quota telemetry available
                                                </p>
                                            )}
                                        </div>
                                    </div>

                                    {/* Action button: rendered only for non-current accounts */}
                                    {!isCurrent && (
                                        <div className="inline-flex items-center rounded-full border border-slate-200/90 dark:border-[var(--ui-border-subtle)] bg-slate-100/80 dark:bg-[var(--ui-canvas)] p-0.5 backdrop-blur-xs shadow-2xs shrink-0">
                                            <button
                                                type="button"
                                                disabled={isAnySwitching}
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    handleSelect(acc);
                                                }}
                                                className={cn(
                                                    'px-3.5 py-1.5 rounded-full text-xs font-semibold transition-all duration-200 cursor-pointer',
                                                    isAnySwitching
                                                        ? 'opacity-60 cursor-not-allowed bg-[var(--ui-interactive)] text-white'
                                                        : 'bg-[var(--ui-interactive)] hover:bg-purple-600 text-white shadow-2xs hover:shadow-xs active:scale-95'
                                                )}
                                            >
                                                {isThisSwitching ? (
                                                    <span className="flex items-center gap-1.5">
                                                        <RotateCw className="w-3.5 h-3.5 animate-spin" />
                                                        <span>Switching...</span>
                                                    </span>
                                                ) : (
                                                    'Select'
                                                )}
                                            </button>
                                        </div>
                                    )}
                                </div>
                            );
                        })
                    )}
                </div>
            </div>
        </div>
    );
}
