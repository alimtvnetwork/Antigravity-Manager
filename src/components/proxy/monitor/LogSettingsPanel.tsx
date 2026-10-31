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
                            {t('monitor.settings.title', { defaultValue: 'Log storage period and maintenance settings' })}
                        </span>
                        <span className="text-xs text-gray-500 dark:text-gray-400 hidden sm:inline">
                            {t('monitor.settings.subtitle', { defaultValue: 'Unified management of request log retention days, thinking block sliding window and disk space recycling' })}
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
                            title="Save all logs and think block configurations"
                        >
                            <Check size={13} />
                            <span>{saveSuccess ? t('common.saved', { defaultValue: 'Already effective' }) : (isSavingConfig ? t('common.saving', { defaultValue: 'Saving...' }) : t('common.save', { defaultValue: 'Save all and hot take effect' }))}</span>
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
                    {/* 1. Request log and message retention policy */}
                    <div className="p-3.5 bg-white dark:bg-base-100 rounded-xl border border-gray-200/90 dark:border-base-200 shadow-xs flex flex-col justify-between space-y-3">
                        <div className="space-y-3">
                            <span className="text-xs font-bold text-gray-800 dark:text-gray-200 flex items-center gap-1.5">
                                <Clock size={13} className="text-indigo-500 dark:text-indigo-400" />
                                {t('monitor.settings.retention_title', { defaultValue: 'Request log and message retention policy (sliding window)' })}
                            </span>
                            <div className="space-y-2.5">
                                {/* Space limit */}
                                <div>
                                    <div className="flex items-center justify-between mb-1">
                                        <label className="text-xs font-medium text-gray-600 dark:text-gray-300">
                                            {t('proxy.config.log_retention_storage_gb', { defaultValue: 'Maximum log retention space (GB)' })}
                                        </label>
                                        <span className="text-[10px] text-gray-500 dark:text-gray-400">
                                            {t('proxy.config.log_retention_current_usage', { defaultValue: 'Current library occupation' })}: <strong className="font-mono text-gray-700 dark:text-gray-200">{dbDiskSizeBytes !== null ? formatBytes(dbDiskSizeBytes) : '...'}</strong>
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
                                        {t('proxy.config.log_retention_storage_gb_desc', { defaultValue: 'It is completely hosted by the sliding window of the capacity limit, and complete messages are retained from being emptyed in advance; the last 30% of records are automatically eliminated when the upper limit is reached.' })}
                                    </p>
                                </div>

                                {/* Maximum number of retained items side by side with message mode */}
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-1">
                                    <div>
                                        <label className="block text-xs font-medium text-gray-600 dark:text-gray-300 mb-1">
                                            {t('proxy.config.log_retention_rows', { defaultValue: 'Maximum number of retained items' })}
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
                                            {t('proxy.config.experimental.payload_storage_mode_label', { defaultValue: 'Monitoring message storage mode' })}
                                        </label>
                                        <select
                                            className="select select-xs select-bordered bg-gray-50 dark:bg-base-200 border-gray-300 dark:border-base-300 text-gray-800 dark:text-white w-full text-xs"
                                            value={appConfig.proxy.experimental?.payload_storage_mode || 'simple'}
                                            onChange={(e) => onUpdateExperimental('payload_storage_mode', e.target.value)}
                                        >
                                            <option value="simple">{t('proxy.config.experimental.payload_mode_simple', { defaultValue: 'Brief mode (recommended)' })}</option>
                                            <option value="full">{t('proxy.config.experimental.payload_mode_full', { defaultValue: 'Complete original text (troubleshooting)' })}</option>
                                        </select>
                                    </div>
                                </div>
                                <p className="text-[10px] text-gray-500 dark:text-gray-400 leading-tight">
                                    {t('proxy.config.experimental.payload_storage_mode_desc', { defaultValue: 'Brief mode avoids overloading the log library with tool parameters and images; you can switch to full mode when troubleshooting.' })}
                                </p>
                            </div>
                        </div>
                    </div>

                    {/* 2. Maintenance and cleaning operations */}
                    <div className="p-3.5 bg-white dark:bg-base-100 rounded-xl border border-gray-200/90 dark:border-base-200 shadow-xs flex flex-col justify-between space-y-3">
                        <div>
                            <span className="text-xs font-bold text-gray-800 dark:text-gray-200 flex items-center gap-1.5 mb-1.5">
                                <HardDrive size={13} className="text-amber-500 dark:text-amber-400" />
                                {t('monitor.settings.maintenance_title', { defaultValue: 'Log maintenance and space cleaning' })}
                            </span>
                            <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                                {t('settings.advanced.logs_desc', { defaultValue: 'Clear the log cache files generated by the application or clear all historical request records to free up disk space.' })}
                            </p>
                        </div>
                        <div className="space-y-2 pt-2">
                            <button
                                type="button"
                                onClick={onOpenClearCacheModal}
                                className="btn btn-xs w-full btn-outline btn-warning gap-1.5 text-xs font-semibold"
                            >
                                <Trash2 size={12} />
                                {t('settings.advanced.clear_logs', { defaultValue: 'Clean log cache files' })}
                            </button>
                            <button
                                type="button"
                                onClick={onClearLogs}
                                className="btn btn-xs w-full btn-outline btn-error gap-1.5 text-xs font-semibold"
                            >
                                <Trash2 size={12} />
                                {t('monitor.actions.clear_all_requests', { defaultValue: 'Clear all historical requests' })}
                            </button>
                            {cacheClearedSuccess && (
                                <p className="text-xs text-emerald-600 dark:text-emerald-400 text-center font-medium">
                                    ✓ {t('settings.advanced.logs_cleared', { defaultValue: 'Log cache cleared' })}
                                </p>
                            )}
                        </div>
                    </div>
                </div>
            </div>
    );
};
