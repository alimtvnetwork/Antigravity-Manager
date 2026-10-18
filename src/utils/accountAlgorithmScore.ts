import type { Account, LiveLimitStatus } from '../types/account';
import {
    extractWeeklyQuotaPercent,
    extract4hWindowQuotaPercent,
    getSubscriptionTierMultiplier,
    DEFAULT_TIER_MULTIPLIERS,
} from '../services/instanceService/scoring';
import type { TierMultipliers } from '../services/instanceService/scoring';
import { parseFlexibleDate } from './format';
import { formatCompactDuration } from './liveLimit';

export interface AccountColorScale {
    text: string; // e.g. 'text-emerald-500 dark:text-emerald-400'
    bg: string; // e.g. 'bg-emerald-500'
    gradient: string; // e.g. 'from-emerald-500 to-teal-400'
    border: string; // e.g. 'border-emerald-500/30 dark:border-emerald-400/30'
}

export interface AccountMathBreakdown {
    formulaString: string; // 'Score = (S_active × M_tier × Q_weekly) / 1000'
    activeTerm: string; // 'S_active = 1 (Idle & Ready)' or 'S_active = 0 (In Use)'
    tierTerm: string; // 'M_tier = 2.0 (Pro Tier)'
    quotaTerm: string; // 'Q_weekly = 85%'
    fourHourTerm: string; // 'Q_4h = 100% (Pass 1 Primary)' or 'Q_4h = 60% (Sub-100% Fallback)'
    multiplicationStep: string; // '1 × 2.0 × 85 = 170.0'
    divisionStep: string; // '170.0 ÷ 1000 = 0.1700'
    percentageStep: string; // '(0.1700 ÷ 0.2000) × 100% = 85.0%'
    cooldownSummary: string; // 'Idle 5.2h (No Cooldown)'
}

export interface AccountAlgorithmScoreResult {
    score: number; // normalized score, e.g. 0.2000
    percentage: number; // 0 - 100 percentage for progress bar
    rawScore: number; // raw points, e.g. 200
    tier: 'ultra' | 'pro' | 'free';
    tierMultiplier: number;
    activeFactor: number; // 1 or 0
    weeklyQuotaPercent: number; // bottleneck weekly %
    fourHourQuotaPercent: number; // 4h window %
    isFull4h: boolean;
    isFallback: boolean;
    disqualificationReason: string | null; // e.g. 'In Use', 'Disabled', 'Forbidden (403)', 'Rate Limited (429)', or null
    colorScale: AccountColorScale;
    mathBreakdown: AccountMathBreakdown;
}

export interface ComputeAccountScoreOptions {
    inUseAccountIds?: string[];
    currentAccountId?: string;
    tierMultipliers?: TierMultipliers;
    targetModel?: string;
    nowMs?: number;
}

export function resolveTier(tierName?: string): 'ultra' | 'pro' | 'free' {
    const lower = (tierName || '').toLowerCase();
    if (lower.includes('ultra')) return 'ultra';
    if (lower.includes('pro')) return 'pro';
    return 'free';
}

export function getColorScale(percentage: number): AccountColorScale {
    if (percentage >= 70) {
        return {
            text: 'text-emerald-500 dark:text-emerald-400',
            bg: 'bg-emerald-500',
            gradient: 'from-emerald-500 to-teal-400',
            border: 'border-emerald-500/30 dark:border-emerald-400/30',
        };
    }
    if (percentage >= 40) {
        return {
            text: 'text-amber-500 dark:text-amber-400',
            bg: 'bg-amber-500',
            gradient: 'from-amber-500 to-yellow-400',
            border: 'border-amber-500/30 dark:border-amber-400/30',
        };
    }
    return {
        text: 'text-rose-500 dark:text-rose-400',
        bg: 'bg-rose-500',
        gradient: 'from-rose-600 to-red-500',
        border: 'border-rose-500/30 dark:border-rose-400/30',
    };
}

function findActiveRateLimit(
    account: Account,
    nowSec: number,
    targetModel?: string
): LiveLimitStatus | null {
    if (!account.live_limited_models) return null;

    if (targetModel && account.live_limited_models[targetModel]) {
        const item = account.live_limited_models[targetModel];
        if (item && item.status === 429 && item.until > nowSec) {
            return item;
        }
    }

    for (const item of Object.values(account.live_limited_models)) {
        if (item && item.status === 429 && item.until > nowSec) {
            return item;
        }
    }
    return null;
}

/**
 * Computes the unified algorithm score and rich mathematical breakdown
 * for an account candidate for visualization, progress bars, and tooltips.
 */
export function computeAccountAlgorithmScore(
    account: Account,
    options?: ComputeAccountScoreOptions
): AccountAlgorithmScoreResult {
    const nowMs = options?.nowMs ?? Date.now();
    const nowSec = Math.floor(nowMs / 1000);

    const isInUse = Boolean(options?.inUseAccountIds && options.inUseAccountIds.includes(account.id));
    const isCurrent = Boolean(options?.currentAccountId && options.currentAccountId === account.id);
    const isDisabled = Boolean(account.disabled) || Boolean(account.proxy_disabled);
    const isForbidden = Boolean(account.quota?.is_forbidden);
    const isBlocked = Boolean(account.validation_blocked);
    const activeRateLimit = findActiveRateLimit(account, nowSec, options?.targetModel);

    let disqualificationReason: string | null = null;
    if (isForbidden) {
        disqualificationReason = 'Forbidden (403)';
    } else if (isDisabled) {
        disqualificationReason = 'Disabled';
    } else if (isBlocked) {
        disqualificationReason = 'Validation Blocked';
    } else if (activeRateLimit) {
        disqualificationReason = 'Rate Limited (429)';
    } else if (isInUse || isCurrent) {
        disqualificationReason = 'In Use';
    }

    const activeFactor = disqualificationReason ? 0 : 1;
    const tierMultipliers = options?.tierMultipliers ?? DEFAULT_TIER_MULTIPLIERS;
    const tier = resolveTier(account.quota?.subscription_tier);
    const tierMultiplier = getSubscriptionTierMultiplier(account.quota?.subscription_tier, tierMultipliers);

    const weeklyQuotaPercent = extractWeeklyQuotaPercent(account);
    const fourHourQuotaPercent = extract4hWindowQuotaPercent(account, options?.targetModel);

    const hasResetTimePassed = (account.quota?.models || []).some(m => {
        if (!m.reset_time) return false;
        const rDate = parseFlexibleDate(m.reset_time);
        return rDate ? rDate.getTime() <= nowMs : false;
    });

    const isFull4h = fourHourQuotaPercent >= 100 || hasResetTimePassed;
    const isFallback = !isFull4h;

    const effectiveFourHour = hasResetTimePassed ? 100 : fourHourQuotaPercent;
    let rawScore = 0;
    let score = 0;
    let percentage = 0;
    const maxTierScore = (tierMultiplier * 100) / 1000;

    let propQuotaVal = weeklyQuotaPercent;

    if (activeFactor === 1) {
        if (!isFallback) {
            // Pass 1: Primary scoring
            rawScore = activeFactor * tierMultiplier * weeklyQuotaPercent;
            score = Number((rawScore / 1000).toFixed(4));
            percentage = maxTierScore > 0 ? Number(((score / maxTierScore) * 100).toFixed(1)) : 0;
            percentage = Math.min(100, Math.max(0, percentage));
        } else {
            // Pass 2: Sub-100% Proportional fallback scoring
            if (effectiveFourHour > 0 && weeklyQuotaPercent >= 8) {
                const proportionalQuota = (weeklyQuotaPercent / 100) * (effectiveFourHour / 100) * 100;
                propQuotaVal = Number(proportionalQuota.toFixed(1));
                rawScore = activeFactor * tierMultiplier * proportionalQuota;
                score = Number((rawScore / 1000).toFixed(4));
                percentage = maxTierScore > 0 ? Number(((score / maxTierScore) * 100).toFixed(1)) : 0;
                percentage = Math.min(100, Math.max(0, percentage));
            } else {
                propQuotaVal = (weeklyQuotaPercent * effectiveFourHour) / 100;
                rawScore = 0;
                score = 0;
                percentage = 0;
            }
        }
    } else {
        // Disqualified account
        rawScore = 0;
        score = 0;
        percentage = 0;
        if (isFallback) {
            propQuotaVal = (weeklyQuotaPercent * effectiveFourHour) / 100;
        }
    }

    const formulaString = isFallback
        ? 'Score = (S_active × M_tier × (Q_weekly × Q_4h / 100)) / 1000'
        : 'Score = (S_active × M_tier × Q_weekly) / 1000';

    const activeTerm = activeFactor === 1
        ? 'S_active = 1 (Idle & Ready)'
        : `S_active = 0 (${disqualificationReason || 'In Use'})`;

    const tierTitle = tier === 'ultra' ? 'Ultra' : tier === 'pro' ? 'Pro' : 'Free';
    const tierTerm = `M_tier = ${tierMultiplier.toFixed(1)} (${tierTitle} Tier)`;

    const quotaTerm = `Q_weekly = ${weeklyQuotaPercent}%`;

    const fourHourTerm = !isFallback
        ? `Q_4h = ${fourHourQuotaPercent}% (Pass 1 Primary)`
        : `Q_4h = ${fourHourQuotaPercent}% (Sub-100% Fallback)`;

    const multStepOperand = !isFallback
        ? weeklyQuotaPercent
        : (Number.isInteger(propQuotaVal) ? propQuotaVal : propQuotaVal.toFixed(1));

    const multiplicationStep = `${activeFactor} × ${tierMultiplier.toFixed(1)} × ${multStepOperand} = ${rawScore.toFixed(1)}`;

    const divisionStep = `${rawScore.toFixed(1)} ÷ 1000 = ${score.toFixed(4)}`;

    const percentageStep = maxTierScore > 0
        ? `(${score.toFixed(4)} ÷ ${maxTierScore.toFixed(4)}) × 100% = ${percentage.toFixed(1)}%`
        : `(0.0000 ÷ 1.0000) × 100% = 0.0%`;

    // Idle & Cooldown calculations
    const idleSec = account.last_used && account.last_used > 0
        ? Math.max(0, nowSec - account.last_used)
        : null;
    const idleStr = idleSec !== null
        ? `Idle ${(idleSec / 3600).toFixed(1)}h`
        : 'Never used';

    let cooldownSummary: string;
    if (activeRateLimit) {
        const remainingSec = Math.max(0, activeRateLimit.until - nowSec);
        cooldownSummary = `${idleStr} (Cooldown: ${formatCompactDuration(remainingSec)})`;
    } else if (account.validation_blocked && account.validation_blocked_until && account.validation_blocked_until > nowSec) {
        const remainingSec = Math.max(0, account.validation_blocked_until - nowSec);
        cooldownSummary = `${idleStr} (Blocked: ${formatCompactDuration(remainingSec)})`;
    } else {
        cooldownSummary = `${idleStr} (No Cooldown)`;
    }

    const colorScale = getColorScale(percentage);

    return {
        score,
        percentage,
        rawScore: Number(rawScore.toFixed(1)),
        tier,
        tierMultiplier,
        activeFactor,
        weeklyQuotaPercent,
        fourHourQuotaPercent,
        isFull4h,
        isFallback,
        disqualificationReason,
        colorScale,
        mathBreakdown: {
            formulaString,
            activeTerm,
            tierTerm,
            quotaTerm,
            fourHourTerm,
            multiplicationStep,
            divisionStep,
            percentageStep,
            cooldownSummary,
        },
    };
}

/**
 * Compares two scored account results descending (highest score first, then 4h quota, then weekly quota, then deterministic email).
 */
export function compareAccountAlgorithmScores(
    a: AccountAlgorithmScoreResult,
    b: AccountAlgorithmScoreResult,
    accountA?: Account,
    accountB?: Account
): number {
    if (b.score !== a.score) {
        return b.score - a.score;
    }
    if (b.fourHourQuotaPercent !== a.fourHourQuotaPercent) {
        return b.fourHourQuotaPercent - a.fourHourQuotaPercent;
    }
    if (b.weeklyQuotaPercent !== a.weeklyQuotaPercent) {
        return b.weeklyQuotaPercent - a.weeklyQuotaPercent;
    }
    const emailA = (accountA?.email || '').toLowerCase();
    const emailB = (accountB?.email || '').toLowerCase();
    return emailA.localeCompare(emailB);
}
