import {
    Laptop,
    RotateCw,
    Plus,
    FolderSync,
    Sparkles,
    SlidersHorizontal,
    LayoutGrid,
    Grid3X3,
    List,
    ToggleLeft,
    ToggleRight,
} from 'lucide-react';
import { cn } from '../../utils/cn';
import type { InstancePageApi } from './instancePageTypes';

export function InstancesHeader({ api }: { api: InstancePageApi }) {
    const {
        t,
        instances,
        runningCount,
        isLoading,
        switcherStatus,
        daemonStatus,
        daemonCountdown,
        viewMode,
        cardDensity,
        fetchInstances,
        fetchSwitcherStatus,
        handleSetViewMode,
        handleSetCardDensity,
        handleSyncAll,
        handleCleanRestart,
        handleToggleAutoSwitcher,
        handleTriggerRotation,
        setNewInstanceName,
        setIsCreateOpen,
        setSettingsModalTarget,
        setIsSettingsModalOpen,
        isSyncingAll,
    } = api;

    const switcher = switcherStatus as { is_running?: boolean } | null;

    return (
        <div className="space-y-3">
            <div className="flex items-center gap-2.5 min-w-0">
                <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 shrink-0">
                    <Laptop className="w-5 h-5" />
                </div>
                <div className="min-w-0">
                    <h1 className="text-lg sm:text-xl font-bold text-gray-900 dark:text-base-content">
                        {t('instances.page_title', 'Instances & Profiles')}
                    </h1>
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                        {t('instances.page_desc', 'Run multiple Antigravity windows in parallel with isolated credentials and extensions')}
                    </p>
                </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-3">
                {/* Segmented Group 1: Status & Maintenance */}
                <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                    <div className="px-3 py-1 text-xs text-slate-700 dark:text-slate-300 font-semibold whitespace-nowrap rounded-l-[4px] rounded-r-none">
                        {t('instances.running_summary', 'Running {{running}} of {{total}}', {
                            running: runningCount,
                            total: instances.length,
                        })}
                    </div>
                    <button
                        type="button"
                        onClick={() => {
                            fetchInstances();
                            fetchSwitcherStatus();
                        }}
                        disabled={isLoading}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title={t('common.refresh', 'Refresh')}
                    >
                        <RotateCw className={cn('w-3.5 h-3.5 text-blue-500', isLoading && 'animate-spin')} />
                        <span>{t('common.refresh', 'Refresh')}</span>
                    </button>
                    <button
                        type="button"
                        onClick={handleSyncAll}
                        disabled={isLoading || isSyncingAll}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Synchronize process PIDs and account quotas across all instances"
                    >
                        <FolderSync className={cn('w-3.5 h-3.5 text-cyan-500', isSyncingAll && 'animate-spin')} />
                        <span>Sync All</span>
                    </button>
                    <button
                        type="button"
                        onClick={handleCleanRestart}
                        disabled={isLoading}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-amber-700 dark:text-amber-400 hover:bg-amber-100/50 dark:hover:bg-amber-950/40 rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Force-terminate lingering background Electron/Antigravity processes, purge lockfiles, and cleanly relaunch Antigravity"
                    >
                        <Sparkles className="w-3.5 h-3.5" />
                        <span>Clean & Restart</span>
                    </button>
                </div>

                {/* Segmented Group 2: Automation & Settings */}
                <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                    <button
                        type="button"
                        onClick={handleToggleAutoSwitcher}
                        disabled={isLoading}
                        className={cn(
                            'flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            switcher?.is_running
                                ? 'text-emerald-700 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40'
                                : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-[#15334d]'
                        )}
                        title="Toggle background auto-profile switcher daemon"
                    >
                        {switcher?.is_running ? (
                            <ToggleRight className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                        ) : (
                            <ToggleLeft className="w-4 h-4 text-slate-400" />
                        )}
                        <span>Auto-Switch: {switcher?.is_running ? 'ON' : 'OFF'}</span>
                    </button>
                    {switcher?.is_running && (
                        <div
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-cyan-300 rounded-none"
                            title={`Auto-Switcher daemon next check in ${daemonCountdown}s (${(daemonStatus as { current_stage?: string })?.current_stage || 'Normal'} stage, interval ${(daemonStatus as { check_interval_seconds?: number })?.check_interval_seconds || 60}s)`}
                        >
                            <span className="relative flex h-2 w-2 shrink-0">
                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)] animate-pulse"></span>
                            </span>
                            <span className="font-mono text-[11px] whitespace-nowrap">⏱ {daemonCountdown}s Next Check</span>
                        </div>
                    )}
                    <button
                        type="button"
                        onClick={handleTriggerRotation}
                        disabled={isLoading}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Evaluate rolling quota across all monitored instances and auto-rotate any low quota accounts"
                    >
                        <Sparkles className={cn('w-3.5 h-3.5 text-amber-500', isLoading && 'animate-spin')} />
                        <span>Eval Quota</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => {
                            setSettingsModalTarget(null);
                            setIsSettingsModalOpen(true);
                        }}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Instance Settings & Sync: Turbo mode, plan review, copy settings, folder sync, JSON tools"
                    >
                        <SlidersHorizontal className="w-3.5 h-3.5 text-blue-500" />
                        <span>Settings & Sync</span>
                    </button>
                </div>

                {/* Segmented Group 3: View Mode & Creation */}
                <div className="flex items-center gap-2">
                    {viewMode === 'card' && (
                        <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                            <button
                                type="button"
                                onClick={() => handleSetCardDensity('normal')}
                                className={cn(
                                    'flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                                    cardDensity === 'normal'
                                        ? 'bg-white dark:bg-[#15334d] text-blue-600 dark:text-cyan-300 shadow-xs'
                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                                )}
                                title="Normal Cards"
                            >
                                <LayoutGrid className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline">Normal Cards</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => handleSetCardDensity('compact')}
                                className={cn(
                                    'flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                                    cardDensity === 'compact'
                                        ? 'bg-white dark:bg-[#15334d] text-blue-600 dark:text-cyan-300 shadow-xs'
                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                                )}
                                title="Compact Cards"
                            >
                                <Grid3X3 className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline">Compact Cards</span>
                            </button>
                        </div>
                    )}

                    <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                        <button
                            type="button"
                            onClick={() => handleSetViewMode('card')}
                            className={cn(
                                'flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                                viewMode === 'card'
                                    ? 'bg-white dark:bg-[#15334d] text-blue-600 dark:text-cyan-300 shadow-xs'
                                    : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                            )}
                            title="Card View"
                        >
                            <LayoutGrid className="w-3.5 h-3.5" />
                            <span className="hidden sm:inline">Cards</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => handleSetViewMode('list')}
                            className={cn(
                                'flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                                viewMode === 'list'
                                    ? 'bg-white dark:bg-[#15334d] text-blue-600 dark:text-cyan-300 shadow-xs'
                                    : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                            )}
                            title="List View"
                        >
                            <List className="w-3.5 h-3.5" />
                            <span className="hidden sm:inline">List</span>
                        </button>
                    </div>

                    <button
                        type="button"
                        onClick={() => {
                            setNewInstanceName('');
                            setIsCreateOpen(true);
                        }}
                        className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-[4px] bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-xs cursor-pointer transition-all duration-150 ease-out active:scale-[0.98]"
                    >
                        <Plus className="w-3.5 h-3.5" />
                        <span>{t('instances.create_btn', 'New Instance')}</span>
                    </button>
                </div>
            </div>
        </div>
    );
}
