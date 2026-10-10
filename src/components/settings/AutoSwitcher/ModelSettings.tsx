import { AutoSwitcherApi } from './useAutoSwitcher';

export function ModelSettings(props: AutoSwitcherApi) {
    const { t, currentConfig, onChange } = props;

    return (
        <>
                            <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 flex flex-col justify-between space-y-4 shadow-2xs">
                                <div>
                                    <div className="flex items-center justify-between mb-2">
                                        <div className="flex items-center gap-2">
                                            <div className="p-1.5 rounded-lg bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400 border border-blue-200/60 dark:border-blue-500/30">
                                                <Cpu className="w-4 h-4" />
                                            </div>
                                            <div>
                                                <h4 className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider">
                                                    {t('settings.auto_switcher.target_model_label', 'Primary Evaluated Model')}
                                                </h4>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                                    Quota benchmark evaluated to trigger automated account failover
                                                </p>
                                            </div>
                                        </div>
                                        <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-500/10 text-blue-600 dark:text-blue-400 border border-blue-200/50 dark:border-blue-500/20 font-semibold">
                                            Trigger Metric
                                        </span>
                                    </div>

                                    <div className="relative mt-3">
                                        <select
                                            value={currentConfig.target_model === 'gemini-pro' || currentConfig.target_model === 'gemini-3.8-flash' ? 'gemini-3.8-flash-high' : currentConfig.target_model}
                                            onChange={(e) => onChange({ ...currentConfig, target_model: e.target.value })}
                                            className="w-full appearance-none px-3 py-2 pr-9 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-medium text-slate-800 dark:text-slate-100 shadow-2xs focus:outline-none focus:ring-2 focus:ring-blue-500/30 transition-all cursor-pointer"
                                        >
                                            <option value="gemini-3.8-flash-high">Gemini 3.8 Flash High (Primary, Recommended)</option>
                                            <option value="claude-sonnet-4.6">Claude Sonnet 4.6 (Failover / Fallback)</option>
                                            <option value="gemini-pro">Gemini Pro</option>
                                        </select>
                                        <ChevronDown className="w-4 h-4 text-slate-400 pointer-events-none absolute right-3 top-2.5" />
                                    </div>

                                    {/* Contextual Model Traits / Hints */}
                                    <div className="mt-3 p-2.5 rounded-lg bg-blue-50/60 dark:bg-slate-900/60 border border-blue-100/80 dark:border-slate-700/60 text-[11px] space-y-1">
                                        {(currentConfig.target_model === 'gemini-3.8-flash-high' || currentConfig.target_model === 'gemini-3.8-flash' || currentConfig.target_model === 'gemini-pro') && (
                                            <div className="flex items-start gap-1.5 text-blue-700 dark:text-blue-300">
                                                <Sparkles className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                                <span><strong>High Throughput &amp; Large Budget:</strong> Gemini 3.8 Flash High polls every 5m (&ge;15%), 1m (&lt;15%), and 40s (&le;12%), automatically rotating and dispatching prompts + plaintext email alerts.</span>
                                            </div>
                                        )}
                                        {currentConfig.target_model === 'claude-sonnet-4.6' && (
                                            <div className="flex items-start gap-1.5 text-purple-700 dark:text-purple-300">
                                                <Sparkles className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                                <span><strong>Deep Reasoning Budget:</strong> Failover activates when Claude Sonnet extended thinking budget hits low threshold.</span>
                                            </div>
                                        )}
                                        {currentConfig.target_model === 'gemini-pro' && (
                                            <div className="flex items-start gap-1.5 text-slate-700 dark:text-slate-300">
                                                <Sparkles className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                                <span><strong>Standard Chat Baseline:</strong> Evaluates standard Gemini Pro quota headroom across all profile slots.</span>
                                            </div>
                                        )}
                                    </div>
                                </div>

                                {/* Rotation Cooldown Period Setting */}
                                <div className="pt-3 border-t border-slate-200 dark:border-slate-700/60">
                                    <div className="flex justify-between items-center text-xs font-semibold text-slate-800 dark:text-slate-200 mb-1.5">
                                        <span className="flex items-center gap-1.5">
                                            <Clock className="w-3.5 h-3.5 text-slate-500" />
                                            <span>Rotation Cooldown Guard</span>
                                        </span>
                                        <span className="text-blue-600 dark:text-blue-400 font-mono font-bold">
                                            {Math.round((currentConfig.cooldown_seconds || 180) / 60)} min ({currentConfig.cooldown_seconds || 180}s)
                                        </span>
                                    </div>
                                    <input
                                        type="range"
                                        min="60"
                                        max="600"
                                        step="30"
                                        value={currentConfig.cooldown_seconds || 180}
                                        onChange={(e) => onChange({ ...currentConfig, cooldown_seconds: Number(e.target.value) })}
                                        className="w-full accent-blue-600 cursor-pointer"
                                    />
                                    <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono mt-1">
                                        <span>1 min</span>
                                        <span>3 min (Default)</span>
                                        <span>5 min</span>
                                        <span>10 min</span>
                                    </div>
                                </div>

                                {/* Account Reuse Cooldown Window Setting */}
                                <div className="pt-3 border-t border-slate-200 dark:border-slate-700/60">
                                    <div className="flex justify-between items-center text-xs font-semibold text-slate-800 dark:text-slate-200 mb-1.5">
                                        <span className="flex items-center gap-1.5">
                                            <Clock className="w-3.5 h-3.5 text-indigo-500" />
                                            <span>Account Reuse Cooldown (Minutes)</span>
                                        </span>
                                        <span className="text-indigo-600 dark:text-indigo-400 font-mono font-bold">
                                            {currentConfig.account_cooldown_minutes ?? 60} min
                                        </span>
                                    </div>
                                    <div className="relative mt-1">
                                        <select
                                            value={currentConfig.account_cooldown_minutes ?? 60}
                                            onChange={(e) => {
                                                const val = Number(e.target.value);
                                                onChange({
                                                    ...currentConfig,
                                                    account_cooldown_minutes: val,
                                                    account_lockout_window_minutes: val,
                                                });
                                            }}
                                            className="w-full appearance-none px-3 py-1.5 pr-8 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 rounded-[5px] text-xs font-medium text-slate-800 dark:text-slate-200 shadow-2xs focus:outline-none focus:ring-2 focus:ring-indigo-500/30 cursor-pointer"
                                        >
                                            <option value={15}>15 Minutes</option>
                                            <option value={30}>30 Minutes</option>
                                            <option value={45}>45 Minutes</option>
                                            <option value={60}>60 Minutes (Default)</option>
                                            <option value={120}>120 Minutes (2 Hours)</option>
                                        </select>
                                        <ChevronDown className="w-3.5 h-3.5 text-slate-400 pointer-events-none absolute right-2.5 top-2.5" />
                                    </div>
                                    <div className="flex gap-1.5 pt-1.5 flex-wrap">
                                        {[15, 30, 45, 60, 120].map((mins) => (
                                            <button
                                                key={mins}
                                                type="button"
                                                onClick={() =>
                                                    onChange({
                                                        ...currentConfig,
                                                        account_cooldown_minutes: mins,
                                                        account_lockout_window_minutes: mins,
                                                    })
                                                }
                                                className={`px-2 py-0.5 text-[10px] font-mono font-medium rounded-[5px] border transition-all cursor-pointer ${
                                                    (currentConfig.account_cooldown_minutes ?? 60) === mins
                                                        ? 'bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border-indigo-300 dark:border-indigo-800 shadow-xs'
                                                        : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-slate-300'
                                                }`}
                                            >
                                                {mins}m
                                            </button>
                                        ))}
                                    </div>
                                    <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed pt-1">
                                        Enforces cross-machine lease lock and skips recently used accounts until cooldown expires, with automatic fallback if all accounts are cooling down.
                                    </p>
                                </div>
                            </div>

        </>
    );
}
