import { useMemo } from 'react';
import type { Account } from '../../types/account';
import { findQuotaModel, findImageQuotaModel } from '../../utils/modelCategory';

interface DashboardStats {
    abnormal: number;
    normalCount: number;
    basePoolCount: number;
    gemini: { avg: number | null; count: number };
    geminiImage: { avg: number | null; count: number };
    claude: { avg: number | null; count: number };
}

/**
 * Computes dashboard statistics from accounts.
 * Extracted from Dashboard to keep the component under the 500-line limit.
 */
export function useDashboardStats(accounts: Account[], onlyAvailable: boolean): DashboardStats {
    return useMemo(() => {
// 1. Account ecosystem health status classification (risk control & enabled state determine viability, not quota)
// Abnormal accounts: triggered Google captcha/risk control block, or quota API returned 403 Forbidden
const abnormalAccounts = accounts.filter(
    a => a.validation_blocked || a.quota?.is_forbidden
);
// Disabled accounts: not abnormal, but user manually disabled proxy or disabled account
const disabledAccounts = accounts.filter(
    a => !a.validation_blocked && !a.quota?.is_forbidden && (a.disabled || a.proxy_disabled)
);
// Available accounts: enabled, healthy state, risk-control clear production accounts
const availableAccounts = accounts.filter(
    a => !a.disabled && !a.proxy_disabled && !a.validation_blocked && !a.quota?.is_forbidden
);
// All normal status accounts (includes disabled and non-disabled, strictly excluding risk-control abnormal accounts)
const normalAccounts = accounts.filter(
    a => !a.validation_blocked && !a.quota?.is_forbidden
);

// Select base statistics pool based on button state: default only available accounts; if disabled, view all normal accounts
const basePool = onlyAvailable
    ? (availableAccounts.length > 0 ? availableAccounts : normalAccounts)
    : normalAccounts;

// 2. Single account quota extraction helper functions
const get5hQuota = (a: Account, modelKey: 'gemini-pro' | 'gemini-image' | 'claude'): number | null => {
    if (modelKey === 'gemini-image') {
        return findImageQuotaModel(a.quota?.models)?.percentage ?? null;
    }
    return findQuotaModel(a.quota?.models, modelKey)?.percentage ?? null;
};

const getWeeklyQuota = (a: Account, modelKey: 'gemini-pro' | 'gemini-image' | 'claude'): number | null => {
    const isClaude = modelKey === 'claude';
    if (a.quota?.quota_groups) {
        for (const group of a.quota.quota_groups) {
            const gname = group.display_name.toLowerCase();
            const matches = isClaude
                ? (gname.includes('claude') || gname.includes('gpt') || gname.includes('3p'))
                : (gname.includes('gemini') || (!gname.includes('claude') && !gname.includes('gpt') && !gname.includes('3p')));
            if (matches) {
                const weekly = group.buckets?.find(b =>
                    b.window?.toLowerCase().includes('week') ||
                    b.bucket_id?.toLowerCase().includes('week') ||
                    b.window?.toLowerCase().includes('7d')
                );
                if (weekly && typeof weekly.remaining_fraction === 'number') {
                    return Math.round(weekly.remaining_fraction * 100);
                }
            }
        }
    }
    return null;
};

// Compute combined metrics for target base pool under specified model (5h average, weekly quota average, weekly quota circuit-breaker weighting)
const computeMetrics = (modelKey: 'gemini-pro' | 'gemini-image' | 'claude') => {
    const pool = basePool;
    if (pool.length === 0) {
        return { avg5h: 0, avgWeekly: 0, weightedEffective: 0, zeroWeeklyCount: 0 };
    }

    let sum5h = 0, count5h = 0;
    let sumWeekly = 0, countWeekly = 0;
    let sumWeighted = 0, totalWeight = 0;
    let zeroWeeklyCount = 0;

    for (const a of pool) {
        const q5h = get5hQuota(a, modelKey);
        const qWeekly = getWeeklyQuota(a, modelKey);

        if (q5h !== null && q5h >= 0) {
            sum5h += q5h;
            count5h++;
        }

        if (qWeekly !== null && qWeekly >= 0) {
            sumWeekly += qWeekly;
            countWeekly++;
            if (qWeekly <= 0) {
                zeroWeeklyCount++;
            }
        }

        // Core bottleneck special handling: when weekly quota reaches 0%, upstream will reject, effective available quota circuit breaks to 0!
        let effectiveVal = q5h ?? 0;
        if (qWeekly !== null && qWeekly <= 0) {
            effectiveVal = 0;
        }

        // Tier weighting (ULTRA 2.0, PRO 1.5, FREE 1.0)
        const tier = (a.quota?.subscription_tier || '').toUpperCase();
        const weight = tier.includes('ULTRA') ? 2.0 : tier.includes('PRO') ? 1.5 : 1.0;
        sumWeighted += effectiveVal * weight;
        totalWeight += weight;
    }

    return {
        avg5h: count5h > 0 ? Math.round(sum5h / count5h) : 0,
        avgWeekly: countWeekly > 0 ? Math.round(sumWeekly / countWeekly) : 0,
        weightedEffective: totalWeight > 0 ? Math.round(sumWeighted / totalWeight) : 0,
        zeroWeeklyCount,
    };
};

const gemini = computeMetrics('gemini-pro');
const geminiImage = computeMetrics('gemini-image');
const claude = computeMetrics('claude');

return {
    total: accounts.length,
    available: availableAccounts.length,
    disabled: disabledAccounts.length,
    abnormal: abnormalAccounts.length,
    normalCount: normalAccounts.length,
    basePoolCount: basePool.length,
    gemini,
    geminiImage,
    claude,
};
    }, [accounts, onlyAvailable]);
}
