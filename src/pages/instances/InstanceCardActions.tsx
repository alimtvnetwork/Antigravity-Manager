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
    FileDown,
    FileUp,
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { showToast } from '../../components/common/ToastContainer';
import {
    downloadPromptTreeStatusJson,
    openAndRestorePromptBackupFile,
    resendPromptsForInstance,
} from '../../components/instances/PromptTreeView/usePromptTreeCoreActions';
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
        <div className="pt-2.5 border-t border-gray-100 dark:border-[#40285e]/80 mt-2 space-y-1.5">
            {/* Level 1: Primary Lifecycle & Switching (5 items) */}
            <div className="relative flex w-full items-stretch overflow-hidden rounded-full border border-slate-200/80 bg-slate-100 shadow-2xs divide-x divide-slate-200 dark:border-[#40285e] dark:bg-[#090611] dark:divide-[#40285e]">
                {/* Slot 1: Lifecycle (Stop/Restart or Launch) */}
                {inst.is_running ? (
                    <div className="flex-1 flex items-center divide-x divide-slate-200 dark:divide-[#40285e]">
                        <button
                            type="button"
                            disabled={isBusy}
                            onClick={() => handleStop(inst.config.id, inst.pid ?? undefined)}
                            className="flex-1 flex items-center justify-center py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer disabled:opacity-50"
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
                            className="flex-1 flex items-center justify-center py-1 text-sky-700 dark:text-sky-300 hover:bg-sky-600 hover:!text-white hover:shadow-[0_2px_0_0_#0369a1] active:translate-y-px transition-all cursor-pointer disabled:opacity-50"
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
                        className="flex-1 flex items-center justify-center py-1 text-teal-600 dark:text-teal-400 hover:bg-teal-50 dark:hover:bg-teal-950/40 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                        title="Launch Instance"
                    >
                        {currentAction === 'launch' ? (
                            <RotateCw className="w-3.5 h-3.5 animate-spin text-teal-500" />
                        ) : (
                            <Play className="w-3.5 h-3.5 fill-current" />
                        )}
                    </button>
                )}

                {/* Slot 2: Switch Account (3D Yellow Hover) */}
                <button
                    type="button"
                    disabled={isBusy}
                    onClick={() => onSwitchAccount()}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                        "bg-transparent text-slate-600 dark:text-slate-400",
                        "hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:!text-black hover:[&_svg]:!text-black hover:font-bold hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.25)]",
                        "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
                    )}
                    title="Switch Account"
                >
                    {currentAction === 'switch' ? (
                        <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-700" />
                    ) : (
                        <ArrowRightLeft className="w-3.5 h-3.5" />
                    )}
                </button>

                {/* Slot 3: Fast Forward (3D Yellow Hover) */}
                <button
                    type="button"
                    disabled={isBusy}
                    onClick={() => handleFastForward(inst.config.id)}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                        "bg-transparent text-slate-600 dark:text-slate-400",
                        "hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:!text-black hover:[&_svg]:!text-black hover:font-bold hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.25)]",
                        "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
                    )}
                    title="Fast Forward to Next Best"
                >
                    {currentAction === 'fast-forward' ? (
                        <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-700" />
                    ) : (
                        <FastForward className="w-3.5 h-3.5" />
                    )}
                </button>

                {/* Slot 4: Audit Trail */}
                <button
                    type="button"
                    disabled={isBusy}
                    onClick={() => openAuditDialog(inst.config.id, inst.config.name)}
                    className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-amber-500 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Audit Trail"
                >
                    <History className="w-3.5 h-3.5" />
                </button>

                {/* Slot 5: Sync PID & Quota */}
                <button
                    type="button"
                    disabled={isBusy || Boolean(syncingInstanceIds[inst.config.id])}
                    onClick={() => handleSync(inst.config.id)}
                    className="flex-1 flex items-center justify-center py-1 text-teal-600 dark:text-teal-400 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Sync PID and Quota"
                >
                    <Cpu className={cn("w-3.5 h-3.5 text-teal-500", (currentAction === 'sync' || syncingInstanceIds[inst.config.id]) && "animate-pulse")} />
                </button>
            </div>

            {/* Level 2: Utilities & Profile Management (6 items) */}
            <div className="relative flex w-full items-stretch overflow-hidden rounded-full border border-slate-200/80 bg-slate-100 shadow-2xs divide-x divide-slate-200 dark:border-[#40285e] dark:bg-[#090611] dark:divide-[#40285e]">
                {/* Slot 1: Prompt Tree (Active task cyan dot) */}
                <button
                    id={`prompt-tree-btn-${inst.config.id}`}
                    type="button"
                    disabled={isBusy}
                    onClick={() => openPromptTree(inst.config.id)}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 transition-colors cursor-pointer relative disabled:opacity-50 disabled:cursor-not-allowed",
                        hasActiveTask
                            ? "bg-cyan-50 dark:bg-cyan-950/40 text-cyan-600 dark:text-cyan-300 hover:bg-cyan-100 dark:hover:bg-cyan-900/50"
                            : "text-slate-600 dark:text-slate-400 hover:text-cyan-600 dark:hover:text-cyan-300 hover:bg-slate-200 dark:hover:bg-[#40285e]"
                    )}
                    title={hasActiveTask ? "Prompt Tree (Active Task Running)" : "Prompt Tree"}
                >
                    <Layers className="w-3.5 h-3.5" />
                    {hasActiveTask && (
                        <span className="absolute top-0.5 right-0.5 w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse" />
                    )}
                </button>

                {/* Slot 2: Settings & Sync */}
                <button
                    type="button"
                    disabled={isBusy}
                    onClick={() => openSettingsDialog(inst.config.id)}
                    className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-300 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Settings & Sync"
                >
                    <SlidersHorizontal className="w-3.5 h-3.5" />
                </button>

                {/* Slot 3: Clone Profile */}
                <button
                    type="button"
                    disabled={isBusy}
                    onClick={() => openCopyDialog(inst.config.id, inst.config.name)}
                    className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Clone Profile"
                >
                    <Copy className="w-3.5 h-3.5" />
                </button>

                {/* Slot 4: Clone Executable */}
                <button
                    type="button"
                    disabled={isBusy}
                    onClick={() => handleCloneExecutable(inst.config.id)}
                    className="flex-1 flex items-center justify-center py-1 text-slate-600 dark:text-slate-400 hover:text-amber-500 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                    title="Clone Executable"
                >
                    <Cpu className="w-3.5 h-3.5" />
                </button>

                {/* Slot 5: Wipe Session / Reset */}
                <button
                    type="button"
                    disabled={inst.is_running || isBusy}
                    onClick={() => handleWipeSession(inst.config.id)}
                    className="flex-1 flex items-center justify-center py-1 text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                    title="Wipe Session / Credentials"
                >
                    <KeyRound className="w-3.5 h-3.5" />
                </button>

                {/* Slot 6: Delete Profile / More Actions Popover */}
                <div className="flex-1 relative">
                    <button
                        type="button"
                        disabled={isBusy}
                        onClick={(e) => {
                            e.stopPropagation();
                            setCardMoreId(cardMoreId === inst.config.id ? null : inst.config.id);
                        }}
                        className={cn(
                            "w-full flex items-center justify-center py-1 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                            cardMoreId === inst.config.id
                                ? "bg-slate-200 dark:bg-[#40285e] text-slate-900 dark:text-white"
                                : "text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-[#40285e]"
                        )}
                        title="More Actions"
                    >
                        <MoreHorizontal className="w-3.5 h-3.5" />
                    </button>

                    {/* More Popover Dropdown */}
                    {cardMoreId === inst.config.id && (
                        <div
                            className="absolute right-0 bottom-full mb-1 z-30 min-w-[170px] rounded-[5px] border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] py-1 text-slate-800 dark:text-slate-200 shadow-xl text-xs"
                            onClick={(e) => e.stopPropagation()}
                        >
                            <button
                                type="button"
                                disabled={isBusy}
                                onClick={async () => {
                                    setCardMoreId(null);
                                    try {
                                        const count = await resendPromptsForInstance(inst.config.id, (msg) => {
                                            if (msg) showToast(msg, 'info');
                                        });
                                        showToast(`Resent ${count} prompt(s) for ${inst.config.name}`, 'success');
                                    } catch (e: any) {
                                        showToast(e?.message || 'Failed to resend prompts', 'error');
                                    }
                                }}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-blue-600 dark:text-cyan-300 cursor-pointer disabled:opacity-50"
                                title="Re-send running and queued prompts in FIFO order"
                            >
                                <RotateCcw className="w-3.5 h-3.5 text-cyan-500" />
                                <span>Resend Prompts (FIFO)</span>
                            </button>
                            <button
                                type="button"
                                disabled={isBusy}
                                onClick={async () => {
                                    setCardMoreId(null);
                                    await downloadPromptTreeStatusJson('running_and_queued', undefined, (msg) => {
                                        if (msg) showToast(msg, 'info');
                                    });
                                }}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-purple-600 dark:text-purple-300 cursor-pointer disabled:opacity-50"
                                title="Export running and queued prompts to JSON"
                            >
                                <FileDown className="w-3.5 h-3.5 text-purple-500" />
                                <span>Export Prompts (.json)</span>
                            </button>
                            <button
                                type="button"
                                disabled={isBusy}
                                onClick={async () => {
                                    setCardMoreId(null);
                                    try {
                                        await openAndRestorePromptBackupFile(inst.config.id, (msg) => {
                                            if (msg) showToast(msg, 'info');
                                        });
                                    } catch (e: any) {
                                        showToast(e?.message || 'Failed to restore prompts', 'error');
                                    }
                                }}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-emerald-600 dark:text-emerald-400 cursor-pointer disabled:opacity-50"
                                title="Restore prompts from JSON backup file"
                            >
                                <FileUp className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Restore Prompts (.json)</span>
                            </button>
                            {!inst.config.is_default && (
                                <>
                                    <div className="my-1 border-t border-slate-100 dark:border-[#40285e]" />
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
                                </>
                            )}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
