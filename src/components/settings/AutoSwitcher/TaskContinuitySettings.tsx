import { AutoSwitcherApi } from './useAutoSwitcher';

export function TaskContinuitySettings(props: AutoSwitcherApi) {
    const { t, currentConfig, onChange } = props;

    return (
        <>
                            <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 flex flex-col justify-between space-y-3 shadow-2xs">
                                <div>
                                    <div className="flex items-center justify-between mb-2">
                                        <div className="flex items-center gap-2">
                                            <div className="p-1.5 rounded-lg bg-emerald-100 dark:bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 border border-emerald-200/60 dark:border-emerald-500/30">
                                                <ShieldCheck className="w-4 h-4" />
                                            </div>
                                            <div>
                                                <h4 className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider">
                                                    Task Continuity &amp; Watchdog
                                                </h4>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                                    Zero-loss task resumption, prompt recovery, and process healing
                                                </p>
                                            </div>
                                        </div>
                                        <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-200/50 dark:border-emerald-500/20 font-semibold">
                                            Active Guard
                                        </span>
                                    </div>

                                    <div className="space-y-2 mt-3">
                                        {/* Toggle 1: Auto-Resume Pending Tasks */}
                                        <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                            <input
                                                type="checkbox"
                                                id="auto_resume_cb"
                                                checked={currentConfig.has_auto_resume}
                                                onChange={(e) => onChange({ ...currentConfig, has_auto_resume: e.target.checked })}
                                                className="checkbox checkbox-xs checkbox-primary rounded mt-0.5"
                                            />
                                            <div className="text-xs">
                                                <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                    {t('settings.auto_switcher.auto_resume_label', 'Snapshot and auto-resume pending tasks on restart')}
                                                </span>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                    Preserves in-flight task queue state and restores session context upon account swap.
                                                </p>
                                            </div>
                                        </label>

                                        {/* Toggle 2: Fast Forward on Critical */}
                                        <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                            <input
                                                type="checkbox"
                                                id="auto_ff_cb"
                                                checked={currentConfig.auto_fast_forward_on_critical ?? true}
                                                onChange={(e) => onChange({ ...currentConfig, auto_fast_forward_on_critical: e.target.checked })}
                                                className="checkbox checkbox-xs checkbox-secondary rounded mt-0.5"
                                            />
                                            <div className="text-xs">
                                                <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                    Auto-trigger Fast-Forward when credits drop &le; 12%
                                                </span>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                    Instantly delegates to the highest-credit candidate without waiting for full exhaustion.
                                                </p>
                                            </div>
                                        </label>

                                        {/* Toggle 3: Auto-Resume Recent Active Prompts */}
                                        <div className="space-y-1.5">
                                            <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                                <input
                                                    type="checkbox"
                                                    id="auto_resume_recent_cb"
                                                    checked={currentConfig.auto_resume_recent_prompts ?? true}
                                                    onChange={(e) => onChange({ ...currentConfig, auto_resume_recent_prompts: e.target.checked })}
                                                    className="checkbox checkbox-xs checkbox-accent rounded mt-0.5"
                                                />
                                                <div className="text-xs">
                                                    <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                        Auto-Resume Recent Active Prompts on Fast-Forward
                                                    </span>
                                                    <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                        Transfers recent prompt history and attached images into freshly booted IDE instance.
                                                    </p>
                                                </div>
                                            </label>

                                            {/* Recency Cutoff Window Sub-Panel */}
                                            {(currentConfig.auto_resume_recent_prompts ?? true) && (
                                                <div className="ml-7 p-3 bg-slate-100/90 dark:bg-slate-900/90 rounded-xl border border-slate-200 dark:border-slate-700/80 space-y-2 animate-in fade-in slide-in-from-top-1 duration-150 shadow-2xs">
                                                    <div className="flex items-center justify-between gap-3">
                                                        <span className="flex items-center gap-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
                                                            <Clock className="w-3.5 h-3.5 text-cyan-500" />
                                                            <span>Recency Cutoff Window:</span>
                                                        </span>
                                                        <div className="relative">
                                                            <select
                                                                value={currentConfig.prompt_recency_threshold_seconds || 3600}
                                                                onChange={(e) => onChange({
                                                                    ...currentConfig,
                                                                    prompt_recency_threshold_seconds: Number(e.target.value),
                                                                })}
                                                                className="appearance-none px-2.5 py-1 pr-7 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-medium text-slate-800 dark:text-slate-200 shadow-2xs focus:outline-none focus:ring-2 focus:ring-cyan-500/30 cursor-pointer"
                                                            >
                                                                <option value={1800}>30 Minutes</option>
                                                                <option value={3600}>1 Hour (Recommended)</option>
                                                                <option value={7200}>2 Hours</option>
                                                                <option value={18000}>5 Hours</option>
                                                                <option value={86400}>1 Day (24 Hours)</option>
                                                            </select>
                                                            <ChevronDown className="w-3.5 h-3.5 text-slate-400 pointer-events-none absolute right-2 top-1.5" />
                                                        </div>
                                                    </div>
                                                    <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                                                        Projects executed within this timeframe are seamlessly restored with full prompt context. Older idle workspaces are safely bypassed for a clean startup.
                                                    </p>
                                                </div>
                                            )}
                                        </div>

                                        {/* Toggle: Auto-Reopen Workspace on Profile Switch */}
                                        <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                            <input
                                                type="checkbox"
                                                id="auto_reopen_workspace_cb"
                                                checked={currentConfig.auto_reopen_on_switch ?? true}
                                                onChange={(e) => onChange({ ...currentConfig, auto_reopen_on_switch: e.target.checked })}
                                                className="checkbox checkbox-xs checkbox-primary rounded mt-0.5"
                                            />
                                            <div className="text-xs">
                                                <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                    Auto-Reopen Workspace on Profile Switch
                                                </span>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                    Automatically reopens the active IDE workspace window in the target instance upon quota rotation.
                                                </p>
                                            </div>
                                        </label>

                                        {/* Toggle 4: 2-Minute IDE Crash & Focus Watchdog */}
                                        <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                            <input
                                                type="checkbox"
                                                id="auto_focus_cb"
                                                checked={currentConfig.auto_focus_window ?? true}
                                                onChange={(e) => onChange({ ...currentConfig, auto_focus_window: e.target.checked })}
                                                className="checkbox checkbox-xs checkbox-info rounded mt-0.5"
                                            />
                                            <div className="text-xs">
                                                <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                    2-Minute IDE Crash &amp; Focus Watchdog
                                                </span>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                    Auto-focuses foreground PID and automatically relaunches non-responsive IDE windows.
                                                </p>
                                            </div>
                                        </label>

                                        {/* Fast-Forward Shortcut Configuration */}
                                        <div className="p-3 bg-slate-100/90 dark:bg-slate-900/90 rounded-xl border border-slate-200 dark:border-slate-700/80 space-y-2 shadow-2xs">
                                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                                                <span className="flex items-center gap-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
                                                    <FastForward className="w-3.5 h-3.5 text-blue-500" />
                                                    <span>Fast-Forward Shortcut Key:</span>
                                                </span>
                                                <div className="flex items-center gap-2">
                                                    <input
                                                        type="text"
                                                        value={currentConfig.fast_forward_shortcut || 'Ctrl+Shift+F'}
                                                        onChange={(e) => onChange({
                                                            ...currentConfig,
                                                            fast_forward_shortcut: e.target.value.trim() || 'Ctrl+Shift+F',
                                                        })}
                                                        placeholder="Ctrl+Shift+F"
                                                        className="w-32 px-2.5 py-1 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-mono font-bold text-center text-blue-600 dark:text-blue-400 shadow-2xs focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                                                    />
                                                    <button
                                                        type="button"
                                                        onClick={() => onChange({
                                                            ...currentConfig,
                                                            fast_forward_shortcut: 'Ctrl+Shift+F',
                                                        })}
                                                        className="btn btn-ghost btn-xs text-[10px] text-gray-500 hover:text-blue-600 cursor-pointer"
                                                        title="Reset to default (Ctrl+Shift+F)"
                                                    >
                                                        Reset
                                                    </button>
                                                </div>
                                            </div>
                                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1.5 text-[11px] text-slate-500 dark:text-slate-400">
                                                <span>Triggers smart account rotation and active window fast-forward from anywhere in the app.</span>
                                                <div className="flex items-center gap-1.5 shrink-0">
                                                    <button
                                                        type="button"
                                                        onClick={() => onChange({ ...currentConfig, fast_forward_shortcut: 'Ctrl+Shift+F' })}
                                                        className="px-1.5 py-0.5 rounded text-[10px] bg-slate-200/70 dark:bg-slate-800 font-mono hover:bg-blue-100 hover:text-blue-600 transition-colors"
                                                    >
                                                        Ctrl+Shift+F
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onClick={() => onChange({ ...currentConfig, fast_forward_shortcut: 'Alt+Shift+F' })}
                                                        className="px-1.5 py-0.5 rounded text-[10px] bg-slate-200/70 dark:bg-slate-800 font-mono hover:bg-blue-100 hover:text-blue-600 transition-colors"
                                                    >
                                                        Alt+Shift+F
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onClick={() => onChange({ ...currentConfig, fast_forward_shortcut: 'Ctrl+Alt+F' })}
                                                        className="px-1.5 py-0.5 rounded text-[10px] bg-slate-200/70 dark:bg-slate-800 font-mono hover:bg-blue-100 hover:text-blue-600 transition-colors"
                                                    >
                                                        Ctrl+Alt+F
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
        </>
    );
}
