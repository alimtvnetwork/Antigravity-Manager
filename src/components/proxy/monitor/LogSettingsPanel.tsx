import React from 'react';
import type { TFunction } from 'i18next';
import { Database, Check, X, Clock, HardDrive, Trash2 } from 'lucide-react';
import type { AppConfig } from '../../../types/config';

export interface LogSettingsPanelProps {
    t: TFunction;
    appConfig: AppConfig;
    dbDiskSizeBytes: number | null;
    isSavingConfig: boolean;
    saveSuccess: boolean;
    cacheClearedSuccess: boolean;
    onUpdateLogRetention: (field: 'max_body_age_hours' | 'max_storage_gb' | 'max_rows', value: number) => void;
    onUpdateExperimental: (field: 'payload_storage_mode' | 'thinking_retention_days', value: any) => void;
    onSaveLogSettings: () => void;
    onClose: () => void;
    onOpenClearCacheModal: () => void;
    onClearLogs: () => void;
}

const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

export const LogSettingsPanel: React.FC<LogSettingsPanelProps> = ({
    t,
    appConfig,
    dbDiskSizeBytes,
    isSavingConfig,
    saveSuccess,
    cacheClearedSuccess,
    onUpdateLogRetention,
    onUpdateExperimental,
    onSaveLogSettings,
    onClose,
    onOpenClearCacheModal,
    onClearLogs,
}) => {
    return (
            <div className="bg-gray-50/90 dark:bg-base-200 border-b border-gray-200 dark:border-base-300 p-4 space-y-3.5 shadow-xs">
                {/* Panel Header */}
                <div className="flex items-center justify-between border-b border-gray-200/80 dark:border-base-200 pb-2.5">
                    <div className="flex items-center gap-2">
                        <Database size={16} className="text-blue-600 dark:text-blue-400" />
                        <span className="text-sm font-bold text-gray-900 dark:text-white">
                            {t('monitor.settings.title', { defaultValue: '日志存储周期与维护设置' })}
                        </span>
                        <span className="text-xs text-gray-500 dark:text-gray-400 hidden sm:inline">
                            {t('monitor.settings.subtitle', { defaultValue: '统一管理请求日志保留天数、思考块滑动窗口与磁盘空间回收' })}
                        </span>
                    </div>
                    <div className="flex items-center gap-2">
                        <button
                            onClick={onSaveLogSettings}
                            disabled={isSavingConfig}
                            className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-all text-white active:scale-95 ${
                                saveSuccess
                                    ? 'bg-emerald-600 hover:bg-emerald-700'
                                    : 'bg-blue-600 hover:bg-blue-700'
                            }`}
                            title="保存全部日志与思考块配置"
                        >
                            <Check size={13} />
                            <span>{saveSuccess ? t('common.saved', { defaultValue: '已生效' }) : (isSavingConfig ? t('common.saving', { defaultValue: '保存中...' }) : t('common.save', { defaultValue: '全部保存并热生效' }))}</span>
                        </button>
                        <button
                            onClick={onClose}
                            className="btn btn-xs btn-ghost text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                        >
                            <X size={15} />
                        </button>
                    </div>
                </div>

                {/* 2-Column Balanced Settings Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                    {/* 1. 请求日志与报文保留策略 */}
                    <div className="p-3.5 bg-white dark:bg-base-100 rounded-xl border border-gray-200/90 dark:border-base-200 shadow-xs flex flex-col justify-between space-y-3">
                        <div className="space-y-3">
                            <span className="text-xs font-bold text-gray-800 dark:text-gray-200 flex items-center gap-1.5">
                                <Clock size={13} className="text-indigo-500 dark:text-indigo-400" />
                                {t('monitor.settings.retention_title', { defaultValue: '请求日志与报文保留策略 (滑动窗口)' })}
                            </span>
                            <div className="space-y-2.5">
                                {/* 空间上限 */}
                                <div>
                                    <div className="flex items-center justify-between mb-1">
                                        <label className="text-xs font-medium text-gray-600 dark:text-gray-300">
                                            {t('proxy.config.log_retention_storage_gb', { defaultValue: '日志保留空间上限 (GB)' })}
                                        </label>
                                        <span className="text-[10px] text-gray-500 dark:text-gray-400">
                                            {t('proxy.config.log_retention_current_usage', { defaultValue: '当前库占用' })}: <strong className="font-mono text-gray-700 dark:text-gray-200">{dbDiskSizeBytes !== null ? formatBytes(dbDiskSizeBytes) : '...'}</strong>
                                        </span>
                                    </div>
                                    <input
                                        type="number"
                                        min={0.1}
                                        max={100}
                                        step={0.1}
                                        value={appConfig.proxy.log_retention?.max_storage_gb ?? 1.0}
                                        onChange={(e) => onUpdateLogRetention('max_storage_gb', parseFloat(e.target.value))}
                                        className="input input-xs input-bordered bg-gray-50 dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-800 dark:text-white w-full font-mono text-xs focus:border-blue-500"
                                    />
                                    <p className="text-[10px] text-gray-500 dark:text-gray-400 mt-0.5 leading-tight">
                                        {t('proxy.config.log_retention_storage_gb_desc', { defaultValue: '完全由容量上限滑动窗口托管，保留完整报文不被提前掏空；达到上限自动淘汰最尾部 30% 记录' })}
                                    </p>
                                </div>

                                {/* 最大保留条数与报文模式并排 */}
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-1">
                                    <div>
                                        <label className="block text-xs font-medium text-gray-600 dark:text-gray-300 mb-1">
                                            {t('proxy.config.log_retention_rows', { defaultValue: '最大保留条数' })}
                                        </label>
                                        <input
                                            type="number"
                                            min={100}
                                            step={1000}
                                            value={appConfig.proxy.log_retention?.max_rows ?? 100000}
                                            onChange={(e) => onUpdateLogRetention('max_rows', Number(e.target.value))}
                                            className="input input-xs input-bordered bg-gray-50 dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-800 dark:text-white w-full font-mono text-xs focus:border-blue-500"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-xs font-medium text-gray-600 dark:text-gray-300 mb-1">
                                            {t('proxy.config.experimental.payload_storage_mode_label', { defaultValue: '监控报文存储模式' })}
                                        </label>
                                        <select
                                            className="select select-xs select-bordered bg-gray-50 dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-800 dark:text-white w-full text-xs"
                                            value={appConfig.proxy.experimental?.payload_storage_mode || 'simple'}
                                            onChange={(e) => onUpdateExperimental('payload_storage_mode', e.target.value)}
                                        >
                                            <option value="simple">{t('proxy.config.experimental.payload_mode_simple', { defaultValue: '简要模式 (推荐)' })}</option>
                                            <option value="full">{t('proxy.config.experimental.payload_mode_full', { defaultValue: '完整原文 (排错)' })}</option>
                                        </select>
                                    </div>
                                </div>
                                <p className="text-[10px] text-gray-500 dark:text-gray-400 leading-tight">
                                    {t('proxy.config.experimental.payload_storage_mode_desc', { defaultValue: '简要模式避免工具参数与图片撑爆日志库；排错时可切完整模式。' })}
                                </p>
                            </div>
                        </div>
                    </div>

                    {/* 2. 维护与清理操作 */}
                    <div className="p-3.5 bg-white dark:bg-base-100 rounded-xl border border-gray-200/90 dark:border-base-200 shadow-xs flex flex-col justify-between space-y-3">
                        <div>
                            <span className="text-xs font-bold text-gray-800 dark:text-gray-200 flex items-center gap-1.5 mb-1.5">
                                <HardDrive size={13} className="text-amber-500 dark:text-amber-400" />
                                {t('monitor.settings.maintenance_title', { defaultValue: '日志维护与空间清理' })}
                            </span>
                            <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                                {t('settings.advanced.logs_desc', { defaultValue: '清理应用产生的日志缓存文件或清空全部历史请求记录，释放磁盘空间。' })}
                            </p>
                        </div>
                        <div className="space-y-2 pt-2">
                            <button
                                type="button"
                                onClick={onOpenClearCacheModal}
                                className="btn btn-xs w-full btn-outline btn-warning gap-1.5 text-xs font-semibold"
                            >
                                <Trash2 size={12} />
                                {t('settings.advanced.clear_logs', { defaultValue: '清理日志缓存文件' })}
                            </button>
                            <button
                                type="button"
                                onClick={onClearLogs}
                                className="btn btn-xs w-full btn-outline btn-error gap-1.5 text-xs font-semibold"
                            >
                                <Trash2 size={12} />
                                {t('monitor.actions.clear_all_requests', { defaultValue: '清空全部历史请求' })}
                            </button>
                            {cacheClearedSuccess && (
                                <p className="text-xs text-emerald-600 dark:text-emerald-400 text-center font-medium">
                                    ✓ {t('settings.advanced.logs_cleared', { defaultValue: '日志缓存已清理' })}
                                </p>
                            )}
                        </div>
                    </div>
                </div>
            </div>
    );
};
