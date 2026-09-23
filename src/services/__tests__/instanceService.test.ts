/**
 * Standalone assertion test for Multiplicative Candidate Scoring and Weekly Quota extraction.
 *
 * Anonymized test fixtures: uses synthetic identifiers only (zero real emails).
 * Run: npx tsx src/services/__tests__/instanceService.test.ts
 */
import {
    extractWeeklyQuotaPercent,
    calculateMultiplicativeScore,
    rankSmartCandidates,
    getSubscriptionTierMultiplier,
} from '../instanceService';
import type { Account, QuotaData } from '../../types/account';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
    try {
        fn();
        passed++;
    } catch (e: unknown) {
        failed++;
        const msg = e instanceof Error ? e.message : String(e);
        console.error(`  FAIL: ${description} — ${msg}`);
    }
}

function assertEqual<T>(actual: T, expected: T): void {
    if (actual !== expected) {
        throw new Error(`expected "${expected}", got "${actual}"`);
    }
}

function makeSyntheticAccount(
    id: string,
    email: string,
    tier: string,
    geminiWeekly: number,
    claudeWeekly: number
): Account {
    const quota: QuotaData = {
        models: [
            { name: 'gemini-2.5-pro', percentage: 100, reset_time: '2026-09-30T00:00:00Z' },
            { name: 'claude-3-7-sonnet', percentage: 100, reset_time: '2026-09-30T00:00:00Z' },
        ],
        last_updated: Date.now(),
        subscription_tier: tier,
        quota_groups: [
            {
                display_name: 'Gemini Models',
                buckets: [
                    {
                        bucket_id: 'gemini-weekly',
                        window: 'weekly',
                        remaining_fraction: geminiWeekly / 100,
                        reset_time: '2026-09-30T00:00:00Z',
                    },
                ],
            },
            {
                display_name: 'Claude and GPT models',
                buckets: [
                    {
                        bucket_id: 'claude-weekly',
                        window: 'weekly',
                        remaining_fraction: claudeWeekly / 100,
                        reset_time: '2026-09-30T00:00:00Z',
                    },
                ],
            },
        ],
    };

    return {
        id,
        email,
        token: {
            access_token: 'fake-token',
            refresh_token: 'fake-refresh',
            expires_in: 3600,
            expiry_timestamp: Date.now() + 3600000,
            token_type: 'Bearer',
        },
        quota,
        disabled: false,
        created_at: 1700000000,
        last_used: 1700000000,
    };
}

// ── Test 1: Subscription Tier Multipliers ────────────────────────────────────
test('getSubscriptionTierMultiplier maps Ultra=5, Pro=3, Free/other=1', () => {
    assertEqual(getSubscriptionTierMultiplier('ULTRA'), 5);
    assertEqual(getSubscriptionTierMultiplier('pro'), 3);
    assertEqual(getSubscriptionTierMultiplier('FREE'), 1);
    assertEqual(getSubscriptionTierMultiplier(undefined), 1);
});

// ── Test 2: Weekly Quota Group Extraction & Bottleneck ───────────────────────
test('extractWeeklyQuotaPercent extracts bottleneck from quota_groups', () => {
    // Synthetic Candidate A: 100% Gemini, 100% Claude -> 100%
    const accA = makeSyntheticAccount('synth_user_a', 'synth_a@test.local', 'pro', 100, 100);
    assertEqual(extractWeeklyQuotaPercent(accA), 100);

    // Synthetic Candidate B: 21% Gemini, 100% Claude -> 21% (bottleneck)
    const accB = makeSyntheticAccount('synth_user_b', 'synth_b@test.local', 'pro', 21, 100);
    assertEqual(extractWeeklyQuotaPercent(accB), 21);
});

// ── Test 3: Multiplicative Scoring Formula: S_active * M_tier * Q_weekly ────
test('calculateMultiplicativeScore computes active * tier * weeklyQuota', () => {
    const accA = makeSyntheticAccount('synth_user_a', 'synth_a@test.local', 'pro', 100, 100);
    const scoreA = calculateMultiplicativeScore(accA, []);
    // S_active=1, M_tier=3, Q_weekly=100 -> 300
    assertEqual(scoreA.score, 300);

    const accB = makeSyntheticAccount('synth_user_b', 'synth_b@test.local', 'pro', 21, 100);
    const scoreB = calculateMultiplicativeScore(accB, []);
    // S_active=1, M_tier=3, Q_weekly=21 -> 63
    assertEqual(scoreB.score, 63);

    // In-use account: S_active=0 -> score 0
    const inUseScore = calculateMultiplicativeScore(accA, ['synth_user_a']);
    assertEqual(inUseScore.score, 0);

    // Current account: S_active=0 -> score 0
    const currentScore = calculateMultiplicativeScore(accA, [], 'synth_user_a');
    assertEqual(currentScore.score, 0);
});

// ── Test 4: Candidate Ranking Prioritizes Higher Weekly Quota ───────────────
test('rankSmartCandidates ranks 100% weekly quota over 21% weekly quota', () => {
    const accA = makeSyntheticAccount('synth_user_a', 'synth_a@test.local', 'pro', 100, 100);
    const accB = makeSyntheticAccount('synth_user_b', 'synth_b@test.local', 'pro', 21, 100);

    // Put accB first in input to verify sorting re-orders
    const ranked = rankSmartCandidates([accB, accA], []);
    assertEqual(ranked.length, 2);
    assertEqual(ranked[0].account.id, 'synth_user_a');
    assertEqual(ranked[0].score, 300);
    assertEqual(ranked[1].account.id, 'synth_user_b');
    assertEqual(ranked[1].score, 63);
});

if (failed > 0) {
    throw new Error(`${failed} test(s) failed`);
}
