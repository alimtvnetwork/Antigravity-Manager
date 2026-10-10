import React from 'react';
import { useTranslation } from 'react-i18next';
import { Clock, Calendar, CalendarDays, Users, Zap, TrendingUp, RefreshCw, Cpu } from 'lucide-react';
import { useTokenStats, formatNumber } from './token-stats/useTokenStats';
import { StatsCharts } from './token-stats/StatsCharts';

const TokenStats: React.FC = () => {
    const { t } = useTranslation();
    const {
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
    } = useTokenStats();

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="px-4 sm:px-6 pt-2 pb-4 space-y-4 max-w-[1920px] mx-auto">
                <div className="flex items-center justify-between">
                    <h1 className="text-2xl font-bold text-gray-800 dark:text-white flex items-center gap-2">
                        <Zap className="w-6 h-6 text-blue-500" />
                        {t('token_stats.title', 'Token 消费统计')}
                    </h1>
                    <div className="flex items-center gap-2">
                        <div className="flex bg-gray-100 dark:bg-gray-800 rounded-lg p-1">
                            <button
                                onClick={() => setTimeRange('hourly')}
                                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors flex items-center gap-1.5 ${timeRange === 'hourly'
                                    ? 'bg-white dark:bg-gray-700 text-blue-600 shadow-sm'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-800'
                                    }`}
                            >
                                <Clock className="w-4 h-4" />
                                {t('token_stats.hourly', '小时')}
                            </button>
                            <button
                                onClick={() => setTimeRange('daily')}
                                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors flex items-center gap-1.5 ${timeRange === 'daily'
                                    ? 'bg-white dark:bg-gray-700 text-blue-600 shadow-sm'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-800'
                                    }`}
                            >
                                <Calendar className="w-4 h-4" />
                                {t('token_stats.daily', '日')}
                            </button>
                            <button
                                onClick={() => setTimeRange('weekly')}
                                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors flex items-center gap-1.5 ${timeRange === 'weekly'
                                    ? 'bg-white dark:bg-gray-700 text-blue-600 shadow-sm'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-800'
                                    }`}
                            >
                                <CalendarDays className="w-4 h-4" />
                                {t('token_stats.weekly', '周')}
                            </button>
                        </div>
                        <button
                            onClick={fetchData}
                            disabled={loading}
                            className="p-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 transition-colors disabled:opacity-50"
                        >
                            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                        </button>
                    </div>
                </div>

                {summary && (
                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
                        <div className="bg-gradient-to-br from-white to-gray-50 dark:from-gray-800 dark:to-gray-800/50 rounded-xl p-4 shadow-sm border border-gray-200 dark:border-gray-700 hover:shadow-md transition-shadow">
                            <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400 text-sm mb-2">
                                <div className="p-1.5 rounded-lg bg-gray-100 dark:bg-gray-700">
                                    <Zap className="w-4 h-4 text-gray-600 dark:text-gray-300" />
                                </div>
                                {t('token_stats.total_tokens', '总 Token')}
                            </div>
                            <div className="text-2xl font-bold text-gray-800 dark:text-white">
                                {formatNumber(summary.total_tokens)}
                            </div>
                        </div>
                        <div className="bg-gradient-to-br from-blue-50/50 to-white dark:from-blue-900/10 dark:to-gray-800 rounded-xl p-4 shadow-sm border border-blue-100 dark:border-blue-900/30 hover:shadow-md transition-shadow">
                            <div className="flex items-center gap-2 text-blue-600/80 dark:text-blue-400/80 text-sm mb-2">
                                <div className="p-1.5 rounded-lg bg-blue-100/50 dark:bg-blue-900/30">
                                    <TrendingUp className="w-4 h-4 text-blue-600 dark:text-blue-400" />
                                </div>
                                {t('token_stats.input_tokens', '输入 Token')}
                            </div>
                            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                                {formatNumber(summary.total_input_tokens)}
                            </div>
                        </div>
                        <div className="bg-gradient-to-br from-purple-50/50 to-white dark:from-purple-900/10 dark:to-gray-800 rounded-xl p-4 shadow-sm border border-purple-100 dark:border-purple-900/30 hover:shadow-md transition-shadow">
                            <div className="flex items-center gap-2 text-purple-600/80 dark:text-purple-400/80 text-sm mb-2">
                                <div className="p-1.5 rounded-lg bg-purple-100/50 dark:bg-purple-900/30">
                                    <TrendingUp className="w-4 h-4 rotate-180 text-purple-600 dark:text-purple-400" />
                                </div>
                                {t('token_stats.output_tokens', '输出 Token')}
                            </div>
                            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                                {formatNumber(summary.total_output_tokens)}
                            </div>
                        </div>
                        <div className="bg-gradient-to-br from-sky-50/50 to-white dark:from-sky-900/10 dark:to-gray-800 rounded-xl p-4 shadow-sm border border-sky-100 dark:border-sky-900/30 hover:shadow-md transition-shadow">
                            <div className="flex items-center gap-2 text-sky-600/80 dark:text-sky-400/80 text-sm mb-2">
                                <div className="p-1.5 rounded-lg bg-sky-100/50 dark:bg-sky-900/30">
                                    <Zap className="w-4 h-4 text-sky-600 dark:text-sky-400" />
                                </div>
                                {t('token_stats.cached_token', '缓存命中')}
                            </div>
                            <div className="text-2xl font-bold text-sky-600 dark:text-sky-400">
                                {formatNumber(summary.total_cached_tokens)}
                            </div>
                        </div>
                        <div className="bg-gradient-to-br from-green-50/50 to-white dark:from-green-900/10 dark:to-gray-800 rounded-xl p-4 shadow-sm border border-green-100 dark:border-green-900/30 hover:shadow-md transition-shadow">
                            <div className="flex items-center gap-2 text-green-600/80 dark:text-green-400/80 text-sm mb-2">
                                <div className="p-1.5 rounded-lg bg-green-100/50 dark:bg-green-900/30">
                                    <Users className="w-4 h-4 text-green-600 dark:text-green-400" />
                                </div>
                                {t('token_stats.accounts_used', '活跃账号')}
                            </div>
                            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                                {summary.unique_accounts}
                            </div>
                        </div>
                        <div className="bg-gradient-to-br from-orange-50/50 to-white dark:from-orange-900/10 dark:to-gray-800 rounded-xl p-4 shadow-sm border border-orange-100 dark:border-orange-900/30 hover:shadow-md transition-shadow">
                            <div className="flex items-center gap-2 text-orange-600/80 dark:text-orange-400/80 text-sm mb-2">
                                <div className="p-1.5 rounded-lg bg-orange-100/50 dark:bg-orange-900/30">
                                    <Cpu className="w-4 h-4 text-orange-600 dark:text-orange-400" />
                                </div>
                                {t('token_stats.models_used', '使用模型')}
                            </div>
                            <div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
                                {modelData.length}
                            </div>
                        </div>
                    </div>
                )}

                <StatsCharts
                    viewMode={viewMode}
                    setViewMode={setViewMode}
                    timeRange={timeRange}
                    chartData={chartData}
                    modelTrendData={modelTrendData}
                    accountTrendData={accountTrendData}
                    allModels={allModels}
                    allAccounts={allAccounts}
                    pieData={pieData}
                    accountData={accountData}
                    modelData={modelData}
                    summary={summary}
                    loading={loading}
                />
            </div>
        </div>
    );
};

export default TokenStats;
