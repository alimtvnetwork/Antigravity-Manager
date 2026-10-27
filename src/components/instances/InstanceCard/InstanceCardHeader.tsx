import React from 'react';
import { Star, Pencil, Copy, Trash2, RotateCw } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { useErrorStore } from '../../../stores/error-store';
import type { InstanceStatus } from '../../../services/instanceService';

export interface ResumeIndicatorState {
    instanceId: string;
    restoredCount?: number;
    at?: number;
    status?: 'scheduled' | 'restoring' | 'restored';
    secondsLeft?: number;
}

export interface ResumeIndicatorBadgeProps {
    instanceId: string;
    resumeIndicator?: ResumeIndicatorState | null;
    className?: string;
}

/**
 * Charming post-switch settling countdown & restore indicator badge.
 * - When countdown is active: pulsing yellow/amber badge `⏳ Restoring Prompts in {secondsLeft}s...`
 * - When restored: subtle green badge `✓ Prompts Restored`
 */
export function ResumeIndicatorBadge({
    instanceId,
    resumeIndicator,
    className,
}: ResumeIndicatorBadgeProps) {
    if (!resumeIndicator || resumeIndicator.instanceId !== instanceId) {
        return null;
    }

    const isCountdownActive =
        resumeIndicator.status === 'scheduled' ||
        (resumeIndicator.secondsLeft !== undefined && resumeIndicator.secondsLeft > 0);

    const isRestoring = resumeIndicator.status === 'restoring';

    if (isCountdownActive) {
        const seconds = resumeIndicator.secondsLeft ?? 10;
        return (
            <span
                className={cn(
                    'inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold',
                    'bg-amber-500/15 dark:bg-amber-500/20 text-amber-700 dark:text-amber-300',
                    'border border-amber-500/30 dark:border-amber-500/40 backdrop-blur-xs shadow-xs animate-pulse font-mono tracking-tight',
                    className
                )}
                title={`Post-switch prompt restore in progress: ${seconds}s settling delay remaining`}
            >
                <span className="w-1.5 h-1.5 rounded-full bg-amber-500 shadow-[0_0_4px_rgba(245,158,11,0.6)] animate-ping shrink-0" />
                <span>⏳ Restoring Prompts in {seconds}s...</span>
            </span>
        );
    }

    if (isRestoring) {
        return (
            <span
                className={cn(
                    'inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold',
                    'bg-amber-500/20 dark:bg-amber-500/25 text-amber-800 dark:text-amber-200',
                    'border border-amber-500/40 backdrop-blur-xs shadow-xs animate-pulse font-mono tracking-tight',
                    className
                )}
                title="Injecting restored prompts into instance..."
            >
                <RotateCw className="w-2.5 h-2.5 animate-spin text-amber-600 dark:text-amber-400 shrink-0" />
                <span>⏳ Restoring Prompts...</span>
            </span>
        );
    }

    // Default restored state
    return (
        <span
            className={cn(
                'inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold',
                'bg-emerald-500/15 dark:bg-emerald-500/20 text-emerald-700 dark:text-emerald-400',
                'border border-emerald-500/30 dark:border-emerald-500/40 backdrop-blur-xs shadow-xs font-mono tracking-tight',
                className
            )}
            title={`Prompts restored successfully (${resumeIndicator.restoredCount ?? 0} prompt(s))` }
        >
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_3px_rgba(16,185,129,0.5)] shrink-0" />
            <span>✓ Prompts Restored</span>
        </span>
    );
}

export interface InstanceCardHeaderProps {
    instance: {
        id: string;
        name: string;
        is_default?: boolean;
        is_running?: boolean;
        [key: string]: unknown;
    };
    inst?: InstanceStatus;
    seqNumber?: number;
    isActive?: boolean;
    isBusy?: boolean;
    isRunning?: boolean;
    hasActiveTask?: boolean;
    deletingId?: string | null;
    currentAction?: string | null;
    resumeIndicator?: ResumeIndicatorState | null;
    onOpenPromptTree?: (instanceId: string) => void;
    onSetDefault?: (instanceId: string) => Promise<void> | void;
    onSetActive?: (instanceId: string) => void;
    onEdit?: (instanceId: string, name: string) => void;
    onCopy?: (instanceId: string, name: string) => void;
    onDelete?: (instanceId: string) => void;
    t?: (key: string, fallback: string) => string;
}

export function InstanceCardHeader({
    instance,
    inst,
    seqNumber,
    isActive = false,
    isBusy = false,
    isRunning,
    hasActiveTask = false,
    deletingId,
    currentAction,
    resumeIndicator,
    onOpenPromptTree,
    onSetDefault,
    onSetActive,
    onEdit,
    onCopy,
    onDelete,
    t = (_k, fallback) => fallback,
}: InstanceCardHeaderProps) {
    const instanceId = instance.id || inst?.config.id || '';
    const instanceName = instance.name || inst?.config.name || '';
    const isDefault = Boolean(instance.is_default ?? inst?.config.is_default);
    const running = isRunning ?? Boolean(inst?.is_running);

    const handleSetDefaultClick = async (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            if (onSetDefault) {
                await onSetDefault(instanceId);
            }
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceCardHeader.handleSetDefaultClick',
                triggerAction: 'set_default_instance',
            });
        }
    };

    const handleSetActiveClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            onSetActive?.(instanceId);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceCardHeader.handleSetActiveClick',
                triggerAction: 'set_active_instance',
            });
        }
    };

    const handleEditClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            onEdit?.(instanceId, instanceName);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceCardHeader.handleEditClick',
                triggerAction: 'edit_instance_name',
            });
        }
    };

    const handleCopyClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            onCopy?.(instanceId, instanceName);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceCardHeader.handleCopyClick',
                triggerAction: 'copy_instance',
            });
        }
    };

    const handleDeleteClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        try {
            onDelete?.(instanceId);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceCardHeader.handleDeleteClick',
                triggerAction: 'delete_instance',
            });
        }
    };

    return (
        <div className="flex items-center justify-between gap-2 mb-2.5 h-6 flex-nowrap min-w-0">
            <div className="flex items-center gap-1.5 min-w-0 flex-1 flex-nowrap overflow-hidden">
                <span
                    className={cn(
                        'w-2.5 h-2.5 rounded-full shrink-0',
                        running
                            ? 'bg-emerald-500 shadow-[0_0_5px_rgba(16,185,129,0.5)]'
                            : 'bg-gray-300 dark:bg-gray-600'
                    )}
                />
                {seqNumber !== undefined && (
                    <span className="px-1.5 py-0.5 rounded-[5px] text-xs font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
                        #{seqNumber}
                    </span>
                )}
                <h3
                    className={cn(
                        'font-bold text-xs truncate shrink min-w-0',
                        isActive ? 'text-blue-900 dark:text-blue-100' : 'text-gray-900 dark:text-base-content'
                    )}
                    title={instanceName}
                >
                    {instanceName}
                </h3>

                {/* Surface settling countdown or restored badge */}
                <ResumeIndicatorBadge
                    instanceId={instanceId}
                    resumeIndicator={resumeIndicator}
                    className="shrink-0"
                />

                {hasActiveTask && onOpenPromptTree && (
                    <button
                        type="button"
                        onClick={() => onOpenPromptTree(instanceId)}
                        className="inline-flex items-center gap-1 px-1.5 h-5 rounded-[5px] text-[9px] font-bold bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-700 dark:text-emerald-400 border border-emerald-500/25 shrink-0 cursor-pointer transition-colors shadow-2xs"
                        title="Active prompt/task running - Click to open Prompt Tree"
                    >
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_3px_rgba(16,185,129,0.5)] shrink-0" />
                        <span>Prompt</span>
                    </button>
                )}

                {isDefault ? (
                    <span className="h-5 px-1.5 rounded-[5px] text-[9px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-400/30 flex items-center justify-center shrink-0">
                        DEFAULT
                    </span>
                ) : (
                    <button
                        disabled={isBusy}
                        onClick={handleSetDefaultClick}
                        className="h-5 px-1.5 rounded-[5px] text-[9px] font-medium text-gray-500 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950/30 border border-slate-200/80 dark:border-[#40285e] transition-colors cursor-pointer flex items-center gap-1 shrink-0 disabled:opacity-50 disabled:cursor-not-allowed"
                        title="Set as default profile"
                    >
                        <Star className="w-2.5 h-2.5" />
                        <span>Set Default</span>
                    </button>
                )}
            </div>

            <div className="shrink-0 inline-flex items-center overflow-hidden rounded-full border border-slate-200 dark:border-[#40285e] divide-x divide-slate-200 dark:divide-[#40285e]">
                {isActive ? (
                    <span className="inline-flex items-center justify-center h-5 px-2 bg-amber-500/20 text-amber-600 dark:text-amber-300 text-[9px] font-bold tracking-wider rounded-none border-0">
                        Active
                    </span>
                ) : (
                    <button
                        disabled={isBusy}
                        onClick={handleSetActiveClick}
                        className="text-[10px] text-gray-500 hover:text-blue-600 transition-colors font-medium px-2 py-0.5 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                        title="Set as active instance for account switches"
                    >
                        Set Active
                    </button>
                )}
                <button
                    disabled={isBusy}
                    onClick={handleEditClick}
                    className="p-1 text-gray-500 dark:text-[var(--ui-text-secondary)] hover:text-blue-600 dark:hover:text-[var(--ui-interactive)] hover:bg-blue-50 dark:hover:bg-[var(--ui-surface-hover)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title={t('instances.edit_title', 'Rename profile')}
                >
                    <Pencil className="w-3 h-3" />
                </button>
                <button
                    disabled={isBusy}
                    onClick={handleCopyClick}
                    className="p-1 text-gray-500 dark:text-[var(--ui-text-secondary)] hover:text-blue-600 dark:hover:text-[var(--ui-interactive)] hover:bg-blue-50 dark:hover:bg-[var(--ui-surface-hover)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Clone / Duplicate profile settings and extensions"
                >
                    <Copy className="w-3 h-3" />
                </button>
                {!isDefault && (
                    <button
                        onClick={handleDeleteClick}
                        disabled={running || isBusy || deletingId === instanceId}
                        className="p-1 text-gray-500 dark:text-[var(--ui-text-secondary)] hover:text-red-600 dark:hover:text-[var(--ui-danger)] hover:bg-red-50 dark:hover:bg-[var(--ui-surface-hover)] disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer"
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
    );
}

export default InstanceCardHeader;
