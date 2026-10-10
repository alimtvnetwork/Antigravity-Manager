import type { Account } from '../../types/account';
import { parseFlexibleDate } from '../../utils/format';

export function findBestSmartPlayAccount(
    accounts: Account[],
    targetModel: string = 'gemini-pro'
): Account | null {
    const available = accounts.filter(acc => {
        const isForbidden = Boolean(acc.quota?.is_forbidden);
        if (isForbidden) return false;
        const isDisabled = Boolean(acc.disabled);
        if (isDisabled) return false;
        return true;
    });

    const hasAvailable = available.length > 0;
    if (!hasAvailable) return null;

    const scored = available.map(acc => {
        const res = calculateMultiplicativeScore(acc, [], undefined, targetModel);
        return { account: acc, score: res.score };
    });

    scored.sort((a, b) => b.score - a.score);
    return scored[0]?.account || null;
}

/**
 * Smart Profile Rotation Algorithm
 * Prioritizes profiles used longer ago (e.g. 15 hours ago > 30 minutes ago).
 * Evaluates candidate profiles in batches of 3, picking the best idle profile.
 */

export function findBestRotationProfile(
    instances: InstanceStatus[],
    currentInstanceId?: string
): InstanceStatus | null {
    if (instances.length === 0) return null;

    // Filter candidates: prefer those not currently running
    const nonRunning = instances.filter(i => {
        if (i.is_running) return false;
        if (instances.length > 1 && i.config.id === currentInstanceId) return false;
        return true;
    });

    const pool = nonRunning.length > 0 ? nonRunning : instances.filter(i => i.config.id !== currentInstanceId);
    if (pool.length === 0) return instances[0] || null;

    // Sort by longest delay since last used (ascending timestamp, where 0 or lowest = oldest / longest ago)
    const sorted = [...pool].sort((a, b) => {
        const lastA = a.config.last_used || 0;
        const lastB = b.config.last_used || 0;
        return lastA - lastB;
    });

    // Batched evaluation: step through chunks of 3 and pick the first available
    for (let i = 0; i < sorted.length; i += 3) {
        const batch = sorted.slice(i, i + 3);
        const match = batch.find(item => !item.is_running);
        if (match) {
            return match;
        }
    }

    return sorted[0] || null;
}

export function formatTimeAgo(timestampSec?: number): string {
    if (!timestampSec || timestampSec <= 0) return 'Never used';
    const nowSec = Math.floor(Date.now() / 1000);
    const diffSec = Math.max(0, nowSec - timestampSec);
    if (diffSec < 60) return 'Just now';
    const mins = Math.floor(diffSec / 60);
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
}

export interface MultiplicativeCandidateResult {
    account: Account;
    score: number;
    activeFactor: number;
    tierMultiplier: number;
    weeklyQuotaPercent: number;
    fourHourQuotaPercent: number;
    daysUntilRefill: number;
    quotaPercentage: number;
    idleHours: number;
    isVerified?: boolean;
}

export type SmartCandidateResult = MultiplicativeCandidateResult;

/**
 * Subscription-tier score weights for the best-account selection algorithm.
 * User-configurable via Settings → Auto Profile Switcher → Best-Account Scoring
 * Algorithm; these are the defaults applied when no settings are loaded.
 */

export interface TierMultipliers {
    ultra: number;
    pro: number;
    free: number;
}

export const DEFAULT_TIER_MULTIPLIERS: TierMultipliers = { ultra: 4, pro: 2, free: 1 };

/** Build effective multipliers from the settings object, falling back to defaults per-tier. */

export function tierMultipliersFromSettings(settings?: {
    ultra_tier_multiplier?: number;
    pro_tier_multiplier?: number;
    free_tier_multiplier?: number;
} | null): TierMultipliers {
    return {
        ultra: settings?.ultra_tier_multiplier ?? DEFAULT_TIER_MULTIPLIERS.ultra,
        pro: settings?.pro_tier_multiplier ?? DEFAULT_TIER_MULTIPLIERS.pro,
        free: settings?.free_tier_multiplier ?? DEFAULT_TIER_MULTIPLIERS.free,
    };
}

export function getSubscriptionTierMultiplier(tierName?: string, multipliers: TierMultipliers = DEFAULT_TIER_MULTIPLIERS): number {
    const lower = (tierName || '').toLowerCase();
    const isUltra = lower.includes('ultra');
    if (isUltra) return multipliers.ultra;
    const isPro = lower.includes('pro');
    if (isPro) return multipliers.pro;
    return multipliers.free;
}

export function extractWeeklyQuotaPercent(acc: Account): number {
    const quotaGroups = acc.quota?.quota_groups;
    if (quotaGroups) {
        if (quotaGroups.length > 0) {
            const weeklyValues: number[] = [];
            for (const group of quotaGroups) {
                const buckets = group.buckets || [];
                for (const b of buckets) {
                    const win = (b.window || '').toLowerCase();
                    const id = (b.bucket_id || '').toLowerCase();
                    const isWeekly = win.includes('week') || id.includes('week');
                    if (isWeekly) {
                        if (typeof b.remaining_fraction === 'number') {
                            weeklyValues.push(Math.round(b.remaining_fraction * 100));
                        }
                    }
                }
            }
            const hasWeekly = weeklyValues.length > 0;
            if (hasWeekly) {
                // Bottleneck rule: take the minimum weekly remaining quota across groups
                return Math.min(...weeklyValues);
            }
        }
    }

    const models = acc.quota?.models || [];
    const validModels = models.filter(m => {
        return typeof m.percentage === 'number';
    });

    const hasValid = validModels.length > 0;
    if (!hasValid) return 100;

    let total = 0;
    for (const m of validModels) {
        total += m.percentage;
    }
    return Math.round(total / validModels.length);
}

/**
 * Specifically extract the 4-hour / 5-hour rolling window quota percentage (0-100).
 * Accounts MUST have 100% in this window to be prioritized for fast-forward rotation.
 */

export function extract4hWindowQuotaPercent(acc: Account, _targetModel?: string): number {
    if (!acc.quota) return 0;
    if (acc.quota.is_forbidden) return 0;
    if (acc.disabled || acc.proxy_disabled || acc.validation_blocked) return 0;

    const minValues: number[] = [];

    // 1. Check all short-window buckets from quota_groups
    const quotaGroups = acc.quota.quota_groups;
    if (quotaGroups && quotaGroups.length > 0) {
        for (const group of quotaGroups) {
            const buckets = group.buckets || [];
            for (const b of buckets) {
                const win = (b.window || '').toLowerCase();
                const id = (b.bucket_id || '').toLowerCase();
                const isShort =
                    win.includes('4h') ||
                    win.includes('5h') ||
                    id.includes('4h') ||
                    id.includes('5h') ||
                    (!win.includes('week') && !win.includes('7d') && !id.includes('week') && !id.includes('7d'));
                if (isShort && typeof b.remaining_fraction === 'number') {
                    minValues.push(Math.round(b.remaining_fraction * 100));
                }
            }
        }
    }

    // 2. Check all active non-banned models (ignore 3.0, 3.1)
    const models = acc.quota.models || [];
    if (models.length > 0) {
        for (const m of models) {
            const name = (m.name || '').toLowerCase();
            if (name.includes('3.0') || name.includes('3.1')) continue;
            if (typeof m.percentage === 'number') {
                minValues.push(m.percentage);
            }
        }
    }

    if (minValues.length > 0) {
        return Math.min(...minValues);
    }

    return 0;
}

function calculateAccountRefillDays(acc: Account, nowMs: number): number {
    let maxDays = 0;
    const models = acc.quota?.models || [];
    for (const m of models) {
        if (!m.reset_time) continue;
        const resetDate = parseFlexibleDate(m.reset_time);
        if (!resetDate) continue;
        const diffMs = resetDate.getTime() - nowMs;
        if (diffMs > 0) {
            const days = diffMs / 86400000;
            if (days > maxDays) maxDays = days;
        }
    }
    return Math.round(maxDays * 10) / 10;
}

/**
 * Normalized Multiplicative Candidate Scoring Algorithm (divided by 1000 for minimal compact numbers):
 * - Anyone with < 100% 4h quota gets a score of `0`.
 * - Otherwise: Score = (S_active * M_tier * Q_weekly) / 1000
 *   where S_active = 1 if unused (0 if in use), M_tier comes from Settings → Algorithm
 *   (defaults: Ultra 4, Pro 2, Free 1), Q_weekly = 0..100.
 *   Example scores (defaults): Ultra 100% = 0.4, Pro 100% = 0.2, Free 100% = 0.1, < 100% 4h quota = 0.
 */

export function calculateMultiplicativeScore(
    acc: Account,
    activeInUseAccountIds: string[] = [],
    currentAccountId?: string,
    targetModel?: string,
    tierMultipliers: TierMultipliers = DEFAULT_TIER_MULTIPLIERS
): MultiplicativeCandidateResult {
    const isInUse = activeInUseAccountIds.includes(acc.id);
    const isCurrent = Boolean(currentAccountId && acc.id === currentAccountId);
    const activeFactor = isInUse || isCurrent ? 0 : 1;

    const tierMultiplier = getSubscriptionTierMultiplier(acc.quota?.subscription_tier, tierMultipliers);
    const weeklyQuotaPercent = extractWeeklyQuotaPercent(acc);
    const fourHourQuotaPercent = extract4hWindowQuotaPercent(acc, targetModel);

    const nowMs = Date.now();
    const hasResetTimePassed = (acc.quota?.models || []).some(m => {
        if (!m.reset_time) return false;
        const rDate = parseFlexibleDate(m.reset_time);
        return rDate ? rDate.getTime() <= nowMs : false;
    });

    const isFull4h = fourHourQuotaPercent >= 100 || hasResetTimePassed;
    if (!isFull4h || activeFactor === 0) {
        return {
            account: acc,
            score: 0,
            activeFactor: 0,
            tierMultiplier,
            weeklyQuotaPercent,
            fourHourQuotaPercent,
            daysUntilRefill: 0,
            quotaPercentage: 0,
            idleHours: 999,
        };
    }

    const rawScore = activeFactor * tierMultiplier * weeklyQuotaPercent;
    const score = Number((rawScore / 1000).toFixed(4));

    const daysUntilRefill = calculateAccountRefillDays(acc, nowMs);
    const nowSec = Math.floor(nowMs / 1000);
    const idleSec = acc.last_used ? Math.max(0, nowSec - acc.last_used) : 0;
    const idleHours = acc.last_used ? Math.round((idleSec / 3600) * 10) / 10 : 999;

    return {
        account: acc,
        score,
        activeFactor,
        tierMultiplier,
        weeklyQuotaPercent,
        fourHourQuotaPercent,
        daysUntilRefill,
        quotaPercentage: fourHourQuotaPercent,
        idleHours,
    };
}

export function rankSmartCandidates(
    accounts: Account[],
    activeInUseAccountIds: string[] = [],
    currentAccountId?: string,
    tierMultipliers: TierMultipliers = DEFAULT_TIER_MULTIPLIERS
): MultiplicativeCandidateResult[] {
    const eligible = accounts.filter(acc => {
        // Never allow the current account or active in-use accounts to be considered as candidates to rotate into
        if (currentAccountId && acc.id === currentAccountId) return false;
        if (activeInUseAccountIds.includes(acc.id)) return false;

        const isDisabled = Boolean(acc.disabled) || Boolean(acc.proxy_disabled);
        if (isDisabled) return false;
        const isForbidden = Boolean(acc.quota?.is_forbidden);
        if (isForbidden) return false;
        const isBlocked = Boolean(acc.validation_blocked);
        if (isBlocked) return false;

        const q4h = extract4hWindowQuotaPercent(acc);
        const nowMs = Date.now();
        const hasResetTimePassed = (acc.quota?.models || []).some(m => {
            if (!m.reset_time) return false;
            const rDate = parseFlexibleDate(m.reset_time);
            return rDate ? rDate.getTime() <= nowMs : false;
        });
        if (q4h < 100 && !hasResetTimePassed) return false;
        return true;
    });

    const hasEligible = eligible.length > 0;
    if (!hasEligible) return [];

    const scored = eligible
        .map(acc => {
            return calculateMultiplicativeScore(
                acc,
                activeInUseAccountIds,
                currentAccountId,
                undefined,
                tierMultipliers
            );
        })
        .filter(item => item.score > 0 && item.activeFactor > 0);

    scored.sort((a, b) => {
        // 1. Highest normalized score first (M_tier * Q_weekly / 1000)
        if (b.score !== a.score) {
            return b.score - a.score;
        }

        // 2. 4-hour quota comparison
        if (b.fourHourQuotaPercent !== a.fourHourQuotaPercent) {
            return b.fourHourQuotaPercent - a.fourHourQuotaPercent;
        }

        // 3. Weekly quota comparison
        if (b.weeklyQuotaPercent !== a.weeklyQuotaPercent) {
            return b.weeklyQuotaPercent - a.weeklyQuotaPercent;
        }

        // 4. Deterministic tie-breaker
        const emailA = (a.account.email || '').toLowerCase();
        const emailB = (b.account.email || '').toLowerCase();
        return emailA.localeCompare(emailB);
    });

    return scored;
}

export function findSmartRotationAccount(
    accounts: Account[],
    currentAccountId?: string,
    activeInUseAccountIds: string[] = [],
    tierMultipliers: TierMultipliers = DEFAULT_TIER_MULTIPLIERS
): MultiplicativeCandidateResult | null {
    const ranked = rankSmartCandidates(
        accounts,
        activeInUseAccountIds,
        currentAccountId,
        tierMultipliers
    );
    const hasRanked = ranked.length > 0;
    if (!hasRanked) return null;
    return ranked[0];
}

export function pickBestCandidateAccount(
    accounts: Account[],
    activeInUseAccountIds: string[] = [],
    currentAccountId?: string,
    tierMultipliers: TierMultipliers = DEFAULT_TIER_MULTIPLIERS
): Account | null {
    const smart = findSmartRotationAccount(accounts, currentAccountId, activeInUseAccountIds, tierMultipliers);
    if (smart?.account && (smart.fourHourQuotaPercent ?? 0) >= 100) {
        return smart.account;
    }
    return null;
}

export const selectNextBestProfile = findBestRotationProfile;
