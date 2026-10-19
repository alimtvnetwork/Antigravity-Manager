import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
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
    FolderArchive,
    ChevronDown,
    FileDown,
    FileUp,
    RotateCcw,
    Mail,
    Layers,
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { showToast } from '../../components/common/ToastContainer';
import {
    downloadPromptTreeStatusJson,
    openAndRestorePromptBackupFile,
    resendPromptsForInstance,
} from '../../components/instances/PromptTreeView/usePromptTreeCoreActions';
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
        setPromptTreeInstance,
        isSyncingAll,
        accounts,
        activeInstanceId,
    } = api;

    const switcher = switcherStatus as { is_running?: boolean } | null;

    const [isImportExportOpen, setIsImportExportOpen] = useState(false);
    const [isRunningDetailsOpen, setIsRunningDetailsOpen] = useState(false);
    const [isExporting, setIsExporting] = useState(false);
    const [isResending, setIsResending] = useState(false);

    const importExportBtnRef = useRef<HTMLButtonElement>(null);
    const runningBadgeRef = useRef<HTMLButtonElement>(null);
    const [importExportPos, setImportExportPos] = useState<{ top: number; left: number } | null>(null);
    const [runningDetailsPos, setRunningDetailsPos] = useState<{ top: number; left: number } | null>(null);

    const updateImportExportPos = () => {
        if (importExportBtnRef.current) {
            const rect = importExportBtnRef.current.getBoundingClientRect();
            setImportExportPos({
                top: rect.bottom + 6,
                left: Math.max(10, Math.min(rect.left, window.innerWidth - 340)),
            });
        }
    };

    const updateRunningDetailsPos = () => {
        if (runningBadgeRef.current) {
            const rect = runningBadgeRef.current.getBoundingClientRect();
            setRunningDetailsPos({
                top: rect.bottom + 6,
                left: Math.max(10, Math.min(rect.left, window.innerWidth - 420)),
            });
        }
    };

    useEffect(() => {
        if (!isImportExportOpen && !isRunningDetailsOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsImportExportOpen(false);
                setIsRunningDetailsOpen(false);
            }
        };
        const handleScrollOrResize = () => {
            if (isImportExportOpen) updateImportExportPos();
            if (isRunningDetailsOpen) updateRunningDetailsPos();
        };
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('resize', handleScrollOrResize);
        window.addEventListener('scroll', handleScrollOrResize, true);
        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('resize', handleScrollOrResize);
            window.removeEventListener('scroll', handleScrollOrResize, true);
        };
    }, [isImportExportOpen, isRunningDetailsOpen]);

    const runningInstances = instances.filter((i) => i.is_running);
    const runningInstancesDetails = runningInstances.map((i, idx) => {
        const acc = accounts.find((a) => a.id === i.config.bound_account_id);
        const email = i.config.bound_email || acc?.email || 'No email bound';
        const seq = i.config.seq_num ?? (idx + 1);
        return {
            id: i.config.id,
            name: i.config.name,
            seq,
            email,
            pid: i.pid,
            memoryMb: i.memory_mb,
            executablePath: i.config.executable_path,
        };
    });

    const runningTooltipText =
        runningInstancesDetails.length > 0
            ? `Running Instances (${runningInstancesDetails.length}):\n` +
              runningInstancesDetails
                  .map((d) => `• #${d.seq} ${d.name} (${d.email}) [PID: ${d.pid ?? 'N/A'}]`)
                  .join('\n')
            : 'No instances currently running';

    const handleOpenBackup = () => {
        setIsImportExportOpen(false);
        if ('setIsBackupModalOpen' in api && typeof (api as any).setIsBackupModalOpen === 'function') {
            (api as any).setBackupInitialTab?.('export');
            (api as any).setIsBackupModalOpen(true);
        }
        window.dispatchEvent(new CustomEvent('agm:open-instance-backup', { detail: { tab: 'export' } }));
    };

    const handleExportPrompts = async (kind: 'running' | 'queued' | 'running_and_queued') => {
        setIsExporting(true);
        try {
            await downloadPromptTreeStatusJson(kind, undefined, (msg) => {
                if (msg) showToast(msg, 'info');
            });
        } finally {
            setIsExporting(false);
            setIsImportExportOpen(false);
        }
    };

    const handleRestorePrompts = async () => {
        setIsImportExportOpen(false);
        try {
            await openAndRestorePromptBackupFile(activeInstanceId || 'default', (msg) => {
                if (msg) showToast(msg, 'info');
            });
        } catch (err: unknown) {
            showToast((err as Error)?.message || 'Failed to restore prompts backup', 'error');
        }
    };

    const handleResendAllPrompts = async () => {
        setIsImportExportOpen(false);
        setIsResending(true);
        try {
            const count = await resendPromptsForInstance(null, (msg) => {
                if (msg) showToast(msg, 'info');
            });
            showToast(`Successfully resent ${count} prompt(s) in FIFO order.`, 'success');
        } catch (err: unknown) {
            showToast((err as Error)?.message || 'Failed to resend prompts', 'error');
        } finally {
            setIsResending(false);
        }
    };

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
                <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#171025] border border-slate-200/80 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs">
                    <button
                        ref={runningBadgeRef}
                        type="button"
                        onClick={() => {
                            updateRunningDetailsPos();
                            setIsRunningDetailsOpen((prev) => !prev);
                        }}
                        className="px-3 py-1 text-xs text-slate-700 dark:text-slate-300 font-semibold whitespace-nowrap rounded-l-[4px] rounded-r-none hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer flex items-center gap-1.5"
                        title={runningTooltipText}
                    >
                        <span
                            className={cn(
                                'w-2 h-2 rounded-full shrink-0',
                                runningCount > 0
                                    ? 'bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)] animate-pulse'
                                    : 'bg-slate-400'
                            )}
                        />
                        <span>
                            {t('instances.running_summary', 'Running {{running}} of {{total}}', {
                                running: runningCount,
                                total: instances.length,
                            })}
                        </span>
                        <ChevronDown className={cn('w-3 h-3 text-slate-400 transition-transform', isRunningDetailsOpen && 'rotate-180')} />
                    </button>
                    <button
                        type="button"
                        onClick={() => {
                            fetchInstances();
                            fetchSwitcherStatus();
                        }}
                        disabled={isLoading}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title={t('common.refresh', 'Refresh')}
                    >
                        <RotateCw className={cn('w-3.5 h-3.5 text-blue-500', isLoading && 'animate-spin')} />
                        <span>{t('common.refresh', 'Refresh')}</span>
                    </button>
                    <button
                        type="button"
                        onClick={handleSyncAll}
                        disabled={isLoading || isSyncingAll}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Synchronize process PIDs and account quotas across all instances"
                    >
                        <FolderSync className={cn('w-3.5 h-3.5 text-cyan-500', isSyncingAll && 'animate-spin')} />
                        <span>Sync All</span>
                    </button>
                    <button
                        ref={importExportBtnRef}
                        id="instances-open-backup-btn"
                        type="button"
                        onClick={() => {
                            updateImportExportPos();
                            setIsImportExportOpen((prev) => !prev);
                        }}
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-purple-700 dark:text-purple-300 hover:bg-purple-100/50 dark:hover:bg-purple-950/40 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Import / Export: Backup and restore running & queued prompts or unified vault"
                    >
                        <FolderArchive className="w-3.5 h-3.5 text-purple-500" />
                        <span>Import / Export</span>
                        <ChevronDown className={cn('w-3 h-3 text-purple-400 transition-transform', isImportExportOpen && 'rotate-180')} />
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

                {/* Portal Popover 1: Running Instances & Emails Details */}
                {isRunningDetailsOpen &&
                    runningDetailsPos &&
                    createPortal(
                        <>
                            <div
                                className="fixed inset-0 z-[99998] bg-transparent"
                                onClick={() => setIsRunningDetailsOpen(false)}
                            />
                            <div
                                style={{ top: `${runningDetailsPos.top}px`, left: `${runningDetailsPos.left}px` }}
                                className="fixed z-[99999] w-96 rounded-xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl p-3 space-y-2 text-xs"
                                onClick={(e) => e.stopPropagation()}
                            >
                                <div className="flex items-center justify-between pb-2 border-b border-slate-100 dark:border-[#40285e]">
                                    <div className="flex items-center gap-2">
                                        <span className="w-2.5 h-2.5 rounded-full bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)] animate-pulse" />
                                        <span className="font-bold text-slate-800 dark:text-slate-100">
                                            Active Running Instances ({runningInstancesDetails.length})
                                        </span>
                                    </div>
                                    <span className="text-[11px] text-slate-400 font-mono">
                                        Total: {instances.length}
                                    </span>
                                </div>

                                {runningInstancesDetails.length === 0 ? (
                                    <div className="py-4 text-center text-slate-400 italic">
                                        No Antigravity instances currently running.
                                    </div>
                                ) : (
                                    <div className="space-y-1.5 max-h-72 overflow-y-auto pr-1">
                                        {runningInstancesDetails.map((instDetail) => (
                                            <div
                                                key={instDetail.id}
                                                className="p-2 rounded-lg bg-slate-50 dark:bg-[#201435] border border-slate-100 dark:border-[#40285e]/80 flex flex-col gap-1.5"
                                            >
                                                <div className="flex items-center justify-between">
                                                    <div className="flex items-center gap-1.5 min-w-0">
                                                        <span className="px-1.5 py-0.2 rounded font-black text-[10px] bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
                                                            #{instDetail.seq}
                                                        </span>
                                                        <span className="font-bold text-slate-800 dark:text-slate-200 truncate">
                                                            {instDetail.name}
                                                        </span>
                                                    </div>
                                                    <div className="flex items-center gap-1.5 shrink-0 font-mono text-[11px] text-slate-500 dark:text-slate-400">
                                                        <span>PID: {instDetail.pid ?? 'N/A'}</span>
                                                        {instDetail.memoryMb ? (
                                                            <span className="text-cyan-600 dark:text-cyan-400">
                                                                · {Math.round(instDetail.memoryMb)}MB
                                                            </span>
                                                        ) : null}
                                                    </div>
                                                </div>
                                                <div className="flex items-center justify-between pt-1 border-t border-slate-200/50 dark:border-[#40285e]/50">
                                                    <div className="flex items-center gap-1 text-[11px] text-slate-600 dark:text-slate-300 truncate">
                                                        <Mail className="w-3 h-3 text-purple-500 shrink-0" />
                                                        <span className="truncate">{instDetail.email}</span>
                                                    </div>
                                                    <button
                                                        type="button"
                                                        onClick={() => {
                                                            setIsRunningDetailsOpen(false);
                                                            setPromptTreeInstance({
                                                                id: instDetail.id,
                                                                name: instDetail.name,
                                                                seqNum: instDetail.seq,
                                                                executablePath: instDetail.executablePath,
                                                            });
                                                        }}
                                                        className="px-2 py-0.5 rounded text-[10px] font-semibold bg-cyan-50 dark:bg-cyan-950/40 text-cyan-600 dark:text-cyan-300 hover:bg-cyan-100 dark:hover:bg-cyan-900/50 transition-colors flex items-center gap-1 cursor-pointer shrink-0"
                                                        title="Open Prompt Tree for this instance"
                                                    >
                                                        <Layers className="w-3 h-3" />
                                                        <span>Prompts</span>
                                                    </button>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        </>,
                        document.body
                    )}

                {/* Portal Popover 2: Import / Export Menu */}
                {isImportExportOpen &&
                    importExportPos &&
                    createPortal(
                        <>
                            <div
                                className="fixed inset-0 z-[99998] bg-transparent"
                                onClick={() => setIsImportExportOpen(false)}
                            />
                            <div
                                style={{ top: `${importExportPos.top}px`, left: `${importExportPos.left}px` }}
                                className="fixed z-[99999] w-80 rounded-xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl p-2 space-y-1 text-xs"
                                onClick={(e) => e.stopPropagation()}
                            >
                                <div className="px-2 py-1 text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                                    Prompts Backup & Restore (JSON)
                                </div>
                                <button
                                    type="button"
                                    disabled={isExporting}
                                    onClick={() => handleExportPrompts('running')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <FileDown className="w-3.5 h-3.5 text-blue-500" />
                                        <span>Export Running Prompts (.json)</span>
                                    </div>
                                </button>
                                <button
                                    type="button"
                                    disabled={isExporting}
                                    onClick={() => handleExportPrompts('queued')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <FileDown className="w-3.5 h-3.5 text-amber-500" />
                                        <span>Export Queued Prompts (.json)</span>
                                    </div>
                                </button>
                                <button
                                    type="button"
                                    disabled={isExporting}
                                    onClick={() => handleExportPrompts('running_and_queued')}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <FileDown className="w-3.5 h-3.5 text-purple-500" />
                                        <span className="font-semibold">Export Running & Queued (.json)</span>
                                    </div>
                                </button>
                                <button
                                    type="button"
                                    onClick={handleRestorePrompts}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <FileUp className="w-3.5 h-3.5 text-emerald-500" />
                                        <span>Restore Prompts from Backup (.json)</span>
                                    </div>
                                </button>
                                <button
                                    type="button"
                                    disabled={isResending}
                                    onClick={handleResendAllPrompts}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <RotateCcw className={cn('w-3.5 h-3.5 text-cyan-500', isResending && 'animate-spin')} />
                                        <span>Resend All Prompts (FIFO Order)</span>
                                    </div>
                                </button>

                                <div className="my-1 border-t border-slate-100 dark:border-[#40285e]" />

                                <div className="px-2 py-1 text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                                    Instance Configuration Vault
                                </div>
                                <button
                                    type="button"
                                    onClick={handleOpenBackup}
                                    className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] text-left cursor-pointer transition-colors"
                                >
                                    <div className="flex items-center gap-2">
                                        <FolderArchive className="w-3.5 h-3.5 text-purple-500" />
                                        <span>Unified Instance Vault Backup & Restore...</span>
                                    </div>
                                </button>
                            </div>
                        </>,
                        document.body
                    )}

                {/* Segmented Group 2: Automation & Settings */}
                <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#171025] border border-slate-200/80 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs">
                    <button
                        type="button"
                        onClick={handleToggleAutoSwitcher}
                        disabled={isLoading}
                        className={cn(
                            'flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                            switcher?.is_running
                                ? 'text-emerald-700 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40'
                                : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-[#40285e]'
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
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
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
                        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                        title="Instance Settings & Sync: Turbo mode, plan review, copy settings, folder sync, JSON tools"
                    >
                        <SlidersHorizontal className="w-3.5 h-3.5 text-blue-500" />
                        <span>Settings & Sync</span>
                    </button>
                </div>

                {/* Segmented Group 3: View Mode & Creation */}
                <div className="flex items-center gap-2">
                    {viewMode === 'card' && (
                        <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#171025] border border-slate-200/80 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs">
                            <button
                                type="button"
                                onClick={() => handleSetCardDensity('normal')}
                                className={cn(
                                    'flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                                    cardDensity === 'normal'
                                        ? 'bg-white dark:bg-[#40285e] text-blue-600 dark:text-cyan-300 shadow-xs'
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
                                        ? 'bg-white dark:bg-[#40285e] text-blue-600 dark:text-cyan-300 shadow-xs'
                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                                )}
                                title="Compact Cards"
                            >
                                <Grid3X3 className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline">Compact Cards</span>
                            </button>
                        </div>
                    )}

                    <div className="flex items-center rounded-[4px] bg-slate-100 dark:bg-[#171025] border border-slate-200/80 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs">
                        <button
                            type="button"
                            onClick={() => handleSetViewMode('card')}
                            className={cn(
                                'flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer',
                                viewMode === 'card'
                                    ? 'bg-white dark:bg-[#40285e] text-blue-600 dark:text-cyan-300 shadow-xs'
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
                                    ? 'bg-white dark:bg-[#40285e] text-blue-600 dark:text-cyan-300 shadow-xs'
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
