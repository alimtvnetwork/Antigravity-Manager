import { ArrowRightLeft, Search, RotateCw } from 'lucide-react';
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

    if (!switchTargetInstance) return null;

    const close = () => {
        setSwitchTargetInstance(null);
        setAccountSearchQuery('');
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
        const targetId = switchTargetInstance.config.id;
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
        }
    };

    const isSwitching = actionState[switchTargetInstance.config.id] === 'switch';

    return (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4 overflow-y-auto">
            <div className="bg-white dark:bg-[#171025] rounded-2xl p-6 w-full max-w-lg shadow-2xl border border-slate-200 dark:border-[#40285e] max-h-[85vh] flex flex-col">
                <div className="flex items-center justify-between pb-3 border-b border-slate-200 dark:border-[#40285e]">
                    <div className="flex items-center gap-2.5">
                        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-teal-500/10 text-teal-600 dark:text-teal-300 border border-teal-500/20">
                            <ArrowRightLeft className="w-4 h-4" />
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
                    <button
                        type="button"
                        onClick={close}
                        className="px-2.5 py-1 rounded-full text-xs font-semibold text-slate-500 hover:text-slate-800 dark:text-slate-300 dark:hover:text-white hover:bg-slate-100 dark:hover:bg-[#40285e]"
                    >
                        Close
                    </button>
                </div>

                <div className="pt-3 pb-1">
                    <div className="relative">
                        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400" />
                        <input
                            type="text"
                            value={accountSearchQuery}
                            onChange={(e) => setAccountSearchQuery(e.target.value)}
                            placeholder="Filter accounts by email or ID..."
                            className="w-full pl-8 pr-2.5 py-1.5 text-xs bg-slate-50 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e] rounded-full focus:outline-none focus:ring-1 focus:ring-teal-500 text-slate-900 dark:text-slate-100"
                        />
                    </div>
                </div>

                <div className="overflow-y-auto py-2 space-y-2 flex-1 my-1 pr-1">
                    {filtered.length === 0 ? (
                        <p className="text-xs text-slate-400 text-center py-4">No accounts matching filter.</p>
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

                            return (
                                <div
                                    key={acc.id}
                                    className={cn(
                                        'p-3 rounded-xl border flex items-center justify-between gap-3 transition-colors',
                                        isCurrent
                                            ? 'bg-teal-50/70 dark:bg-teal-950/30 border-teal-300/60 dark:border-teal-700'
                                            : 'bg-slate-50/80 dark:bg-[#090611] border-slate-200 dark:border-[#40285e] hover:border-teal-400/50'
                                    )}
                                >
                                    <div className="min-w-0 flex-1">
                                        <div className="flex items-center gap-2 mb-1">
                                            <span className="font-mono text-xs font-semibold text-slate-900 dark:text-slate-100 truncate">
                                                {acc.email}
                                            </span>
                                            <span
                                                className={cn(
                                                    'text-[9px] font-bold px-1.5 py-0.5 rounded',
                                                    tier.includes('ULTRA')
                                                        ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300'
                                                        : tier.includes('PRO')
                                                          ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
                                                          : 'bg-gray-100 text-gray-600 dark:bg-[#40285e] dark:text-slate-300'
                                                )}
                                            >
                                                {tier}
                                            </span>
                                            {isCurrent ? (
                                                <span className="text-[10px] text-teal-700 dark:text-teal-300 font-semibold">
                                                    Current
                                                </span>
                                            ) : null}
                                        </div>
                                        <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-[11px] text-slate-500 dark:text-slate-400">
                                            <span>4h {formatQuotaPercent(fourH)}</span>
                                            <span>Weekly {formatQuotaPercent(weekly)}</span>
                                            <span className="font-semibold text-slate-700 dark:text-slate-200">
                                                Total {formatQuotaPercent(total)}
                                            </span>
                                            <div className="w-24 h-1.5 bg-slate-200 dark:bg-[#40285e] rounded-full overflow-hidden">
                                                <div
                                                    className={cn(
                                                        'h-full rounded-full',
                                                        barPct >= 50 ? 'bg-emerald-500' : barPct >= 20 ? 'bg-amber-500' : 'bg-rose-500'
                                                    )}
                                                    style={{ width: `${barPct}%` }}
                                                />
                                            </div>
                                        </div>
                                    </div>

                                    <button
                                        type="button"
                                        disabled={isCurrent || isLoading || Boolean(actionState[switchTargetInstance.config.id])}
                                        onClick={() => handleSelect(acc)}
                                        className={cn(
                                            'px-3 py-1 rounded-full text-[11px] font-semibold border',
                                            isCurrent
                                                ? 'opacity-50 cursor-not-allowed border-slate-200 dark:border-[#40285e] text-slate-400'
                                                : 'bg-teal-600 hover:bg-teal-500 text-white border-teal-500'
                                        )}
                                    >
                                        {isSwitching ? (
                                            <span className="flex items-center gap-1">
                                                <RotateCw className="w-3 h-3 animate-spin" />
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

                <div className="flex justify-end pt-3 border-t border-slate-200 dark:border-[#40285e]">
                    <button
                        type="button"
                        onClick={close}
                        className="px-4 py-1.5 rounded-full text-xs font-semibold bg-slate-100 hover:bg-slate-200 dark:bg-[#40285e] dark:hover:bg-[#1d4366] text-slate-700 dark:text-slate-200"
                    >
                        {t('common.close', 'Close')}
                    </button>
                </div>
            </div>
        </div>
    );
}
