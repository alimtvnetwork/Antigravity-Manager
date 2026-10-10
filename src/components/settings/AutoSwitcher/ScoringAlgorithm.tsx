import { AutoSwitcherApi } from './useAutoSwitcher';
import { cn } from '../../utils/cn';

export function ScoringAlgorithm(props: AutoSwitcherApi) {
    const { t, currentConfig, onChange } = props;

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
                                onClick={() => onChange({ ...currentConfig, ultra_tier_multiplier: 4.0, pro_tier_multiplier: 2.0, free_tier_multiplier: 1.0 })}
                                className="p-1.5 rounded-lg text-slate-400 hover:text-violet-500 hover:bg-violet-100/60 dark:hover:bg-violet-500/10 transition-colors cursor-pointer"
                                title={t('settings.auto_switcher.algorithm_reset', 'Reset to defaults (Ultra 4.0, Pro 2.0, Free 1.0)')}
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
                    </div>
                </div>

        </>
    );
}
