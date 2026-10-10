import {
    Copy,
    Trash2,
    RotateCw,
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
} from 'lucide-react';
import { cn } from '../../utils/cn';
import type { InstanceStatus } from '../../services/instanceService';
import type { InstanceCardData } from './instanceCardData';
import type { InstancePageApi } from './instancePageTypes';

interface InstanceCardActionsProps {
    api: InstancePageApi;
    inst: InstanceStatus;
    data: InstanceCardData;
    hasActiveTask: boolean;
}

export function InstanceCardActions({ api, inst, data, hasActiveTask }: InstanceCardActionsProps) {
    const {
        syncingInstanceIds,
        deletingId,
        cardMoreId,
        setCardMoreId,
        handleStop,
        handleRestart,
        handleLaunch,
        handleFastForward,
        handleSync,
        handleCloneExecutable,
        handleWipeSession,
        handleDelete,
        openPromptTree,
        openSettingsDialog,
        openCopyDialog,
        openAuditDialog,
        setSwitchTargetInstance,
    } = api;
    const { currentAction, isBusy } = data;

    const onSwitchAccount = () => setSwitchTargetInstance(inst);

    return (
        <div className="pt-2.5 border-t border-gray-100 dark:border-[#15334d]/80 mt-2 space-y-1.5">
                                                {/* Row 1: Primary Actions Capsule (Launch/Stop, Switch Account, Fast-Forward, Sync PID) */}
                                                <div className="flex items-center rounded-[5px] overflow-hidden bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs w-full">
                                                    {inst.is_running ? (
                                                        <div className="flex-1 flex items-center divide-x divide-slate-200 dark:divide-[#15334d]">
                                                            <button
                                                                type="button"
                                                                disabled={isBusy}
                                                                onClick={() => handleStop(inst.config.id)}
                                                                className="flex-1 flex items-center justify-center py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-l-[4px] transition-colors cursor-pointer disabled:opacity-50"
                                                                title="Stop Instance"
                                                            >
                                                                {currentAction === 'stop' ? (
                                                                    <RotateCw className="w-3.5 h-3.5 animate-spin text-rose-500" />
                                                                ) : (
                                                                    <Square className="w-3.5 h-3.5 fill-current" />
                                                                )}
                                                            </button>
                                                            <button
                                                                type="button"
                                                                disabled={isBusy}
                                                                onClick={() => handleRestart(inst.config.id)}
                                                                className="flex-1 flex items-center justify-center py-1 text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors cursor-pointer disabled:opacity-50"
                                                                title="Restart Instance on Current Account"
                                                            >
                                                                {currentAction === 'restart' ? (
                                                                    <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-500" />
                                                                ) : (
                                                                    <RotateCcw className="w-3.5 h-3.5" />
                                                                )}
                                                            </button>
                                                        </div>
                                                    ) : (
                                                        <button
                                                            type="button"
                                                            disabled={isBusy}
                                                            onClick={() => handleLaunch(inst.config.id)}
                                                            className="flex-1 flex items-center justify-center py-1 text-teal-600 dark:text-teal-400 hover:bg-teal-50 dark:hover:bg-teal-950/40 rounded-l-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
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
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => onSwitchAccount()}
                                                        className="flex-1 flex items-center justify-center py-1 text-sky-600 dark:text-sky-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Switch Account"
                                                    >
                                                        {currentAction === 'switch' ? (
                                                            <RotateCw className="w-3.5 h-3.5 animate-spin text-sky-500" />
                                                        ) : (
                                                            <ArrowRightLeft className="w-3.5 h-3.5" />
                                                        )}
                                                    </button>

                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => handleFastForward(inst.config.id)}
                                                        className="flex-1 flex items-center justify-center py-1 text-amber-600 dark:text-amber-400 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Fast Forward to Next Best"
                                                    >
                                                        {currentAction === 'fast-forward' ? (
                                                            <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-500" />
                                                        ) : (
                                                            <FastForward className="w-3.5 h-3.5" />
                                                        )}
                                                    </button>

                                                    <button
                                                        type="button"
                                                        disabled={isBusy || Boolean(syncingInstanceIds[inst.config.id])}
                                                        onClick={() => handleSync(inst.config.id)}
                                                        className="flex-1 flex items-center justify-center py-1 text-teal-600 dark:text-teal-400 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-r-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Sync PID and Quota"
                                                    >
                                                        <Cpu className={cn("w-3.5 h-3.5 text-teal-500", (currentAction === 'sync' || syncingInstanceIds[inst.config.id]) && "animate-pulse")} />
                                                    </button>
                                                </div>

                                                {/* Row 2: Secondary Actions Capsule (Prompts Tree, Settings, Clone Profile, Audit, More Popover) */}
                                                <div className="flex items-center rounded-[5px] overflow-hidden bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs w-full relative">
                                                    {/* Slot 1: Prompts Tree */}
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => openPromptTree(inst.config.id)}
                                                        className={cn(
                                                            "flex-1 flex items-center justify-center py-1 transition-colors cursor-pointer relative rounded-l-[4px] disabled:opacity-50 disabled:cursor-not-allowed",
                                                            hasActiveTask
                                                                ? "bg-cyan-50 dark:bg-cyan-950/40 text-cyan-600 dark:text-cyan-300 hover:bg-cyan-100 dark:hover:bg-cyan-900/50"
                                                                : "text-slate-600 dark:text-slate-400 hover:text-cyan-600 dark:hover:text-cyan-300 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                                                        )}
                                                        title={hasActiveTask ? "Prompt Tree (Active Task Running)" : "Prompt Tree"}
                                                    >
                                                        <Layers className="w-3.5 h-3.5" />
                                                        {hasActiveTask && (
                                                            <span className="absolute top-0.5 right-0.5 w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse" />
                                                        )}
                                                    </button>

                                                    {/* Slot 2: Settings */}
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => openSettingsDialog(inst.config.id)}
                                                        className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Settings & Sync"
                                                    >
                                                        <SlidersHorizontal className="w-3.5 h-3.5" />
                                                    </button>

                                                    {/* Slot 3: Clone Profile */}
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => openCopyDialog(inst.config.id, inst.config.name)}
                                                        className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Clone Profile"
                                                    >
                                                        <Copy className="w-3.5 h-3.5" />
                                                    </button>

                                                    {/* Slot 4: Audit Trail */}
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => openAuditDialog(inst.config.id, inst.config.name)}
                                                        className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-amber-500 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Audit Trail"
                                                    >
                                                        <History className="w-3.5 h-3.5" />
                                                    </button>

                                                    {/* Slot 5: More Popover Trigger */}
                                                    <div className="flex-1 relative">
                                                        <button
                                                            type="button"
                                                            disabled={isBusy}
                                                            onClick={(e) => {
                                                                e.stopPropagation();
                                                                setCardMoreId(cardMoreId === inst.config.id ? null : inst.config.id);
                                                            }}
                                                            className={cn(
                                                                "w-full flex items-center justify-center py-1 rounded-r-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                                                                cardMoreId === inst.config.id
                                                                    ? "bg-slate-200 dark:bg-[#15334d] text-slate-900 dark:text-white"
                                                                    : "text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                                                            )}
                                                            title="More Actions (Clone Executable, Wipe, Delete)"
                                                        >
                                                            <MoreHorizontal className="w-3.5 h-3.5" />
                                                        </button>

                                                        {/* More Popover Dropdown */}
                                                        {cardMoreId === inst.config.id && (
                                                            <div
                                                                className="absolute right-0 bottom-full mb-1 z-30 min-w-[170px] rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] py-1 text-slate-800 dark:text-slate-200 shadow-xl text-xs"
                                                                onClick={(e) => e.stopPropagation()}
                                                            >
                                                                <button
                                                                    type="button"
                                                                    disabled={isBusy}
                                                                    onClick={() => {
                                                                        setCardMoreId(null);
                                                                        handleCloneExecutable(inst.config.id);
                                                                    }}
                                                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-purple-600 dark:text-purple-400 cursor-pointer disabled:opacity-50"
                                                                >
                                                                    <Cpu className="w-3.5 h-3.5" />
                                                                    <span>Clone Executable</span>
                                                                </button>
                                                                <button
                                                                    type="button"
                                                                    disabled={inst.is_running || isBusy}
                                                                    onClick={() => {
                                                                        setCardMoreId(null);
                                                                        handleWipeSession(inst.config.id);
                                                                    }}
                                                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#15334d] text-amber-600 dark:text-amber-400 cursor-pointer disabled:opacity-40"
                                                                >
                                                                    <KeyRound className="w-3.5 h-3.5 text-amber-500" />
                                                                    <span>Wipe Credentials</span>
                                                                </button>
                                                                {!inst.config.is_default && (
                                                                    <button
                                                                        type="button"
                                                                        disabled={inst.is_running || isBusy || deletingId === inst.config.id}
                                                                        onClick={() => {
                                                                            setCardMoreId(null);
                                                                            handleDelete(inst.config.id);
                                                                        }}
                                                                        className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-rose-50 dark:hover:bg-rose-950/40 text-rose-600 dark:text-rose-400 cursor-pointer disabled:opacity-40"
                                                                    >
                                                                        <Trash2 className="w-3.5 h-3.5" />
                                                                        <span>Delete Profile</span>
                                                                    </button>
                                                                )}
                                                            </div>
                                                        )}
                                                    </div>
                                                </div>
                                            </div>
    );
}
