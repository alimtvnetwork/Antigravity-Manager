import { Zap, Sliders, CheckCircle2, Sparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../../utils/cn';
import type { InstanceSettingsApi } from './instanceSettingsTypes';

export function SettingsQuickToggles({ api }: { api: InstanceSettingsApi }) {
    const { t } = useTranslation();
    const { isTurboMode, isAlwaysProceed, isOperating, handleSetTurboMode, handleSetPlanReview, handleEnforceDefaults } = api;
    void t;

    return (
        <div>
            <span className="block text-[11px] font-bold text-gray-400 uppercase tracking-wider mb-2">
                Common Automation Settings
            </span>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {/* Turbo Mode Card */}
                <div className="p-3.5 rounded-xl border border-gray-200 dark:border-[#15334d] bg-gray-50/60 dark:bg-[#071a27]/60 flex flex-col justify-between space-y-2.5">
                    <div className="flex items-start justify-between gap-2">
                        <div className="flex items-center gap-2">
                            <div className="p-1.5 rounded-lg bg-amber-500/15 text-amber-600 dark:text-amber-400 shrink-0">
                                <Zap className="w-4 h-4 fill-current" />
                            </div>
                            <div>
                                <div className="font-bold text-xs text-gray-900 dark:text-gray-100">Turbo Mode</div>
                                <div className="text-[10px] text-gray-500 dark:text-gray-400">Auto-confirm safe edits</div>
                            </div>
                        </div>
                        <span
                            className={cn(
                                'px-2 py-0.5 rounded text-[10px] font-bold uppercase shrink-0',
                                isTurboMode
                                    ? 'bg-amber-500/20 text-amber-700 dark:text-amber-300 border border-amber-500/30'
                                    : 'bg-gray-200 dark:bg-slate-800 text-gray-600 dark:text-gray-400'
                            )}
                        >
                            {isTurboMode ? 'ON' : 'OFF'}
                        </span>
                    </div>

                    <div className="flex items-center rounded-lg bg-gray-100 dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs">
                        <button
                            type="button"
                            disabled={isOperating}
                            onClick={() => handleSetTurboMode(false)}
                            title={isTurboMode ? 'Click to disable Auto-Confirm (Turbo Mode)' : 'Click to enable Auto-Confirm (Turbo Mode)'}
                            className={cn(
                                'flex-1 py-1 px-2.5 rounded-md text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5',
                                isTurboMode
                                    ? 'bg-amber-600 text-white shadow-xs'
                                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]'
                            )}
                        >
                            <Zap className="w-3.5 h-3.5 fill-current" />
                            <span>Auto-Confirm: {isTurboMode ? 'ON' : 'OFF'}</span>
                        </button>
                        <button
                            type="button"
                            disabled={isOperating}
                            onClick={() => handleSetTurboMode(true)}
                            className="py-1 px-2.5 text-xs font-medium text-blue-600 dark:text-cyan-400 hover:bg-gray-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer shrink-0 flex items-center gap-1"
                            title="Apply this Auto-Confirm state across all instances"
                        >
                            <Sliders className="w-3 h-3" />
                            <span>Apply All</span>
                        </button>
                    </div>
                </div>

                {/* Plan Review Card */}
                <div className="p-3.5 rounded-xl border border-gray-200 dark:border-[#15334d] bg-gray-50/60 dark:bg-[#071a27]/60 flex flex-col justify-between space-y-2.5">
                    <div className="flex items-start justify-between gap-2">
                        <div className="flex items-center gap-2">
                            <div className="p-1.5 rounded-lg bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 shrink-0">
                                <CheckCircle2 className="w-4 h-4" />
                            </div>
                            <div>
                                <div className="font-bold text-xs text-gray-900 dark:text-gray-100">Plan Review</div>
                                <div className="text-[10px] text-gray-500 dark:text-gray-400">Proceed vs require prompt</div>
                            </div>
                        </div>
                        <span
                            className={cn(
                                'px-2 py-0.5 rounded text-[10px] font-bold shrink-0',
                                isAlwaysProceed
                                    ? 'bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30'
                                    : 'bg-blue-500/20 text-blue-700 dark:text-blue-300 border border-blue-500/30'
                            )}
                        >
                            {isAlwaysProceed ? 'Always Proceed' : 'Ask Permission'}
                        </span>
                    </div>

                    <div className="flex items-center rounded-lg bg-gray-100 dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs">
                        <button
                            type="button"
                            disabled={isOperating}
                            onClick={() => handleSetPlanReview(false, !isAlwaysProceed)}
                            title={isAlwaysProceed ? 'Click to require permission before executing plans' : 'Click to always proceed without asking'}
                            className={cn(
                                'flex-1 py-1 px-2.5 rounded-md text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5',
                                isAlwaysProceed ? 'bg-emerald-600 text-white shadow-xs' : 'bg-blue-600 text-white shadow-xs'
                            )}
                        >
                            <CheckCircle2 className="w-3.5 h-3.5" />
                            <span>{isAlwaysProceed ? 'Always Proceed' : 'Ask Permission'}</span>
                        </button>
                        <button
                            type="button"
                            disabled={isOperating}
                            onClick={() => handleSetPlanReview(true, isAlwaysProceed)}
                            className="py-1 px-2.5 text-xs font-medium text-blue-600 dark:text-cyan-400 hover:bg-gray-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer shrink-0 flex items-center gap-1"
                            title="Apply current Plan Review setting across all instances"
                        >
                            <Sliders className="w-3 h-3" />
                            <span>Apply All</span>
                        </button>
                    </div>
                </div>
            </div>

            {/* Enforce Baseline Defaults Action */}
            <div className="mt-3 p-3 rounded-xl border border-gray-200 dark:border-[#15334d] bg-gray-50/40 dark:bg-[#071a27]/40 flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-2">
                    <Sparkles className="w-4 h-4 text-indigo-500 shrink-0" />
                    <div>
                        <div className="font-semibold text-xs text-gray-900 dark:text-gray-100">
                            Default Baseline Alignment
                        </div>
                        <div className="text-[10px] text-gray-500 dark:text-gray-400">
                            Reset and propagate standard baseline settings & themes from default profile
                        </div>
                    </div>
                </div>
                <div className="flex items-center rounded-lg bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs shrink-0">
                    <button
                        type="button"
                        disabled={isOperating}
                        onClick={() => handleEnforceDefaults(false)}
                        title="Enforce baseline settings onto target profile"
                        className="px-2.5 py-1 text-xs font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-l-md transition-colors cursor-pointer flex items-center gap-1"
                    >
                        <Sparkles className="w-3 h-3 text-indigo-500" />
                        <span>Enforce Target</span>
                    </button>
                    <button
                        type="button"
                        disabled={isOperating}
                        onClick={() => handleEnforceDefaults(true)}
                        title="Enforce baseline settings across all profiles"
                        className="px-2.5 py-1 text-xs font-semibold text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 rounded-r-md transition-colors cursor-pointer flex items-center gap-1"
                    >
                        <Sliders className="w-3 h-3" />
                        <span>Enforce All</span>
                    </button>
                </div>
            </div>
        </div>
    );
}
