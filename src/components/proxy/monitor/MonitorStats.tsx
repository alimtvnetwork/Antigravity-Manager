import React from 'react';
import type { TFunction } from 'i18next';
import { formatCompactNumber } from '../../../utils/format';
import type { ProxyStats } from './types';

export interface MonitorStatsProps {
    t: TFunction;
    stats: ProxyStats;
}

/** Inline total / success / error counters shown in the monitor toolbar (large screens). */
export const MonitorStats: React.FC<MonitorStatsProps> = ({ t, stats }) => {
    return (
        <div className="hidden lg:flex items-center gap-3 text-xs font-bold font-mono">
            <span className="text-blue-600 dark:text-blue-400">
                {formatCompactNumber(stats.total_requests)} <span className="font-sans font-semibold text-[11px] text-gray-500 dark:text-gray-400">{t('monitor.stats.total')}</span>
            </span>
            <span className="text-emerald-600 dark:text-emerald-400">
                {formatCompactNumber(stats.success_count)} <span className="font-sans font-semibold text-[11px] text-gray-500 dark:text-gray-400">{t('monitor.stats.ok')}</span>
            </span>
            <span className="text-rose-600 dark:text-rose-400">
                {formatCompactNumber(stats.error_count)} <span className="font-sans font-semibold text-[11px] text-gray-500 dark:text-gray-400">{t('monitor.stats.err')}</span>
            </span>
        </div>
    );
};
