import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../utils/request';
import { showToast } from '../../components/common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import type {
    AppConfig,
    ProxyConfig,
    StickySessionConfig,
    ExperimentalConfig,
    CircuitBreakerConfig,
} from '../../types/config';
import {
    DEFAULT_MODEL_MAPPINGS,
    DEFAULT_SCHEDULING,
    DEFAULT_EXPERIMENTAL_THRESHOLDS,
    BUILT_IN_WILDCARD_KEY,
} from './constants';

export interface ProxyConfigState {
    appConfig: AppConfig | null;
    configLoading: boolean;
    configError: string | null;
    loadConfig: () => Promise<void>;
    saveConfig: (newConfig: AppConfig) => Promise<void>;
    handleSaveProxySettings: () => Promise<void>;
    updateProxyConfig: (updates: Partial<ProxyConfig>) => void;
    updateSchedulingConfig: (updates: Partial<StickySessionConfig>) => void;
    updateExperimentalConfig: (updates: Partial<ExperimentalConfig>) => void;
    updateCircuitBreakerConfig: (newBreakerConfig: CircuitBreakerConfig) => void;
    handleMappingUpdate: (type: 'custom', key: string, value: string) => Promise<void>;
    handleRemoveCustomMapping: (key: string) => Promise<void>;
    handleResetMapping: () => void;
    executeResetMapping: () => Promise<void>;
    isResetConfirmOpen: boolean;
    setIsResetConfirmOpen: (open: boolean) => void;
    setAppConfig: React.Dispatch<React.SetStateAction<AppConfig | null>>;
}

export function useProxyConfig(): ProxyConfigState {
    const { t } = useTranslation();

    const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
    const [configLoading, setConfigLoading] = useState(true);
    const [configError, setConfigError] = useState<string | null>(null);
    const [isResetConfirmOpen, setIsResetConfirmOpen] = useState(false);

    const loadConfig = useCallback(async () => {
        setConfigLoading(true);
        setConfigError(null);
        try {
            const config = await invoke<AppConfig>('load_config');
            setAppConfig(config);
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyConfig.loadConfig',
                triggerAction: 'load_config',
            });
            setConfigError(String(error));
        } finally {
            setConfigLoading(false);
        }
    }, []);

    const saveConfig = useCallback(async (newConfig: AppConfig) => {
        // 1. 立即更新 UI 状态，确保流畅
        setAppConfig(newConfig);
        try {
            await invoke('save_config', { config: newConfig });
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyConfig.saveConfig',
                triggerAction: 'save_config',
            });
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    }, [t]);

    const handleSaveProxySettings = useCallback(async () => {
        if (!appConfig) return;
        try {
            await invoke('save_config', { config: appConfig });
            showToast(t('common.saved'), 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyConfig.handleSaveProxySettings',
                triggerAction: 'save_config',
            });
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    }, [appConfig, t]);

    const updateProxyConfig = useCallback((updates: Partial<ProxyConfig>) => {
        if (!appConfig) return;
        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                ...updates
            }
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    const updateSchedulingConfig = useCallback((updates: Partial<StickySessionConfig>) => {
        if (!appConfig) return;
        const currentScheduling = appConfig.proxy.scheduling || DEFAULT_SCHEDULING;
        const newScheduling = { ...currentScheduling, ...updates };

        const newAppConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                scheduling: newScheduling
            }
        };
        saveConfig(newAppConfig);
    }, [appConfig, saveConfig]);

    const updateExperimentalConfig = useCallback((updates: Partial<ExperimentalConfig>) => {
        if (!appConfig) return;
        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                experimental: {
                    ...(appConfig.proxy.experimental || {
                        enable_usage_scaling: true,
                        context_compression_threshold_l1: DEFAULT_EXPERIMENTAL_THRESHOLDS.l1,
                        context_compression_threshold_l2: DEFAULT_EXPERIMENTAL_THRESHOLDS.l2,
                        context_compression_threshold_l3: DEFAULT_EXPERIMENTAL_THRESHOLDS.l3
                    }),
                    ...updates
                }
            }
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    const updateCircuitBreakerConfig = useCallback((newBreakerConfig: CircuitBreakerConfig) => {
        if (!appConfig) return;
        const newConfig = {
            ...appConfig,
            circuit_breaker: newBreakerConfig
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    // 专门处理模型映射的热更新 (全量)
    const handleMappingUpdate = useCallback(async (type: 'custom', key: string, value: string) => {
        if (!appConfig) return;

        const trimmedKey = key.trim();
        const lowerKey = trimmedKey.toLowerCase();

        // 新增通配符仅允许内置 gemini-3.x-flash（x 必须大于 8）；已存在的项可继续编辑目标
        const isExisting = !!(appConfig.proxy.custom_mapping && Object.prototype.hasOwnProperty.call(appConfig.proxy.custom_mapping, trimmedKey));
        if (!isExisting && (trimmedKey.includes('*') || /gemini-3\.x-flash/i.test(trimmedKey) || lowerKey.includes('.x'))) {
            if (lowerKey !== BUILT_IN_WILDCARD_KEY) {
                showToast(t('proxy.router.wildcard_only_gemini_3x') || '仅允许内置通配符 gemini-3.x-flash（x 必须大于 8），其他请填写具体模型 ID', 'warning');
                return;
            }
        }

        const newConfig = { ...appConfig.proxy };
        newConfig.custom_mapping = { ...(newConfig.custom_mapping || {}), [trimmedKey]: value };

        try {
            await invoke('update_model_mapping', { config: newConfig });
            setAppConfig({ ...appConfig, proxy: newConfig });
            showToast(t('common.saved'), 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyConfig.handleMappingUpdate',
                triggerAction: 'update_model_mapping',
            });
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    }, [appConfig, t]);

    const handleRemoveCustomMapping = useCallback(async (key: string) => {
        if (!appConfig || !appConfig.proxy.custom_mapping) return;
        const newCustom = { ...appConfig.proxy.custom_mapping };
        delete newCustom[key];
        const newConfig = { ...appConfig.proxy, custom_mapping: newCustom };
        try {
            await invoke('update_model_mapping', { config: newConfig });
            setAppConfig({ ...appConfig, proxy: newConfig });
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyConfig.handleRemoveCustomMapping',
                triggerAction: 'update_model_mapping',
            });
        }
    }, [appConfig]);

    const handleResetMapping = useCallback(() => {
        setIsResetConfirmOpen(true);
    }, []);

    const executeResetMapping = useCallback(async () => {
        if (!appConfig) return;
        setIsResetConfirmOpen(false);

        // 恢复到默认预设映射值
        const newConfig = {
            ...appConfig.proxy,
            custom_mapping: { ...DEFAULT_MODEL_MAPPINGS }
        };

        try {
            await invoke('update_model_mapping', { config: newConfig });
            setAppConfig({ ...appConfig, proxy: newConfig });
            showToast(t('common.success'), 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyConfig.executeResetMapping',
                triggerAction: 'update_model_mapping',
            });
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    }, [appConfig, t]);

    // 初始化加载
    useEffect(() => {
        loadConfig();
    }, [loadConfig]);

    return {
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
    };
}
