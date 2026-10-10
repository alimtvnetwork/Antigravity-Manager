import { useTranslation } from 'react-i18next';
import { formatNumber, shortenModelName } from './useTokenStats';
import type { ViewMode } from './useTokenStats';

interface TrendTooltipEntry {
    name?: string;
    value?: number;
    color?: string;
}

export interface CustomTrendTooltipProps {
    active?: boolean;
    payload?: TrendTooltipEntry[];
    label?: string;
    viewMode: ViewMode;
}

// Custom Tooltip for Trend Chart
export function CustomTrendTooltip({ active, payload, label, viewMode }: CustomTrendTooltipProps) {
    if (!active || !payload || !payload.length) return null;

    // Sort payload by value descending
    const sortedPayload = [...payload].sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

    return (
        <div className="bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm p-2.5 rounded-xl shadow-xl border border-gray-100 dark:border-gray-700 text-xs z-[100] min-w-[180px] pointer-events-none">
            <p className="font-semibold text-gray-700 dark:text-gray-200 mb-1.5 border-b border-gray-100 dark:border-gray-700 pb-1.5">
                {label}
            </p>
            <div className="max-h-[180px] overflow-y-auto space-y-1 pr-1.5 scrollbar-thin scrollbar-thumb-gray-200 dark:scrollbar-thumb-gray-700">
                {sortedPayload.map((entry, index) => {
                    const name = entry.name ?? '';
                    const displayName = viewMode === 'model' ? shortenModelName(name) : name.split('@')[0];
                    return (
                        <div key={index} className="flex items-center justify-between gap-4">
                            <div className="flex items-center gap-2 overflow-hidden">
                                <div className="w-2 h-2 rounded-full flex-shrink-0" style={{ backgroundColor: entry.color }} />
                                <span className="text-gray-500 dark:text-gray-400 truncate max-w-[120px]" title={name}>
                                    {displayName}
                                </span>
                            </div>
                            <span className="font-mono font-medium text-gray-700 dark:text-gray-200">
                                {formatNumber(entry.value ?? 0)}
                            </span>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

interface UsageTrendRow {
    total_tokens?: number;
    total_input_tokens?: number;
    total_cached_tokens?: number;
    total_output_tokens?: number;
    request_count?: number;
}

export interface UsageTrendTooltipProps {
    active?: boolean;
    payload?: { payload?: UsageTrendRow }[];
    label?: string;
}

export function UsageTrendTooltip({ active, payload, label }: UsageTrendTooltipProps) {
    const { t } = useTranslation();
    if (!active || !payload || !payload.length) return null;
    const row = payload[0]?.payload || {};
    const items = [
        { label: t('token_stats.total', '合计'), value: row.total_tokens || 0, color: '#111827' },
        { label: t('token_stats.input', '输入'), value: row.total_input_tokens || 0, color: '#3b82f6' },
        { label: t('token_stats.cached_token', '缓存命中'), value: row.total_cached_tokens || 0, color: '#93c5fd' },
        { label: t('token_stats.output', '输出'), value: row.total_output_tokens || 0, color: '#8b5cf6' },
    ];
    return (
        <div className="bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm p-2.5 rounded-xl shadow-xl border border-gray-100 dark:border-gray-700 text-xs z-[100] pointer-events-none min-w-[170px]">
            {label && <p className="font-semibold text-gray-700 dark:text-gray-200 mb-2">{label}</p>}
            <div className="space-y-1">
                {items.map((item) => (
                    <div key={item.label} className="flex items-center justify-between gap-4">
                        <div className="flex items-center gap-2">
                            <div className="w-2 h-2 rounded-full" style={{ backgroundColor: item.color }} />
                            <span className="text-gray-500 dark:text-gray-400">
                                {item.label}:
                            </span>
                        </div>
                        <span className="font-mono font-medium text-gray-700 dark:text-gray-200">
                            {formatNumber(item.value)}
                        </span>
                    </div>
                ))}
                <div className="flex items-center justify-between gap-4 pt-1 border-t border-gray-100 dark:border-gray-700">
                    <span className="text-gray-500 dark:text-gray-400">
                        {t('token_stats.requests', '请求数')}:
                    </span>
                    <span className="font-mono font-medium text-gray-700 dark:text-gray-200">
                        {(row.request_count || 0).toLocaleString()}
                    </span>
                </div>
            </div>
        </div>
    );
}

interface PieTooltipEntry {
    name?: string;
    value?: number;
    color?: string;
    payload?: {
        color?: string;
        fullEmail?: string;
    };
}

export interface CustomPieTooltipProps {
    active?: boolean;
    payload?: PieTooltipEntry[];
}

// Custom Tooltip for Pie Chart
export function CustomPieTooltip({ active, payload }: CustomPieTooltipProps) {
    if (!active || !payload || !payload.length) return null;
    const entry = payload[0];
    return (
        <div className="bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm p-2.5 rounded-xl shadow-xl border border-gray-100 dark:border-gray-700 text-xs z-[100] pointer-events-none">
            <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full" style={{ backgroundColor: entry.payload?.color || entry.color }} />
                <span className="text-gray-500 dark:text-gray-400">
                    {entry.payload?.fullEmail || entry.name}:
                </span>
                <span className="font-mono font-medium text-gray-700 dark:text-gray-200">
                    {formatNumber(entry.value ?? 0)}
                </span>
            </div>
        </div>
    );
}
