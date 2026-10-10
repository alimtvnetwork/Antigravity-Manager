import { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Power, RefreshCw, Save, Settings, Share2, Terminal } from 'lucide-react';
import { request as invoke } from '../utils/request';
import { copyToClipboard } from '../utils/clipboard';
import ModalDialog from '../components/common/ModalDialog';
import { showToast } from '../components/common/ToastContainer';
import { useErrorStore } from '../stores/error-store';
import { useProxyModels } from '../hooks/useProxyModels';
import type { SelectOption } from '../components/common/GroupedSelect';
import { useProxyConfig } from './api-proxy/useProxyConfig';
import { useProxyStatus } from './api-proxy/useProxyStatus';
import { useCloudflared } from './api-proxy/useCloudflared';
import { usePresetManager } from './api-proxy/usePresetManager';
import { useCredentialEditing } from './api-proxy/useCredentialEditing';
import { useFixedAccount } from './api-proxy/useFixedAccount';
import { useZaiDispatcher } from './api-proxy/useZaiDispatcher';
import { ServiceSettingsTab } from './api-proxy/ServiceSettingsTab';
import { CliSetupTab } from './api-proxy/CliSetupTab';
import { ProtocolsTab } from './api-proxy/ProtocolsTab';
import { ExternalProvidersSection } from './api-proxy/ExternalProvidersSection';
import { PresetManagerDialog } from './api-proxy/PresetManagerDialog';
import type { MenuTab, ProtocolKind } from './api-proxy/types';
import { COPY_FEEDBACK_TIMEOUT_MS, DEFAULT_SELECTED_MODEL_ID } from './api-proxy/constants';

export default function ApiProxy() {
    const { t } = useTranslation();

    const { models } = useProxyModels();

    const [activeMenuTab, setActiveMenuTab] = useState<MenuTab>('settings');
    const [copied, setCopied] = useState<string | null>(null);
    const [selectedProtocol, setSelectedProtocol] = useState<ProtocolKind>('openai');
    const [selectedModelId, setSelectedModelId] = useState(DEFAULT_SELECTED_MODEL_ID);

    const {
        appConfig,
        configLoading,
        configError,
        loadConfig,
        saveConfig,
        handleSaveProxySettings,
        updateProxyConfig,
        updateSchedulingConfig,
        updateExperimentalConfig,
        updateCircuitBreakerConfig,
        handleMappingUpdate,
        handleRemoveCustomMapping,
        handleResetMapping,
        executeResetMapping,
        isResetConfirmOpen,
        setIsResetConfirmOpen,
        setAppConfig,
    } = useProxyConfig();

    const { status, loading, handleToggle } = useProxyStatus(appConfig);

    const cloudflared = useCloudflared({
        appConfig,
        saveConfig,
        proxyRunning: status.running,
        onCopied: setCopied,
    });

    const presetManager = usePresetManager({ appConfig, setAppConfig, loadConfig });

    const credentials = useCredentialEditing({ appConfig, updateProxyConfig });

    const fixedAccount = useFixedAccount();

    const zai = useZaiDispatcher({ appConfig, saveConfig });

    // Modal states
    const [isClearBindingsConfirmOpen, setIsClearBindingsConfirmOpen] = useState(false);
    const [isClearRateLimitsConfirmOpen, setIsClearRateLimitsConfirmOpen] = useState(false);

    const copyToClipboardHandler = (text: string, label: string) => {
        copyToClipboard(text).then((success) => {
            if (success) {
                setCopied(label);
                setTimeout(() => setCopied(null), COPY_FEEDBACK_TIMEOUT_MS);
            }
        }).catch((error: unknown) => {
            useErrorStore.getState().trackWarning(error, {
                source: 'ApiProxy.copyToClipboardHandler',
                triggerAction: 'copyToClipboard',
            });
        });
    };

    // 生成自定义映射表单的选项 (从 models 动态生成，统一纯正 Model ID 风格)
    const customMappingOptions: SelectOption[] = useMemo(() => {
        return models.map(model => ({
            value: model.id,
            label: model.id,
            group: model.group || 'Other'
        }));
    }, [models]);

    const handleClearSessionBindings = () => {
        setIsClearBindingsConfirmOpen(true);
    };

    const executeClearSessionBindings = async () => {
        setIsClearBindingsConfirmOpen(false);
        try {
            await invoke('clear_proxy_session_bindings');
            showToast(t('common.success'), 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'ApiProxy.executeClearSessionBindings',
                triggerAction: 'clear_proxy_session_bindings',
            });
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleClearRateLimits = () => {
        setIsClearRateLimitsConfirmOpen(true);
    };

    const executeClearRateLimits = async () => {
        setIsClearRateLimitsConfirmOpen(false);
        try {
            await invoke('clear_all_proxy_rate_limits');
            showToast(t('common.success'), 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'ApiProxy.executeClearRateLimits',
                triggerAction: 'clear_all_proxy_rate_limits',
            });
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    return (
        <div className="h-full w-full overflow-y-auto overflow-x-hidden">
            <div className="px-4 sm:px-6 pt-2 pb-4 space-y-4 max-w-[1920px] mx-auto">

                {/* Loading State */}
                {configLoading && (
                    <div className="flex items-center justify-center py-20">
                        <div className="flex flex-col items-center gap-4">
                            <RefreshCw size={32} className="animate-spin text-blue-500" />
                            <span className="text-sm text-gray-500 dark:text-gray-400">
                                {t('common.loading')}
                            </span>
                        </div>
                    </div>
                )}

                {/* Error State */}
                {!configLoading && configError && (
                    <div className="flex items-center justify-center py-20">
                        <div className="flex flex-col items-center gap-4 text-center">
                            <div className="w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                                <Settings size={32} className="text-red-500" />
                            </div>
                            <div className="space-y-2">
                                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                    {t('proxy.error.load_failed')}
                                </h3>
                                <p className="text-sm text-gray-500 dark:text-gray-400 max-w-md">
                                    {configError}
                                </p>
                            </div>
                            <button
                                onClick={loadConfig}
                                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
                            >
                                <RefreshCw size={16} />
                                {t('common.retry')}
                            </button>
                        </div>
                    </div>
                )}

                {/* 配置区 */}
                {!configLoading && !configError && appConfig && (
                    <div className="bg-white dark:bg-base-100 rounded-xl shadow-xs border border-gray-200/80 dark:border-base-200">
                        <div className="px-4 sm:px-5 py-2.5 border-b border-gray-100 dark:border-base-200 bg-gray-50/60 dark:bg-base-200/80 flex flex-wrap items-center justify-between gap-3">
                            {/* Three selectable menu tabs: Service Settings, CLI Setup, Multi-Protocol */}
                            <div className="flex items-center gap-1.5 bg-gray-200/70 dark:bg-base-300/60 p-1 rounded-xl">
                                <button
                                    type="button"
                                    onClick={() => setActiveMenuTab('settings')}
                                    className={`px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1.5 transition-all ${
                                        activeMenuTab === 'settings'
                                            ? 'bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs'
                                            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                    }`}
                                >
                                    <Settings size={14} className={activeMenuTab === 'settings' ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'} />
                                    {t('proxy.config.title')}
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setActiveMenuTab('cli')}
                                    className={`px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1.5 transition-all ${
                                        activeMenuTab === 'cli'
                                            ? 'bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs'
                                            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                    }`}
                                >
                                    <Terminal size={14} className={activeMenuTab === 'cli' ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'} />
                                    {t('proxy.cli_sync.title', { defaultValue: 'CLI Setup' })}
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setActiveMenuTab('protocols')}
                                    className={`px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1.5 transition-all ${
                                        activeMenuTab === 'protocols'
                                            ? 'bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs'
                                            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                    }`}
                                >
                                    <Share2 size={14} className={activeMenuTab === 'protocols' ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'} />
                                    {t('proxy.multi_protocol.title', { defaultValue: 'Multi-Protocol' })}
                                </button>
                            </div>

                            {/* Control Buttons */}
                            <div className="flex items-center gap-2.5 ml-auto">
                                <button
                                    onClick={handleSaveProxySettings}
                                    disabled={!appConfig}
                                    className={`px-3 py-1 rounded-lg text-xs font-medium transition-colors flex items-center gap-2 bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-800 hover:bg-blue-100 dark:hover:bg-blue-900/40 ${!appConfig ? 'opacity-50 cursor-not-allowed' : ''}`}
                                >
                                    <Save size={14} />
                                    {t('settings.save')}
                                </button>
                                <button
                                    onClick={handleToggle}
                                    disabled={loading || !appConfig}
                                    className={`px-3 py-1 rounded-lg text-xs font-medium transition-colors flex items-center gap-2 ${status.running
                                        ? 'bg-red-50 to-red-600 text-red-600 hover:bg-red-100 border border-red-200'
                                        : 'bg-blue-600 hover:bg-blue-700 text-white shadow-sm shadow-blue-500/30'
                                        } ${(loading || !appConfig) ? 'opacity-50 cursor-not-allowed' : ''}`}
                                >
                                    <Power size={14} />
                                    {loading ? t('proxy.status.processing') : (status.running ? t('proxy.action.stop') : t('proxy.action.start'))}
                                </button>
                            </div>
                        </div>

                        {/* TAB 1: Service Settings (settings) */}
                        {activeMenuTab === 'settings' && (
                            <ServiceSettingsTab
                                config={appConfig}
                                t={t}
                                status={status}
                                updateProxyConfig={updateProxyConfig}
                                copied={copied}
                                copyToClipboardHandler={copyToClipboardHandler}
                                credentials={credentials}
                            />
                        )}

                        {/* TAB 2: CLI 一键配置 (cli) */}
                        {activeMenuTab === 'cli' && (
                            <CliSetupTab
                                config={appConfig}
                                status={status}
                            />
                        )}

                        {/* TAB 3: 多协议支持 (protocols) */}
                        {activeMenuTab === 'protocols' && (
                            <ProtocolsTab
                                config={appConfig}
                                t={t}
                                status={status}
                                selectedProtocol={selectedProtocol}
                                onSelectProtocol={setSelectedProtocol}
                                selectedModelId={selectedModelId}
                                onSelectModelId={setSelectedModelId}
                                models={models}
                                copied={copied}
                                copyToClipboardHandler={copyToClipboardHandler}
                            />
                        )}
                    </div>
                )}

                {/* External Providers Integration - 仅在服务配置 Tab 下展示 */}
                {
                    !configLoading && !configError && appConfig && activeMenuTab === 'settings' && (
                        <ExternalProvidersSection
                            config={appConfig}
                            t={t}
                            status={status}
                            copied={copied}
                            saveConfig={saveConfig}
                            updateProxyConfig={updateProxyConfig}
                            updateSchedulingConfig={updateSchedulingConfig}
                            updateExperimentalConfig={updateExperimentalConfig}
                            updateCircuitBreakerConfig={updateCircuitBreakerConfig}
                            handleSaveProxySettings={handleSaveProxySettings}
                            handleMappingUpdate={handleMappingUpdate}
                            handleRemoveCustomMapping={handleRemoveCustomMapping}
                            handleResetMapping={handleResetMapping}
                            customMappingOptions={customMappingOptions}
                            presetManager={presetManager}
                            cloudflared={cloudflared}
                            fixedAccount={fixedAccount}
                            zai={zai}
                            onClearSessionBindings={handleClearSessionBindings}
                            onClearRateLimits={handleClearRateLimits}
                        />
                    )
                }


                {/* 各种对话框 */}
                <ModalDialog
                    isOpen={isResetConfirmOpen}
                    title={t('proxy.dialog.reset_mapping_title') || '重置映射'}
                    message={t('proxy.dialog.reset_mapping_msg') || '确定要重置所有模型映射为系统默认吗？'}
                    type="confirm"
                    isDestructive={true}
                    onConfirm={executeResetMapping}
                    onCancel={() => setIsResetConfirmOpen(false)}
                />

                <ModalDialog
                    isOpen={credentials.isRegenerateKeyConfirmOpen}
                    title={t('proxy.dialog.regenerate_key_title') || t('proxy.dialog.confirm_regenerate')}
                    message={t('proxy.dialog.regenerate_key_msg') || t('proxy.dialog.confirm_regenerate')}
                    type="confirm"
                    isDestructive={true}
                    onConfirm={credentials.executeGenerateApiKey}
                    onCancel={() => credentials.setIsRegenerateKeyConfirmOpen(false)}
                />

                <ModalDialog
                    isOpen={isClearBindingsConfirmOpen}
                    title={t('proxy.dialog.clear_bindings_title') || '清除会话绑定'}
                    message={t('proxy.dialog.clear_bindings_msg') || '确定要清除所有会话与账号的绑定映射吗？'}
                    type="confirm"
                    isDestructive={true}
                    onConfirm={executeClearSessionBindings}
                    onCancel={() => setIsClearBindingsConfirmOpen(false)}
                />

                <ModalDialog
                    isOpen={isClearRateLimitsConfirmOpen}
                    title={t('proxy.dialog.clear_rate_limits_title') || '清除限流记录'}
                    message={t('proxy.dialog.clear_rate_limits_confirm') || '确定要清除所有本地限流记录吗？'}
                    type="confirm"
                    isDestructive={true}
                    onConfirm={executeClearRateLimits}
                    onCancel={() => setIsClearRateLimitsConfirmOpen(false)}
                />

                <PresetManagerDialog
                    t={t}
                    presetManager={presetManager}
                />
            </div >
        </div >
    );
}
