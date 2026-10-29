import { createPortal } from 'react-dom';
import {
    Play,
    Square,
    RotateCcw,
    RotateCw,
    History,
    FastForward,
    SlidersHorizontal,
    Copy,
    Trash2,
    MoreHorizontal,
    Cpu,
    Layers,
    ArrowLeftRight,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { InstanceStatus } from '../../../services/instanceService';
import type { InstanceActionType } from './instanceTableTypes';

interface ActionsCellProps {
    inst: InstanceStatus;
    currentAction: InstanceActionType;
    isBusy: boolean;
    isMoreOpen: boolean;
    onLaunch: () => void;
    onStop: () => void;
    onRestart: () => void;
    onSwitch: () => void;
    onFastForward: () => void;
    onOpenPromptTree: () => void;
    onAudit?: () => void;
    onToggleMore: (anchor: HTMLElement) => void;
}

export function ActionsCell({
    inst,
    currentAction,
    isBusy,
    isMoreOpen,
    onLaunch,
    onStop,
    onRestart,
    onSwitch,
    onFastForward,
    onOpenPromptTree,
    onAudit,
    onToggleMore,
}: ActionsCellProps) {
    return (
        <td className="px-2 py-1 text-right whitespace-nowrap min-w-[150px]">
            <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-zinc-900/90 backdrop-blur-md border border-slate-200/80 dark:border-zinc-700/60 p-0.5 divide-x divide-slate-200/70 dark:divide-zinc-700/60 shadow-sm">
                {inst.is_running ? (
                    <>
                        <button
                            id={`instance-stop-${inst.config.id}`}
                            type="button"
                            disabled={isBusy}
                            onClick={onStop}
                            data-ui-name="instance-stop-btn"
                            data-ui-id={`instance-stop-${inst.config.id}`}
                            className="flex items-center justify-center h-7 px-2.5 text-rose-600 dark:text-rose-400 hover:bg-rose-600 hover:text-white rounded-l-full transition-all duration-300 ease-out font-semibold cursor-pointer disabled:opacity-50"
                            title="Stop Instance"
                        >
                            {currentAction === 'stop' ? (
                                <RotateCw className="w-3.5 h-3.5 animate-spin text-rose-500" />
                            ) : (
                                <Square className="w-3.5 h-3.5 fill-current" />
                            )}
                        </button>
                        <button
                            id={`instance-restart-${inst.config.id}`}
                            type="button"
                            disabled={isBusy}
                            onClick={onRestart}
                            data-ui-name="instance-restart-btn"
                            data-ui-id={`instance-restart-${inst.config.id}`}
                            className="flex items-center justify-center h-7 px-2.5 text-amber-600 dark:text-amber-400 hover:bg-amber-500 hover:text-zinc-950 font-semibold transition-all duration-300 ease-out cursor-pointer disabled:opacity-50"
                            title="Restart Instance on Current Account"
                        >
                            {currentAction === 'restart' ? (
                                <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-500" />
                            ) : (
                                <RotateCcw className="w-3.5 h-3.5" />
                            )}
                        </button>
                    </>
                ) : (
                    <button
                        id={`instance-launch-${inst.config.id}`}
                        type="button"
                        disabled={isBusy}
                        onClick={onLaunch}
                        data-ui-name="instance-launch-btn"
                        data-ui-id={`instance-launch-${inst.config.id}`}
                        className="flex items-center justify-center h-7 px-2.5 text-teal-600 dark:text-teal-400 hover:bg-emerald-600 hover:text-white rounded-l-full transition-all duration-300 ease-out font-semibold cursor-pointer disabled:opacity-50"
                        title="Launch Instance"
                    >
                        {currentAction === 'launch' ? (
                            <RotateCw className="w-3.5 h-3.5 animate-spin text-teal-500" />
                        ) : (
                            <Play className="w-3.5 h-3.5 fill-current" />
                        )}
                    </button>
                )}

                <button
                    id={`instance-switch-${inst.config.id}`}
                    type="button"
                    disabled={isBusy}
                    onClick={onSwitch}
                    data-ui-name="instance-switch-btn"
                    data-ui-id={`instance-switch-${inst.config.id}`}
                    className="flex items-center justify-center h-7 px-2.5 text-amber-600 dark:text-amber-300 bg-amber-500/10 dark:bg-amber-500/20 hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:text-black hover:!text-black font-semibold transition-all duration-300 ease-out cursor-pointer disabled:opacity-50"
                    title="Switch Account"
                >
                    {currentAction === 'switch' ? (
                        <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-500" />
                    ) : (
                        <ArrowLeftRight className="w-3.5 h-3.5" />
                    )}
                </button>

                <button
                    id={`instance-fastforward-${inst.config.id}`}
                    type="button"
                    disabled={isBusy}
                    onClick={onFastForward}
                    data-ui-name="instance-fastforward-btn"
                    data-ui-id={`instance-fastforward-${inst.config.id}`}
                    className="flex items-center justify-center h-7 px-2.5 text-amber-600 dark:text-amber-300 hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:text-black hover:!text-black font-semibold transition-all duration-300 ease-out cursor-pointer disabled:opacity-50"
                    title="Fast Forward to Best Candidate"
                >
                    {currentAction === 'fast-forward' ? (
                        <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-500" />
                    ) : (
                        <FastForward className="w-3.5 h-3.5 fill-current" />
                    )}
                </button>

                {/* Primary Prompts Button */}
                <button
                    id={`instance-tree-${inst.config.id}`}
                    type="button"
                    disabled={isBusy}
                    onClick={onOpenPromptTree}
                    data-ui-name="instance-tree-btn"
                    data-ui-id={`instance-tree-${inst.config.id}`}
                    className="flex items-center justify-center h-7 px-2.5 text-cyan-600 dark:text-cyan-400 hover:bg-cyan-600 hover:text-white font-semibold transition-all duration-300 ease-out cursor-pointer disabled:opacity-50"
                    title="Prompts & History"
                >
                    <Layers className="w-3.5 h-3.5" />
                </button>

                {/* Explicit Audit Trail Button */}
                {onAudit && (
                    <button
                        id={`instance-audit-${inst.config.id}`}
                        type="button"
                        disabled={isBusy}
                        onClick={onAudit}
                        data-ui-name="instance-audit-btn"
                        data-ui-id={`instance-audit-${inst.config.id}`}
                        className="flex items-center justify-center h-7 px-2.5 text-amber-600 dark:text-amber-300 hover:bg-amber-500 hover:text-zinc-950 font-semibold transition-all duration-300 ease-out cursor-pointer disabled:opacity-50"
                        title="Audit Trail & Switch History"
                    >
                        <History className="w-3.5 h-3.5" />
                    </button>
                )}

                <button
                    id={`instance-more-${inst.config.id}`}
                    type="button"
                    disabled={isBusy}
                    onClick={(e) => {
                        e.stopPropagation();
                        onToggleMore(e.currentTarget);
                    }}
                    data-ui-name="instance-more-btn"
                    data-ui-id={`instance-more-${inst.config.id}`}
                    className={cn(
                        'flex items-center justify-center h-7 px-2 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded-r-full font-semibold transition-all duration-300 ease-out cursor-pointer disabled:opacity-50',
                        isMoreOpen && 'bg-slate-200 dark:bg-[#40285e]'
                    )}
                    title="More Actions (Prompts, Audit, Sync, Settings, Clone, Delete)"
                >
                    <MoreHorizontal className="w-3.5 h-3.5" />
                </button>
            </div>
        </td>
    );
}

interface MoreMenuPortalProps {
    inst: InstanceStatus;
    menuPos: { top: number; left: number };
    menuRef: React.RefObject<HTMLDivElement | null>;
    onAudit?: (id: string, name: string) => void;
    onSync?: (id: string) => void;
    onClose: () => void;
    onOpenPromptTree: (id: string) => void;
    onRestart: (id: string) => void;
    onSettings: (id: string) => void;
    onClone: (id: string, name: string) => void;
    onDelete: (id: string) => void;
}

export function MoreMenuPortal({
    inst,
    menuPos,
    menuRef,
    onAudit,
    onSync,
    onClose,
    onOpenPromptTree,
    onRestart,
    onSettings,
    onClone,
    onDelete,
}: MoreMenuPortalProps) {
    const isDefault = inst.config.is_default || inst.config.id === 'default';
    const itemClass =
        'flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] cursor-pointer';

    return createPortal(
        <div
            ref={menuRef}
            className="fixed z-[10000] min-w-[180px] rounded-[5px] border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] py-1 text-slate-800 dark:text-slate-200 shadow-xl text-xs"
            style={{ top: menuPos.top, left: menuPos.left, transform: 'translateX(-100%)' }}
            onClick={(e) => e.stopPropagation()}
        >
            <button
                type="button"
                onClick={() => {
                    onClose();
                    onOpenPromptTree(inst.config.id);
                }}
                data-ui-name="instance-menu-prompts-btn"
                data-ui-id={`instance-menu-prompts-${inst.config.id}`}
                className={`${itemClass} text-slate-700 dark:text-slate-200`}
            >
                <Layers className="w-3.5 h-3.5 text-cyan-600 dark:text-cyan-400" />
                <span>Prompts & History</span>
            </button>
            {onAudit && (
                <button
                    type="button"
                    onClick={() => {
                        onClose();
                        onAudit(inst.config.id, inst.config.name);
                    }}
                    data-ui-name="instance-menu-audit-btn"
                    data-ui-id={`instance-menu-audit-${inst.config.id}`}
                    className={`${itemClass} text-slate-700 dark:text-slate-200`}
                >
                    <History className="w-3.5 h-3.5 text-amber-500" />
                    <span>Audit Trail</span>
                </button>
            )}
            {onSync && (
                <button
                    type="button"
                    onClick={() => {
                        onClose();
                        onSync(inst.config.id);
                    }}
                    data-ui-name="instance-menu-sync-btn"
                    data-ui-id={`instance-menu-sync-${inst.config.id}`}
                    className={`${itemClass} text-teal-600 dark:text-teal-400`}
                >
                    <Cpu className="w-3.5 h-3.5 text-teal-500" />
                    <span>Sync PID & Quota</span>
                </button>
            )}
            {inst.is_running && (
                <button
                    type="button"
                    onClick={() => {
                        onClose();
                        onRestart(inst.config.id);
                    }}
                    data-ui-name="instance-menu-restart-btn"
                    data-ui-id={`instance-menu-restart-${inst.config.id}`}
                    className={`${itemClass} text-amber-600 dark:text-amber-400`}
                >
                    <RotateCcw className="w-3.5 h-3.5 text-amber-500" />
                    <span>Restart Instance</span>
                </button>
            )}
            <button
                type="button"
                onClick={() => {
                    onClose();
                    onSettings(inst.config.id);
                }}
                data-ui-name="instance-menu-settings-btn"
                data-ui-id={`instance-menu-settings-${inst.config.id}`}
                className={`${itemClass} text-slate-700 dark:text-slate-200`}
            >
                <SlidersHorizontal className="w-3.5 h-3.5 text-blue-500" />
                <span>Settings & Sync</span>
            </button>
            <button
                type="button"
                onClick={() => {
                    onClose();
                    onClone(inst.config.id, inst.config.name);
                }}
                data-ui-name="instance-menu-clone-btn"
                data-ui-id={`instance-menu-clone-${inst.config.id}`}
                className={`${itemClass} text-indigo-600 dark:text-indigo-400`}
            >
                <Copy className="w-3.5 h-3.5 text-indigo-500" />
                <span>Clone Profile</span>
            </button>
            {!isDefault && (
                <button
                    type="button"
                    disabled={inst.is_running}
                    onClick={() => {
                        onClose();
                        onDelete(inst.config.id);
                    }}
                    data-ui-name="instance-menu-delete-btn"
                    data-ui-id={`instance-menu-delete-${inst.config.id}`}
                    className={`${itemClass} hover:bg-rose-50 dark:hover:bg-rose-950/40 text-rose-600 dark:text-rose-400 disabled:opacity-40`}
                >
                    <Trash2 className="w-3.5 h-3.5" />
                    <span>Delete Profile</span>
                </button>
            )}
        </div>,
        document.body
    );
}
