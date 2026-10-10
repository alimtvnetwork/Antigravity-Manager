import React, { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import ModalDialog from '../common/ModalDialog';
import { request as invoke } from '../../utils/request';
import { useErrorStore } from '../../stores/error-store';
import type { AppConfig, ExperimentalConfig } from '../../types/config';
import { useProxyLogs } from './monitor/useProxyLogs';
import { MonitorToolbar } from './monitor/MonitorToolbar';
import { LogSettingsPanel } from './monitor/LogSettingsPanel';
import { LogDetailPanel } from './monitor/LogDetailPanel';
import { LogTable } from './monitor/LogTable';
import { CACHE_CLEAR_SUCCESS_TIMEOUT_MS, CONFIG_SAVE_SUCCESS_TIMEOUT_MS } from './monitor/types';
import type { PayloadViewMode, ProxyMonitorProps, ProxyRequestLog } from './monitor/types';

export const ProxyMonitor: React.FC<ProxyMonitorProps> = ({ className }) => {
    const { t } = useTranslation();

    const [selectedLog, setSelectedLog] = useState<ProxyRequestLog | null>(null);
    const [isLoggingEnabled, setIsLoggingEnabled] = useState(false);
    const [captureHealthLogs, setCaptureHealthLogs] = useState(false);
    const [isClearConfirmOpen, setIsClearConfirmOpen] = useState(false);
    const [payloadViewMode, setPayloadViewMode] = useState<PayloadViewMode>('concise');
    const [showMetadata, setShowMetadata] = useState(true);
    const [copiedCard, setCopiedCard] = useState<string | null>(null);

    // 日志存储与维护配置状态
    const [showLogSettings, setShowLogSettings] = useState(false);
    const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
    const [isSavingConfig, setIsSavingConfig] = useState(false);
    const [saveSuccess, setSaveSuccess] = useState(false);
    const [isClearCacheModalOpen, setIsClearCacheModalOpen] = useState(false);

    const [dbDiskSizeBytes, setDbDiskSizeBytes] = useState<number | null>(null);
    const [cacheClearedSuccess, setCacheClearedSuccess] = useState(false);
    const [loadingDetail, setLoadingDetail] = useState(false);

    const handleConfigLoaded = useCallback((config: AppConfig) => {
        setAppConfig(config);
        setIsLoggingEnabled(config.proxy.enable_logging);
        setCaptureHealthLogs(!!config.proxy.capture_health_logs);
    }, []);

    const {
        stats,
        setStats,
        setLogs,
        loading,
        filter,
        setFilter,
        accountFilter,
        setAccountFilter,
        pageSize,
        setPageSize,
        currentPage,
        totalCount,
        setTotalCount,
        totalPages,
        pageStart,
        pageEnd,
        pageSizeOptions,
        filteredLogs,
        quickFilters,
        uniqueAccounts,
        filterInputRef,
        loadData,
        goToPage,
    } = useProxyLogs({ captureHealthLogs, onConfigLoaded: handleConfigLoaded });

    const fetchDbDiskSize = useCallback(async () => {
        try {
            const bytes = await invoke<number>('get_proxy_db_disk_size');
            setDbDiskSizeBytes(bytes);
        } catch (e) {
            console.error('Failed to get proxy db disk size', e);
            // Tracked in the error module; disk-size label keeps previous value.
            useErrorStore.getState().trackWarning(e, {
              source: 'ProxyMonitor.fetchDbDiskSize',
              endpoint: 'get_proxy_db_disk_size',
              triggerAction: 'fetch_db_disk_size',
            });
        }
    }, []);

    useEffect(() => {
        if (showLogSettings) {
            fetchDbDiskSize();
        }
    }, [showLogSettings, fetchDbDiskSize]);

    const toggleLogging = async () => {
        const newState = !isLoggingEnabled;
        try {
            const config = await invoke<AppConfig>('load_config');
            if (config && config.proxy) {
                config.proxy.enable_logging = newState;
                await invoke('save_config', { config });
                await invoke('set_proxy_monitor_enabled', { enabled: newState });
                setIsLoggingEnabled(newState);
            }
        } catch (e) {
            console.error("Failed to toggle logging", e);
            // Tracked in the error module; toggle reverts to previous state on next load.
            useErrorStore.getState().trackWarning(e, {
              source: 'ProxyMonitor.toggleLogging',
              endpoint: 'set_proxy_monitor_enabled',
              triggerAction: 'toggle_logging',
            });
        }
    };

    const toggleCaptureHealthLogs = async () => {
        const newState = !captureHealthLogs;
        try {
            const config = await invoke<AppConfig>('load_config');
            if (config && config.proxy) {
                config.proxy.capture_health_logs = newState;
                await invoke('save_config', { config });
                await invoke('set_proxy_capture_health_logs', { enabled: newState });
                setCaptureHealthLogs(newState);
                loadData(1, filter, accountFilter);
            }
        } catch (e) {
            console.error("Failed to toggle capture health logs", e);
        }
    };

    useEffect(() => {
        setCopiedCard(null);
    }, [selectedLog?.id]);


    const clearLogs = () => {
        setIsClearConfirmOpen(true);
    };

    const executeClearLogs = async () => {
        setIsClearConfirmOpen(false);
        try {
            await invoke('clear_proxy_logs');
            setLogs([]);
            setStats({ total_requests: 0, success_count: 0, error_count: 0 });
            setTotalCount(0);
            fetchDbDiskSize();
        } catch (e) {
            console.error("Failed to clear logs", e);
        }
    };

    const updateLogRetentionField = (field: 'max_body_age_hours' | 'max_storage_gb' | 'max_rows', value: number) => {
        if (!appConfig) return;
        const currentRetention = appConfig.proxy?.log_retention || { max_body_age_hours: 24, max_storage_gb: 0.5, max_rows: 100000 };
        const safeVal = field === 'max_storage_gb'
            ? Math.max(0.1, isNaN(value) ? 0.5 : value)
            : Math.max(1, isNaN(value) ? 1 : value);
        const updated = {
            ...currentRetention,
            [field]: safeVal,
        };
        const currentExp: ExperimentalConfig = appConfig.proxy?.experimental || {
            enable_usage_scaling: true,
        };
        const updatedConfig: AppConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                log_retention: updated,
                experimental: {
                    ...currentExp,
                }
            }
        };
        setAppConfig(updatedConfig);
    };

    const updateExperimentalField = (field: 'payload_storage_mode' | 'thinking_retention_days', value: any) => {
        if (!appConfig) return;
        const currentExp: ExperimentalConfig = appConfig.proxy?.experimental || {
            enable_usage_scaling: true,
        };
        const updatedExp: ExperimentalConfig = {
            ...currentExp,
            [field]: field === 'thinking_retention_days' ? Math.max(1, parseInt(value) || 15) : value,
        };
        const updatedConfig: AppConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                experimental: updatedExp
            }
        };
        setAppConfig(updatedConfig);
    };

    const handleSaveLogSettings = async () => {
        if (!appConfig) return;
        setIsSavingConfig(true);
        try {
            await invoke('save_config', { config: appConfig });
            setSaveSuccess(true);
            fetchDbDiskSize();
            setTimeout(() => setSaveSuccess(false), CONFIG_SAVE_SUCCESS_TIMEOUT_MS);
        } catch (e) {
            console.error('Failed to save log settings', e);
        } finally {
            setIsSavingConfig(false);
        }
    };

    const handleClearCache = async () => {
        setIsClearCacheModalOpen(false);
        try {
            await invoke('clear_log_cache');
            setCacheClearedSuccess(true);
            setTimeout(() => setCacheClearedSuccess(false), CACHE_CLEAR_SUCCESS_TIMEOUT_MS);
        } catch (e) {
            console.error('Failed to clear log cache', e);
        }
    };

    const handleLogClick = async (log: ProxyRequestLog) => {
        setLoadingDetail(true);
        try {
            const detail = await invoke<ProxyRequestLog>('get_proxy_log_detail', { logId: log.id, log_id: log.id });
            setSelectedLog(detail || log);
        } catch (e) {
            console.error('Failed to load log detail', e);
            setSelectedLog(log);
        } finally {
            setLoadingDetail(false);
        }
    };
    return (
        <div className={`flex flex-col bg-white dark:bg-base-100 rounded-xl shadow-xs border border-gray-200/80 dark:border-base-200 overflow-hidden ${className || 'flex-1'}`}>
            <MonitorToolbar
                t={t}
                filter={filter}
                onFilterChange={setFilter}
                accountFilter={accountFilter}
                onAccountFilterChange={setAccountFilter}
                uniqueAccounts={uniqueAccounts}
                stats={stats}
                loading={loading}
                isLoggingEnabled={isLoggingEnabled}
                onToggleLogging={toggleLogging}
                captureHealthLogs={captureHealthLogs}
                onToggleCaptureHealthLogs={toggleCaptureHealthLogs}
                showLogSettings={showLogSettings}
                onToggleLogSettings={() => setShowLogSettings((prev) => !prev)}
                onClearLogs={clearLogs}
                onRefresh={() => loadData(currentPage, filter)}
                onResetFilters={() => { setFilter(''); setAccountFilter(''); }}
                quickFilters={quickFilters}
                filterInputRef={filterInputRef}
            />

            {showLogSettings && appConfig && (
                <LogSettingsPanel
                    t={t}
                    appConfig={appConfig}
                    dbDiskSizeBytes={dbDiskSizeBytes}
                    isSavingConfig={isSavingConfig}
                    saveSuccess={saveSuccess}
                    cacheClearedSuccess={cacheClearedSuccess}
                    onUpdateLogRetention={updateLogRetentionField}
                    onUpdateExperimental={updateExperimentalField}
                    onSaveLogSettings={handleSaveLogSettings}
                    onClose={() => setShowLogSettings(false)}
                    onOpenClearCacheModal={() => setIsClearCacheModalOpen(true)}
                    onClearLogs={clearLogs}
                />
            )}

            <LogTable
                logs={filteredLogs}
                loading={loading}
                onLogClick={handleLogClick}
                t={t}
            />

            {/* Pagination Controls */}
            <div className="flex items-center justify-between px-4 py-3 bg-gray-50 dark:bg-base-200 border-t border-gray-200 dark:border-base-300 text-xs">
                <div className="flex items-center gap-2 whitespace-nowrap">
                    <span className="text-gray-500">{t('common.per_page')}</span>
                    <select
                        value={pageSize}
                        onChange={(e) => setPageSize(Number(e.target.value))}
                        className="select select-xs select-bordered w-16"
                    >
                        {pageSizeOptions.map(size => (
                            <option key={size} value={size}>{size}</option>
                        ))}
                    </select>
                </div>

                <div className="flex items-center gap-3">
                    <button
                        onClick={() => goToPage(currentPage - 1)}
                        disabled={currentPage <= 1 || loading}
                        className="btn btn-xs btn-ghost"
                    >
                        <ChevronLeft size={14} />
                    </button>
                    <span className="text-gray-600 dark:text-gray-400 min-w-[80px] text-center">
                        {currentPage} / {totalPages || 1}
                    </span>
                    <button
                        onClick={() => goToPage(currentPage + 1)}
                        disabled={currentPage >= totalPages || loading}
                        className="btn btn-xs btn-ghost"
                    >
                        <ChevronRight size={14} />
                    </button>
                </div>

                <div className="text-gray-500">
                    {t('common.pagination_info', { start: pageStart, end: pageEnd, total: totalCount })}
                </div>
            </div>

            {selectedLog && (
                <LogDetailPanel
                    t={t}
                    log={selectedLog}
                    loadingDetail={loadingDetail}
                    onClose={() => setSelectedLog(null)}
                    payloadViewMode={payloadViewMode}
                    onPayloadViewModeChange={setPayloadViewMode}
                    showMetadata={showMetadata}
                    onToggleMetadata={() => setShowMetadata((prev) => !prev)}
                    copiedCard={copiedCard}
                    onCopiedCardChange={setCopiedCard}
                />
            )}

            <ModalDialog
                isOpen={isClearConfirmOpen}
                title={t('monitor.dialog.clear_title')}
                message={t('monitor.dialog.clear_msg')}
                type="confirm"
                confirmText={t('common.delete')}
                isDestructive={true}
                onConfirm={executeClearLogs}
                onCancel={() => setIsClearConfirmOpen(false)}
            />

            <ModalDialog
                isOpen={isClearCacheModalOpen}
                title={t('settings.advanced.clear_logs_title', { defaultValue: '清理日志缓存确认' })}
                message={t('settings.advanced.clear_logs_msg', { defaultValue: '确定要清理所有日志缓存文件吗？这不会影响历史请求记录和账号数据。' })}
                type="confirm"
                confirmText={t('common.clear', { defaultValue: '清理' })}
                cancelText={t('common.cancel', { defaultValue: '取消' })}
                isDestructive={true}
                onConfirm={handleClearCache}
                onCancel={() => setIsClearCacheModalOpen(false)}
            />
        </div>
    );
};
