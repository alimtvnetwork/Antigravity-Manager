import { useState } from 'react';
import { ArrowRightLeft, Search, RotateCw, X, Check } from 'lucide-react';
import { cn } from '../../utils/cn';
import { showToast } from '../../components/common/ToastContainer';
import { findQuotaModel } from '../../config/modelConfig';
import type { Account, QuotaBucket, QuotaGroup } from '../../types/account';
import type { InstancePageApi } from './instancePageTypes';

function readWeeklyPercent(groups: QuotaGroup[] | undefined): number | null {
    if (!groups?.length) return null;
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
        return null;
    }
    return Math.min(100, Math.max(0, bucket.remaining_fraction * 100));
}

function accountQuotaScores(acc: Account): { fourH: number | null; weekly: number | null } {
    if (acc.disabled || acc.quota?.is_forbidden) {
        return { fourH: null, weekly: null };
    }
    const proModel = findQuotaModel(acc.quota?.models, 'gemini-pro');
    const flashModel = findQuotaModel(acc.quota?.models, 'gemini-flash');
    const model = proModel || flashModel;
    const raw4h = model && Number.isFinite(model.percentage) ? model.percentage : null;
    const fourH = raw4h === null ? null : Math.min(100, Math.max(0, raw4h));
    return { fourH, weekly: readWeeklyPercent(acc.quota?.quota_groups) };
}

function sortScore(value: number | null): number {
    return value === null || !Number.isFinite(value) ? 0 : value;
}

function formatQuotaPercent(value: number | null): string {
    if (value === null || !Number.isFinite(value)) return '—';
    const rounded = Math.round(value * 10) / 10;
    return Number.isInteger(rounded) ? `${rounded}%` : `${rounded.toFixed(1)}%`;
}

export function SwitchAccountDialog({ api }: { api: InstancePageApi }) {
    const {
        t,
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

    if (!switchTargetInstance) return null;

    const targetId = switchTargetInstance.config.id;
    const isSwitching = actionState[targetId] === 'switch' || switchingAccountId !== null;
    const isAnySwitching = isSwitching || isLoading;

    const close = () => {
        if (isAnySwitching) return;
        setSwitchTargetInstance(null);
        setAccountSearchQuery('');
        setSwitchingAccountId(null);
    };

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

    return (
        <div
            className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4 overflow-y-auto animate-in fade-in duration-200"
            onClick={(e) => {
                if (e.target === e.currentTarget && !isAnySwitching) {
                    close();
                }
            }}
        >
            <div
                className="backdrop-blur-md bg-white/95 dark:bg-[#120d1d]/95 border border-slate-200 dark:border-[#40285e] shadow-2xl rounded-2xl p-6 w-full max-w-lg max-h-[85vh] flex flex-col"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between pb-3.5 border-b border-slate-200/80 dark:border-[#40285e]">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-teal-500/10 text-teal-600 dark:text-teal-300 border border-teal-500/20 shadow-2xs">
                            <ArrowRightLeft className="w-5 h-5" />
                        </div>
                        <div>
                            <h3 className="font-bold text-base text-slate-900 dark:text-white">
                                Switch Account for {switchTargetInstance.config.name}
                            </h3>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                                Currently bound:{' '}
                                <span className="font-mono font-medium text-slate-700 dark:text-slate-200">
                                    {switchTargetInstance.config.bound_email || 'None'}
                                </span>
                            </p>
                        </div>
                    </div>

                    {/* Top-right X close button */}
                    <button
                        type="button"
                        onClick={close}
                        disabled={isAnySwitching}
                        className="p-1.5 rounded-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                        title="Close dialog"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Filter Search Input */}
                <div className="pt-3.5 pb-1">
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400" />
                        <input
                            type="text"
                            value={accountSearchQuery}
                            onChange={(e) => setAccountSearchQuery(e.target.value)}
                            disabled={isAnySwitching}
                            placeholder="Filter accounts by email or ID..."
                            className="w-full pl-9 pr-3 py-1.5 text-xs bg-slate-50/80 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e] rounded-full focus:outline-none focus:ring-1 focus:ring-teal-500 text-slate-900 dark:text-slate-100 disabled:opacity-50"
                        />
                    </div>
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
                            const { fourH, weekly } = accountQuotaScores(acc);
                            const total =
                                fourH === null && weekly === null ? null : sortScore(fourH) + sortScore(weekly);
                            const barPct = fourH === null ? 0 : Math.min(100, Math.max(0, fourH));
                            const tier = (acc.quota?.subscription_tier || 'FREE').toUpperCase();
                            const isThisSwitching = switchingAccountId === acc.id;

                            return (
                                <div
                                    key={acc.id}
                                    className={cn(
                                        'p-3.5 rounded-xl border flex items-center justify-between gap-3 transition-all',
                                        isCurrent
                                            ? 'bg-teal-50/80 dark:bg-teal-950/40 border-teal-400/60 dark:border-teal-700/80 shadow-2xs'
                                            : 'bg-white/80 dark:bg-[#181126]/80 border-slate-200/90 dark:border-[#382252] hover:bg-slate-50/90 dark:hover:bg-[#201633] hover:border-teal-400/50 hover:shadow-2xs'
                                    )}
                                >
                                    <div className="min-w-0 flex-1">
                                        <div className="flex items-center gap-2 mb-1.5">
                                            <span className="font-mono text-xs font-semibold text-slate-900 dark:text-slate-100 truncate">
                                                {acc.email}
                                            </span>
                                            <span
                                                className={cn(
                                                    'text-[9px] font-bold px-1.5 py-0.5 rounded shadow-2xs',
                                                    tier.includes('ULTRA')
                                                        ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300'
                                                        : tier.includes('PRO')
                                                          ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
                                                          : 'bg-gray-100 text-gray-600 dark:bg-[#40285e] dark:text-slate-300'
                                                )}
                                            >
                                                {tier}
                                            </span>
                                            {isCurrent && (
                                                <span className="inline-flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-teal-100 dark:bg-teal-900/60 text-teal-800 dark:text-teal-200 border border-teal-300/50 dark:border-teal-700/50 shadow-2xs">
                                                    <Check className="w-3 h-3 text-teal-600 dark:text-teal-300" />
                                                    <span>Current</span>
                                                </span>
                                            )}
                                        </div>

                                        {/* Crisp Quota Badges */}
                                        <div className="flex flex-wrap items-center gap-1.5 text-[11px]">
                                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md font-medium bg-slate-100/90 dark:bg-[#211636] text-slate-700 dark:text-slate-300 border border-slate-200/80 dark:border-[#40285e]">
                                                <span className="text-[10px] text-slate-400 uppercase font-semibold">4h</span>
                                                <span className="font-mono font-semibold">{formatQuotaPercent(fourH)}</span>
                                            </span>
                                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md font-medium bg-slate-100/90 dark:bg-[#211636] text-slate-700 dark:text-slate-300 border border-slate-200/80 dark:border-[#40285e]">
                                                <span className="text-[10px] text-slate-400 uppercase font-semibold">Weekly</span>
                                                <span className="font-mono font-semibold">{formatQuotaPercent(weekly)}</span>
                                            </span>
                                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md font-medium bg-teal-50 dark:bg-teal-950/40 text-teal-800 dark:text-teal-200 border border-teal-200/80 dark:border-teal-800/60">
                                                <span className="text-[10px] text-teal-600/70 dark:text-teal-400/70 uppercase font-bold">Total</span>
                                                <span className="font-mono font-bold">{formatQuotaPercent(total)}</span>
                                            </span>
                                            <div className="w-20 h-1.5 ml-1 bg-slate-200 dark:bg-[#341e4d] rounded-full overflow-hidden">
                                                <div
                                                    className={cn(
                                                        'h-full rounded-full transition-all duration-300',
                                                        barPct >= 50 ? 'bg-emerald-500' : barPct >= 20 ? 'bg-amber-500' : 'bg-rose-500'
                                                    )}
                                                    style={{ width: `${barPct}%` }}
                                                />
                                            </div>
                                        </div>
                                    </div>

                                    {/* Action button */}
                                    <button
                                        type="button"
                                        disabled={isCurrent || isAnySwitching}
                                        onClick={() => handleSelect(acc)}
                                        className={cn(
                                            'px-3.5 py-1.5 rounded-full text-xs font-semibold border transition-all cursor-pointer shrink-0',
                                            isCurrent
                                                ? 'opacity-50 cursor-not-allowed border-slate-200 dark:border-[#40285e] text-slate-400 dark:text-slate-500'
                                                : isAnySwitching
                                                  ? 'opacity-60 cursor-not-allowed bg-teal-600 text-white border-teal-500'
                                                  : 'bg-teal-600 hover:bg-teal-500 text-white border-teal-500 shadow-2xs hover:shadow-xs active:scale-95'
                                        )}
                                    >
                                        {isThisSwitching ? (
                                            <span className="flex items-center gap-1.5">
                                                <RotateCw className="w-3.5 h-3.5 animate-spin" />
                                                <span>Switching...</span>
                                            </span>
                                        ) : isCurrent ? (
                                            'Current'
                                        ) : (
                                            'Select'
                                        )}
                                    </button>
                                </div>
                            );
                        })
                    )}
                </div>

                {/* Footer with Close Button */}
                <div className="flex justify-end pt-3.5 border-t border-slate-200/80 dark:border-[#40285e]">
                    <button
                        type="button"
                        onClick={close}
                        disabled={isAnySwitching}
                        className="px-4 py-1.5 rounded-full text-xs font-semibold bg-slate-100 hover:bg-slate-200 dark:bg-[#40285e] dark:hover:bg-[#1d4366] text-slate-700 dark:text-slate-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                    >
                        {t('common.close', 'Close')}
                    </button>
                </div>
            </div>
        </div>
    );
}
