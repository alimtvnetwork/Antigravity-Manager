import { useEffect, useRef } from 'react';
import { useConfigStore } from '../../stores/useConfigStore';
import { useAccountStore } from '../../stores/useAccountStore';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { showToast } from './ToastContainer';
import { isTauri } from '../../utils/env';

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

const APP_START_TIME = Date.now();
const STARTUP_QUIET_PERIOD_MS = 60_000;

function getStartupDelayMs(): number {
    const elapsed = Date.now() - APP_START_TIME;
    return Math.max(0, STARTUP_QUIET_PERIOD_MS - elapsed);
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
        let initialTimerId: ReturnType<typeof setTimeout> | null = null;
        const { auto_refresh, refresh_interval } = config;
        const quietDelay = getStartupDelayMs();

        // Check if we just turned it on
        if (auto_refresh && !prevAutoRefreshRef.current) {
            if (quietDelay > 0) {
                console.log(`[BackgroundTask] Deferring initial quota refresh for ${Math.round(quietDelay / 1000)}s quiet period...`);
                initialTimerId = setTimeout(() => {
                    refreshAllQuotas();
                }, quietDelay);
            } else {
                console.log('[BackgroundTask] Auto-refresh enabled, executing immediately...');
                refreshAllQuotas();
            }
        }
        prevAutoRefreshRef.current = auto_refresh;

        const effectiveIntervalMinutes = Math.max(5, refresh_interval || 5);
        if (auto_refresh) {
            console.log(`[BackgroundTask] Starting auto-refresh quota timer: ${effectiveIntervalMinutes} mins`);
            intervalId = setInterval(() => {
                console.log('[BackgroundTask] Auto-refreshing all quotas...');
                refreshAllQuotas();
            }, effectiveIntervalMinutes * 60 * 1000);
        }

        return () => {
            if (initialTimerId) {
                clearTimeout(initialTimerId);
            }
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
        let initialTimerId: ReturnType<typeof setTimeout> | null = null;
        const { auto_sync, sync_interval } = config;
        const { syncAccountFromDb } = useAccountStore.getState();
        const quietDelay = getStartupDelayMs();

        // Check if we just turned it on
        if (auto_sync && !prevAutoSyncRef.current) {
            if (quietDelay > 0) {
                console.log(`[BackgroundTask] Deferring initial account sync for ${Math.round(quietDelay / 1000)}s quiet period...`);
                initialTimerId = setTimeout(() => {
                    syncAccountFromDb();
                }, quietDelay);
            } else {
                console.log('[BackgroundTask] Auto-sync enabled, executing immediately...');
                syncAccountFromDb();
            }
        }
        prevAutoSyncRef.current = auto_sync;

        const effectiveSyncMinutes = Math.max(5, sync_interval || 5);
        if (auto_sync) {
            console.log(`[BackgroundTask] Starting auto-sync account timer: ${effectiveSyncMinutes} mins`);
            intervalId = setInterval(() => {
                console.log('[BackgroundTask] Auto-syncing current account from DB...');
                syncAccountFromDb();
            }, effectiveSyncMinutes * 60 * 1000);
        }

        return () => {
            if (initialTimerId) {
                clearTimeout(initialTimerId);
            }
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

        const threshold = switcher.low_quota_threshold_percent ?? 15.0;
        const targetModel = switcher.target_model || 'gemini-2.5-flash';
        const intervalSecs = Math.max(60, switcher.check_interval_seconds || 300);

        const checkAndSmartFastForward = async () => {
            if (isRotatingRef.current) return;
            // In desktop Tauri environment, the native Rust background daemon (start_auto_switcher)
            // runs the authoritative unified single-circle loop and broadcasts account://auto-switched events.
            if (isTauri()) return;

            try {
                const accountStore = useAccountStore.getState();
                const instanceStore = useInstanceStore.getState();

                const instancesToEvaluate = instanceStore.instances.length > 0
                    ? instanceStore.instances
                    : [{ config: { id: 'default', bound_account_id: accountStore.currentAccount?.id } }];

                for (const inst of instancesToEvaluate) {
                    const instId = inst.config.id;
                    const boundAccId = inst.config.bound_account_id || accountStore.currentAccount?.id;
                    let boundAcc = accountStore.accounts.find(a => a.id === boundAccId);
                    if (!boundAcc && accountStore.currentAccount) {
                        boundAcc = accountStore.currentAccount;
                    }
                    if (!boundAcc) continue;

                    const remainingQuota = getAccountRemainingQuota(boundAcc, targetModel);
                    if (remainingQuota <= threshold && accountStore.accounts.length > 1) {
                        const now = Date.now();
                        const lastSwitch = lastSwitchTimeRef.current[instId] || 0;
                        if (now - lastSwitch < 15000) {
                            continue; // Cooldown of 15 seconds per instance
                        }

                        isRotatingRef.current = true;
                        lastSwitchTimeRef.current[instId] = now;
                        console.log(`[AutoSwitcher] Instance ${instId} account ${boundAcc.email} quota ${remainingQuota}% <= ${threshold}%. Triggering Smart Fast-Forward in unified circle...`);

                        const result = await instanceStore.smartRotateProfileAccount(instId);
                        if (result) {
                            showToast(`Smart Fast-Forward: Switched instance ${instId} to ${result.accountEmail} (quota ${remainingQuota}% <= ${threshold}%)`, 'success');
                        }
                    }
                }
            } catch (err) {
                console.error('[AutoSwitcher] Smart Fast-Forward error:', err);
            } finally {
                isRotatingRef.current = false;
            }
        };

        const quietDelay = getStartupDelayMs();
        const initialDelay = quietDelay > 0 ? quietDelay : 500;
        const timerId = setTimeout(() => {
            checkAndSmartFastForward();
        }, initialDelay);

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
