import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../utils/request';
import { showToast } from '../../components/common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import type { AppConfig } from '../../types/config';
import type { ProxyStatus } from './types';
import { STATUS_POLL_INTERVAL_MS } from './constants';

export interface ProxyStatusState {
    status: ProxyStatus;
    loading: boolean;
    loadStatus: () => Promise<void>;
    handleToggle: () => Promise<void>;
}

export function useProxyStatus(appConfig: AppConfig | null): ProxyStatusState {
    const { t } = useTranslation();

    const [status, setStatus] = useState<ProxyStatus>({
        running: false,
        port: 0,
        base_url: '',
        active_accounts: 0,
    });
    const [loading, setLoading] = useState(false);

    const loadStatus = useCallback(async () => {
        try {
            const s = await invoke<ProxyStatus>('get_proxy_status');
            // 如果后端返回 starting 或 busy，则在 UI 上表现为加载中
            if (s.base_url === 'starting' || s.base_url === 'busy') {
                // 如果当前已经是运行状态，不要被覆盖为 false
                setStatus(prev => ({ ...s, running: prev.running }));
            } else {
                setStatus(s);
            }
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyStatus.loadStatus',
                triggerAction: 'get_proxy_status',
            });
        }
    }, []);

    const handleToggle = useCallback(async () => {
        if (!appConfig) return;
        setLoading(true);
        try {
            if (status.running) {
                await invoke('stop_proxy_service');
            } else {
                // 使用当前的 appConfig.proxy 启动
                await invoke('start_proxy_service', { config: appConfig.proxy });
            }
            await loadStatus();
        } catch (error: unknown) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useProxyStatus.handleToggle',
                triggerAction: status.running ? 'stop_proxy_service' : 'start_proxy_service',
            });
            showToast(t('proxy.dialog.operate_failed', { error: String(error) }), 'error');
        } finally {
            setLoading(false);
        }
    }, [appConfig, status.running, loadStatus, t]);

    useEffect(() => {
        loadStatus();
        const interval = setInterval(loadStatus, STATUS_POLL_INTERVAL_MS);
        return () => {
            clearInterval(interval);
        };
    }, [loadStatus]);

    return { status, loading, loadStatus, handleToggle };
}
