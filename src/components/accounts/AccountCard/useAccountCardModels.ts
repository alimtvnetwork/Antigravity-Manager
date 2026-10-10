import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Bot, Sparkles } from 'lucide-react';
import type { Account, ModelQuota } from '../../../types/account';
import { MODEL_CONFIG, sortModels, getModelProtectionKey, resolveQuotaModels, ensurePinnedImageSelector } from '../../../config/modelConfig';
import { DEFAULT_MODELS, type DisplayModel, type WeeklyItem } from './types';

interface UseAccountCardModelsParams {
    account: Account;
    showAllQuotas: boolean;
    pinnedModels: string[] | undefined;
    quotaProtectionEnabled: boolean;
}

/**
 * Computes display models and weekly quota items for an account card.
 * Extracted from AccountCard to keep the component under the 500-line limit.
 */
export function useAccountCardModels({
    account,
    showAllQuotas,
    pinnedModels,
    quotaProtectionEnabled,
}: UseAccountCardModelsParams): {
    displayModels: DisplayModel[];
    weeklyItems: WeeklyItem[];
    isModelProtected: (key?: string) => boolean;
} {
    const { t } = useTranslation();

    const displayModels = useMemo((): DisplayModel[] => {
        // Build map of friendly labels and icons from DEFAULT_MODELS
        const iconMap = new Map(DEFAULT_MODELS.map(m => [m.id, m.Icon]));

        // Get all models from account (source of truth)
        const accountModels = account.quota?.models?.map(m => {
            // Note: DEFAULT_MODELS contains shortLabel from MODEL_CONFIG
            // Ensure shortLabel is present in mapped models
            // Safely retrieve directly from MODEL_CONFIG
            const fullConfig = MODEL_CONFIG[m.name.toLowerCase()];
            return {
                id: m.name,
                label: m.display_name || fullConfig?.shortLabel || fullConfig?.label || m.name,
                protectedKey: getModelProtectionKey(m.name) ?? fullConfig?.protectedKey ?? m.name,
                Icon: iconMap.get(m.name) || Bot,
                data: m
            };
        }) || [];

        let models: typeof accountModels;

        if (showAllQuotas) {
            models = accountModels;
        } else {
            // Filter for pinned or defaults
            const pinned = pinnedModels;
            if (pinned && pinned.length > 0) {
                const selections = resolveQuotaModels(
                    accountModels.map(m => m.data),
                    ensurePinnedImageSelector(pinned),
                );
                models = selections
                    .map(sel => sel.model ? accountModels.find(am => am.data === sel.model) : undefined)
                    .filter((m): m is typeof accountModels[number] => m !== undefined);
                // Preserve pinned models without quota data (display 0%)
                for (const sel of selections) {
                    if (!sel.model) {
                        const selectorConfig = MODEL_CONFIG[sel.selectorId.toLowerCase()];
                        if (selectorConfig) {
                            models = [...models, {
                                id: sel.selectorId,
                                label: selectorConfig.shortLabel || selectorConfig.label,
                                protectedKey: selectorConfig.protectedKey,
                                Icon: selectorConfig.Icon,
                                data: { name: sel.selectorId, percentage: 0 } as ModelQuota,
                            }];
                        }
                    }
                }
            } else {
                // Default fallback: show known default models, plus we show all dynamic pinned models
                // Fallback: if pinned is not configured, show built-in models and models with a display_name.
                models = accountModels.filter(m => DEFAULT_MODELS.some(d => d.id === m.id) || m.data.display_name);
            }
        }

        // Apply sorting and filter expired models
        return sortModels(models).filter(m => m.id !== 'claude-sonnet-4-6-thinking' && m.id !== 'claude-sonnet-4-5-thinking' && m.id !== 'claude-opus-4-5-thinking');
    }, [account, showAllQuotas, pinnedModels]);

    // Parse weekly quota entries when in weekly view
    const weeklyItems = useMemo((): WeeklyItem[] => {
        return (account.quota?.quota_groups || []).flatMap(group => {
            return (group.buckets || [])
                .filter(b => b.window.toLowerCase().includes('week') || b.bucket_id.toLowerCase().includes('week'))
                .map(b => {
                    const shortGroupName = (group.display_name || '')
                        .replace(/ models?$/i, '')
                        .replace(/Claude and GPT/i, 'Claude/GPT');
                    const weeklySuffix = t('accounts.quota_window_weekly_short', 'Semanal');
                    return {
                        id: `${group.display_name}-${b.bucket_id}`,
                        label: b.display_name ? `${shortGroupName} (${b.display_name})` : `${shortGroupName} (${weeklySuffix})`,
                        percentage: Math.round((b.remaining_fraction || 0) * 100),
                        resetTime: b.reset_time,
                        cycleTokens: b.cycle_tokens,
                        Icon: shortGroupName.toLowerCase().includes('claude') ? Sparkles : Bot,
                    };
                });
        });
    }, [account.quota?.quota_groups, t]);

    const isModelProtected = (key?: string): boolean => {
        if (!quotaProtectionEnabled) return false;
        if (!key) return false;
        return account.protected_models?.includes(key) ?? false;
    };

    return { displayModels, weeklyItems, isModelProtected };
}
