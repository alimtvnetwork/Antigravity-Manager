import {
    computeAccountAlgorithmScore,
    getColorScale,
    resolveTier,
    compareAccountAlgorithmScores,
} from './accountAlgorithmScore';
import type { Account, LiveLimitStatus, QuotaData } from '../types/account';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
    try {
        fn();
        passed++;
        console.log(`  PASS: ${description}`);
    } catch (e: unknown) {
        failed++;
        const msg = e instanceof Error ? e.message : String(e);
        console.error(`  FAIL: ${description} — ${msg}`);
    }
}

function assertEqual<T>(actual: T, expected: T, msg?: string): void {
    if (actual !== expected) {
        throw new Error(`${msg ? msg + ': ' : ''}expected "${expected}", got "${actual}"`);
    }
}

function assert(condition: boolean, msg?: string): void {
    if (!condition) {
        throw new Error(msg || 'Assertion failed');
    }
}

function createMockAccount(params: {
    id?: string;
    email?: string;
    tier?: string;
    weeklyQuota?: number;
    fourHourQuota?: number;
    disabled?: boolean;
    proxy_disabled?: boolean;
    is_forbidden?: boolean;
    validation_blocked?: boolean;
    validation_blocked_until?: number;
    live_limited_models?: Record<string, LiveLimitStatus>;
    last_used?: number;
    reset_time?: string;
}): Account {
    const weeklyPct = params.weeklyQuota ?? 100;
    const fourHourPct = params.fourHourQuota ?? 100;
    const resetTime = params.reset_time || '2099-01-01T00:00:00Z';

    const quota: QuotaData = {
        models: [
            { name: 'gemini-2.5-pro', percentage: fourHourPct, reset_time: resetTime },
        ],
        last_updated: Date.now(),
        subscription_tier: params.tier ?? 'PRO',
        is_forbidden: params.is_forbidden,
        quota_groups: [
            {
                display_name: 'Gemini Models',
                buckets: [
                    {
                        bucket_id: 'gemini-5h',
                        window: '5h',
                        remaining_fraction: fourHourPct / 100,
                        reset_time: resetTime,
                    },
                    {
                        bucket_id: 'gemini-weekly',
                        window: 'weekly',
                        remaining_fraction: weeklyPct / 100,
                        reset_time: resetTime,
                    },
                ],
            },
        ],
    };

    return {
        id: params.id ?? 'acc-001',
        email: params.email ?? 'dev@example.com',
        token: {
            access_token: 'synthetic-token',
            refresh_token: 'synthetic-refresh',
            expires_in: 3600,
            expiry_timestamp: Date.now() + 3600000,
            token_type: 'Bearer',
        },
        quota,
        disabled: params.disabled,
        proxy_disabled: params.proxy_disabled,
        validation_blocked: params.validation_blocked,
        validation_blocked_until: params.validation_blocked_until,
        live_limited_models: params.live_limited_models,
        created_at: Math.floor(Date.now() / 1000) - 86400,
        last_used: params.last_used ?? Math.floor(Date.now() / 1000) - 18720, // 5.2h ago
    };
}

console.log('--- Running Account Algorithm Score Tests ---');

// Test 1: Pro Tier primary score calculation with 85% weekly and 100% 4h quota
test('Pro Tier primary score calculation (Weekly 85%, 4h 100%)', () => {
    const nowMs = 1700000000000;
    const lastUsed = Math.floor(nowMs / 1000) - Math.round(5.2 * 3600);
    const account = createMockAccount({
        tier: 'PRO',
        weeklyQuota: 85,
        fourHourQuota: 100,
        last_used: lastUsed,
    });

    const result = computeAccountAlgorithmScore(account, { nowMs });

    assertEqual(result.tier, 'pro');
    assertEqual(result.tierMultiplier, 2);
    assertEqual(result.activeFactor, 1);
    assertEqual(result.weeklyQuotaPercent, 85);
    assertEqual(result.fourHourQuotaPercent, 100);
    assertEqual(result.isFull4h, true);
    assertEqual(result.isFallback, false);
    assertEqual(result.disqualificationReason, null);
    assertEqual(result.rawScore, 170);
    assertEqual(result.score, 0.17);
    assertEqual(result.percentage, 85.0);

    // Color scaling: >= 70% should be emerald
    assert(result.colorScale.text.includes('emerald-500'));
    assert(result.colorScale.gradient.includes('from-emerald-500 to-teal-400'));

    // Math breakdown verification
    assertEqual(result.mathBreakdown.formulaString, 'Score = (S_active × M_tier × Q_weekly) / 1000');
    assertEqual(result.mathBreakdown.activeTerm, 'S_active = 1 (Idle & Ready)');
    assertEqual(result.mathBreakdown.tierTerm, 'M_tier = 2.0 (Pro Tier)');
    assertEqual(result.mathBreakdown.quotaTerm, 'Q_weekly = 85%');
    assertEqual(result.mathBreakdown.fourHourTerm, 'Q_4h = 100% (Pass 1 Primary)');
    assertEqual(result.mathBreakdown.multiplicationStep, '1 × 2.0 × 85 = 170.0');
    assertEqual(result.mathBreakdown.divisionStep, '170.0 ÷ 1000 = 0.1700');
    assertEqual(result.mathBreakdown.percentageStep, '(0.1700 ÷ 0.2000) × 100% = 85.0%');
    assertEqual(result.mathBreakdown.cooldownSummary, 'Idle 5.2h (No Cooldown)');
});

// Test 2: Ultra Tier primary calculation (100% weekly, 100% 4h)
test('Ultra Tier primary score calculation (Weekly 100%, 4h 100%)', () => {
    const nowMs = 1700000000000;
    const account = createMockAccount({
        tier: 'ULTRA',
        weeklyQuota: 100,
        fourHourQuota: 100,
    });

    const result = computeAccountAlgorithmScore(account, { nowMs });

    assertEqual(result.tier, 'ultra');
    assertEqual(result.tierMultiplier, 4);
    assertEqual(result.rawScore, 400);
    assertEqual(result.score, 0.4);
    assertEqual(result.percentage, 100.0);
    assertEqual(result.mathBreakdown.tierTerm, 'M_tier = 4.0 (Ultra Tier)');
    assertEqual(result.mathBreakdown.multiplicationStep, '1 × 4.0 × 100 = 400.0');
    assertEqual(result.mathBreakdown.divisionStep, '400.0 ÷ 1000 = 0.4000');
    assertEqual(result.mathBreakdown.percentageStep, '(0.4000 ÷ 0.4000) × 100% = 100.0%');
});

// Test 3: Free Tier calculation (60% weekly, 100% 4h)
test('Free Tier primary score calculation (Weekly 60%, 4h 100%)', () => {
    const nowMs = 1700000000000;
    const account = createMockAccount({
        tier: 'FREE',
        weeklyQuota: 60,
        fourHourQuota: 100,
    });

    const result = computeAccountAlgorithmScore(account, { nowMs });

    assertEqual(result.tier, 'free');
    assertEqual(result.tierMultiplier, 1);
    assertEqual(result.rawScore, 60);
    assertEqual(result.score, 0.06);
    assertEqual(result.percentage, 60.0);
    // Color scaling 40% - 69%: amber
    assert(result.colorScale.text.includes('amber-500'));
    assert(result.colorScale.gradient.includes('from-amber-500 to-yellow-400'));
    assertEqual(result.mathBreakdown.tierTerm, 'M_tier = 1.0 (Free Tier)');
    assertEqual(result.mathBreakdown.percentageStep, '(0.0600 ÷ 0.1000) × 100% = 60.0%');
});

// Test 4: In-use disqualification scenario
test('In-use account disqualification', () => {
    const account = createMockAccount({
        id: 'acc-in-use',
        tier: 'PRO',
        weeklyQuota: 85,
        fourHourQuota: 100,
    });

    const result = computeAccountAlgorithmScore(account, {
        inUseAccountIds: ['acc-in-use'],
    });

    assertEqual(result.activeFactor, 0);
    assertEqual(result.disqualificationReason, 'In Use');
    assertEqual(result.score, 0);
    assertEqual(result.rawScore, 0);
    assertEqual(result.percentage, 0);
    // Color scaling < 40%: rose
    assert(result.colorScale.text.includes('rose-500'));
    assertEqual(result.mathBreakdown.activeTerm, 'S_active = 0 (In Use)');
    assertEqual(result.mathBreakdown.multiplicationStep, '0 × 2.0 × 85 = 0.0');
    assertEqual(result.mathBreakdown.divisionStep, '0.0 ÷ 1000 = 0.0000');
    assertEqual(result.mathBreakdown.percentageStep, '(0.0000 ÷ 0.2000) × 100% = 0.0%');
});

// Test 5: Current active profile disqualification
test('Current active profile disqualification', () => {
    const account = createMockAccount({
        id: 'acc-current',
        tier: 'PRO',
    });

    const result = computeAccountAlgorithmScore(account, {
        currentAccountId: 'acc-current',
    });

    assertEqual(result.activeFactor, 0);
    assertEqual(result.disqualificationReason, 'In Use');
    assertEqual(result.score, 0);
});

// Test 6: Disabled account disqualification
test('Disabled account disqualification', () => {
    const account = createMockAccount({
        tier: 'PRO',
        disabled: true,
    });

    const result = computeAccountAlgorithmScore(account);

    assertEqual(result.activeFactor, 0);
    assertEqual(result.disqualificationReason, 'Disabled');
    assertEqual(result.score, 0);
    assertEqual(result.mathBreakdown.activeTerm, 'S_active = 0 (Disabled)');
});

// Test 7: Forbidden (403) account disqualification
test('Forbidden (403) account disqualification', () => {
    const account = createMockAccount({
        tier: 'PRO',
        is_forbidden: true,
    });

    const result = computeAccountAlgorithmScore(account);

    assertEqual(result.activeFactor, 0);
    assertEqual(result.disqualificationReason, 'Forbidden (403)');
    assertEqual(result.score, 0);
    assertEqual(result.mathBreakdown.activeTerm, 'S_active = 0 (Forbidden (403))');
});

// Test 8: Rate Limited (429) account disqualification & cooldown timer
test('Rate limited (429) account disqualification and cooldown', () => {
    const nowMs = 1700000000000;
    const nowSec = Math.floor(nowMs / 1000);
    const account = createMockAccount({
        tier: 'PRO',
        last_used: nowSec - 3600, // 1.0h ago
        live_limited_models: {
            'gemini-2.5-pro': {
                model: 'gemini-2.5-pro',
                status: 429,
                reason: 'Rate limit exceeded',
                until: nowSec + 300, // 5m remaining
                detected_at: nowSec - 60,
            },
        },
    });

    const result = computeAccountAlgorithmScore(account, { nowMs });

    assertEqual(result.activeFactor, 0);
    assertEqual(result.disqualificationReason, 'Rate Limited (429)');
    assertEqual(result.score, 0);
    assertEqual(result.mathBreakdown.activeTerm, 'S_active = 0 (Rate Limited (429))');
    assertEqual(result.mathBreakdown.cooldownSummary, 'Idle 1.0h (Cooldown: 5m 0s)');
});

// Test 9: Sub-100% Fallback scenario (Pass 2)
test('Sub-100% Fallback scenario (Weekly 85%, 4h 60%, Pro Tier)', () => {
    const account = createMockAccount({
        tier: 'PRO',
        weeklyQuota: 85,
        fourHourQuota: 60,
    });

    const result = computeAccountAlgorithmScore(account);

    assertEqual(result.isFull4h, false);
    assertEqual(result.isFallback, true);
    assertEqual(result.fourHourQuotaPercent, 60);
    assertEqual(result.weeklyQuotaPercent, 85);

    // Proportional quota = (85 / 100) * (60 / 100) * 100 = 51.0
    // rawScore = 1 * 2 * 51 = 102
    assertEqual(result.rawScore, 102);
    assertEqual(result.score, 0.102);
    assertEqual(result.percentage, 51.0);

    assertEqual(result.mathBreakdown.fourHourTerm, 'Q_4h = 60% (Sub-100% Fallback)');
    assertEqual(result.mathBreakdown.multiplicationStep, '1 × 2.0 × 51 = 102.0');
    assertEqual(result.mathBreakdown.divisionStep, '102.0 ÷ 1000 = 0.1020');
    assertEqual(result.mathBreakdown.percentageStep, '(0.1020 ÷ 0.2000) × 100% = 51.0%');
});

// Test 10: Fallback low weekly quota threshold (< 8%)
test('Fallback with weekly quota < 8% gives score 0', () => {
    const account = createMockAccount({
        tier: 'PRO',
        weeklyQuota: 5,
        fourHourQuota: 50,
    });

    const result = computeAccountAlgorithmScore(account);

    assertEqual(result.isFallback, true);
    assertEqual(result.score, 0);
    assertEqual(result.rawScore, 0);
    assertEqual(result.percentage, 0);
});

// Test 11: Custom tier multipliers option
test('Custom tier multipliers option', () => {
    const account = createMockAccount({
        tier: 'PRO',
        weeklyQuota: 100,
        fourHourQuota: 100,
    });

    const result = computeAccountAlgorithmScore(account, {
        tierMultipliers: { ultra: 5, pro: 3, free: 1.5 },
    });

    assertEqual(result.tierMultiplier, 3);
    assertEqual(result.rawScore, 300);
    assertEqual(result.score, 0.3);
    assertEqual(result.mathBreakdown.tierTerm, 'M_tier = 3.0 (Pro Tier)');
});

// Test 12: Color scale helper thresholds
test('Color scale helper returns correct classes per threshold', () => {
    const high = getColorScale(75);
    assert(high.text.includes('emerald-500'));
    assert(high.gradient.includes('from-emerald-500 to-teal-400'));

    const mid = getColorScale(45);
    assert(mid.text.includes('amber-500'));
    assert(mid.gradient.includes('from-amber-500 to-yellow-400'));

    const low = getColorScale(35);
    assert(low.text.includes('rose-500'));
    assert(low.gradient.includes('from-rose-600 to-red-500'));
});

// Test 13: resolveTier helper
test('resolveTier handles different tier name formats', () => {
    assertEqual(resolveTier('ULTRA'), 'ultra');
    assertEqual(resolveTier('Pro-tier'), 'pro');
    assertEqual(resolveTier('Free'), 'free');
    assertEqual(resolveTier(undefined), 'free');
    assertEqual(resolveTier('unknown'), 'free');
});

// Test 14: Sorting comparator
test('compareAccountAlgorithmScores sorts descending by score then quotas', () => {
    const acc1 = createMockAccount({ id: 'acc1', email: 'a@test.com', tier: 'PRO', weeklyQuota: 85, fourHourQuota: 100 });
    const acc2 = createMockAccount({ id: 'acc2', email: 'b@test.com', tier: 'ULTRA', weeklyQuota: 100, fourHourQuota: 100 });

    const res1 = computeAccountAlgorithmScore(acc1);
    const res2 = computeAccountAlgorithmScore(acc2);

    const diff = compareAccountAlgorithmScores(res1, res2, acc1, acc2);
    assert(diff > 0, 'Ultra (score 0.40) should rank before Pro (score 0.17)');
});

console.log(`\nTests finished: ${passed} passed, ${failed} failed.`);
if (failed > 0) {
    throw new Error(`${failed} test(s) failed`);
}
