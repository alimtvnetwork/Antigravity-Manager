import React, { useCallback } from 'react';
import { Flame, ShieldCheck, Leaf, Sparkles } from 'lucide-react';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';

export type WarmupPresetTier = 'ultra' | 'pro' | 'eco';

export interface WarmupTierConfig {
  id: WarmupPresetTier;
  label: string;
  tagline: string;
  description: string;
  models: string[];
  icon: React.ComponentType<{ className?: string }>;
}

export const WARMUP_TIER_MODELS: Record<WarmupPresetTier, string[]> = {
  ultra: ['gemini-2.5-pro', 'claude-3-7-sonnet', 'gemini-2.5-flash'],
  pro: ['gemini-2.5-flash', 'gemini-2.0-flash'],
  eco: ['gemini-2.0-flash'],
};

export const WARMUP_TIER_CONFIGS: Record<WarmupPresetTier, WarmupTierConfig> = {
  ultra: {
    id: 'ultra',
    label: 'Ultra Profile',
    tagline: 'Pro + Claude',
    description: 'Warms high-tier reasoning engines (Gemini 2.5 Pro, Claude 3.7 Sonnet, 2.5 Flash)',
    models: WARMUP_TIER_MODELS.ultra,
    icon: Flame,
  },
  pro: {
    id: 'pro',
    label: 'Standard Pro Profile',
    tagline: 'Flash Models',
    description: 'Balanced throughput warming (Gemini 2.5 Flash, 2.0 Flash)',
    models: WARMUP_TIER_MODELS.pro,
    icon: ShieldCheck,
  },
  eco: {
    id: 'eco',
    label: 'Eco Free Profile',
    tagline: 'Minimal Flash',
    description: 'Zero quota burn warming strictly lightweight Gemini 2.0 Flash',
    models: WARMUP_TIER_MODELS.eco,
    icon: Leaf,
  },
};

export interface WarmupTierPresetsProps {
  activePreset?: WarmupPresetTier;
  onSelectPreset?: (preset: WarmupPresetTier) => void;
  className?: string;
  showDescriptions?: boolean;
}

interface PresetPillButtonProps {
  config: WarmupTierConfig;
  isActive: boolean;
  onSelect: (preset: WarmupPresetTier) => void;
}

/**
 * Individual segmented pill button representing a warmup tier profile.
 */
const PresetPillButton: React.FC<PresetPillButtonProps> = ({
  config,
  isActive,
  onSelect,
}) => {
  const Icon = config.icon;
  const handleClick = () => onSelect(config.id);

  return (
    <button
      type="button"
      onClick={handleClick}
      title={config.description}
      className={cn(
        'px-3 py-1 rounded-full text-xs font-medium transition-all duration-200 inline-flex items-center gap-1.5 cursor-pointer select-none',
        isActive
          ? 'bg-orange-500 text-white font-semibold shadow-md'
          : 'text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800'
      )}
    >
      <Icon className={cn('w-3.5 h-3.5', isActive ? 'text-white' : 'text-zinc-400')} />
      <span>{config.label}</span>
      <span
        className={cn(
          'text-[10px] px-1.5 py-0.2 rounded-full font-mono',
          isActive ? 'bg-orange-600/60 text-orange-100' : 'bg-zinc-800 text-zinc-400'
        )}
      >
        {config.tagline}
      </span>
    </button>
  );
};

/**
 * Tier-Based Warmup Profiles.
 * Offers one-click preset buttons configuring model sets across accounts.
 */
export const WarmupTierPresets: React.FC<WarmupTierPresetsProps> = ({
  activePreset = 'ultra',
  onSelectPreset,
  className,
  showDescriptions = false,
}) => {
  const tiers: WarmupTierConfig[] = [
    WARMUP_TIER_CONFIGS.ultra,
    WARMUP_TIER_CONFIGS.pro,
    WARMUP_TIER_CONFIGS.eco,
  ];

  const handleSelect = useCallback(
    (preset: WarmupPresetTier) => {
      try {
        if (onSelectPreset) {
          onSelectPreset(preset);
        }
      } catch (err) {
        useErrorStore.getState().trackWarning(err, {
          source: 'WarmupTierPresets',
          triggerAction: 'select_preset',
        });
      }
    },
    [onSelectPreset]
  );

  const activeConfig = WARMUP_TIER_CONFIGS[activePreset] ?? WARMUP_TIER_CONFIGS.ultra;

  return (
    <div className={cn('flex flex-col gap-2', className)}>
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Sparkles className="w-3.5 h-3.5 text-orange-400" />
          <span className="text-xs font-semibold text-zinc-300">Warmup Tier Profiles</span>
        </div>
        <div className="inline-flex p-1 bg-zinc-900 border border-zinc-800 rounded-full gap-1">
          {tiers.map((tier) => (
            <PresetPillButton
              key={tier.id}
              config={tier}
              isActive={activePreset === tier.id}
              onSelect={handleSelect}
            />
          ))}
        </div>
      </div>

      {showDescriptions && (
        <div className="flex items-center justify-between text-[11px] text-zinc-400 bg-zinc-900/60 border border-zinc-800/80 rounded-lg px-3 py-1.5">
          <span>{activeConfig.description}</span>
          <div className="flex items-center gap-1">
            {activeConfig.models.map((m) => (
              <span
                key={m}
                className="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-300 font-mono"
              >
                {m}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
