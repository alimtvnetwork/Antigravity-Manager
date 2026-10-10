import type { TFunction } from 'i18next';
import { BrainCircuit } from 'lucide-react';
import ThinkingBudget from '../../components/settings/ThinkingBudget';
import type { AppConfig, ThinkingBudgetConfig } from '../../types/config';
import type { ExperimentalConfigUpdater, ProxyConfigUpdater } from './types';
import { DEFAULT_THINKING_MAX_MEMORY_TURNS, DEFAULT_THINKING_RETENTION_DAYS } from './constants';
import { CollapsibleCard } from './CollapsibleCard';

interface ThinkingSectionProps {
    config: AppConfig;
    t: TFunction;
    updateProxyConfig: ProxyConfigUpdater;
    updateExperimentalConfig: ExperimentalConfigUpdater;
    onSave: () => Promise<void> | void;
}

export function ThinkingSection({
    config,
    t,
    updateProxyConfig,
    updateExperimentalConfig,
    onSave,
}: ThinkingSectionProps) {
    return (
        <CollapsibleCard
            title={t('proxy.config.thinking_settings.title', { defaultValue: 'Thinking Settings' })}
            icon={<BrainCircuit size={18} className="text-purple-500" />}
            defaultExpanded={true}
        >
            <ThinkingBudget
                config={config.proxy.thinking_budget}
                onChange={(tbConfig: ThinkingBudgetConfig) => updateProxyConfig({ thinking_budget: tbConfig })}
                onSave={onSave}
                thinkingStoreEnabled={config.proxy.experimental?.thinking_store_enabled !== false}
                onThinkingStoreChange={(enabled: boolean) =>
                    updateExperimentalConfig({ thinking_store_enabled: enabled })
                }
                thinkingMaxMemoryTurns={config.proxy.experimental?.thinking_max_memory_turns ?? DEFAULT_THINKING_MAX_MEMORY_TURNS}
                onThinkingMaxMemoryTurnsChange={(turns: number) =>
                    updateExperimentalConfig({ thinking_max_memory_turns: turns })
                }
                thinkingRetentionDays={config.proxy.experimental?.thinking_retention_days ?? DEFAULT_THINKING_RETENTION_DAYS}
                onThinkingRetentionDaysChange={(days: number) =>
                    updateExperimentalConfig({ thinking_retention_days: days })
                }
            />
        </CollapsibleCard>
    );
}
