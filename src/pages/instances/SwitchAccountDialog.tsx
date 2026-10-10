import { ArrowRightLeft, Search, RotateCw } from 'lucide-react';
import { cn } from '../../utils/cn';
import { showToast } from '../../components/common/ToastContainer';
import { findQuotaModel } from '../../config/modelConfig';
import type { Account } from '../../types/account';
import type { InstancePageApi } from './instancePageTypes';

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

    const filtered = accounts.filter((a) => {
        const q = accountSearchQuery.toLowerCase().trim();
        if (!q) return true;
        return a.email.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
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
            setActionError((e as Error)?.toString?.() || 'Failed to switch account');
        } finally {
            setActionState((prev) => ({ ...prev, [targetId]: null }));
        }
    };

    const isSwitching = actionState[switchTargetInstance.config.id] === 'switch';

    return (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
            <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-lg shadow-2xl border border-gray-100 dark:border-base-100 max-h-[85vh] flex flex-col">
                <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100">
                    <div className="flex items-center gap-2.5">
                        <ArrowRightLeft className="w-5 h-5 text-blue-600" />
                        <div>
                            <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                Switch Account for {switchTargetInstance.config.name}
                            </h3>
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                Currently bound:{' '}
                                <span className="font-mono font-medium">
                                    {switchTargetInstance.config.bound_email || 'None'}
                                </span>
                            </p>
                        </div>
                    </div>
                    <button onClick={close} className="btn btn-ghost btn-xs btn-circle">
                        ✕
                    </button>
                </div>

                <div className="pt-2 pb-1">
                    <div className="relative">
                        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
                        <input
                            type="text"
                            value={accountSearchQuery}
                            onChange={(e) => setAccountSearchQuery(e.target.value)}
                            placeholder="Filter accounts by email or ID..."
                            className="w-full pl-8 pr-2.5 py-1 text-xs bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500 text-gray-900 dark:text-base-content"
                        />
                    </div>
                </div>

                <div className="overflow-y-auto py-2 space-y-2 flex-1 my-1 pr-1">
                    {filtered.length === 0 ? (
                        <p className="text-xs text-gray-400 text-center py-4">No accounts matching filter.</p>
                    ) : (
                        filtered.map((acc) => {
                            const isCurrent =
                                switchTargetInstance.config.bound_account_id === acc.id ||
                                (switchTargetInstance.config.bound_email &&
                                    switchTargetInstance.config.bound_email.toLowerCase() === acc.email.toLowerCase());
                            const proModel = findQuotaModel(acc.quota?.models, 'gemini-pro');
                            const flashModel = findQuotaModel(acc.quota?.models, 'gemini-flash');
                            const model = proModel || flashModel;
                            const pct = model ? Math.min(100, Math.max(0, model.percentage)) : 0;
                            const tier = (acc.quota?.subscription_tier || 'FREE').toUpperCase();

                            return (
                                <div
                                    key={acc.id}
                                    className={cn(
                                        'p-3 rounded-xl border flex items-center justify-between gap-3 transition-colors',
                                        isCurrent
                                            ? 'bg-blue-50/50 dark:bg-blue-950/20 border-blue-200 dark:border-blue-800'
                                            : 'bg-gray-50/60 dark:bg-base-100/60 border-gray-100 dark:border-base-100 hover:border-gray-300 dark:hover:border-base-300'
                                    )}
                                >
                                    <div className="min-w-0 flex-1">
                                        <div className="flex items-center gap-2 mb-1">
                                            <span className="font-mono text-xs font-semibold text-gray-900 dark:text-base-content truncate">
                                                {acc.email}
                                            </span>
                                            <span
                                                className={cn(
                                                    'text-[9px] font-bold px-1.5 py-0.5 rounded',
                                                    tier.includes('ULTRA')
                                                        ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300'
                                                        : tier.includes('PRO')
                                                          ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
                                                          : 'bg-gray-100 text-gray-600 dark:bg-[#15334d] dark:text-slate-300'
                                                )}
                                            >
                                                {tier}
                                            </span>
                                            {isCurrent ? (
                                                <span className="text-[10px] text-blue-600 dark:text-blue-400 font-semibold">
                                                    Active
                                                </span>
                                            ) : null}
                                        </div>
                                        <div className="flex items-center gap-2 text-[11px] text-gray-500">
                                            <span>Quota: {pct}%</span>
                                            <div className="w-24 h-1.5 bg-gray-200 dark:bg-base-300 rounded-full overflow-hidden">
                                                <div
                                                    className={cn(
                                                        'h-full rounded-full',
                                                        pct >= 50 ? 'bg-emerald-500' : pct >= 20 ? 'bg-amber-500' : 'bg-rose-500'
                                                    )}
                                                    style={{ width: `${pct}%` }}
                                                />
                                            </div>
                                        </div>
                                    </div>

                                    <button
                                        disabled={isCurrent || isLoading || Boolean(actionState[switchTargetInstance.config.id])}
                                        onClick={() => handleSelect(acc)}
                                        className={cn('btn btn-xs rounded-[5px]', isCurrent ? 'btn-disabled opacity-50' : 'btn-primary')}
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

                <div className="flex justify-end pt-3 border-t border-gray-100 dark:border-base-100">
                    <button
                        onClick={close}
                        className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400 rounded-[5px]"
                    >
                        {t('common.close', 'Close')}
                    </button>
                </div>
            </div>
        </div>
    );
}
