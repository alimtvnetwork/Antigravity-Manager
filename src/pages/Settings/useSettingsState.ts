import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { request as invoke } from '../../utils/request';
import { useConfigStore } from '../../stores/useConfigStore';
import { AppConfig } from '../../types/config';
import { useDebugConsole } from '../../stores/useDebugConsole';
import { useTranslation } from 'react-i18next';
import { isTauri } from '../../utils/env';
import { useErrorStore } from '../../stores/error-store';
import versionData from '../../../version.json';

export function normalizeDataDirDisplay(path: string): string {
    const trimmed = path.trim();
    if (trimmed.startsWith('\\\\?\\UNC\\')) {
        return `\\\\${trimmed.slice('\\\\?\\UNC\\'.length)}`;
    }
    if (trimmed.startsWith('\\\\?\\')) {
        return trimmed.slice('\\\\?\\'.length);
    }
    if (trimmed.startsWith('//?/UNC/')) {
        return `//${trimmed.slice('//?/UNC/'.length)}`;
    }
    if (trimmed.startsWith('//?/')) {
        return trimmed.slice('//?/'.length);
    }
    return trimmed;
}

export function useSettingsState() {
    const { t, i18n } = useTranslation();
    const { tab: routeTab } = useParams<{ tab?: string }>();
    const { config, loadConfig, saveConfig, updateLanguage, updateTheme } = useConfigStore();
    const { enable, disable, isEnabled, open: openDebugModal } = useDebugConsole();
    const [activeTab, setActiveTab] = useState<'general' | 'account' | 'proxy' | 'email' | 'ssh' | 'themes' | 'supabase' | 'advanced' | 'debug' | 'about'>('general');

    useEffect(() => {
        if (!routeTab) return;
        const normalized = routeTab.toLowerCase();
        if (normalized === 'auto-switcher') {
            setActiveTab('account');
            return;
        }
        const validTabs = ['general', 'account', 'proxy', 'email', 'ssh', 'themes', 'supabase', 'advanced', 'debug', 'about'] as const;
        if ((validTabs as readonly string[]).includes(normalized)) {
            setActiveTab(normalized as typeof validTabs[number]);
        }
    }, [routeTab]);
    const [appVersion, setAppVersion] = useState<string>(versionData.version || versionData.Version || '4.183.0');
    const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);
    const [isMoreDropdownOpen, setIsMoreDropdownOpen] = useState(false);
    const [formData, setFormData] = useState<AppConfig>({
        language: 'en',
        theme: 'system',
        auto_refresh: false,
        refresh_interval: 15,
        auto_sync: true,
        auto_sync_migrated: true,
        sync_interval: 5,
        proxy: {
            enabled: false,
            port: 8080,
            api_key: '',
            auto_start: false,
            request_timeout: 120,
            enable_logging: false,
            upstream_proxy: {
                enabled: false,
                url: ''
            },
            debug_logging: {
                enabled: false,
                output_dir: undefined
            } as { enabled: boolean; output_dir?: string },
            proxy_pool: {
                enabled: false,
                proxies: [],
                health_check_interval: 300,
                auto_failover: true,
                strategy: 'priority',
                account_bindings: {}
            }
        },
        scheduled_warmup: {
            enabled: false,
            monitored_models: []
        },
        quota_protection: {
            enabled: false,
            threshold_percentage: 15,
            monitored_models: []
        },
        pinned_quota_models: {
            models: ['gemini-pro-agent', 'gemini-3-flash-agent', 'gemini-3.1-flash-image', 'claude-opus-4-6-thinking']
        },
        cloudflared: {
            enabled: false,
            mode: 'quick',
            port: 7860,
            use_http2: true
        },
        circuit_breaker: {
            enabled: false,
            backoff_steps: [30, 60, 120, 300, 600]
        },
        hidden_menu_items: [],  // Menu display settings: do not hide any menu items by default
        instance_clone_mode: 'full',
        instance_card_max_projects: 3,
        auto_profile_switcher: {
            is_enabled: true,
            check_interval_seconds: 60,
            low_quota_threshold_percent: 15.0,
            target_model: 'gemini-pro',
            has_auto_resume: true,
            cooldown_seconds: 180,
            ...({ account_lockout_window_minutes: 60 } as any),
        },
        conversation_cleanup: {
            is_enabled: false,
            interval_hours: 1,
            keep_count: 40,
        },
        training_api_enabled: false,
        notify_on_update: true,
        notify_via_email: true,
        notify_via_telegram: true,
    });

    // Dialog state
    // Dialog state
    const [isClearLogsOpen, setIsClearLogsOpen] = useState(false);
    const [isSupportModalOpen, setIsSupportModalOpen] = useState(false);
    const [dataDirPath, setDataDirPath] = useState<string>('~/.antigravity_tools/');
    const [pendingDataDir, setPendingDataDir] = useState<string>('');
    const [isMigrateDataDirOpen, setIsMigrateDataDirOpen] = useState(false);
    const [isMigratingDataDir, setIsMigratingDataDir] = useState(false);

    // Antigravity cache clearing state
    const [isClearCacheOpen, setIsClearCacheOpen] = useState(false);
    const [cachePaths, setCachePaths] = useState<string[]>([]);
    const [isClearingCache, setIsClearingCache] = useState(false);

    // Update check state
    const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
    const [updateInfo, setUpdateInfo] = useState<{
        hasUpdate: boolean;
        latestVersion: string;
        currentVersion: string;
        downloadUrl: string;
        source?: string;
        channel?: 'stable' | 'beta';
    } | null>(null);
    const [isInstallerUpdating, setIsInstallerUpdating] = useState(false);

    // Homebrew Cask state
    const [isBrewInstalled, setIsBrewInstalled] = useState(false);
    const [isBrewUpgrading, setIsBrewUpgrading] = useState(false);
    const [isBrewConfirmOpen, setIsBrewConfirmOpen] = useState(false);
    const [isBrewSuccessOpen, setIsBrewSuccessOpen] = useState(false);


    useEffect(() => {
        loadConfig();

        // Get actual data directory path
        invoke<string>('get_data_dir_path')
            .then(path => setDataDirPath(normalizeDataDirDisplay(path)))
            .catch(err => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'useSettingsState.getDataDirPath',
                    triggerAction: 'get_data_dir_path',
                });
            });

        // Load update settings
        invoke<{
            auto_check: boolean;
            last_check_time: number;
            check_interval_hours: number;
            notify_on_update?: boolean;
            notify_via_email?: boolean;
            notify_via_telegram?: boolean;
            update_channel?: 'stable' | 'beta';
        }>('get_update_settings')
            .then(settings => {
                setFormData(prev => ({
                    ...prev,
                    auto_check_update: settings.auto_check,
                    update_check_interval: settings.check_interval_hours,
                    notify_on_update: settings.notify_on_update ?? true,
                    notify_via_email: settings.notify_via_email ?? true,
                    notify_via_telegram: settings.notify_via_telegram ?? true,
                    update_channel: settings.update_channel || (appVersion.includes('-') ? 'beta' : 'stable'),
                }));
            })
            .catch(err => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'useSettingsState.getUpdateSettings',
                    triggerAction: 'get_update_settings',
                });
            });

        // Get actual auto launch status
        invoke<boolean>('is_auto_launch_enabled')
            .then(enabled => {
                setFormData(prev => ({ ...prev, auto_launch: enabled }));
            })
            .catch(err => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'useSettingsState.isAutoLaunchEnabled',
                    triggerAction: 'is_auto_launch_enabled',
                });
            });

        // Get actual app version
        if (isTauri()) {
            import('@tauri-apps/api/app').then(({ getVersion }) => {
                getVersion()
                    .then(v => setAppVersion(v))
                    .catch(err => {
                        useErrorStore.getState().trackWarning(err, {
                            source: 'useSettingsState.getVersion',
                            triggerAction: 'getVersion',
                        });
                    });
            });
        }

        // Check if installed via Homebrew Cask (Tauri environment only)
        if (isTauri()) {
            invoke<boolean>('check_homebrew_installation')
                .then(installed => setIsBrewInstalled(installed))
                .catch(err => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'useSettingsState.checkHomebrewInstallation',
                        triggerAction: 'check_homebrew_installation',
                    });
                });
        }

    }, [loadConfig]);

    useEffect(() => {
        if (config) {
            setFormData({
                ...config,
                auto_sync: config.auto_sync ?? true,
                auto_sync_migrated: config.auto_sync_migrated ?? true,
                conversation_cleanup: config.conversation_cleanup ?? {
                    is_enabled: false,
                    interval_hours: 1,
                    keep_count: 40,
                },
                training_api_enabled: config.training_api_enabled ?? false,
                instance_card_max_projects: config.instance_card_max_projects ?? 3,
            });
        }
    }, [config]);

    // User-controlled debug console logic


    return {
        t,
        i18n,
        config,
        loadConfig,
        saveConfig,
        updateLanguage,
        updateTheme,
        enable,
        disable,
        isEnabled,
        openDebugModal,
        activeTab,
        setActiveTab,
        appVersion,
        setAppVersion,
        isBackupModalOpen,
        setIsBackupModalOpen,
        isMoreDropdownOpen,
        setIsMoreDropdownOpen,
        formData,
        setFormData,
        isClearLogsOpen,
        setIsClearLogsOpen,
        isSupportModalOpen,
        setIsSupportModalOpen,
        dataDirPath,
        setDataDirPath,
        pendingDataDir,
        setPendingDataDir,
        isMigrateDataDirOpen,
        setIsMigrateDataDirOpen,
        isMigratingDataDir,
        setIsMigratingDataDir,
        isClearCacheOpen,
        setIsClearCacheOpen,
        cachePaths,
        setCachePaths,
        isClearingCache,
        setIsClearingCache,
        isCheckingUpdate,
        setIsCheckingUpdate,
        updateInfo,
        setUpdateInfo,
        isInstallerUpdating,
        setIsInstallerUpdating,
        isBrewInstalled,
        setIsBrewInstalled,
        isBrewUpgrading,
        setIsBrewUpgrading,
        isBrewConfirmOpen,
        setIsBrewConfirmOpen,
        isBrewSuccessOpen,
        setIsBrewSuccessOpen,
    };
}

export type SettingsState = ReturnType<typeof useSettingsState>;
