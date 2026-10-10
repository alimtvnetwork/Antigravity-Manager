import { useState, useMemo, useCallback } from 'react';
import { request as invoke } from '../../utils/request';
import { useErrorStore } from '../../stores/error-store';
import type { AppConfig, ProxyConfig, ZaiConfig } from '../../types/config';
import type { SaveConfigFn } from './types';

export interface ZaiDispatcher {
    zaiModelOptions: string[];
    zaiModelMapping: Record<string, string>;
    zaiModelsLoading: boolean;
    zaiNewMappingFrom: string;
    setZaiNewMappingFrom: (value: string) => void;
    zaiNewMappingTo: string;
    setZaiNewMappingTo: (value: string) => void;
    refreshZaiModels: () => Promise<void>;
    updateZaiDefaultModels: (updates: Partial<NonNullable<ProxyConfig['zai']>['models']>) => void;
    upsertZaiModelMapping: (from: string, to: string) => void;
    removeZaiModelMapping: (from: string) => void;
    updateZaiGeneralConfig: (updates: Partial<ZaiConfig>) => void;
}

interface UseZaiDispatcherArgs {
    appConfig: AppConfig | null;
    saveConfig: SaveConfigFn;
}

export function useZaiDispatcher({ appConfig, saveConfig }: UseZaiDispatcherArgs): ZaiDispatcher {
    const [zaiAvailableModels, setZaiAvailableModels] = useState<string[]>([]);
    const [zaiModelsLoading, setZaiModelsLoading] = useState(false);
    const [, setZaiModelsError] = useState<string | null>(null);
    const [zaiNewMappingFrom, setZaiNewMappingFrom] = useState('');
    const [zaiNewMappingTo, setZaiNewMappingTo] = useState('');

    const zaiModelOptions = useMemo(() => {
        const unique = new Set(zaiAvailableModels);
        return Array.from(unique).sort();
    }, [zaiAvailableModels]);

    const zaiModelMapping = useMemo(() => {
        return appConfig?.proxy.zai?.model_mapping || {};
    }, [appConfig?.proxy.zai?.model_mapping]);

    const refreshZaiModels = useCallback(async () => {
        if (!appConfig?.proxy.zai) return;
        setZaiModelsLoading(true);
        setZaiModelsError(null);
        try {
            const models = await invoke<string[]>('fetch_zai_models', {
                zai: appConfig.proxy.zai,
                upstreamProxy: appConfig.proxy.upstream_proxy,
                requestTimeout: appConfig.proxy.request_timeout,
            });
            setZaiAvailableModels(models);
        } catch (error: unknown) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useZaiDispatcher.refreshZaiModels',
                triggerAction: 'fetch_zai_models',
            });
            setZaiModelsError(String(error));
        } finally {
            setZaiModelsLoading(false);
        }
    }, [appConfig]);

    const updateZaiDefaultModels = useCallback((updates: Partial<NonNullable<ProxyConfig['zai']>['models']>) => {
        if (!appConfig?.proxy.zai) return;
        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                zai: {
                    ...appConfig.proxy.zai,
                    models: { ...appConfig.proxy.zai.models, ...updates }
                }
            }
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    const upsertZaiModelMapping = useCallback((from: string, to: string) => {
        if (!appConfig?.proxy.zai) return;
        const currentMapping = appConfig.proxy.zai.model_mapping || {};
        const newMapping = { ...currentMapping, [from]: to };

        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                zai: {
                    ...appConfig.proxy.zai,
                    model_mapping: newMapping
                }
            }
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    const removeZaiModelMapping = useCallback((from: string) => {
        if (!appConfig?.proxy.zai) return;
        const currentMapping = appConfig.proxy.zai.model_mapping || {};
        const newMapping = { ...currentMapping };
        delete newMapping[from];

        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                zai: {
                    ...appConfig.proxy.zai,
                    model_mapping: newMapping
                }
            }
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    const updateZaiGeneralConfig = useCallback((updates: Partial<ZaiConfig>) => {
        if (!appConfig?.proxy.zai) return;
        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                zai: {
                    ...appConfig.proxy.zai,
                    ...updates
                }
            }
        };
        saveConfig(newConfig);
    }, [appConfig, saveConfig]);

    return {
        zaiModelOptions,
        zaiModelMapping,
        zaiModelsLoading,
        zaiNewMappingFrom,
        setZaiNewMappingFrom,
        zaiNewMappingTo,
        setZaiNewMappingTo,
        refreshZaiModels,
        updateZaiDefaultModels,
        upsertZaiModelMapping,
        removeZaiModelMapping,
        updateZaiGeneralConfig,
    };
}
