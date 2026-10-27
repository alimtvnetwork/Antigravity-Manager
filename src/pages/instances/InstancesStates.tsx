import { copyToClipboard } from '../../utils/clipboard';
import { useState } from 'react';
import { Search, Plus, Play, RotateCw, Laptop, AlertCircle, Terminal, ChevronDown, ChevronRight, Copy, Check, X } from 'lucide-react';
import type { InstancePageApi } from './instancePageTypes';
import { forceDispatchPendingResume } from '../../services/instanceService';

export function InstancesFilterBar({ api }: { api: InstancePageApi }) {
    const { t, searchQuery, setSearchQuery, setNewInstanceName, setIsCreateOpen } = api;

    return (
        <div className="flex items-center justify-between gap-3 flex-wrap">
            <div className="flex items-center gap-2 flex-1 max-w-md bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 rounded-[5px] px-3 py-2 shadow-xs">
                <Search className="w-4 h-4 text-gray-400 shrink-0" />
                <input
                    type="text"
                    placeholder={t('instances.search_placeholder', 'Search by profile name, ID, or bound email...')}
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full bg-transparent border-none outline-hidden text-xs text-gray-900 dark:text-base-content"
                />
                {Boolean(searchQuery) && (
                    <button
                        type="button"
                        onClick={() => setSearchQuery('')}
                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-0.5 rounded-full hover:bg-gray-100 dark:hover:bg-slate-700/60 transition-colors cursor-pointer"
                        title={t('common.clear', 'Clear search')}
                        aria-label={t('common.clear', 'Clear search')}
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                )}
            </div>
            <button
                onClick={() => {
                    if (api.openCreateDialog) {
                        api.openCreateDialog();
                    } else {
                        const defaultId = api.instances.find((i) => i.config.is_default || i.config.id === 'default')?.config.id || 'default';
                        api.setNewInstanceFromInstance(defaultId);
                        setNewInstanceName('');
                        setIsCreateOpen(true);
                    }
                }}
                className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[4px] group hover:shadow-md hover:shadow-primary/25 hover:-translate-y-0.5 active:translate-y-0 active:scale-[0.98] transition-all duration-200 ease-out"
            >
                <Plus className="w-4 h-4 transition-transform duration-300 group-hover:rotate-90 group-hover:scale-110" />
                <span>{t('instances.create_btn', 'New Instance')}</span>
            </button>
        </div>
    );
}

export function InstancesAlerts({ api }: { api: InstancePageApi }) {
    const { t, storeError, actionError, setActionError, fetchInstances, resumeIndicator } = api;

    const isSettling =
        resumeIndicator &&
        (resumeIndicator.status === 'scheduled' ||
            (resumeIndicator.secondsLeft !== undefined && resumeIndicator.secondsLeft > 0));

    const handleForceDispatch = async () => {
        if (!resumeIndicator?.instanceId) return;
        await forceDispatchPendingResume(resumeIndicator.instanceId);
    };

    return (
        <>
            {isSettling ? (
                <div className="p-3.5 rounded-[5px] bg-amber-500/10 dark:bg-amber-500/15 border border-amber-500/30 text-amber-800 dark:text-amber-200 text-xs flex items-center justify-between shadow-xs">
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-amber-500 shadow-[0_0_6px_rgba(245,158,11,0.6)] animate-ping shrink-0" />
                        <span className="font-medium">
                            {t(
                                'instances.settling_banner',
                                `Post-switch settling delay active: Restoring prompts in ${resumeIndicator.secondsLeft ?? 10}s for instance '${resumeIndicator.instanceId}'...`
                            )}
                        </span>
                    </div>
                    <button
                        onClick={handleForceDispatch}
                        className="btn btn-xs rounded-full bg-amber-600 hover:bg-amber-700 text-white border-none gap-1 font-semibold px-3 shadow-xs transition-colors cursor-pointer"
                        title="Bypass 10s settling delay and restore prompts immediately"
                    >
                        <Play className="w-3 h-3 fill-current" />
                        <span>{t('instances.force_restore', 'Force Dispatch Now')}</span>
                    </button>
                </div>
            ) : null}
            {storeError ? (
                <div className="p-3.5 rounded-[5px] bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{String(storeError)}</span>
                    </div>
                    <button
                        onClick={() => fetchInstances()}
                        className="btn btn-xs btn-outline btn-error gap-1 shrink-0 rounded-[5px]"
                    >
                        <RotateCw className="w-3 h-3" />
                        <span>{t('common.retry', 'Retry')}</span>
                    </button>
                </div>
            ) : null}

            {actionError ? (
                <div className="p-3.5 rounded-[5px] bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{actionError}</span>
                    </div>
                    <button onClick={() => setActionError(null)} className="text-xs font-semibold">
                        ✕
                    </button>
                </div>
            ) : null}
        </>
    );
}

export function InstancesLoadingState({ api }: { api: InstancePageApi }) {
    const { t } = api;
    return (
        <div className="flex flex-col items-center justify-center py-20 bg-white dark:bg-base-200 rounded-[5px] border border-gray-200/80 dark:border-base-100">
            <RotateCw className="w-8 h-8 text-blue-600 dark:text-blue-400 animate-spin mb-3" />
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('instances.loading', 'Loading instances and profiles...')}
            </p>
        </div>
    );
}

export function InstancesEmptyState({ api }: { api: InstancePageApi }) {
    const { t, setNewInstanceName, setIsCreateOpen } = api;

    const { handleInitDefault } = api;

    return (
        <div className="flex flex-col items-center justify-center py-16 px-4 text-center bg-white dark:bg-base-200 rounded-[5px] border border-dashed border-gray-300 dark:border-base-100">
            <div className="p-4 rounded-[5px] bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 mb-4">
                <Laptop className="w-10 h-10" />
            </div>
            <h3 className="text-base font-bold text-gray-900 dark:text-base-content mb-1">
                {t('instances.empty_title', 'No Profiles Found')}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400 max-w-md mb-6 leading-relaxed">
                {t(
                    'instances.empty_desc',
                    'Instances allow you to run isolated Antigravity windows with dedicated account tokens, extensions, and workspaces. Initialize your default profile or create a custom one to get started.'
                )}
            </p>
            <div className="flex items-center gap-3">
                <button onClick={handleInitDefault} className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[5px]">
                    <Play className="w-3.5 h-3.5" />
                    <span>{t('instances.init_default', 'Initialize Default Profile')}</span>
                </button>
                <button
                    onClick={() => {
                        if (api.openCreateDialog) {
                            api.openCreateDialog();
                        } else {
                            const defaultId = api.instances.find((i) => i.config.is_default || i.config.id === 'default')?.config.id || 'default';
                            api.setNewInstanceFromInstance(defaultId);
                            setNewInstanceName('');
                            setIsCreateOpen(true);
                        }
                    }}
                    className="btn btn-outline btn-sm gap-1.5 rounded-[5px] group hover:shadow-md hover:-translate-y-0.5 active:translate-y-0 active:scale-[0.98] transition-all duration-200 ease-out"
                >
                    <Plus className="w-3.5 h-3.5 transition-transform duration-300 group-hover:rotate-90 group-hover:scale-110" />
                    <span>{t('instances.create_custom', 'Create Custom Profile')}</span>
                </button>
            </div>
        </div>
    );
}

export function InstancesNoResultsState({ api }: { api: InstancePageApi }) {
    const { t, setSearchQuery } = api;
    return (
        <div className="flex flex-col items-center justify-center py-12 text-center bg-white dark:bg-base-200 rounded-[5px] border border-gray-200/80 dark:border-base-100">
            <Search className="w-8 h-8 text-gray-400 mb-2" />
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('instances.no_search_results', 'No profiles match your search')}
            </p>
            <p className="text-xs text-gray-400 mt-1 mb-4">
                {t('instances.no_search_results_desc', 'Try searching with a different name, profile ID, or email')}
            </p>
            <button
                type="button"
                onClick={() => setSearchQuery('')}
                className="btn btn-primary btn-sm rounded-[5px] shadow-sm gap-1.5"
            >
                <RotateCw className="w-3.5 h-3.5" />
                <span>{t('instances.reset_search', 'Reset Search')}</span>
            </button>
        </div>
    );
}

export function InstancesCliHelpCard() {
    const [isExpanded, setIsExpanded] = useState(false);
    const [copiedCmd, setCopiedCmd] = useState<string | null>(null);

    const commands = [
        {
            category: 'Lifecycle',
            cmd: 'agm list',
            desc: 'View active instances, PIDs, and running tasks',
            notes: 'Real-time status',
        },
        {
            category: 'Sync',
            cmd: 'agm sync',
            desc: 'Synchronize process PIDs and account quotas',
            notes: 'Automated multi-process audit',
        },
        {
            category: 'Switch',
            cmd: 'agm switch <id>',
            desc: 'Rotate profile, transfer credentials, and resume tasks',
            notes: 'Atomic handoff',
        },
        {
            category: 'Vault',
            cmd: 'agm backup',
            desc: 'Export encrypted profile backup archive',
            notes: 'AES-256-GCM',
        },
        {
            category: 'Vault',
            cmd: 'agm restore <file>',
            desc: 'Restore accounts and configurations from archive',
            notes: 'Safe merge',
        },
        {
            category: 'Hygiene',
            cmd: 'agm clean-restart',
            desc: 'Terminate orphaned processes and clean lockfiles',
            notes: 'Resolves port & lock conflicts',
        },
        {
            category: 'Prompt',
            cmd: 'agm prompts list',
            desc: 'Inspect historical and active prompts across profiles',
            notes: 'FIFO ordered',
        },
    ];

    const copyCommand = (cmd: string) => {
        copyToClipboard(cmd);
        setCopiedCmd(cmd);
        setTimeout(() => setCopiedCmd(null), 2000);
    };

    return (
        <div className="rounded-xl border border-gray-200/80 dark:border-[#40285e] bg-white/60 dark:bg-[#171025]/60 backdrop-blur-md shadow-xs overflow-hidden transition-all duration-200">
            <button
                type="button"
                onClick={() => setIsExpanded((prev) => !prev)}
                className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-gray-50/50 dark:hover:bg-[#40285e]/30 transition-colors cursor-pointer"
            >
                <div className="flex items-center gap-2.5">
                    <div className="p-1.5 rounded-lg bg-blue-500/10 text-blue-600 dark:text-cyan-400">
                        <Terminal className="w-4 h-4" />
                    </div>
                    <div>
                        <span className="text-xs font-bold text-gray-900 dark:text-white">
                            agm CLI Reference & Operator Shortcuts
                        </span>
                        <p className="text-[11px] text-gray-500 dark:text-gray-400">
                            Command-line interface commands for headless environments, scripts, and quick automation
                        </p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-600 dark:text-cyan-400 font-semibold">
                        7 Commands
                    </span>
                    {isExpanded ? (
                        <ChevronDown className="w-4 h-4 text-gray-400" />
                    ) : (
                        <ChevronRight className="w-4 h-4 text-gray-400" />
                    )}
                </div>
            </button>

            {isExpanded && (
                <div className="px-4 pb-4 pt-1 border-t border-gray-100 dark:border-[#40285e]/60 space-y-3">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                        {commands.map((c) => (
                            <div
                                key={c.cmd}
                                className="flex items-start justify-between gap-2 p-2.5 rounded-lg bg-gray-50 dark:bg-[#090611] border border-gray-200/70 dark:border-[#40285e]/70 group hover:border-blue-400/50 transition-colors"
                            >
                                <div className="space-y-1 min-w-0">
                                    <div className="flex items-center gap-1.5">
                                        <span className="px-1.5 py-0.2 rounded text-[9px] font-bold uppercase tracking-wider bg-slate-200 dark:bg-slate-800 text-slate-700 dark:text-slate-300">
                                            {c.category}
                                        </span>
                                        <code className="font-mono text-[11px] font-semibold text-blue-600 dark:text-cyan-400">
                                            {c.cmd}
                                        </code>
                                    </div>
                                    <p className="text-[11px] text-gray-600 dark:text-gray-300 line-clamp-1">
                                        {c.desc}
                                    </p>
                                    <span className="text-[10px] text-gray-400 dark:text-gray-500">
                                        {c.notes}
                                    </span>
                                </div>
                                <button
                                    type="button"
                                    onClick={() => copyCommand(c.cmd)}
                                    className="p-1 rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-200 dark:hover:bg-slate-800 transition-colors shrink-0 cursor-pointer"
                                    title="Copy command to clipboard"
                                >
                                    {copiedCmd === c.cmd ? (
                                        <Check className="w-3.5 h-3.5 text-emerald-500" />
                                    ) : (
                                        <Copy className="w-3.5 h-3.5" />
                                    )}
                                </button>
                            </div>
                        ))}
                    </div>

                    <div className="p-2.5 rounded-lg bg-blue-50/60 dark:bg-blue-950/20 border border-blue-200/60 dark:border-blue-900/40 text-[11px] text-blue-700 dark:text-blue-300 flex items-center gap-2">
                        <span className="font-semibold">💡 Tip:</span>
                        <span>
                            All instances run in conscious process sandboxes with isolated Electron storage. Use <code className="font-mono bg-blue-100 dark:bg-blue-900/40 px-1 py-0.5 rounded">agm switch &lt;id&gt;</code> to seamlessly rotate between profiles without losing active task contexts.
                        </span>
                    </div>
                </div>
            )}
        </div>
    );
}
