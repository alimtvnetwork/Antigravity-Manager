import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { showToast } from '../../common/ToastContainer';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { getAutoSwitcherDaemonStatus, type AutoSwitcherDaemonStatus } from '../../../services/instanceService';
import { DEFAULT_CONFIG } from './switcherTypes';
import type { AutoProfileSwitcherConfig } from '../../../types/config';

export function useAutoSwitcher(config?: AutoProfileSwitcherConfig, onChange: (c: AutoProfileSwitcherConfig) => void = () => {}) {
    const { t } = useTranslation();
    const currentConfig = config || DEFAULT_CONFIG;
    const { smartRotateProfileAccount, activeInstanceId } = useInstanceStore();
    const [rotationFeedback, setRotationFeedback] = useState<string | null>(null);
    const [isRotating, setIsRotating] = useState(false);
    const [isActionsOpen, setIsActionsOpen] = useState(false);
    const actionsRef = React.useRef<HTMLDivElement>(null);
    const fileInputRef = React.useRef<HTMLInputElement>(null);
    const [daemonStatus, setDaemonStatus] = useState<AutoSwitcherDaemonStatus | null>(null);
    const [remainingSeconds, setRemainingSeconds] = useState<number>(0);

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;
        let isMounted = true;

        const fetchStatus = () => {
            getAutoSwitcherDaemonStatus()
                .then((status) => {
                    if (!isMounted) return;
                    setDaemonStatus(status);
                    if (status.next_check_in_seconds !== undefined) {
                        setRemainingSeconds(Math.max(0, status.next_check_in_seconds));
                    }
                })
                .catch(() => {});
        };

        fetchStatus();

        import('@tauri-apps/api/event').then(({ listen }) => {
            if (!isMounted) return;
            listen<AutoSwitcherDaemonStatus>('auto-switcher://status-tick', (event) => {
                if (!isMounted) return;
                if (event.payload) {
                    setDaemonStatus(event.payload);
                    if (event.payload.next_check_in_seconds !== undefined) {
                        setRemainingSeconds(Math.max(0, event.payload.next_check_in_seconds));
                    }
                }
            }).then((unsub) => {
                if (isMounted) {
                    unlisten = unsub;
                } else {
                    unsub();
                }
            }).catch(() => {});
        });

        const pollTimer = setInterval(fetchStatus, 3000);
        const tickTimer = setInterval(() => {
            setRemainingSeconds((prev) => (prev > 0 ? prev - 1 : 0));
        }, 1000);

        return () => {
            isMounted = false;
            clearInterval(pollTimer);
            clearInterval(tickTimer);
            if (unlisten) unlisten();
        };
    }, []);

    React.useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (actionsRef.current && !actionsRef.current.contains(e.target as Node)) {
                setIsActionsOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const handleExport = () => {
        setIsActionsOpen(false);
        const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(currentConfig, null, 2));
        const downloadAnchor = document.createElement('a');
        downloadAnchor.setAttribute("href", dataStr);
        downloadAnchor.setAttribute("download", `auto_switcher_config_${Date.now()}.json`);
        document.body.appendChild(downloadAnchor);
        downloadAnchor.click();
        downloadAnchor.remove();
        showToast('Auto-Switcher configuration exported', 'success');
    };

    const handleImportFile = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = (event) => {
            try {
                const parsed = JSON.parse(event.target?.result as string);
                if (typeof parsed === 'object' && parsed !== null) {
                    onChange({ ...DEFAULT_CONFIG, ...parsed });
                    showToast('Auto-Switcher configuration imported successfully', 'success');
                } else {
                    showToast('Invalid configuration file format', 'error');
                }
            } catch (err: any) {
                showToast(`Failed to parse JSON: ${err?.message || err}`, 'error');
            }
        };
        reader.readAsText(file);
        e.target.value = '';
    };

    const handleResetDefaults = () => {
        setIsActionsOpen(false);
        onChange({ ...DEFAULT_CONFIG });
        showToast('Auto-Switcher configuration reset to defaults', 'info');
    };

    const handleToggleEnabled = (is_enabled: boolean) => {
        onChange({ ...currentConfig, is_enabled });
    };

    const handleIntervalChange = (val: number) => {
        const check_interval_seconds = Math.max(15, Math.min(600, val));
        onChange({ ...currentConfig, check_interval_seconds });
    };

    const handleThresholdChange = (val: number) => {
        const low_quota_threshold_percent = Math.max(1.0, Math.min(99.0, val));
        onChange({ ...currentConfig, low_quota_threshold_percent });
    };

    const handleManualRotate = async () => {
        setIsRotating(true);
        setRotationFeedback(null);
        try {
            const targetId = activeInstanceId || 'default';
            const result = await smartRotateProfileAccount(targetId);
            const resumeNote = (result.resumedProjectsCount ?? 0) > 0
                ? ` · Auto-resumed ${result.resumedProjectsCount} project(s) (<1h)`
                : '';
            setRotationFeedback(`Successfully rotated to profile '${result.instanceName}' with account '${result.accountEmail}'${resumeNote}`);
        } catch (e: any) {
            setRotationFeedback(`Error: ${e?.message || e?.toString() || 'Rotation failed'}`);
        } finally {
            setIsRotating(false);
        }
    };


    return {
        t,
        currentConfig,
        onChange,
        rotationFeedback,
        setRotationFeedback,
        isRotating,
        setIsRotating,
        isActionsOpen,
        setIsActionsOpen,
        actionsRef,
        fileInputRef,
        daemonStatus,
        setDaemonStatus,
        remainingSeconds,
        setRemainingSeconds,
        handleExport,
        handleImportFile,
        handleResetDefaults,
        handleToggleEnabled,
        handleIntervalChange,
        handleThresholdChange,
        handleManualRotate,
    };
}

export type AutoSwitcherApi = ReturnType<typeof useAutoSwitcher>;
