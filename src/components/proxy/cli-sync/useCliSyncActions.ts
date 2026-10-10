import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'i18next';
import { request as invoke } from '../../../utils/request';
import { showToast } from '../../common/ToastContainer';
import { useProxyModels } from '../../../hooks/useProxyModels';
import { useErrorStore } from '../../../stores/error-store';
import {
    ALL_CLI_APPS,
    CliAppType,
    CliStatus,
    DEDICATED_STATUS_COMMAND,
    DEFAULT_SELECTED_MODELS,
    V1_PATH_APPS,
    ViewingConfig,
} from './types';

const emptyFlagRecord = (): Record<CliAppType, boolean> => ({
    Claude: false,
    Codex: false,
    JeikCode: false,
    GrokBuild: false,
    Gemini: false,
    OpenCode: false,
    Droid: false,
    Hermes: false,
    OpenClaw: false,
});

const nullStatusRecord = (): Record<CliAppType, CliStatus | null> => ({
    Claude: null,
    Codex: null,
    JeikCode: null,
    GrokBuild: null,
    Gemini: null,
    OpenCode: null,
    Droid: null,
    Hermes: null,
    OpenClaw: null,
});

const toErrorMessage = (error: unknown): string =>
    error instanceof Error ? error.message : String(error);

const RESTORE_COMMANDS: Record<string, string> = {
    Droid: 'execute_droid_restore',
    OpenCode: 'execute_opencode_restore',
    Hermes: 'execute_hermes_restore',
    OpenClaw: 'execute_openclaw_restore',
};

const CONFIG_CONTENT_COMMANDS: Record<string, string> = {
    Droid: 'get_droid_config_content',
    OpenCode: 'get_opencode_config_content',
    Hermes: 'get_hermes_config_content',
    OpenClaw: 'get_openclaw_config_content',
};

export interface CliSyncActions {
    t: TFunction;
    statuses: Record<CliAppType, CliStatus | null>;
    loading: Record<CliAppType, boolean>;
    syncing: Record<CliAppType, boolean>;
    syncAccounts: boolean;
    setSyncAccounts: (v: boolean) => void;
    selectedModels: Record<CliAppType, string>;
    setSelectedModels: React.Dispatch<React.SetStateAction<Record<CliAppType, string>>>;
    viewingConfig: ViewingConfig | null;
    setViewingConfig: (v: ViewingConfig | null) => void;
    restoreConfirmApp: CliAppType | null;
    setRestoreConfirmApp: (v: CliAppType | null) => void;
    syncConfirmApp: CliAppType | null;
    setSyncConfirmApp: (v: CliAppType | null) => void;
    clearConfirmApp: CliAppType | null;
    setClearConfirmApp: (v: CliAppType | null) => void;
    droidSyncModal: boolean;
    setDroidSyncModal: (v: boolean) => void;
    hermesSyncModal: boolean;
    setHermesSyncModal: (v: boolean) => void;
    openClawSyncModal: boolean;
    setOpenClawSyncModal: (v: boolean) => void;
    openCodeSyncModal: boolean;
    setOpenCodeSyncModal: (v: boolean) => void;
    modelOptions: { value: string; label: string; group: string }[];
    getFormattedProxyUrl: (app: CliAppType) => string;
    checkStatus: (app: CliAppType) => Promise<void>;
    handleSync: (app: CliAppType) => void;
    executeSync: () => Promise<void>;
    handleRestore: (app: CliAppType) => void;
    executeRestore: () => Promise<void>;
    handleClear: (app: CliAppType) => void;
    executeClear: () => Promise<void>;
    handleViewConfig: (app: CliAppType, fileName?: string) => Promise<void>;
    openExternalUrl: (url: string) => Promise<void>;
}

export function useCliSyncActions(proxyUrl: string, apiKey: string): CliSyncActions {
    const { t } = useTranslation();
    const [statuses, setStatuses] = useState<Record<CliAppType, CliStatus | null>>(nullStatusRecord);
    const [loading, setLoading] = useState<Record<CliAppType, boolean>>(emptyFlagRecord);
    const [syncing, setSyncing] = useState<Record<CliAppType, boolean>>(emptyFlagRecord);
    const [syncAccounts, setSyncAccounts] = useState(false);
    const [droidSyncModal, setDroidSyncModal] = useState(false);
    const [hermesSyncModal, setHermesSyncModal] = useState(false);
    const [openClawSyncModal, setOpenClawSyncModal] = useState(false);
    const [selectedModels, setSelectedModels] = useState<Record<CliAppType, string>>(DEFAULT_SELECTED_MODELS);
    const [viewingConfig, setViewingConfig] = useState<ViewingConfig | null>(null);
    const [restoreConfirmApp, setRestoreConfirmApp] = useState<CliAppType | null>(null);
    const [syncConfirmApp, setSyncConfirmApp] = useState<CliAppType | null>(null);
    const [openCodeSyncModal, setOpenCodeSyncModal] = useState(false);
    const [clearConfirmApp, setClearConfirmApp] = useState<CliAppType | null>(null);

    const { models: proxyModels } = useProxyModels();

    const modelOptions = proxyModels.map(m => ({
        value: m.id,
        label: m.name,
        group: m.group || 'General',
    }));

    // Format Proxy URL based on CLI application
    const getFormattedProxyUrl = useCallback((app: CliAppType) => {
        if (!proxyUrl) return '';
        const base = proxyUrl.trimEnd().replace(/\/+$/, '');
        // Anthropic / OpenAI / Responses protocol CLIs usually require /v1
        if (V1_PATH_APPS.has(app)) {
            return base.endsWith('/v1') ? base : `${base}/v1`;
        }
        // Claude and Gemini SDKs handle version paths automatically or do not need /v1
        return base.replace(/\/v1$/, '');
    }, [proxyUrl]);

    const checkStatus = useCallback(async (app: CliAppType) => {
        setLoading(prev => ({ ...prev, [app]: true }));
        try {
            const formattedUrl = getFormattedProxyUrl(app);
            let command: string;
            let params: Record<string, unknown>;
            const dedicated = DEDICATED_STATUS_COMMAND[app];
            if (dedicated) {
                command = dedicated;
                params = { proxyUrl: formattedUrl };
            } else {
                command = 'get_cli_sync_status';
                params = { appType: app, proxyUrl: formattedUrl };
            }

            const status = await invoke<CliStatus>(command, params);
            setStatuses(prev => ({ ...prev, [app]: status }));
        } catch (error) {
            console.error(`Failed to check ${app} status:`, error);
            // Tracked in the error module; status chip keeps previous/unknown state, retried on next check.
            useErrorStore.getState().trackWarning(error, {
                source: 'CliSyncCard.checkStatus',
                triggerAction: 'check_cli_status',
                context: { app },
            });
        } finally {
            setLoading(prev => ({ ...prev, [app]: false }));
        }
    }, [getFormattedProxyUrl]);

    const handleSync = (app: CliAppType) => {
        if (app === 'Droid') {
            setDroidSyncModal(true);
            return;
        }
        if (app === 'OpenCode') {
            setOpenCodeSyncModal(true);
            return;
        }
        if (app === 'Hermes') {
            setHermesSyncModal(true);
            return;
        }
        if (app === 'OpenClaw') {
            setOpenClawSyncModal(true);
            return;
        }
        setSyncConfirmApp(app);
    };

    const executeSync = async () => {
        const app = syncConfirmApp;
        if (!app) return;
        setSyncConfirmApp(null);

        if (!proxyUrl || !apiKey) {
            showToast(t('proxy.cli_sync.toast.config_missing', { defaultValue: 'Please generate API Key and start service first' }), 'error');
            return;
        }

        try {
            const formattedUrl = getFormattedProxyUrl(app);
            const command = app === 'OpenCode' ? 'execute_opencode_sync' : 'execute_cli_sync';
            const params = app === 'OpenCode'
                ? { proxyUrl: formattedUrl, apiKey: apiKey, syncAccounts: syncAccounts }
                : { appType: app, proxyUrl: formattedUrl, apiKey: apiKey, model: selectedModels[app] };

            await invoke(command, params);
            showToast(t(app === 'OpenCode' ? 'proxy.opencode_sync.toast.sync_success' : 'proxy.cli_sync.toast.sync_success', { name: app, defaultValue: `${app} synced successfully` }), 'success');
            await checkStatus(app);
        } catch (error: unknown) {
            const msg = toErrorMessage(error);
            showToast(t(app === 'OpenCode' ? 'proxy.opencode_sync.toast.sync_error' : 'proxy.cli_sync.toast.sync_error', { name: app, error: msg, defaultValue: `Sync failed: ${msg}` }), 'error');
        } finally {
            setSyncing(prev => ({ ...prev, [app]: false }));
        }
    };

    const handleRestore = (app: CliAppType) => {
        setRestoreConfirmApp(app);
    };

    const executeRestore = async () => {
        if (!restoreConfirmApp) return;
        const app = restoreConfirmApp;
        setRestoreConfirmApp(null);

        setSyncing(prev => ({ ...prev, [app]: true }));
        try {
            const command = RESTORE_COMMANDS[app] ?? 'execute_cli_restore';
            const params = RESTORE_COMMANDS[app] ? {} : { appType: app };
            await invoke(command, params);
            showToast(t('common.success'), 'success');
            await checkStatus(app);
        } catch (error: unknown) {
            showToast(toErrorMessage(error), 'error');
        } finally {
            setSyncing(prev => ({ ...prev, [app]: false }));
        }
    };

    const handleClear = (app: CliAppType) => {
        setClearConfirmApp(app);
    };

    const executeClear = async () => {
        if (!clearConfirmApp) return;
        const app = clearConfirmApp;
        setClearConfirmApp(null);

        setSyncing(prev => ({ ...prev, [app]: true }));
        try {
            if (app === 'Hermes') {
                await invoke('execute_hermes_clear');
                showToast(t('proxy.hermes_sync.toast.clear_success', { defaultValue: 'Hermes configuration cleared successfully' }), 'success');
            } else if (app === 'OpenClaw') {
                await invoke('execute_openclaw_clear');
                showToast(t('proxy.openclaw_sync.toast.clear_success', { defaultValue: 'OpenClaw configuration cleared successfully' }), 'success');
            } else {
                const formattedUrl = getFormattedProxyUrl(app);
                await invoke('execute_opencode_clear', { proxyUrl: formattedUrl, clearLegacy: true });
                showToast(t('proxy.opencode_sync.toast.clear_success', { defaultValue: 'OpenCode cleared successfully' }), 'success');
            }
            await checkStatus(app);
        } catch (error: unknown) {
            const msg = toErrorMessage(error);
            const toastKey = app === 'Hermes'
                ? 'proxy.hermes_sync.toast.clear_error'
                : app === 'OpenClaw'
                    ? 'proxy.openclaw_sync.toast.clear_error'
                    : 'proxy.opencode_sync.toast.clear_error';
            showToast(t(toastKey, { error: msg, defaultValue: `Clear failed: ${msg}` }), 'error');
        } finally {
            setSyncing(prev => ({ ...prev, [app]: false }));
        }
    };

    const handleViewConfig = async (app: CliAppType, fileName?: string) => {
        try {
            const status = statuses[app];
            if (!status) return;

            const targetFile = fileName || status.files[0];
            const dedicated = CONFIG_CONTENT_COMMANDS[app];
            const command = dedicated ?? 'get_cli_config_content';
            const params: Record<string, unknown> = dedicated
                ? (app === 'OpenCode' ? { request: { fileName: targetFile } } : {})
                : { appType: app, fileName: targetFile };

            const content = await invoke<string>(command, params);
            setViewingConfig({
                app,
                content,
                fileName: targetFile,
                allFiles: status.files,
            });
        } catch (error: unknown) {
            showToast(toErrorMessage(error), 'error');
        }
    };

    useEffect(() => {
        for (const app of ALL_CLI_APPS) {
            void checkStatus(app);
        }
    }, [checkStatus]);

    const openExternalUrl = async (url: string) => {
        try {
            const { openUrl } = await import('@tauri-apps/plugin-opener');
            await openUrl(url);
        } catch {
            window.open(url, '_blank', 'noopener,noreferrer');
        }
    };

    return {
        t,
        statuses,
        loading,
        syncing,
        syncAccounts,
        setSyncAccounts,
        selectedModels,
        setSelectedModels,
        viewingConfig,
        setViewingConfig,
        restoreConfirmApp,
        setRestoreConfirmApp,
        syncConfirmApp,
        setSyncConfirmApp,
        clearConfirmApp,
        setClearConfirmApp,
        droidSyncModal,
        setDroidSyncModal,
        hermesSyncModal,
        setHermesSyncModal,
        openClawSyncModal,
        setOpenClawSyncModal,
        openCodeSyncModal,
        setOpenCodeSyncModal,
        modelOptions,
        getFormattedProxyUrl,
        checkStatus,
        handleSync,
        executeSync,
        handleRestore,
        executeRestore,
        handleClear,
        executeClear,
        handleViewConfig,
        openExternalUrl,
    };
}
