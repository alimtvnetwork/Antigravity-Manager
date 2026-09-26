import { useEffect, useRef } from 'react';
import { useConfigStore } from '../../stores/useConfigStore';
import { useAccountStore } from '../../stores/useAccountStore';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { showToast } from './ToastContainer';

function getAccountRemainingQuota(account: any, targetModel: string): number {
    if (!account?.quota) return 100;
    const quota = account.quota;
    let minPercent = 100;

    // Check models
    if (Array.isArray(quota.models) && quota.models.length > 0) {
        const targetLower = (targetModel || 'gemini-2.5-flash').toLowerCase();
        const nonBanned = quota.models.filter((m: any) => {
            const name = (m.name || '').toLowerCase();
            return !name.includes('3.0') && !name.includes('3.1');
        });

        // 1. Target matching models
        const matching = nonBanned.filter((m: any) => {
            const name = (m.name || '').toLowerCase();
            return name.includes(targetLower) || (targetLower.includes('flash') && name.includes('flash'));
        });

        if (matching.length > 0) {
            for (const m of matching) {
                if (typeof m.percentage === 'number' && m.percentage < minPercent) {
                    minPercent = m.percentage;
                }
            }
        }

        // 2. Any depleted model if below 100
        for (const m of nonBanned) {
            if (typeof m.percentage === 'number' && m.percentage < 100 && m.percentage < minPercent) {
                minPercent = m.percentage;
            }
        }
    }

    // Check quota_groups (e.g. immediate/hourly or weekly buckets)
    if (Array.isArray(quota.quota_groups)) {
        for (const group of quota.quota_groups) {
            if (Array.isArray(group?.buckets)) {
                for (const bucket of group.buckets) {
                    if (typeof bucket?.remaining_fraction === 'number') {
                        const pct = Math.round(bucket.remaining_fraction * 100);
                        if (pct < minPercent) {
                            minPercent = pct;
                        }
                    }
                }
            }
        }
    }

    return minPercent;
}

function BackgroundTaskRunner() {
    const { config } = useConfigStore();
    const { refreshAllQuotas } = useAccountStore();

    // Use refs to track previous state to detect "off -> on" transitions
    const prevAutoRefreshRef = useRef(false);
    const prevAutoSyncRef = useRef(false);
    const lastSwitchTimeRef = useRef<Record<string, number>>({});
    const isRotatingRef = useRef(false);

    // Auto Refresh Quota Effect
    useEffect(() => {
        if (!config) return;

        let intervalId: ReturnType<typeof setTimeout> | null = null;
        const { auto_refresh, refresh_interval } = config;

        // Check if we just turned it on
        if (auto_refresh && !prevAutoRefreshRef.current) {
            console.log('[BackgroundTask] Auto-refresh enabled, executing immediately...');
            refreshAllQuotas();
        }
        prevAutoRefreshRef.current = auto_refresh;

        if (auto_refresh && refresh_interval > 0) {
            console.log(`[BackgroundTask] Starting auto-refresh quota timer: ${refresh_interval} mins`);
            intervalId = setInterval(() => {
                console.log('[BackgroundTask] Auto-refreshing all quotas...');
                refreshAllQuotas();
            }, Math.min(refresh_interval * 60 * 1000, 2147483647));
        }

        return () => {
            if (intervalId) {
                console.log('[BackgroundTask] Clearing auto-refresh timer');
                clearInterval(intervalId);
            }
        };
    }, [config?.auto_refresh, config?.refresh_interval]);

    // Auto Sync Current Account Effect
    useEffect(() => {
        if (!config) return;

        let intervalId: ReturnType<typeof setTimeout> | null = null;
        const { auto_sync, sync_interval } = config;
        const { syncAccountFromDb } = useAccountStore.getState();

        // Check if we just turned it on
        if (auto_sync) {
            if (!prevAutoSyncRef.current) {
                console.log('[BackgroundTask] Auto-sync enabled, executing immediately...');
                syncAccountFromDb();
            }
        }
        prevAutoSyncRef.current = auto_sync;

        if (auto_sync && sync_interval > 0) {
            console.log(`[BackgroundTask] Starting auto-sync account timer: ${sync_interval} mins`);
            intervalId = setInterval(() => {
                console.log('[BackgroundTask] Auto-syncing current account from DB...');
                syncAccountFromDb();
            }, Math.min(sync_interval * 60 * 1000, 2147483647));
        }

        return () => {
            if (intervalId) {
                console.log('[BackgroundTask] Clearing auto-sync timer');
                clearInterval(intervalId);
            }
        };
    }, [config?.auto_sync, config?.sync_interval]);

    // Auto Profile Switcher Effect (Watches threshold slider, e.g. 98%, and triggers Smart Fast-Forward)
    useEffect(() => {
        const switcher = config?.auto_profile_switcher;
        if (!switcher || !switcher.is_enabled) return;

        const threshold = switcher.low_quota_threshold_percent ?? 10;
        const targetModel = switcher.target_model || 'gemini-2.5-flash';
        const intervalSecs = Math.max(5, switcher.check_interval_seconds || 15);

        const checkAndSmartFastForward = async () => {
            if (isRotatingRef.current) return;
            try {
                const accountStore = useAccountStore.getState();
                const instanceStore = useInstanceStore.getState();

                const activeInstId = instanceStore.activeInstanceId || 'default';
                const inst = instanceStore.instances.find(i => i.config.id === activeInstId);
                const boundAccId = inst?.config.bound_account_id || accountStore.currentAccount?.id;

                let boundAcc = accountStore.accounts.find(a => a.id === boundAccId);
                if (!boundAcc && accountStore.currentAccount) {
                    boundAcc = accountStore.currentAccount;
                }

                if (!boundAcc) return;

                const remainingQuota = getAccountRemainingQuota(boundAcc, targetModel);

                if (remainingQuota <= threshold && accountStore.accounts.length > 1) {
                    const now = Date.now();
                    const lastSwitch = lastSwitchTimeRef.current[activeInstId] || 0;
                    if (now - lastSwitch < 15000) {
                        return; // Cooldown of 15 seconds per instance
                    }

                    isRotatingRef.current = true;
                    lastSwitchTimeRef.current[activeInstId] = now;
                    console.log(`[AutoSwitcher] Active account ${boundAcc.email} quota ${remainingQuota}% <= ${threshold}%. Triggering Smart Fast-Forward...`);

                    const result = await instanceStore.smartRotateProfileAccount(activeInstId);
                    if (result) {
                        showToast(`Smart Fast-Forward: Switched to ${result.email} (quota ${remainingQuota}% <= ${threshold}%)`, 'success');
                    }
                }
            } catch (err) {
                console.error('[AutoSwitcher] Smart Fast-Forward error:', err);
            } finally {
                isRotatingRef.current = false;
            }
        };

        // Immediate check when threshold or is_enabled changes (e.g., user moves slider to 98%)
        const timerId = setTimeout(() => {
            checkAndSmartFastForward();
        }, 500);

        const intervalId = setInterval(checkAndSmartFastForward, intervalSecs * 1000);

        return () => {
            clearTimeout(timerId);
            clearInterval(intervalId);
        };
    }, [
        config?.auto_profile_switcher?.is_enabled,
        config?.auto_profile_switcher?.low_quota_threshold_percent,
        config?.auto_profile_switcher?.critical_threshold_percent,
        config?.auto_profile_switcher?.check_interval_seconds,
        config?.auto_profile_switcher?.target_model,
    ]);

    // Render nothing
    return null;
}

export default BackgroundTaskRunner;
