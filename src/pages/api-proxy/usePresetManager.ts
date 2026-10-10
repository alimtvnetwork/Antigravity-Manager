import { useState, useEffect, useMemo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../utils/request';
import { showToast } from '../../components/common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import type { AppConfig } from '../../types/config';
import type { CustomPreset, PresetManager } from './types';
import { CUSTOM_PRESETS_STORAGE_KEY, DEFAULT_PRESET_ID } from './constants';

interface UsePresetManagerArgs {
    appConfig: AppConfig | null;
    setAppConfig: (config: AppConfig) => void;
    loadConfig: () => Promise<void>;
}

export function usePresetManager({ appConfig, setAppConfig, loadConfig }: UsePresetManagerArgs): PresetManager {
    const { t } = useTranslation();

    const [selectedPreset, setSelectedPreset] = useState<string>(DEFAULT_PRESET_ID);
    const [customPresets, setCustomPresets] = useState<CustomPreset[]>([]);
    const [isPresetManagerOpen, setIsPresetManagerOpen] = useState(false);
    const [newPresetName, setNewPresetName] = useState('');

    // 定义多个预设方案
    const defaultPresets = useMemo(() => [
        {
            id: 'default',
            name: t('proxy.router.preset_default'),
            description: t('proxy.router.preset_default_desc'),
            mappings: {
                "gemini-3.6-flash": "gemini-3.6-flash-tiered",
                "gemini-3.7-flash": "gemini-3.7-flash-tiered",
                "gemini-3.8-flash": "gemini-3.8-flash-tiered",
                "gemini-3.x-flash": "3.x-flash-tiered",
                "gpt-4*": "gemini-3.1-pro-high",
                "gpt-4o*": "gemini-3-flash",
                "gpt-3.5*": "gemini-2.5-flash",
                "o1-*": "gemini-3.1-pro-high",
                "o3-*": "gemini-3.1-pro-high",
                "claude-3-5-sonnet-*": "claude-sonnet-4-6",
                "claude-3-opus-*": "claude-opus-4-6-thinking",
                "claude-opus-4-6*": "claude-opus-4-6-thinking",
                "claude-haiku-*": "gemini-2.5-flash",
                "claude-3-haiku-*": "gemini-2.5-flash",
            }
        },
        {
            id: 'performance',
            name: t('proxy.router.preset_performance'),
            description: t('proxy.router.preset_performance_desc'),
            mappings: {
                "gpt-4*": "claude-opus-4-6-thinking",
                "gpt-4o*": "claude-sonnet-4-6",
                "gpt-3.5*": "gemini-3-flash",
                "o1-*": "claude-opus-4-6-thinking",
                "o3-*": "claude-opus-4-6-thinking",
                "claude-3-5-sonnet-*": "claude-sonnet-4-6",
                "claude-3-opus-*": "claude-opus-4-6-thinking",
                "claude-opus-4-6*": "claude-opus-4-6-thinking",
                "claude-haiku-*": "claude-sonnet-4-6",
                "claude-3-haiku-*": "claude-sonnet-4-6",
            }
        },
        {
            id: 'cost-effective',
            name: t('proxy.router.preset_cost'),
            description: t('proxy.router.preset_cost_desc'),
            mappings: {
                "gpt-4*": "gemini-3-flash",
                "gpt-4o*": "gemini-2.5-flash",
                "gpt-3.5*": "gemini-2.5-flash",
                "o1-*": "gemini-3-flash",
                "o3-*": "gemini-3-flash",
                "claude-3-5-sonnet-*": "gemini-3-flash",
                "claude-3-opus-*": "gemini-3-flash",
                "claude-opus-4-*": "gemini-3-flash", // Cost-effective: map all opus 4 to flash
                "claude-haiku-*": "gemini-2.5-flash",
                "claude-3-haiku-*": "gemini-2.5-flash",
            }
        },
        {
            id: 'balanced',
            name: t('proxy.router.preset_balanced'),
            description: t('proxy.router.preset_balanced_desc'),
            mappings: {
                "gpt-4*": "gemini-3.1-pro-high",
                "gpt-4o*": "gemini-3-flash",
                "gpt-3.5*": "gemini-2.5-flash",
                "o1-*": "claude-sonnet-4-6",
                "o3-*": "claude-sonnet-4-6",
                "claude-3-5-sonnet-*": "claude-sonnet-4-6",
                "claude-3-opus-*": "gemini-3.1-pro-high",
                "claude-opus-4-5*": "gemini-3.1-pro-high",
                "claude-opus-4-6*": "claude-opus-4-6-thinking", // Balanced: Keep 4.6 as itself (or map to high?) Let's map to itself for now to utilize header
                "claude-haiku-*": "gemini-2.5-flash",
                "claude-3-haiku-*": "gemini-2.5-flash",
            }
        },
    ], [t]);

    const presetOptions = useMemo(() => {
        return [...defaultPresets, ...customPresets];
    }, [defaultPresets, customPresets]);

    // Custom Presets Logic
    const loadCustomPresets = useCallback(() => {
        try {
            const saved = localStorage.getItem(CUSTOM_PRESETS_STORAGE_KEY);
            if (saved) {
                setCustomPresets(JSON.parse(saved));
            }
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'usePresetManager.loadCustomPresets',
                triggerAction: 'localStorage.getItem',
            });
        }
    }, []);

    const saveCustomPresetsToStorage = useCallback((presets: CustomPreset[]) => {
        try {
            localStorage.setItem(CUSTOM_PRESETS_STORAGE_KEY, JSON.stringify(presets));
            setCustomPresets(presets);
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'usePresetManager.saveCustomPresetsToStorage',
                triggerAction: 'localStorage.setItem',
            });
            showToast('Failed to save preset', 'error');
        }
    }, []);

    const handleSaveCurrentAsPreset = useCallback(() => {
        if (!appConfig?.proxy.custom_mapping || Object.keys(appConfig.proxy.custom_mapping).length === 0) {
            showToast(t('proxy.router.no_mapping_to_save'), 'warning');
            return;
        }
        if (!newPresetName.trim()) {
            showToast(t('proxy.router.preset_name_required'), 'warning');
            return;
        }

        const newPreset: CustomPreset = {
            id: `custom_${Date.now()}`,
            name: newPresetName,
            description: t('proxy.router.custom_preset_desc'),
            mappings: { ...appConfig.proxy.custom_mapping }
        };

        const updatedPresets = [...customPresets, newPreset];
        saveCustomPresetsToStorage(updatedPresets);
        setNewPresetName('');
        setIsPresetManagerOpen(false);
        showToast(t('proxy.router.preset_saved', { defaultValue: 'Preset saved successfully' }), 'success');
        // Auto select the new preset
        setSelectedPreset(newPreset.id);
    }, [appConfig, newPresetName, customPresets, saveCustomPresetsToStorage, t]);

    const handleDeletePreset = useCallback((id: string) => {
        const updatedPresets = customPresets.filter(p => p.id !== id);
        saveCustomPresetsToStorage(updatedPresets);
        setSelectedPreset((prev) => prev === id ? DEFAULT_PRESET_ID : prev);
    }, [customPresets, saveCustomPresetsToStorage]);

    // 应用预设映射 (通配符)
    const handleApplyPresets = useCallback(async () => {
        if (!appConfig) return;

        const selectedPresetData = presetOptions.find(p => p.id === selectedPreset);
        if (!selectedPresetData) return;

        // 构造新配置
        const newConfig = {
            ...appConfig.proxy,
            // 策略:覆盖同名 key,保留其他自定义 key
            // [FIX #1738] Type assertion to ensure Record<string, string> compatibility
            custom_mapping: { ...appConfig.proxy.custom_mapping, ...selectedPresetData.mappings } as Record<string, string>
        };

        // 备份旧配置用于回滚
        const oldConfig = { ...appConfig };

        try {
            // 1. 乐观更新：立即更新 UI
            setAppConfig({ ...appConfig, proxy: newConfig });
            showToast(t('proxy.router.presets_applied') + ` (${selectedPresetData.name})`, 'success');

            // 2. 后台异步保存
            await invoke('update_model_mapping', { config: newConfig });

            // 3. 重新加载配置以确保一致性
            await loadConfig();
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'usePresetManager.handleApplyPresets',
                triggerAction: 'update_model_mapping',
            });
            // 3. 失败回滚
            setAppConfig(oldConfig);
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    }, [appConfig, presetOptions, selectedPreset, setAppConfig, loadConfig, t]);

    useEffect(() => {
        loadCustomPresets();
    }, [loadCustomPresets]);

    return {
        defaultPresets,
        presetOptions,
        customPresets,
        selectedPreset,
        setSelectedPreset,
        newPresetName,
        setNewPresetName,
        isPresetManagerOpen,
        setIsPresetManagerOpen,
        handleSaveCurrentAsPreset,
        handleDeletePreset,
        handleApplyPresets,
    };
}
