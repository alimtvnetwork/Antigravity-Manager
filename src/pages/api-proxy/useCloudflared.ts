import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../utils/request';
import { copyToClipboard } from '../../utils/clipboard';
import { showToast } from '../../components/common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import type { AppConfig } from '../../types/config';
import type { CloudflaredControls, CloudflaredMode, CloudflaredStatus, SaveConfigFn } from './types';
import { CF_STATUS_POLL_INTERVAL_MS, COPY_FEEDBACK_TIMEOUT_MS, DEFAULT_PROXY_PORT } from './constants';

interface UseCloudflaredArgs {
    appConfig: AppConfig | null;
    saveConfig: SaveConfigFn;
    proxyRunning: boolean;
    onCopied: (label: string | null) => void;
}

export function useCloudflared({ appConfig, saveConfig, proxyRunning, onCopied }: UseCloudflaredArgs): CloudflaredControls {
    const { t } = useTranslation();

    const [cfStatus, setCfStatus] = useState<CloudflaredStatus>({
        installed: false,
        running: false,
    });
    const [cfLoading, setCfLoading] = useState(false);
    const [cfMode, setCfMode] = useState<CloudflaredMode>('quick');
    const [cfToken, setCfToken] = useState('');
    const [cfUseHttp2, setCfUseHttp2] = useState(true); // HTTP/2 is enabled by default, which is more stable

    // Cloudflared: Check status
    const loadCfStatus = useCallback(async () => {
        try {
            const status = await invoke<CloudflaredStatus>('cloudflared_get_status');
            setCfStatus(status);
        } catch (error) {
            // Ignore the error, it may be that the manager is not initialized; only the error history is recorded
            useErrorStore.getState().trackWarning(error, {
                source: 'useCloudflared.loadCfStatus',
                triggerAction: 'cloudflared_get_status',
            });
        }
    }, []);

    // Cloudflared: Installation
    const handleCfInstall = useCallback(async () => {
        setCfLoading(true);
        try {
            const status = await invoke<CloudflaredStatus>('cloudflared_install');
            setCfStatus(status);
            showToast(t('proxy.cloudflared.install_success', { defaultValue: 'Cloudflared installed successfully' }), 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useCloudflared.handleCfInstall',
                triggerAction: 'cloudflared_install',
            });
            showToast(String(error), 'error');
        } finally {
            setCfLoading(false);
        }
    }, [t]);

    // Cloudflared: start/stop
    const handleCfToggle = useCallback(async (enable: boolean) => {
        if (enable && !proxyRunning) {
            showToast(
                t('proxy.cloudflared.require_proxy_running', { defaultValue: 'Please start the local proxy service first' }),
                'warning'
            );
            return;
        }
        setCfLoading(true);
        try {
            if (enable) {
                if (!cfStatus.installed) {
                    const installStatus = await invoke<CloudflaredStatus>('cloudflared_install');
                    setCfStatus(installStatus);
                    if (!installStatus.installed) {
                        throw new Error('Cloudflared install failed');
                    }
                    showToast(t('proxy.cloudflared.install_success', { defaultValue: 'Cloudflared installed successfully' }), 'success');
                }

                const config = {
                    enabled: true,
                    mode: cfMode,
                    port: appConfig?.proxy.port || DEFAULT_PROXY_PORT,
                    token: cfMode === 'auth' ? cfToken : null,
                    use_http2: cfUseHttp2,
                };
                const status = await invoke<CloudflaredStatus>('cloudflared_start', { config });
                setCfStatus(status);
                showToast(t('proxy.cloudflared.started', { defaultValue: 'Tunnel started' }), 'success');

                // Persistent "enabled" state
                if (appConfig) {
                    const newConfig = {
                        ...appConfig,
                        cloudflared: {
                            ...appConfig.cloudflared,
                            enabled: true,
                            mode: cfMode,
                            token: cfToken,
                            use_http2: cfUseHttp2,
                            port: appConfig.proxy.port || DEFAULT_PROXY_PORT
                        }
                    };
                    saveConfig(newConfig);
                }
            } else {
                const status = await invoke<CloudflaredStatus>('cloudflared_stop');
                setCfStatus(status);
                showToast(t('proxy.cloudflared.stopped', { defaultValue: 'Tunnel stopped' }), 'success');

                // Persistent "disabled" state
                if (appConfig) {
                    const newConfig = {
                        ...appConfig,
                        cloudflared: {
                            ...appConfig.cloudflared,
                            enabled: false
                        }
                    };
                    saveConfig(newConfig);
                }
            }
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useCloudflared.handleCfToggle',
                triggerAction: enable ? 'cloudflared_start' : 'cloudflared_stop',
            });
            showToast(String(error), 'error');
        } finally {
            setCfLoading(false);
        }
    }, [proxyRunning, cfStatus.installed, cfMode, cfToken, cfUseHttp2, appConfig, saveConfig, t]);

    // Cloudflared: Copy URL
    const handleCfCopyUrl = useCallback(async () => {
        if (cfStatus.url) {
            const success = await copyToClipboard(cfStatus.url);
            if (success) {
                onCopied('cf-url');
                setTimeout(() => onCopied(null), COPY_FEEDBACK_TIMEOUT_MS);
            }
        }
    }, [cfStatus.url, onCopied]);

    // Restore Cloudflared persistence state (originally executed synchronously in loadConfig)
    useEffect(() => {
        const cf = appConfig?.cloudflared;
        if (!cf) return;
        setCfMode(cf.mode || 'quick');
        setCfToken(cf.token || '');
        setCfUseHttp2(cf.use_http2 !== false); // HTTP/2 is enabled by default
    }, [appConfig?.cloudflared?.mode, appConfig?.cloudflared?.token, appConfig?.cloudflared?.use_http2]);

    useEffect(() => {
        loadCfStatus();
        const cfInterval = setInterval(loadCfStatus, CF_STATUS_POLL_INTERVAL_MS);
        return () => {
            clearInterval(cfInterval);
        };
    }, [loadCfStatus]);

    return {
        cfStatus,
        cfLoading,
        cfMode,
        cfToken,
        cfUseHttp2,
        setCfMode,
        setCfToken,
        setCfUseHttp2,
        loadCfStatus,
        handleCfInstall,
        handleCfToggle,
        handleCfCopyUrl,
    };
}
