import { useRef, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { AreaChart, Area, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, Legend } from 'recharts';
import { Cpu, Users } from 'lucide-react';
import { CustomTrendTooltip, UsageTrendTooltip, CustomPieTooltip } from './Tooltips';
import {
    MODEL_COLORS,
    COLORS,
    PIE_LEGEND_LIMIT,
    formatNumber,
    shortenModelName,
} from './useTokenStats';
import type {
    AccountTokenStats,
    ModelTokenStats,
    PieSlice,
    TimeRange,
    TokenStatsAggregated,
    TokenStatsSummary,
    TrendRow,
    ViewMode,
} from './useTokenStats';

export interface StatsChartsProps {
    viewMode: ViewMode;
    setViewMode: (mode: ViewMode) => void;
    timeRange: TimeRange;
    chartData: TokenStatsAggregated[];
    modelTrendData: TrendRow[];
    accountTrendData: TrendRow[];
    allModels: string[];
    allAccounts: string[];
    pieData: PieSlice[];
    accountData: AccountTokenStats[];
    modelData: ModelTokenStats[];
    summary: TokenStatsSummary | null;
    loading: boolean;
}

interface ChartMouseEvent {
    activeCoordinate?: { x: number; y: number };
}

const TOOLTIP_WIDTH = 200;
const PIE_TOOLTIP_WIDTH = 180;
const EDGE_BUFFER = 20;
const TOOLTIP_OFFSET = 15;

export function StatsCharts(props: StatsChartsProps) {
    const { t } = useTranslation();
    const {
        viewMode,
        setViewMode,
        timeRange,
        chartData,
        modelTrendData,
        accountTrendData,
        allModels,
        allAccounts,
        pieData,
        accountData,
        modelData,
        summary,
        loading,
    } = props;

    const trendChartContainerRef = useRef<HTMLDivElement>(null);
    const [tooltipPosition, setTooltipPosition] = useState<{ x: number; y: number } | undefined>(undefined);

    // Ref and state for pie chart tooltip position
    const pieChartContainerRef = useRef<HTMLDivElement>(null);
    const [pieTooltipPosition, setPieTooltipPosition] = useState<{ x: number; y: number } | undefined>(undefined);

    // Handle mouse move to calculate tooltip position
    const handleTrendChartMouseMove = useCallback((e: ChartMouseEvent) => {
        if (!trendChartContainerRef.current || !e?.activeCoordinate) return;

        const containerRect = trendChartContainerRef.current.getBoundingClientRect();
        const rightEdgeThreshold = containerRect.width - TOOLTIP_WIDTH - EDGE_BUFFER;

        const mouseXInContainer = e.activeCoordinate.x;

        if (mouseXInContainer > rightEdgeThreshold) {
            setTooltipPosition({
                x: e.activeCoordinate.x - TOOLTIP_WIDTH - TOOLTIP_OFFSET,
                y: e.activeCoordinate.y
            });
        } else {
            setTooltipPosition(undefined); // Use default positioning
        }
    }, []);

    // Handle mouse move for pie chart to calculate tooltip position
    const handlePieChartMouseMove = useCallback((e: ChartMouseEvent) => {
        if (!pieChartContainerRef.current) return;

        const containerRect = pieChartContainerRef.current.getBoundingClientRect();

        // Get mouse position relative to container
        if (e?.activeCoordinate) {
            const mouseXInContainer = e.activeCoordinate.x;
            const rightEdgeThreshold = containerRect.width - PIE_TOOLTIP_WIDTH - EDGE_BUFFER;

            if (mouseXInContainer > rightEdgeThreshold) {
                setPieTooltipPosition({
                    x: e.activeCoordinate.x - PIE_TOOLTIP_WIDTH - TOOLTIP_OFFSET,
                    y: e.activeCoordinate.y
                });
            } else {
                setPieTooltipPosition(undefined);
            }
        }
    }, []);

    return (
        <>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-lg font-semibold text-gray-800 dark:text-white flex items-center gap-2">
                        {viewMode === 'model' ? (
                            <Cpu className="w-5 h-5 text-purple-500" />
                        ) : (
                            <Users className="w-5 h-5 text-green-500" />
                        )}
                        {viewMode === 'model'
                            ? t('token_stats.model_trend', '分模型使用趋势')
                            : t('token_stats.account_trend', '分账号使用趋势')
                        }
                    </h2>
                    <div className="flex bg-gray-100/80 dark:bg-gray-700/50 rounded-lg p-1">
                        <button
                            onClick={() => setViewMode('model')}
                            className={`px-3 py-1 text-xs font-medium rounded-md transition-all ${viewMode === 'model'
                                ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                                : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                                }`}
                        >
                            {t('token_stats.by_model', '按模型')}
                        </button>
                        <button
                            onClick={() => setViewMode('account')}
                            className={`px-3 py-1 text-xs font-medium rounded-md transition-all ${viewMode === 'account'
                                ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                                : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                                }`}
                        >
                            {t('token_stats.by_account_view', '按账号')}
                        </button>
                    </div>
                </div>
                <div className="h-72" ref={trendChartContainerRef}>
                    {modelTrendData.length > 0 && allModels.length > 0 ? (
                        <ResponsiveContainer width="100%" height="100%">
                            <AreaChart
                                data={viewMode === 'model' ? modelTrendData : accountTrendData}
                                onMouseMove={handleTrendChartMouseMove}
                                onMouseLeave={() => setTooltipPosition(undefined)}
                            >
                                <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#374151" strokeOpacity={0.15} />
                                <XAxis
                                    dataKey="period"
                                    tick={{ fontSize: 11, fill: '#6b7280' }}
                                    tickFormatter={(val) => {
                                        if (timeRange === 'hourly') return val.split(' ')[1] || val;
                                        if (timeRange === 'daily') return val.split('-').slice(1).join('/');
                                        return val;
                                    }}
                                    axisLine={false}
                                    tickLine={false}
                                    dy={10}
                                />
                                <YAxis
                                    tick={{ fontSize: 11, fill: '#6b7280' }}
                                    tickFormatter={(val) => formatNumber(val)}
                                    axisLine={false}
                                    tickLine={false}
                                />
                                <Tooltip
                                    content={<CustomTrendTooltip viewMode={viewMode} />}
                                    cursor={{ stroke: '#6b7280', strokeWidth: 1, strokeDasharray: '4 4', fill: 'transparent' }}
                                    allowEscapeViewBox={{ x: true, y: true }}
                                    position={tooltipPosition}
                                    wrapperStyle={{ zIndex: 100 }}
                                />
                                <Legend
                                    formatter={(value) => viewMode === 'model' ? shortenModelName(value) : value.split('@')[0]}
                                    wrapperStyle={{
                                        fontSize: '11px',
                                        paddingTop: '10px',
                                        maxHeight: '60px',
                                        overflowY: 'auto',
                                        zIndex: 0
                                    }}
                                />
                                {(viewMode === 'model' ? allModels : allAccounts).map((item, index) => (
                                    <Area
                                        key={item}
                                        type="monotone"
                                        dataKey={item}
                                        stackId="1"
                                        stroke={viewMode === 'model' ? MODEL_COLORS[index % MODEL_COLORS.length] : COLORS[index % COLORS.length]}
                                        fill={viewMode === 'model' ? MODEL_COLORS[index % MODEL_COLORS.length] : COLORS[index % COLORS.length]}
                                        fillOpacity={0.6}
                                    />
                                ))}
                            </AreaChart>
                        </ResponsiveContainer>
                    ) : (
                        <div className="h-full flex items-center justify-center text-gray-400">
                            {loading ? t('common.loading', '加载中...') : t('token_stats.no_data', '暂无数据')}
                        </div>
                    )}
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div className="lg:col-span-2 bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700 flex flex-col">
                    <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">
                        {t('token_stats.usage_trend', 'Token 使用趋势')}
                    </h2>
                    <div className="flex-1 min-h-[16rem]">
                        {chartData.length > 0 ? (
                            <ResponsiveContainer width="100%" height="100%">
                                <BarChart data={chartData}>
                                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#374151" strokeOpacity={0.15} />
                                    <XAxis
                                        dataKey="period"
                                        tick={{ fontSize: 11, fill: '#6b7280' }}
                                        tickFormatter={(val) => {
                                            if (timeRange === 'hourly') return val.split(' ')[1] || val;
                                            if (timeRange === 'daily') return val.split('-').slice(1).join('/');
                                            return val;
                                        }}
                                        axisLine={false}
                                        tickLine={false}
                                        dy={10}
                                    />
                                    <YAxis
                                        tick={{ fontSize: 11, fill: '#6b7280' }}
                                        tickFormatter={(val) => formatNumber(val)}
                                        axisLine={false}
                                        tickLine={false}
                                    />
                                    <Tooltip
                                        content={<UsageTrendTooltip />}
                                        cursor={{ fill: 'transparent' }}
                                        allowEscapeViewBox={{ x: true, y: true }}
                                        wrapperStyle={{ zIndex: 100 }}
                                    />
                                    <Bar dataKey="total_cached_tokens" name={t('token_stats.cached_token', '缓存命中')} stackId="input" fill="#93c5fd" radius={[0, 0, 4, 4]} maxBarSize={50} />
                                    <Bar dataKey="uncached_input_tokens" name={t('token_stats.input', '输入')} stackId="input" fill="#3b82f6" radius={[4, 4, 0, 0]} maxBarSize={50} />
                                    <Bar dataKey="total_output_tokens" name={t('token_stats.output', '输出')} fill="#8b5cf6" radius={[4, 4, 0, 0]} maxBarSize={50} />
                                </BarChart>
                            </ResponsiveContainer>
                        ) : (
                            <div className="h-full flex items-center justify-center text-gray-400">
                                {loading ? t('common.loading', '加载中...') : t('token_stats.no_data', '暂无数据')}
                            </div>
                        )}
                    </div>
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
                    <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">
                        {t('token_stats.by_account', '分账号统计')}
                    </h2>
                    <div className="h-48" ref={pieChartContainerRef}>
                        {pieData.length > 0 ? (
                            <ResponsiveContainer width="100%" height="100%">
                                <PieChart
                                    onMouseMove={handlePieChartMouseMove}
                                    onMouseLeave={() => setPieTooltipPosition(undefined)}
                                >
                                    <Pie
                                        data={pieData}
                                        cx="50%"
                                        cy="50%"
                                        innerRadius={40}
                                        outerRadius={70}
                                        paddingAngle={2}
                                        dataKey="value"
                                    >
                                        {pieData.map((entry, index) => (
                                            <Cell key={`cell-${index}`} fill={entry.color} />
                                        ))}
                                    </Pie>
                                    <Tooltip
                                        content={<CustomPieTooltip />}
                                        allowEscapeViewBox={{ x: true, y: true }}
                                        position={pieTooltipPosition}
                                        wrapperStyle={{ zIndex: 100 }}
                                    />
                                </PieChart>
                            </ResponsiveContainer>
                        ) : (
                            <div className="h-full flex items-center justify-center text-gray-400">
                                {loading ? t('common.loading', '加载中...') : t('token_stats.no_data', '暂无数据')}
                            </div>
                        )}
                    </div>
                    <div className="mt-4 space-y-2 max-h-32 overflow-y-auto">
                        {accountData.slice(0, PIE_LEGEND_LIMIT).map((account, index) => (
                            <div key={account.account_email} className="flex items-center justify-between text-sm">
                                <div className="flex items-center gap-2">
                                    <div
                                        className="w-3 h-3 rounded-full"
                                        style={{ backgroundColor: COLORS[index % COLORS.length] }}
                                    />
                                    <span className="text-gray-600 dark:text-gray-300 truncate max-w-[120px]">
                                        {account.account_email.split('@')[0]}
                                    </span>
                                </div>
                                <span className="font-medium text-gray-800 dark:text-white">
                                    {formatNumber(account.total_tokens)}
                                </span>
                            </div>
                        ))}
                    </div>
                </div>
            </div>


            {
                modelData.length > 0 && viewMode === 'model' && (
                    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
                        <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4 flex items-center gap-2">
                            <Cpu className="w-5 h-5 text-blue-500" />
                            {t('token_stats.model_details', '分模型详细统计')}
                        </h2>
                        <div className="overflow-x-auto">
                            <table className="w-full text-sm">
                                <thead>
                                    <tr className="border-b border-gray-200 dark:border-gray-700">
                                        <th className="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.model', '模型')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.requests', '请求数')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.input', '输入')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.output', '输出')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.cached_token', '缓存命中')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.total', '合计')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.percentage', '占比')}
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {modelData.map((model, index) => {
                                        const percentage = summary ? ((model.total_tokens / summary.total_tokens) * 100).toFixed(1) : '0';
                                        return (
                                            <tr
                                                key={model.model}
                                                className="border-b border-gray-100 dark:border-gray-700/50 hover:bg-gray-50 dark:hover:bg-gray-700/30"
                                            >
                                                <td className="py-3 px-4">
                                                    <div className="flex items-center gap-2">
                                                        <div
                                                            className="w-3 h-3 rounded-full"
                                                            style={{ backgroundColor: MODEL_COLORS[index % MODEL_COLORS.length] }}
                                                        />
                                                        <span className="text-gray-800 dark:text-white font-medium">
                                                            {model.model}
                                                        </span>
                                                    </div>
                                                </td>
                                                <td className="py-3 px-4 text-right text-gray-600 dark:text-gray-300">
                                                    {model.request_count.toLocaleString()}
                                                </td>
                                                <td className="py-3 px-4 text-right text-blue-600">
                                                    {formatNumber(model.total_input_tokens)}
                                                </td>
                                                <td className="py-3 px-4 text-right text-purple-600">
                                                    {formatNumber(model.total_output_tokens)}
                                                </td>
                                                <td className="py-3 px-4 text-right text-sky-600">
                                                    {formatNumber(model.total_cached_tokens)}
                                                </td>
                                                <td className="py-3 px-4 text-right font-semibold text-gray-800 dark:text-white">
                                                    {formatNumber(model.total_tokens)}
                                                </td>
                                                <td className="py-3 px-4 text-right">
                                                    <div className="flex items-center justify-end gap-2">
                                                        <div className="w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                                            <div
                                                                className="h-2 rounded-full"
                                                                style={{
                                                                    width: `${percentage}%`,
                                                                    backgroundColor: MODEL_COLORS[index % MODEL_COLORS.length]
                                                                }}
                                                            />
                                                        </div>
                                                        <span className="text-gray-600 dark:text-gray-300 w-12 text-right">
                                                            {percentage}%
                                                        </span>
                                                    </div>
                                                </td>
                                            </tr>
                                        );
                                    })}
                                </tbody>
                            </table>
                        </div>
                    </div>
                )
            }



            {
                accountData.length > 0 && viewMode === 'account' && (
                    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
                        <h2 className="text-lg font-semibold text-gray-800 dark:text-white mb-4">
                            {t('token_stats.account_details', '账号详细统计')}
                        </h2>
                        <div className="overflow-x-auto">
                            <table className="w-full text-sm">
                                <thead>
                                    <tr className="border-b border-gray-200 dark:border-gray-700">
                                        <th className="text-left py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.account', '账号')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.requests', '请求数')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.input', '输入')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.output', '输出')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.cached_token', '缓存命中')}
                                        </th>
                                        <th className="text-right py-3 px-4 font-medium text-gray-500 dark:text-gray-400">
                                            {t('token_stats.total', '合计')}
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {accountData.map((account) => (
                                        <tr
                                            key={account.account_email}
                                            className="border-b border-gray-100 dark:border-gray-700/50 hover:bg-gray-50 dark:hover:bg-gray-700/30"
                                        >
                                            <td className="py-3 px-4 text-gray-800 dark:text-white">
                                                {account.account_email}
                                            </td>
                                            <td className="py-3 px-4 text-right text-gray-600 dark:text-gray-300">
                                                {account.request_count.toLocaleString()}
                                            </td>
                                            <td className="py-3 px-4 text-right text-blue-600">
                                                {formatNumber(account.total_input_tokens)}
                                            </td>
                                            <td className="py-3 px-4 text-right text-purple-600">
                                                {formatNumber(account.total_output_tokens)}
                                            </td>
                                            <td className="py-3 px-4 text-right text-sky-600">
                                                {formatNumber(account.total_cached_tokens)}
                                            </td>
                                            <td className="py-3 px-4 text-right font-semibold text-gray-800 dark:text-white">
                                                {formatNumber(account.total_tokens)}
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    </div>
                )
            }
        </>
    );
}
