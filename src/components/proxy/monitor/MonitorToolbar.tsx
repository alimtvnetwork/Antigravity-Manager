import React from 'react';
import type { TFunction } from 'react-i18next';
import { Search, Trash2, RefreshCw, User, Settings } from 'lucide-react';
import { MonitorStats } from './MonitorStats';
import type { QuickFilter } from './useProxyLogs';
import type { ProxyStats } from './types';

export interface MonitorToolbarProps {
    t: TFunction;
    filter: string;
    onFilterChange: (value: string) => void;
    accountFilter: string;
    onAccountFilterChange: (value: string) => void;
    uniqueAccounts: string[];
    stats: ProxyStats;
    loading: boolean;
    isLoggingEnabled: boolean;
    onToggleLogging: () => void;
    captureHealthLogs: boolean;
    onToggleCaptureHealthLogs: () => void;
    showLogSettings: boolean;
    onToggleLogSettings: () => void;
    onClearLogs: () => void;
    onRefresh: () => void;
    onResetFilters: () => void;
    quickFilters: QuickFilter[];
    filterInputRef: React.RefObject<HTMLInputElement | null>;
}

export const MonitorToolbar: React.FC<MonitorToolbarProps> = ({
    t,
    filter,
    onFilterChange,
    accountFilter,
    onAccountFilterChange,
    uniqueAccounts,
    stats,
    loading,
    isLoggingEnabled,
    onToggleLogging,
    captureHealthLogs,
    onToggleCaptureHealthLogs,
    showLogSettings,
    onToggleLogSettings,
    onClearLogs,
    onRefresh,
    onResetFilters,
    quickFilters,
    filterInputRef,
}) => {
    return (
        <div className="p-3.5 border-b border-gray-200/80 dark:border-base-200 space-y-3 bg-gray-50/80 dark:bg-base-200">
            <div className="flex items-center gap-3">
                <button
                    onClick={onToggleLogging}
                    className={`btn btn-sm gap-2 px-3.5 border font-semibold rounded-lg transition-all ${isLoggingEnabled
                        ? 'bg-rose-600 hover:bg-rose-700 border-rose-600 text-white shadow-xs'
                        : 'bg-white dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-base-300/80 shadow-2xs'
                        }`}
                >
                    <div className={`w-2.5 h-2.5 rounded-full ${isLoggingEnabled ? 'bg-white' : 'bg-gray-400'}`} />
                    {isLoggingEnabled ? t('monitor.logging_status.active') : t('monitor.logging_status.paused')}
                </button>

                <div className="relative flex-1">
                    <Search className="absolute left-2.5 top-2 text-gray-400" size={14} />
                    <input
                        ref={filterInputRef}
                        type="text"
                        placeholder={t('monitor.filters.placeholder')}
                        className="input input-sm input-bordered w-full pl-9 text-xs bg-white dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-900 dark:text-white focus:border-blue-500 focus:ring-1 focus:ring-blue-500/30"
                        value={filter}
                        onChange={(e) => onFilterChange(e.target.value)}
                    />
                </div>

                <div className="relative">
                    <User className="absolute left-2.5 top-2 text-gray-400 z-10" size={14} />
                    <select
                        className="select select-sm select-bordered pl-8 text-xs min-w-[140px] max-w-[220px] bg-white dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-900 dark:text-white focus:border-blue-500 focus:ring-1 focus:ring-blue-500/30"
                        value={accountFilter}
                        onChange={(e) => onAccountFilterChange(e.target.value)}
                        title={t('monitor.filters.by_account')}
                    >
                        <option value="">{t('monitor.filters.all_accounts')}</option>
                        {uniqueAccounts.map(email => (
                            <option key={email} value={email} title={email}>
                                {email}
                            </option>
                        ))}
                    </select>
                </div>

                <MonitorStats t={t} stats={stats} />

                <button onClick={onRefresh} className="btn btn-sm btn-ghost text-gray-400 hover:text-gray-600 dark:hover:text-gray-200" title={t('common.refresh')}>
                    <RefreshCw size={16} className={loading ? 'animate-spin' : ''} />
                </button>
                <button
                    onClick={onToggleLogSettings}
                    className={`btn btn-sm btn-ghost ${
                        showLogSettings
                            ? 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/30'
                            : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-200'
                    }`}
                    title={t('common.settings', { defaultValue: '设置' })}
                    aria-label={t('common.settings', { defaultValue: '设置' })}
                >
                    <Settings size={16} />
                </button>
                <button onClick={onClearLogs} className="btn btn-sm btn-ghost text-gray-400 hover:text-gray-600 dark:hover:text-gray-200" title={t('monitor.actions.clear_all_requests', { defaultValue: '清空请求日志' })}>
                    <Trash2 size={16} />
                </button>
            </div>

            <div className="flex flex-wrap items-center gap-2">
                <span className="text-xs font-bold text-gray-700 dark:text-gray-200 uppercase tracking-wide">{t('monitor.filters.quick_filters')}</span>
                {quickFilters.map(q => (
                    <button
                        key={q.label}
                        onClick={() => onFilterChange(q.value)}
                        className={`px-3 py-0.5 rounded-full text-xs font-semibold border transition-all ${
                            filter === q.value
                                ? 'bg-blue-600 text-white border-blue-600 shadow-xs'
                                : 'bg-white dark:bg-base-200 text-gray-700 dark:text-gray-200 border-gray-300 dark:border-base-300 hover:bg-gray-100 dark:hover:bg-base-300/80 hover:text-gray-900 dark:hover:text-white shadow-2xs'
                        }`}
                    >
                        {q.label}
                    </button>
                ))}
                <button
                    onClick={onToggleCaptureHealthLogs}
                    className={`px-3 py-0.5 rounded-full text-xs font-semibold border transition-all flex items-center gap-1.5 ${
                        captureHealthLogs
                            ? 'bg-emerald-600 text-white border-emerald-600 shadow-xs'
                            : 'bg-white dark:bg-base-200 text-gray-700 dark:text-gray-200 border-gray-300 dark:border-base-300 hover:bg-gray-100 dark:hover:bg-base-300/80 hover:text-gray-900 dark:hover:text-white shadow-2xs'
                    }`}
                    title={t('monitor.filters.capture_health_tip', { defaultValue: '默认关闭：过滤全部 GET 成功请求（含 /v1/models 模型列表轮询与 /health 探针）且不入库；失败请求始终记录；开启后才全部记录并落库' })}
                >
                    <span className={`w-1.5 h-1.5 rounded-full ${captureHealthLogs ? 'bg-white animate-pulse' : 'bg-gray-400 dark:bg-gray-500'}`} />
                    {t('monitor.filters.capture_health', { defaultValue: '捕获健康检查' })}
                </button>
                {(filter || accountFilter) && (
                    <button
                        onClick={onResetFilters}
                        className="text-xs font-semibold text-blue-600 dark:text-blue-400 hover:underline ml-1"
                    >
                        {t('monitor.filters.reset')}
                    </button>
                )}
            </div>
        </div>
    );
};
