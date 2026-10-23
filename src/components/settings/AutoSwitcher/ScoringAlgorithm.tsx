import { AutoSwitcherApi } from './useAutoSwitcher';
import { SlidersHorizontal, RotateCcw } from 'lucide-react';
import { cn } from '../../../utils/cn';

export function ScoringAlgorithm(props: AutoSwitcherApi) {
    const { t, currentConfig, onChange } = props;
    const floorValue = currentConfig.min_weekly_quota_percent ?? 15;

    return (
        <>
                <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 flex flex-col justify-between space-y-3 shadow-2xs">
                    <div>
                        <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center gap-2">
                                <div className="p-1.5 rounded-lg bg-violet-100 dark:bg-violet-500/20 text-violet-600 dark:text-violet-400 border border-violet-200/60 dark:border-violet-500/30">
                                    <SlidersHorizontal className="w-4 h-4" />
                                </div>
                                <div>
                                    <h4 className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider">
                                        {t('settings.auto_switcher.algorithm_title', 'Best-Account Scoring Algorithm')}
                                    </h4>
                                    <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                        {t('settings.auto_switcher.algorithm_subtitle', 'Subscription-tier weights for the next-best-profile ranking')}
                                    </p>
                                </div>
                            </div>
                            <button
                                type="button"
                                onClick={() => onChange({
                                    ...currentConfig,
                                    ultra_tier_multiplier: 4.0,
                                    pro_tier_multiplier: 2.0,
                                    free_tier_multiplier: 1.0,
                                    min_weekly_quota_percent: 15.0,
                                })}
                                className="p-1.5 rounded-lg text-slate-400 hover:text-violet-500 hover:bg-violet-100/60 dark:hover:bg-violet-500/10 transition-colors cursor-pointer"
                                title={t('settings.auto_switcher.algorithm_reset', 'Reset to defaults (Ultra 4.0, Pro 2.0, Free 1.0, Floor 15%)')}
                            >
                                <RotateCcw className="w-3.5 h-3.5" />
                            </button>
                        </div>

                        <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed font-mono bg-slate-100/80 dark:bg-slate-900/80 rounded-lg px-2.5 py-1.5 border border-slate-200 dark:border-slate-700/60">
                            score = (M<sub>tier</sub> × weekly_quota × hours_elapsed) / 100
                        </p>

                        <div className="grid grid-cols-3 gap-2 mt-3">
                            {([
                                { key: 'ultra_tier_multiplier' as const, label: 'Ultra', def: 4.0, color: 'violet' },
                                { key: 'pro_tier_multiplier' as const, label: 'Pro', def: 2.0, color: 'blue' },
                                { key: 'free_tier_multiplier' as const, label: 'Free', def: 1.0, color: 'slate' },
                            ]).map(({ key, label, def, color }) => {
                                const val = currentConfig[key] ?? def;
                                return (
                                    <label key={key} className="flex flex-col gap-1 p-2 rounded-lg bg-white dark:bg-slate-900/70 border border-slate-200 dark:border-slate-700/60">
                                        <span className="text-[11px] font-semibold text-slate-700 dark:text-slate-300 flex items-center justify-between">
                                            {label}
                                            <span className={cn(
                                                "font-mono text-[10px] px-1.5 py-0.2 rounded-full border",
                                                color === 'violet' && "bg-violet-500/10 text-violet-600 dark:text-violet-400 border-violet-500/25",
                                                color === 'blue' && "bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/25",
                                                color === 'slate' && "bg-slate-500/10 text-slate-500 dark:text-slate-400 border-slate-500/25",
                                            )}>
                                                ×{val}
                                            </span>
                                        </span>
                                        <input
                                            type="number"
                                            min={0.1}
                                            max={10}
                                            step={0.1}
                                            value={val}
                                            onChange={(e) => {
                                                const parsed = Number(e.target.value);
                                                if (Number.isFinite(parsed)) {
                                                    const clamped = Math.max(0.1, Math.min(10, parsed));
                                                    onChange({ ...currentConfig, [key]: clamped });
                                                }
                                            }}
                                            className="w-full px-2 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-mono font-medium text-slate-800 dark:text-slate-200 shadow-2xs focus:outline-none focus:ring-2 focus:ring-violet-500/30"
                                        />
                                    </label>
                                );
                            })}
                        </div>
                        <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed pt-1">
                            {t('settings.auto_switcher.algorithm_desc', 'Higher weight favors that tier when ranking healthy candidates. Changes apply to the next best-profile evaluation immediately.')}
                        </p>

                        {/* Minimum Weekly Quota Floor (%) */}
                        <div className="mt-3 p-3.5 rounded-xl bg-white dark:bg-slate-900/70 border border-slate-200 dark:border-slate-700/60 space-y-2.5">
                            <div className="flex justify-between items-center text-xs font-semibold text-slate-800 dark:text-slate-200">
                                <span>{t('settings.auto_switcher.min_weekly_floor_label', 'Minimum Weekly Quota Floor (%)')}</span>
                                <span className="text-violet-600 dark:text-violet-400 font-mono font-bold">
                                    {`≥ ${floorValue}%`}
                                </span>
                            </div>
                            <input
                                type="range"
                                min={0}
                                max={50}
                                step={1}
                                value={floorValue}
                                onChange={(e) => onChange({ ...currentConfig, min_weekly_quota_percent: Number(e.target.value) })}
                                className="w-full accent-violet-600 cursor-pointer"
                            />
                            <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                                <span>0% (Disabled)</span>
                                <span>15% (Default)</span>
                                <span>30%</span>
                                <span>50%</span>
                            </div>
                            <div className="flex flex-wrap gap-2 pt-1">
                                {[
                                    { label: '15% (Default)', value: 15 },
                                    { label: '10%', value: 10 },
                                    { label: '0% (Disabled)', value: 0 },
                                ].map((preset) => {
                                    const isSelected = floorValue === preset.value;
                                    return (
                                        <button
                                            key={preset.value}
                                            type="button"
                                            onClick={() => onChange({ ...currentConfig, min_weekly_quota_percent: preset.value })}
                                            className={cn(
                                                "px-2.5 py-1 text-[11px] font-medium rounded-lg border transition-all cursor-pointer",
                                                isSelected
                                                    ? "bg-violet-50 dark:bg-violet-950/40 text-violet-600 dark:text-violet-400 border-violet-300 dark:border-violet-800 shadow-2xs font-semibold"
                                                    : "bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600"
                                            )}
                                        >
                                            {preset.label}
                                        </button>
                                    );
                                })}
                            </div>
                            <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed pt-0.5">
                                {t('settings.auto_switcher.min_weekly_floor_desc', 'Accounts with weekly quota below this floor are treated as depleted (score 0.0) to prevent premature exhaustion.')}
                            </p>
                        </div>
                    </div>
                </div>

        </>
    );
}
