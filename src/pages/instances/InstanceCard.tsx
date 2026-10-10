import {
    Star,
    Pencil,
    Copy,
    Trash2,
    RotateCw,
    Mail,
    Folder,
    Cpu,
    Layers,
    SlidersHorizontal,
    History,
    MoreHorizontal,
    KeyRound,
    Square,
    RotateCcw,
    Play,
    ArrowRightLeft,
    FastForward,
    ArrowLeftRight,
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { Gemini } from '../../components/common/icons';
import { TierBadge } from '../../components/accounts/TierBadge';
import { QuotaProgressBar } from '../../components/accounts/QuotaProgressBar';
import { SELECTED_CARD_CLASSES, ACTIVE_PILL_CLASSES } from '../../components/common/selectedState';
import { showToast } from '../../components/common/ToastContainer';
import type { InstanceStatus } from '../../services/instanceService';
import type { AgmProjectTreeNode } from '../../components/instances/PromptTreeViewModal';
import { truncatePath, getActionLabel, type InstanceTheme } from './instancePageUtils';
import { useInstanceCardData } from './instanceCardData';
import type { InstancePageApi } from './instancePageTypes';
import { InstanceCardProjects } from './InstanceCardProjects';
import { InstanceCardActions } from './InstanceCardActions';

export interface InstanceCardProps {
    api: InstancePageApi;
    inst: InstanceStatus;
    index: number;
    seqNumber: number;
    theme: InstanceTheme;
}

export function InstanceCard({ api, inst, index, seqNumber, theme }: InstanceCardProps) {
    const {
        t,
        activeInstanceId,
        cardDensity,
        actionState,
        runningTreeNodes,
        deletingId,
        syncingInstanceIds,
        cardMoreId,
        activeCardRef,
        setActiveInstance,
        setDefaultInstance,
        setActionError,
        setEditTargetId,
        setEditInstanceName,
        openCopyDialog,
        handleDelete,
        handleLaunch,
        handleStop,
        handleRestart,
        handleFastForward,
        handleSync,
        handleCloneExecutable,
        handleWipeSession,
        setPromptTreeInstance,
        openSettingsDialog,
        openAuditDialog,
        setCardMoreId,
        refreshQuota,
    } = api;

    const data = useInstanceCardData(inst, runningTreeNodes, actionState);
    const { boundAccount, displayEmail, geminiModel, weeklyQuota, hasActiveTask, effectiveExePath, currentAction, isBusy } =
        data;

    const isActive = inst.config.id === activeInstanceId;
    void index;

    const handleSetDefault = async () => {
        try {
            await setDefaultInstance(inst.config.id);
            showToast(t('instances.set_default_toast', 'Default profile updated successfully'), 'success');
        } catch (e: unknown) {
            setActionError((e as Error)?.toString?.() || 'Failed to set default profile');
        }
    };

    const copyToClipboard = async (text: string, successMsg: string, failMsg: string, e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            await navigator.clipboard.writeText(text);
            showToast(successMsg, 'info');
        } catch {
            showToast(failMsg, 'error');
        }
    };

    return (
        <div
            key={inst.config.id}
            ref={isActive ? activeCardRef : undefined}
            className={cn(
                'group relative rounded-[5px] border transition-all duration-200 flex flex-col justify-between overflow-hidden backdrop-blur-xs',
                isActive
                    ? SELECTED_CARD_CLASSES
                    : 'bg-white dark:bg-[var(--ui-surface-1)] border-gray-200/50 dark:border-[var(--ui-border-subtle)] hover:border-gray-300/80 dark:hover:border-[var(--ui-interactive-40)] hover:bg-slate-50/90 dark:hover:bg-[var(--ui-surface-hover)] shadow-xs'
            )}
        >
            {isBusy && (
                <div className="absolute inset-0 bg-white/75 dark:bg-[#071a27]/85 backdrop-blur-[2px] z-30 flex flex-col items-center justify-center gap-2 rounded-[5px] pointer-events-auto cursor-wait select-none">
                    <RotateCw className="w-5 h-5 animate-spin text-blue-500 dark:text-cyan-400" />
                    <span className="text-xs font-bold text-gray-800 dark:text-cyan-200 tracking-wide font-mono">
                        {getActionLabel(currentAction)}
                    </span>
                </div>
            )}

            <div className={cn('flex flex-col flex-1 justify-between min-w-0', cardDensity === 'compact' ? 'p-2.5' : 'p-3.5')}>
                <div className="min-w-0">
                    <div className="flex items-center justify-between gap-2 mb-2.5 h-6 flex-nowrap min-w-0">
                        <div className="flex items-center gap-1.5 min-w-0 flex-1 flex-nowrap overflow-hidden">
                            <span
                                className={cn(
                                    'w-2.5 h-2.5 rounded-full shrink-0',
                                    inst.is_running
                                        ? 'bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)] animate-pulse'
                                        : 'bg-gray-300 dark:bg-gray-600'
                                )}
                            />
                            <span className="px-1.5 py-0.5 rounded-[5px] text-xs font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
                                #{seqNumber}
                            </span>
                            <h3
                                className={cn(
                                    'font-bold text-xs truncate shrink min-w-0',
                                    isActive ? 'text-blue-900 dark:text-blue-100' : 'text-gray-900 dark:text-base-content'
                                )}
                                title={inst.config.name}
                            >
                                {inst.config.name}
                            </h3>
                            {hasActiveTask && (
                                <button
                                    type="button"
                                    onClick={() => openPromptTree(inst.config.id)}
                                    className="inline-flex items-center gap-1 px-1.5 h-5 rounded-[5px] text-[9px] font-bold bg-[#1af18d]/15 hover:bg-[#1af18d]/25 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 shrink-0 cursor-pointer transition-colors shadow-2xs"
                                    title="Active prompt/task running - Click to open Prompt Tree"
                                >
                                    <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse shrink-0" />
                                    <span>Prompt</span>
                                </button>
                            )}
                            {inst.config.is_default ? (
                                <span className="h-5 px-1.5 rounded-[5px] text-[9px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-400/30 flex items-center justify-center shrink-0">
                                    DEFAULT
                                </span>
                            ) : (
                                <button
                                    disabled={isBusy}
                                    onClick={handleSetDefault}
                                    className="h-5 px-1.5 rounded-[5px] text-[9px] font-medium text-gray-400 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950/30 border border-dashed border-gray-300 dark:border-[#15334d] transition-colors cursor-pointer flex items-center gap-1 shrink-0 disabled:opacity-50 disabled:cursor-not-allowed"
                                    title="Set as default profile"
                                >
                                    <Star className="w-2.5 h-2.5" />
                                    <span>Set Default</span>
                                </button>
                            )}
                        </div>
                        <div className="shrink-0 flex items-center gap-1">
                            {isActive ? (
                                <span className={cn(ACTIVE_PILL_CLASSES, 'text-[9px] px-1.5 py-0.2 tracking-wider')}>
                                    Active
                                </span>
                            ) : (
                                <button
                                    disabled={isBusy}
                                    onClick={() => setActiveInstance(inst.config.id)}
                                    className="text-[10px] text-gray-500 hover:text-blue-600 transition-colors font-medium mr-0.5 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                    title="Set as active instance for account switches"
                                >
                                    Set Active
                                </button>
                            )}
                            <button
                                disabled={isBusy}
                                onClick={() => {
                                    setEditTargetId(inst.config.id);
                                    setEditInstanceName(inst.config.name);
                                }}
                                className="p-1 rounded-[5px] text-gray-500 dark:text-[var(--ui-text-secondary)] hover:text-blue-600 dark:hover:text-[var(--ui-interactive)] hover:bg-blue-50 dark:hover:bg-[var(--ui-surface-hover)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                title={t('instances.edit_title', 'Rename profile')}
                            >
                                <Pencil className="w-3 h-3" />
                            </button>
                            <button
                                disabled={isBusy}
                                onClick={() => openCopyDialog(inst.config.id, inst.config.name)}
                                className="p-1 rounded-[5px] text-gray-500 dark:text-[var(--ui-text-secondary)] hover:text-blue-600 dark:hover:text-[var(--ui-interactive)] hover:bg-blue-50 dark:hover:bg-[var(--ui-surface-hover)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                title="Clone / Duplicate profile settings and extensions"
                            >
                                <Copy className="w-3 h-3" />
                            </button>
                            {!inst.config.is_default && (
                                <button
                                    onClick={() => handleDelete(inst.config.id)}
                                    disabled={inst.is_running || isBusy || deletingId === inst.config.id}
                                    className="p-1 rounded-[5px] text-gray-500 dark:text-[var(--ui-text-secondary)] hover:text-red-600 dark:hover:text-[var(--ui-danger)] hover:bg-red-50 dark:hover:bg-[var(--ui-surface-hover)] disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer"
                                    title="Delete profile"
                                >
                                    {currentAction === 'delete' ? (
                                        <RotateCw className="w-3 h-3 animate-spin text-rose-500" />
                                    ) : (
                                        <Trash2 className="w-3 h-3" />
                                    )}
                                </button>
                            )}
                        </div>
                    </div>

                    <div
                        className={cn(
                            'ui-interactive-row py-1 px-2.5 rounded-md bg-gray-50/80 dark:bg-[var(--ui-surface-2)] border border-gray-100 dark:border-[var(--ui-border-subtle)] mb-2.5 flex items-center justify-between gap-1.5',
                            'group-hover:border-gray-300 dark:group-hover:bg-[var(--ui-surface-hover)] dark:group-hover:border-[var(--ui-interactive-40)]'
                        )}
                    >
                        <div className="flex items-center gap-1.5 min-w-0 flex-1">
                            <Mail className="w-3 h-3 text-gray-400 dark:text-[var(--ui-text-secondary)] group-hover:text-blue-500 dark:group-hover:text-[var(--ui-interactive)] transition-colors shrink-0" />
                            <span className="text-[10px] text-gray-500 dark:text-gray-400 font-medium shrink-0">Account:</span>
                            {displayEmail ? (
                                <span
                                    className={cn(
                                        'px-1.5 py-0.5 rounded-md text-[11px] font-semibold font-mono border flex items-center gap-1 min-w-0 shadow-2xs transition-colors',
                                        'bg-blue-500/10 text-blue-700 border-blue-500/30 dark:bg-[var(--ui-interactive-soft)] dark:text-[var(--ui-interactive)] dark:border-[var(--ui-interactive-border)]'
                                    )}
                                    title={displayEmail}
                                >
                                    <span className="w-1.5 h-1.5 rounded-full shrink-0 bg-blue-500 dark:bg-[var(--ui-interactive)]" />
                                    <span className="truncate">{displayEmail}</span>
                                </span>
                            ) : (
                                <span className="text-[11px] text-gray-400 italic">Unassigned</span>
                            )}
                        </div>
                        {boundAccount && (
                            <TierBadge tier={boundAccount.quota?.subscription_tier} size="xs" className="shrink-0" />
                        )}
                    </div>

                    <div className="mb-2">
                        {geminiModel || weeklyQuota ? (
                            <div className="p-2 rounded-md bg-gray-50/90 dark:bg-[var(--ui-surface-2)] border border-gray-200/70 dark:border-[var(--ui-border-subtle)] space-y-1.5">
                                {geminiModel && (
                                    <div className="space-y-0.5">
                                        <div className="flex items-center justify-between text-xs">
                                            <div className="flex items-center gap-1.5 min-w-0">
                                                <Gemini.Color className="w-3.5 h-3.5 shrink-0" />
                                                <span className="font-semibold text-gray-800 dark:text-gray-200 text-xs truncate">
                                                    {geminiModel.display_name || geminiModel.name || 'Gemini 3.1 Pro'} (4H)
                                                </span>
                                            </div>
                                        </div>
                                        <QuotaProgressBar
                                            percentage={geminiModel.percentage}
                                            resetTime={geminiModel.reset_time}
                                        />
                                    </div>
                                )}
                                {weeklyQuota && (
                                    <div className={cn('space-y-0.5', geminiModel && 'pt-1 border-t border-gray-200/60 dark:border-[#15334d]/60')}>
                                        <div className="flex items-center justify-between text-xs">
                                            <span className="text-[11px] font-medium text-gray-600 dark:text-gray-300">
                                                Weekly Quota
                                            </span>
                                        </div>
                                        <QuotaProgressBar
                                            isWeekly
                                            percentage={weeklyQuota.percentage}
                                            resetTime={weeklyQuota.resetTime}
                                        />
                                    </div>
                                )}
                            </div>
                        ) : boundAccount ? (
                            <div className="p-2 rounded-md bg-gray-50/60 dark:bg-[#0c2438]/90 border border-gray-100 dark:border-[#15334d] flex items-center justify-between text-xs text-gray-500">
                                <div className="flex items-center gap-1.5">
                                    <Gemini.Color className="w-3.5 h-3.5 shrink-0 opacity-70" />
                                    <span className="text-[11px] text-gray-500 dark:text-gray-400">Quota not synced</span>
                                </div>
                                <button
                                    onClick={() => refreshQuota(boundAccount.id)}
                                    className="text-blue-600 dark:text-blue-400 hover:underline gap-1 text-[11px] cursor-pointer flex items-center"
                                >
                                    <ArrowLeftRight className="w-3 h-3 text-blue-500" />
                                    <span>Sync</span>
                                </button>
                            </div>
                        ) : (
                            <div className="p-2 rounded-md bg-gray-50/40 dark:bg-[#0c2438]/60 border border-dashed border-gray-200 dark:border-[#15334d] flex items-center justify-between text-xs text-gray-400">
                                <div className="flex items-center gap-1.5">
                                    <Gemini.Color className="w-3.5 h-3.5 shrink-0 opacity-40 grayscale" />
                                    <span className="text-[10px] italic">No profile bound</span>
                                </div>
                                <span className="text-[10px] text-gray-400/80">Launch to assign</span>
                            </div>
                        )}
                    </div>

                    <div className="space-y-1.5 py-2 border-t border-gray-100 dark:border-[#15334d]/80 text-xs">
                        <div className="flex items-center justify-between gap-1.5 flex-wrap">
                            {inst.is_running ? (
                                <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[10px] font-semibold bg-[#1af18d]/10 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 shadow-2xs">
                                    <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse" />
                                    Running {inst.pid ? `(${inst.pid})` : ''}
                                </span>
                            ) : (
                                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-medium bg-slate-500/10 text-slate-600 dark:text-slate-400 border border-slate-400/20">
                                    <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                                    Idle
                                </span>
                            )}

                            <span
                                className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-medium bg-gray-100 dark:bg-[#0c2438] text-gray-600 dark:text-slate-300 border border-gray-200 dark:border-[#15334d] truncate max-w-[120px]"
                                title={`Profile ID: ${inst.config.id}`}
                            >
                                <span className="text-gray-400 dark:text-slate-500">ID:</span>
                                <span className="truncate">{inst.config.id}</span>
                            </span>
                        </div>

                        <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-gray-50/80 dark:bg-[#0c2438]/70 border border-gray-200/70 dark:border-[#15334d] text-[10px] group transition-colors hover:border-gray-300 dark:hover:border-blue-500/30">
                            <div className="flex items-center gap-1 min-w-0 flex-1 text-gray-500 dark:text-slate-400" title={inst.config.data_dir}>
                                <Folder className="w-3 h-3 shrink-0 text-blue-500/80" />
                                <span className="truncate font-mono" title={inst.config.data_dir}>
                                    {truncatePath(inst.config.data_dir)}
                                </span>
                            </div>
                            <button
                                type="button"
                                onClick={(e) => copyToClipboard(inst.config.data_dir, 'Data dir path copied to clipboard', 'Failed to copy path', e)}
                                className="opacity-60 group-hover:opacity-100 p-0.5 rounded-[5px] hover:bg-gray-200 dark:hover:bg-[#15334d] text-gray-500 dark:text-gray-300 transition-opacity cursor-pointer shrink-0"
                                title="Copy directory path"
                            >
                                <Copy className="w-3 h-3" />
                            </button>
                        </div>

                        {effectiveExePath ? (
                            <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-slate-100/70 dark:bg-slate-900/60 border border-slate-200/80 dark:border-[#15334d] text-[10px] group/exe transition-colors">
                                <div className="flex items-center gap-1 min-w-0 flex-1 text-slate-700 dark:text-slate-300" title={effectiveExePath}>
                                    <Cpu className="w-3 h-3 shrink-0 text-cyan-500" />
                                    <span className="truncate font-mono" title={effectiveExePath}>
                                        {truncatePath(effectiveExePath)}
                                    </span>
                                </div>
                                <button
                                    type="button"
                                    onClick={(e) => copyToClipboard(effectiveExePath, 'Executable path copied to clipboard', 'Failed to copy executable path', e)}
                                    className="opacity-60 group-hover/exe:opacity-100 p-0.5 rounded-[5px] hover:bg-slate-200 dark:hover:bg-[#15334d] text-slate-500 dark:text-slate-300 transition-opacity cursor-pointer shrink-0"
                                    title="Copy executable path"
                                >
                                    <Copy className="w-3 h-3" />
                                </button>
                            </div>
                        ) : null}
                    </div>

                    {cardDensity !== 'compact' && <InstanceCardProjects api={api} inst={inst} />}
                </div>

                <InstanceCardActions api={api} inst={inst} data={data} hasActiveTask={hasActiveTask} />
            </div>

            <div className="relative w-full h-[3px] overflow-hidden rounded-b-[5px]">
                <div
                    className={cn(
                        'absolute inset-0 bg-gradient-to-r transition-all duration-300 ease-out transform',
                        theme.accentBar,
                        isActive
                            ? 'opacity-60 scale-x-100 group-hover:opacity-100'
                            : 'opacity-0 scale-x-95 group-hover:opacity-100 group-hover:scale-x-100'
                    )}
                />
            </div>
        </div>
    );
}

