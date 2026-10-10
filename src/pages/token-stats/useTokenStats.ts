import { useEffect, useState } from 'react';
import { request as invoke } from '../../utils/request';

export interface TokenStatsAggregated {
    period: string;
    total_input_tokens: number;
    total_output_tokens: number;
    total_cached_tokens: number;
    total_tokens: number;
    request_count: number;
    uncached_input_tokens?: number;
}

export interface AccountTokenStats {
    account_email: string;
    total_input_tokens: number;
    total_output_tokens: number;
    total_cached_tokens: number;
    total_tokens: number;
    request_count: number;
}

export interface ModelTokenStats {
    model: string;
    total_input_tokens: number;
    total_output_tokens: number;
    total_cached_tokens: number;
    total_tokens: number;
    request_count: number;
}

export interface ModelTrendPoint {
    period: string;
    model_data: Record<string, number>;
}

export interface AccountTrendPoint {
    period: string;
    account_data: Record<string, number>;
}

export interface TokenStatsSummary {
    total_input_tokens: number;
    total_output_tokens: number;
    total_cached_tokens: number;
    total_tokens: number;
    total_requests: number;
    unique_accounts: number;
}

export interface PieSlice {
    name: string;
    value: number;
    fullEmail: string;
    color: string;
}

export type TimeRange = 'hourly' | 'daily' | 'weekly';
export type ViewMode = 'model' | 'account';

/** Flattened trend row keyed by model/account name plus the period label. */
export type TrendRow = Record<string, string | number>;

export const MODEL_COLORS = [
    '#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981',
    '#06b6d4', '#6366f1', '#f43f5e', '#84cc16', '#a855f7',
    '#14b8a6', '#f97316', '#64748b', '#0ea5e9', '#d946ef'
];

export const COLORS = ['#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981', '#06b6d4', '#6366f1', '#f43f5e'];

export const PIE_SLICE_LIMIT = 8;
export const PIE_LEGEND_LIMIT = 5;

export const formatNumber = (num: number): string => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
};

export const shortenModelName = (model: string): string => {
    return model
        .replace('gemini-', 'g-')
        .replace('claude-', 'c-')
        .replace('-preview', '')
        .replace('-latest', '');
};

export function useTokenStats() {
    const [timeRange, setTimeRange] = useState<TimeRange>('daily');
    const [viewMode, setViewMode] = useState<ViewMode>('model');
    const [chartData, setChartData] = useState<TokenStatsAggregated[]>([]);
    const [accountData, setAccountData] = useState<AccountTokenStats[]>([]);
    const [modelData, setModelData] = useState<ModelTokenStats[]>([]);
    const [modelTrendData, setModelTrendData] = useState<TrendRow[]>([]);
    const [accountTrendData, setAccountTrendData] = useState<TrendRow[]>([]);
    const [allModels, setAllModels] = useState<string[]>([]);
    const [allAccounts, setAllAccounts] = useState<string[]>([]);
    const [summary, setSummary] = useState<TokenStatsSummary | null>(null);
    const [loading, setLoading] = useState(true);

    const fetchData = async () => {
        setLoading(true);
        try {
            let hours = 24;
            let data: TokenStatsAggregated[] = [];
            let modelTrend: ModelTrendPoint[] = [];
            let accountTrend: AccountTrendPoint[] = [];

            switch (timeRange) {
                case 'hourly':
                    hours = 24;
                    data = await invoke<TokenStatsAggregated[]>('get_token_stats_hourly', { hours: 24 });
                    modelTrend = await invoke<ModelTrendPoint[]>('get_token_stats_model_trend_hourly', { hours: 24 });
                    accountTrend = await invoke<AccountTrendPoint[]>('get_token_stats_account_trend_hourly', { hours: 24 });
                    break;
                case 'daily':
                    hours = 168;
                    data = await invoke<TokenStatsAggregated[]>('get_token_stats_daily', { days: 7 });
                    modelTrend = await invoke<ModelTrendPoint[]>('get_token_stats_model_trend_daily', { days: 7 });
                    accountTrend = await invoke<AccountTrendPoint[]>('get_token_stats_account_trend_daily', { days: 7 });
                    break;
                case 'weekly':
                    hours = 720;
                    data = await invoke<TokenStatsAggregated[]>('get_token_stats_weekly', { weeks: 4 });
                    modelTrend = await invoke<ModelTrendPoint[]>('get_token_stats_model_trend_daily', { days: 30 });
                    accountTrend = await invoke<AccountTrendPoint[]>('get_token_stats_account_trend_daily', { days: 30 });
                    break;
            }

            setChartData(data.map(point => ({
                ...point,
                total_cached_tokens: point.total_cached_tokens || 0,
                uncached_input_tokens: Math.max((point.total_input_tokens || 0) - (point.total_cached_tokens || 0), 0)
            })));

            const models = new Set<string>();
            modelTrend.forEach(point => {
                Object.keys(point.model_data).forEach(m => models.add(m));
            });
            const modelList = Array.from(models);
            setAllModels(modelList);

            const transformedTrend = modelTrend.map(point => {
                const row: TrendRow = { period: point.period };
                modelList.forEach(model => {
                    row[model] = point.model_data[model] || 0;
                });
                return row;
            });
            setModelTrendData(transformedTrend);

            // Process Account Trend Data
            const accountsSet = new Set<string>();
            accountTrend.forEach(point => {
                Object.keys(point.account_data).forEach(acc => accountsSet.add(acc));
            });
            const accountList = Array.from(accountsSet);
            setAllAccounts(accountList);

            const transformedAccountTrend = accountTrend.map(point => {
                const row: TrendRow = { period: point.period };
                accountList.forEach(acc => {
                    row[acc] = point.account_data[acc] || 0;
                });
                return row;
            });
            setAccountTrendData(transformedAccountTrend);

            const [accounts, models_stats, summaryData] = await Promise.all([
                invoke<AccountTokenStats[]>('get_token_stats_by_account', { hours }),
                invoke<ModelTokenStats[]>('get_token_stats_by_model', { hours }),
                invoke<TokenStatsSummary>('get_token_stats_summary', { hours })
            ]);

            setAccountData(accounts);
            setModelData(models_stats);
            setSummary(summaryData);
        } catch (error) {
            console.error('Failed to fetch token stats:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
    }, [timeRange]);

    const pieData: PieSlice[] = accountData.slice(0, PIE_SLICE_LIMIT).map((account, index) => ({
        name: account.account_email.split('@')[0] + '...',
        value: account.total_tokens,
        fullEmail: account.account_email,
        color: COLORS[index % COLORS.length]
    }));

    return {
        timeRange,
        setTimeRange,
        viewMode,
        setViewMode,
        chartData,
        accountData,
        modelData,
        modelTrendData,
        accountTrendData,
        allModels,
        allAccounts,
        summary,
        loading,
        fetchData,
        pieData,
    };
}
