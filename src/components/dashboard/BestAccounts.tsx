import { TrendingUp } from 'lucide-react';
import { Account, QuotaGroup } from '../../types/account';
import { findQuotaModel } from '../../config/modelConfig';

interface BestAccountsProps {
    accounts: Account[];
    currentAccountId?: string;
    onSwitch?: (accountId: string) => void;
}

import { useTranslation } from 'react-i18next';

/** Extract 5h or Weekly bucket percentage (0-100) from quota_groups */
function getBucketPercentage(
    quotaGroups: QuotaGroup[] | undefined,
    category: 'gemini' | 'claude',
    targetWindow: '5h' | 'weekly'
): number | null {
    if (!quotaGroups || quotaGroups.length === 0) return null;

    for (const group of quotaGroups) {
        const name = (group.display_name || '').toLowerCase();
        const isTarget = category === 'claude'
            ? (name.includes('claude') || name.includes('gpt'))
            : (name.includes('gemini') || !name.includes('claude'));

        if (isTarget) {
            const bucket = group.buckets?.find(b => {
                const win = (b.window || '').toLowerCase();
                const id = (b.bucket_id || '').toLowerCase();
                if (targetWindow === 'weekly') {
                    return win.includes('week') || id.includes('week');
                } else {
                    return win.includes('5h') || id.includes('5h') || win.includes('hour') || id.includes('hour');
                }
            });

            if (bucket && typeof bucket.remaining_fraction === 'number') {
                return Math.round(bucket.remaining_fraction * 100);
            }
        }
    }
    return null;
}

/**
 * Calculate model composite effective quota
 * Auto-detect: Dual-bucket (take min bottleneck) / Free account weekly-only / 5h-only fallback
 */
function calculateEffectiveQuota(
    fiveHourFromModel: number | null,
    weeklyFromGroup: number | null,
    fiveHourFromGroup: number | null
): number {
    const fiveHour = fiveHourFromGroup !== null ? fiveHourFromGroup : fiveHourFromModel;
    const weekly = weeklyFromGroup;

    // Case 1: Dual buckets exist (Pro/Ultra account) -> Bottleneck min(5h, weekly)
    if (fiveHour !== null && weekly !== null) {
        return Math.min(fiveHour, weekly);
    }

    // Case 2: Only weekly bucket exists (Free tier) -> Base on weekly quota directly
    if (weekly !== null) {
        return weekly;
    }

    // Case 3: Only 5h bucket exists (single-bucket fallback) -> Base on 5h quota
    if (fiveHour !== null) {
        return fiveHour;
    }

    return 0;
}

function BestAccounts({ accounts, currentAccountId, onSwitch }: BestAccountsProps) {
    const { t } = useTranslation();
    // 1. Get list sorted by composite effective quota (exclude current and disabled accounts)
    const geminiSorted = accounts
        .filter(a => a.id !== currentAccountId && !a.disabled && !a.proxy_disabled)
        .map(a => {
            const pro5hModel = findQuotaModel(a.quota?.models, 'gemini-pro')?.percentage ?? null;
            const flash5hModel = findQuotaModel(a.quota?.models, 'gemini-flash')?.percentage ?? null;
            const weeklyGroup = getBucketPercentage(a.quota?.quota_groups, 'gemini', 'weekly');
            const fiveHourGroup = getBucketPercentage(a.quota?.quota_groups, 'gemini', '5h');

            const effectivePro = calculateEffectiveQuota(pro5hModel, weeklyGroup, fiveHourGroup);
            const effectiveFlash = calculateEffectiveQuota(flash5hModel, weeklyGroup, fiveHourGroup);

            // Composite score: Pro has higher weight (70%), Flash has 30%
            let score = Math.round(effectivePro * 0.7 + effectiveFlash * 0.3);

            // If weekly quota is depleted (<= 5%), eliminate directly
            if (weeklyGroup !== null && weeklyGroup <= 5) {
                score = 0;
            }

            return {
                ...a,
                quotaVal: score,
            };
        })
        .filter(a => a.quotaVal > 0)
        .sort((a, b) => b.quotaVal - a.quotaVal);

    const claudeSorted = accounts
        .filter(a => a.id !== currentAccountId && !a.disabled && !a.proxy_disabled)
        .map(a => {
            const claude5hModel = findQuotaModel(a.quota?.models, 'claude')?.percentage ?? null;
            const weeklyGroup = getBucketPercentage(a.quota?.quota_groups, 'claude', 'weekly');
            const fiveHourGroup = getBucketPercentage(a.quota?.quota_groups, 'claude', '5h');

            let score = calculateEffectiveQuota(claude5hModel, weeklyGroup, fiveHourGroup);

            // If weekly quota is depleted (<= 5%), eliminate directly
            if (weeklyGroup !== null && weeklyGroup <= 5) {
                score = 0;
            }

            return {
                ...a,
                quotaVal: score,
            };
        })
        .filter(a => a.quotaVal > 0)
        .sort((a, b) => b.quotaVal - a.quotaVal);

    let bestGemini = geminiSorted[0];
    let bestClaude = claudeSorted[0];

    // 2. If recommended accounts are identical and alternatives exist, find optimal distinct account pair
    if (bestGemini && bestClaude && bestGemini.id === bestClaude.id) {
        const nextGemini = geminiSorted[1];
        const nextClaude = claudeSorted[1];

        // Option A: Keep Gemini optimal, switch Claude to runner-up
        // Option B: Switch Gemini to runner-up, keep Claude optimal
        // Evaluation criterion: maximize sum of quotas (or prioritize preserving 100%)

        const scoreA = bestGemini.quotaVal + (nextClaude?.quotaVal || 0);
        const scoreB = (nextGemini?.quotaVal || 0) + bestClaude.quotaVal;

        if (!nextGemini) {
            if (nextClaude) {
                bestClaude = nextClaude;
            }
        } else if (nextClaude) {
            if (scoreA >= scoreB) {
                bestClaude = nextClaude;
            } else {
                bestGemini = nextGemini;
            }
        } else {
            bestGemini = nextGemini;
        }
        // If no runner-up available (e.g. only 1 account), keep as-is
    }

    // Construct final view model for display (compatible with existing rendering logic)
    const bestGeminiRender = bestGemini ? { ...bestGemini, geminiQuota: bestGemini.quotaVal } : undefined;
    const bestClaudeRender = bestClaude ? { ...bestClaude, claudeQuota: bestClaude.quotaVal } : undefined;

    return (
        <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 h-full flex flex-col">
            <h2 className="text-base font-semibold text-gray-900 dark:text-base-content mb-3 flex items-center gap-2">
                <TrendingUp className="w-4 h-4 text-blue-500 dark:text-blue-400" />
                {t('dashboard.best_accounts')}
            </h2>

            <div className="space-y-2 flex-1">
                {/* Gemini Best */}
                {bestGeminiRender && (
                    <div className="flex items-center justify-between p-2.5 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-100 dark:border-green-900/30">
                        <div className="flex-1 min-w-0">
                            <div className="text-[10px] text-green-600 dark:text-green-400 font-medium mb-0.5">{t('dashboard.for_gemini')}</div>
                            <div className="font-medium text-sm text-gray-900 dark:text-base-content truncate">
                                {bestGeminiRender.email}
                            </div>
                        </div>
                        <div className="ml-2 px-2 py-0.5 bg-green-500 text-white text-xs font-semibold rounded-full">
                            {bestGeminiRender.geminiQuota}%
                        </div>
                    </div>
                )}

                {/* Claude Best */}
                {bestClaudeRender && (
                    <div className="flex items-center justify-between p-2.5 bg-cyan-50 dark:bg-cyan-900/20 rounded-lg border border-cyan-100 dark:border-cyan-900/30">
                        <div className="flex-1 min-w-0">
                            <div className="text-[10px] text-cyan-600 dark:text-cyan-400 font-medium mb-0.5">{t('dashboard.for_claude')}</div>
                            <div className="font-medium text-sm text-gray-900 dark:text-base-content truncate">
                                {bestClaudeRender.email}
                            </div>
                        </div>
                        <div className="ml-2 px-2 py-0.5 bg-cyan-500 text-white text-xs font-semibold rounded-full">
                            {bestClaudeRender.claudeQuota}%
                        </div>
                    </div>
                )}

                {!bestGeminiRender && !bestClaudeRender && (
                    <div className="text-center py-4 text-gray-400 text-sm">
                        {t('accounts.no_data')}
                    </div>
                )}
            </div>

            {(bestGeminiRender || bestClaudeRender) && onSwitch && (
                <div className="mt-auto pt-3">
                    <button
                        className="w-full px-3 py-1.5 bg-blue-500 text-white text-xs font-medium rounded-lg hover:bg-blue-600 transition-colors"
                        onClick={() => {
                            // Prioritize switching to account with higher quota
                            let targetId = bestGeminiRender?.id;
                            if (!bestGeminiRender) {
                                if (bestClaudeRender) {
                                    targetId = bestClaudeRender.id;
                                }
                            } else if (bestClaudeRender) {
                                if (bestClaudeRender.claudeQuota > bestGeminiRender.geminiQuota) {
                                    targetId = bestClaudeRender.id;
                                }
                            }

                            if (onSwitch && targetId) {
                                onSwitch(targetId);
                            }
                        }}
                    >
                        {t('dashboard.switch_best')}
                    </button>
                </div>
            )}
        </div>
    );
}

export default BestAccounts;
