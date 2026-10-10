import React from 'react';
import { useTranslation } from 'react-i18next';
import {
    ExternalLink,
    Eye,
    EyeOff,
    Copy,
    X,
    RefreshCw,
    Activity,
    Layers,
    DollarSign,
    Coins,
    Cpu,
    Flame,
    Hash,
    TrendingUp
} from 'lucide-react';
import { motion } from 'framer-motion';
import { useApiKeyFun } from './api-key-fun/useApiKeyFun';
import { ApiKeyList } from './api-key-fun/ApiKeyList';
import { SyncSection } from './api-key-fun/SyncSection';

export const ApiKeyFun: React.FC = () => {
    const { t } = useTranslation();
    const state = useApiKeyFun();
    const {
        apiKey,
        baseUrl,
        showApiKey,
        setShowApiKey,
        querying,
        usage,
        models,
        queryError,
        modelsError,
        runQuery,
        handleCopy,
        handleApiKeyChange,
    } = state;

    return (
        <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="h-full flex flex-col px-4 sm:px-6 pt-2 pb-4 gap-4 overflow-y-auto max-w-[90rem] mx-auto w-full"
        >
            {/* Header Card */}
            <div
                className="w-full rounded-2xl border border-blue-100 dark:border-indigo-500/20 p-6 md:p-0 md:px-8 md:h-[140px] flex flex-col md:flex-row items-center justify-between gap-6 shadow-xl relative overflow-hidden shrink-0 bg-gradient-to-r from-blue-50 via-indigo-50/60 to-purple-50 dark:from-indigo-950 dark:via-purple-900/40 dark:to-slate-900 transition-colors duration-300"
            >
                {/* Decorative background elements for light mode */}
                <div className="absolute top-0 right-0 w-64 h-64 bg-purple-400/10 dark:bg-purple-500/10 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2"></div>
                <div className="absolute bottom-0 left-0 w-64 h-64 bg-blue-400/10 dark:bg-blue-500/10 rounded-full blur-3xl translate-y-1/2 -translate-x-1/2"></div>

                <div className="flex flex-col md:flex-row items-center md:items-center gap-5 text-center md:text-left z-10 w-full md:w-auto">
                    {/* Branded Logo Box */}
                    <div className="w-14 h-14 md:w-16 md:h-16 bg-white dark:bg-base-100 rounded-2xl flex items-center justify-center shadow-[0_0_20px_rgba(59,130,246,0.15)] dark:shadow-[0_0_25px_rgba(59,130,246,0.3)] border border-blue-100 dark:border-blue-900/50 flex-shrink-0 select-none transform transition-transform duration-300 hover:scale-105">
                        <span className="text-xl md:text-2xl font-bold text-[#e05220] font-sans">
                            {"{AK}"}
                        </span>
                    </div>

                    {/* Info */}
                    <div className="flex flex-col gap-1.5 max-w-4xl">
                        <div className="flex flex-col md:flex-row items-center md:items-end gap-3">
                            <h1 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white tracking-wide leading-none">
                                {t('apiKeyFun.title', { defaultValue: 'APIKEY.FUN 中转站' })}
                            </h1>
                            <span className="bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300 border border-blue-200 dark:border-blue-800/40 px-2.5 py-0.5 rounded-full text-[10px] font-semibold tracking-wide uppercase">
                                {t('apiKeyFun.eyebrow', { defaultValue: '中转站' })}
                            </span>
                        </div>
                        <p className="text-xs md:text-sm text-gray-600 dark:text-gray-300/90 leading-relaxed font-normal mt-1">
                            {t('apiKeyFun.description', { defaultValue: 'Antigravity Tools 官方合作中转站，为用户提供稳定、开放、高性价比的大模型 API 接入服务。支持 Claude、OpenAI、Gemini 等主流模型，适合在 Codex、Gemini CLI、Claude Code 及其他开发工具中统一配置使用。通过 Antigravity Tools 专属链接注册，可享受最高充值永久 95 折优惠。' })}
                        </p>
                    </div>
                </div>

                <a
                    href="https://apikey.fan/register?aff=AntManager"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="bg-white hover:bg-blue-50 dark:bg-base-200 dark:hover:bg-base-300 text-blue-700 dark:text-blue-400 border border-blue-200 dark:border-blue-800/50 px-6 py-3 rounded-xl font-bold text-sm flex items-center gap-2 transition-all shadow-md shadow-blue-500/10 dark:shadow-none flex-shrink-0 hover:scale-[1.02] active:scale-[0.98] duration-200 z-10"
                >
                    <ExternalLink size={16} className="text-blue-500 dark:text-blue-400" />
                    <span>{t('apiKeyFun.viewNow', { defaultValue: '立即查看' })}</span>
                </a>
            </div>

            {/* Stats Grid - Full Width Dashboard Metrics */}
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4 w-full">
                {/* Remaining */}
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-row items-center gap-3.5 transition-all duration-300 hover:shadow-md hover:border-blue-100 dark:hover:border-blue-900/30 group"
                >
                    <div className="p-3 bg-green-50 dark:bg-green-950/20 rounded-xl flex-shrink-0 text-green-500 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-300">
                        <Coins className="w-5 h-5" />
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-xs font-medium text-gray-400 dark:text-gray-500 truncate">
                            {t('apiKeyFun.usage.remainingAmount', { defaultValue: '剩余额度' })}
                        </span>
                        <span className="text-xl font-bold text-gray-900 dark:text-white mt-0.5 tracking-tight truncate">
                            {usage ? usage.remaining : '$0.00'}
                        </span>
                    </div>
                </motion.div>

                {/* Used */}
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-row items-center gap-3.5 transition-all duration-300 hover:shadow-md hover:border-blue-100 dark:hover:border-blue-900/30 group"
                >
                    <div className="p-3 bg-blue-50 dark:bg-blue-950/20 rounded-xl flex-shrink-0 text-blue-500 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-300">
                        <DollarSign className="w-5 h-5" />
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-xs font-medium text-gray-400 dark:text-gray-500 truncate">
                            {t('apiKeyFun.usage.usedAmount', { defaultValue: '已用额度' })}
                        </span>
                        <span className="text-xl font-bold text-gray-900 dark:text-white mt-0.5 tracking-tight truncate">
                            {usage ? usage.used : '--'}
                        </span>
                    </div>
                </motion.div>

                {/* Today Requests */}
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-row items-center gap-3.5 transition-all duration-300 hover:shadow-md hover:border-blue-100 dark:hover:border-blue-900/30 group"
                >
                    <div className="p-3 bg-orange-50 dark:bg-orange-950/20 rounded-xl flex-shrink-0 text-orange-500 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-300">
                        <Flame className="w-5 h-5" />
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-xs font-medium text-gray-400 dark:text-gray-500 truncate">
                            {t('apiKeyFun.usage.todayRequests', { defaultValue: 'Today Requests' })}
                        </span>
                        <span className="text-xl font-bold text-gray-900 dark:text-white mt-0.5 tracking-tight truncate">
                            {usage ? usage.todayRequests : '--'}
                        </span>
                    </div>
                </motion.div>

                {/* Today Tokens */}
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-row items-center gap-3.5 transition-all duration-300 hover:shadow-md hover:border-blue-100 dark:hover:border-blue-900/30 group"
                >
                    <div className="p-3 bg-purple-50 dark:bg-purple-950/20 rounded-xl flex-shrink-0 text-purple-500 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-300">
                        <Cpu className="w-5 h-5" />
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-xs font-medium text-gray-400 dark:text-gray-500 truncate">
                            {t('apiKeyFun.usage.todayTokens', { defaultValue: 'Today Tokens' })}
                        </span>
                        <span className="text-xl font-bold text-gray-900 dark:text-white mt-0.5 tracking-tight truncate">
                            {usage ? usage.todayTokens : '--'}
                        </span>
                    </div>
                </motion.div>

                {/* Total Requests */}
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-row items-center gap-3.5 transition-all duration-300 hover:shadow-md hover:border-blue-100 dark:hover:border-blue-900/30 group"
                >
                    <div className="p-3 bg-indigo-50 dark:bg-indigo-950/20 rounded-xl flex-shrink-0 text-indigo-500 group-hover:scale-110 group-hover:-rotate-3 transition-transform duration-300">
                        <TrendingUp className="w-5 h-5" />
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-xs font-medium text-gray-400 dark:text-gray-500 truncate">
                            {t('apiKeyFun.usage.totalRequests', { defaultValue: 'Total Requests' })}
                        </span>
                        <span className="text-xl font-bold text-gray-900 dark:text-white mt-0.5 tracking-tight truncate">
                            {usage ? usage.totalRequests : '--'}
                        </span>
                    </div>
                </motion.div>

                {/* Total Tokens */}
                <motion.div
                    whileHover={{ y: -2 }}
                    className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-row items-center gap-3.5 transition-all duration-300 hover:shadow-md hover:border-blue-100 dark:hover:border-blue-900/30 group"
                >
                    <div className="p-3 bg-pink-50 dark:bg-pink-950/20 rounded-xl flex-shrink-0 text-pink-500 group-hover:scale-110 group-hover:-rotate-3 transition-transform duration-300">
                        <Hash className="w-5 h-5" />
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-xs font-medium text-gray-400 dark:text-gray-500 truncate">
                            {t('apiKeyFun.usage.totalTokens', { defaultValue: 'Total Tokens' })}
                        </span>
                        <span className="text-xl font-bold text-gray-900 dark:text-white mt-0.5 tracking-tight truncate">
                            {usage ? usage.totalTokens : '--'}
                        </span>
                    </div>
                </motion.div>
            </div>

            {/* Bottom Section Layout */}
            <div className="grid grid-cols-1 xl:grid-cols-12 lg:grid-cols-12 gap-6 items-start mt-2 pb-8">

                {/* Left Sidebar: Saved Keys List */}
                <ApiKeyList
                    managedKeys={state.managedKeys}
                    apiKey={apiKey}
                    baseUrl={baseUrl}
                    editingId={state.editingId}
                    setEditingId={state.setEditingId}
                    editNameValue={state.editNameValue}
                    setEditNameValue={state.setEditNameValue}
                    syncingKey={state.syncingKey}
                    querying={querying}
                    handleSelectKey={state.handleSelectKey}
                    handleCopy={handleCopy}
                    handleDeleteKey={state.handleDeleteKey}
                    startRename={state.startRename}
                    saveRename={state.saveRename}
                    getModelsForKey={state.getModelsForKey}
                    profileInfo={state.profileInfo}
                    handleToggleOpenCodeProfile={state.handleToggleOpenCodeProfile}
                />

                {/* Right Main Panel: Query and Models */}
                <div className="xl:col-span-8 lg:col-span-8 space-y-6">
                    {/* Query Form */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-5 shadow-sm border border-gray-100 dark:border-base-200">
                        <h2 className="text-lg font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                            <Activity size={18} className="text-blue-500" />
                            {t('apiKeyFun.queryTitle', { defaultValue: 'Key Quota Query' })}
                        </h2>

                        <div className="flex flex-col gap-4">
                            <div className="flex flex-col sm:flex-row gap-4 items-end w-full">
                                <div className="form-control w-full flex-1">
                                    <label className="label mb-1">
                                        <span className="label-text font-bold text-slate-700 dark:text-gray-300">{t('apiKeyFun.apiKeyLabel', { defaultValue: 'API Key' })} <span className="text-red-500 ml-0.5">*</span></span>
                                    </label>
                                    <div className="relative">
                                        <input
                                            type={showApiKey ? 'text' : 'password'}
                                            className="w-full h-14 pl-5 pr-36 font-mono text-sm bg-slate-50 dark:bg-black/20 border-2 border-slate-200 dark:border-white/10 rounded-2xl focus:bg-white dark:focus:bg-black/40 focus:border-blue-500 dark:focus:border-blue-500/80 focus:ring-4 focus:ring-blue-500/20 dark:focus:ring-blue-500/10 transition-all shadow-sm outline-none text-gray-800 dark:text-gray-200 placeholder-slate-400 dark:placeholder-gray-600"
                                            placeholder={t('apiKeyFun.apiKeyPlaceholder', { defaultValue: 'Paste your API Key...' })}
                                            value={apiKey}
                                            onChange={e => handleApiKeyChange(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === 'Enter' && !querying && apiKey) {
                                                    runQuery(apiKey, baseUrl);
                                                }
                                            }}
                                        />
                                        <div className="absolute right-2.5 top-1/2 -translate-y-1/2 flex items-center gap-1">
                                            {apiKey && (
                                                <button
                                                    className="p-2 rounded-lg text-slate-400 hover:bg-slate-200 hover:text-slate-600 dark:text-gray-500 dark:hover:bg-white/10 dark:hover:text-gray-300 transition-colors"
                                                    onClick={() => handleApiKeyChange('')}
                                                    title="Clear"
                                                >
                                                    <X size={16} strokeWidth={2.5} />
                                                </button>
                                            )}
                                            <button
                                                className="p-2 rounded-lg text-slate-400 hover:bg-slate-200 hover:text-slate-600 dark:text-gray-500 dark:hover:bg-white/10 dark:hover:text-gray-300 transition-colors"
                                                onClick={() => setShowApiKey(!showApiKey)}
                                                title={showApiKey ? "Hide" : "Show"}
                                            >
                                                {showApiKey ? <EyeOff size={16} strokeWidth={2.5} /> : <Eye size={16} strokeWidth={2.5} />}
                                            </button>
                                            <div className="w-px h-5 bg-slate-200 dark:bg-white/10 mx-1"></div>
                                            <button
                                                className="p-2 rounded-xl bg-white dark:bg-white/5 hover:bg-blue-50 dark:hover:bg-blue-500/20 text-slate-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 transition-all border border-slate-200 dark:border-white/5 shadow-sm"
                                                onClick={() => handleCopy(apiKey)}
                                                disabled={!apiKey}
                                                title="Copy"
                                            >
                                                <Copy size={16} strokeWidth={2.5} />
                                            </button>
                                        </div>
                                    </div>
                                </div>
                                <div className="flex gap-2 w-full sm:w-auto shrink-0 mt-8">
                                    <button
                                        onClick={() => runQuery(apiKey, baseUrl)}
                                        className={`btn h-14 min-h-0 rounded-2xl bg-blue-500 hover:bg-blue-600 text-white border-none flex items-center justify-center gap-2 shadow-md shadow-blue-500/20 transition-all sm:w-32 text-sm font-bold ${querying ? 'opacity-70' : ''}`}
                                        disabled={querying || !apiKey}
                                    >
                                        <RefreshCw size={16} className={querying ? 'animate-spin' : ''} />
                                        <span>{querying ? t('common.loading') : t('common.refresh')}</span>
                                    </button>
                                </div>
                            </div>

                            {/* Premium CLI Actions Banner */}
                            <SyncSection
                                models={models}
                                apiKey={apiKey}
                                baseUrl={baseUrl}
                                querying={querying}
                                syncingKey={state.syncingKey}
                                handleSyncCli={state.handleSyncCli}
                                handleSyncOpenCode={state.handleSyncOpenCode}
                                getModelsForKey={state.getModelsForKey}
                                profileInfo={state.profileInfo}
                            />
                        </div>

                        {queryError && (
                            <div className="alert alert-error text-xs p-3 rounded-lg mt-4 flex items-start gap-2 bg-red-50 dark:bg-red-950/20 text-red-600 dark:text-red-400 border border-red-100 dark:border-red-900/30">
                                <span>{queryError}</span>
                            </div>
                        )}
                    </div>

                    {/* Available Models */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-5 shadow-sm border border-gray-100 dark:border-base-200">
                        <h2 className="text-base font-bold text-gray-900 dark:text-white mb-3 flex items-center gap-2">
                            <Layers size={16} className="text-blue-500" />
                            {t('apiKeyFun.models.title', { defaultValue: 'Available Models' })}
                            {models.length > 0 && <span className="text-xs font-normal text-gray-400">({models.length})</span>}
                        </h2>

                        {modelsError ? (
                            <div className="bg-red-50 dark:bg-red-950/20 text-red-600 dark:text-red-400 p-3 rounded-lg border border-red-100 dark:border-red-900/30 text-xs mt-2">
                                {modelsError}
                            </div>
                        ) : models.length === 0 ? (
                            <p className="text-xs text-gray-400 mt-2">
                                {apiKey ? t('apiKeyFun.models.emptyFromKey', { defaultValue: 'No models returned yet. Query to fetch models.' })
                                       : t('apiKeyFun.models.empty', { defaultValue: 'Enter key and query to load available models.' })}
                            </p>
                        ) : (
                            <div className="flex flex-wrap gap-1.5 max-h-[400px] overflow-y-auto pt-2">
                                {models.map(m => (
                                    <span key={m} className="px-2.5 py-1 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 text-xs rounded-md border border-gray-200 dark:border-base-300 font-mono hover:bg-gray-100 dark:hover:bg-base-300 transition-colors shadow-sm cursor-default">
                                        {m}
                                    </span>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

            </div>
        </motion.div>
    );
};
