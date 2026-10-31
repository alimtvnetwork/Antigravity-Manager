import { useCallback, useEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import {
    Sparkles,
    X,
    Play,
    RefreshCw,
    FileDown,
    AlignLeft,
    CheckCircle2,
    Clock,
    Layers,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';
import {
    emptyFlywheelState,
    improvementsService,
    type FlywheelState,
    type ImprovementKind,
} from '../../../services/improvementsService';
import {
    flywheelActionToast,
    resolveImprovementsInstanceId,
} from '../../../services/improvementsFlywheelUtils';

export interface ImprovementsBreakdownModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId?: string;
    conversationTitle?: string;
    repoPath?: string;
    onPromptDispatched?: (promptId?: string) => void;
}

const KINDS: { id: ImprovementKind; label: string }[] = [
    { id: 'system', label: 'Whole System' },
    { id: 'ui', label: 'UI / UX' },
    { id: 'code', label: 'Code Quality' },
    { id: 'ideas', label: 'Innovations' },
];

export function ImprovementsBreakdownModal({
    isOpen,
    onClose,
    instanceId: propInstanceId,
    conversationTitle,
    repoPath,
    onPromptDispatched,
}: ImprovementsBreakdownModalProps) {
    const instanceId = resolveImprovementsInstanceId(propInstanceId || 'default');
    const [kind, setKind] = useState<ImprovementKind>('code');
    const [dryRun, setDryRun] = useState(false);
    const [busy, setBusy] = useState<string | null>(null);
    const [state, setState] = useState<FlywheelState | null>(null);

    const loadStatus = useCallback(async () => {
        setBusy('status');
        try {
            const next = await improvementsService.status(instanceId);
            setState(next);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ImprovementsBreakdownModal.loadStatus',
                triggerAction: 'improvements_status',
            });
            setState(emptyFlywheelState());
        } finally {
            setBusy(null);
        }
    }, [instanceId]);

    useEffect(() => {
        if (isOpen) {
            void loadStatus();
        }
    }, [isOpen, loadStatus]);

    if (!isOpen) return null;

    const runAction = async (action: 'detail' | 'next' | 'plan' | 'check') => {
        setBusy(action);
        try {
            const next =
                action === 'detail'
                    ? await improvementsService.detail(dryRun, instanceId)
                    : action === 'next'
                      ? await improvementsService.next(dryRun, instanceId)
                      : action === 'plan'
                        ? await improvementsService.plan(kind, dryRun, instanceId)
                        : await improvementsService.check(instanceId);
            setState(next);
            showToast(flywheelActionToast(action, dryRun, next), 'success', 2500);
            if (next.dispatched_prompt_id) {
                onPromptDispatched?.(next.dispatched_prompt_id);
            }
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'ImprovementsBreakdownModal.runAction',
                triggerAction: `improvements_${action}`,
            });
        } finally {
            setBusy(null);
        }
    };

    const exportMarkdown = () => {
        if (!state) return;
        const totalBatches = state.batches?.length || 5;
        let md = `# AGM 100 Improvements Flywheel\n\n`;
        md += `- **Kind:** ${state.kind}\n`;
        md += `- **Wave:** ${state.wave}\n`;
        md += `- **Completed Slots:** ${state.completed_batches}/${totalBatches}\n`;
        md += `- **Pending Slots:** ${state.batches.filter((b) => b.status === 'pending').length}\n`;
        md += `- **Workspace:** \`${state.workspace_root || repoPath || 'Current Workspace'}\`\n`;
        md += `- **Conversation:** ${conversationTitle || 'Active Context'}\n`;
        md += `- **Ledger:** \`${state.ledger_rel}\`\n\n`;
        md += `## Task Backlog & Hints\n\n`;
        if (state.task_hints && state.task_hints.length > 0) {
            for (const hint of state.task_hints) {
                md += `- ${hint}\n`;
            }
        } else {
            md += `*(No active pending task files found)*\n`;
        }
        md += `\n## Improvement Slots (Batches of ${state.batch_size || 20})\n\n`;
        for (const b of state.batches) {
            const mark = b.status === 'pending' ? '[ ]' : '[x]';
            md += `### ${mark} ${b.title} (${b.status.toUpperCase()})\n`;
            md += `- **Items:** ${b.item_start}–${b.item_end}\n`;
            md += `- **Prompt Reference:** \`${b.prompt_rel}\`\n\n`;
        }
        md += `\n## AI Execution Protocol\n\n`;
        md += `1. Execute improvements in slots of ${state.batch_size || 20}.\n`;
        md += `2. After each slot of 20 improvements, make an atomic GitMap commit.\n`;
        md += `3. Consolidate completed subtask plans upon completion.\n`;

        const blob = new Blob([md], { type: 'text/markdown' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `flywheel-100-improvements-wave-${state.wave}-${Date.now()}.md`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        showToast('Exported 100 Improvements as Markdown', 'success', 2200);
    };

    const completedCount = state?.completed_batches ?? 0;
    const totalSlots = state?.batches?.length ?? 5;
    const pendingCount = state?.batches?.filter((b) => b.status === 'pending').length ?? totalSlots;

    const modalContent = (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
            <div
                className="w-full max-w-2xl rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between px-5 py-3.5 border-b border-slate-200 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#1f1533]/80">
                    <div className="flex items-center gap-2.5">
                        <div className="w-8 h-8 rounded-lg bg-amber-500/15 border border-amber-500/30 flex items-center justify-center text-amber-500 shadow-2xs">
                            <Sparkles className="w-4 h-4" />
                        </div>
                        <div>
                            <h3 className="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                                <span>Next 100 Improvements Breakdown</span>
                                <span className="text-[10px] font-mono font-semibold px-2 py-0.5 rounded-full bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border border-amber-200/70 dark:border-amber-800/60">
                                    Wave {state?.wave || 1}
                                </span>
                            </h3>
                            <p className="text-[11px] text-slate-500 dark:text-slate-400 truncate max-w-md">
                                {conversationTitle ? `Context: ${conversationTitle}` : `Workspace: ${repoPath || instanceId}`}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="w-7 h-7 rounded-lg flex items-center justify-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 transition-colors cursor-pointer"
                        title="Close modal"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Body */}
                <div className="p-5 overflow-y-auto space-y-4 text-xs">
                    {/* Action Grid */}
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <button
                            type="button"
                            onClick={() => void runAction('detail')}
                            disabled={busy !== null}
                            className="flex flex-col items-start p-3.5 rounded-xl border border-amber-500/30 bg-amber-50/60 dark:bg-amber-950/20 hover:bg-amber-100/70 dark:hover:bg-amber-950/40 text-amber-900 dark:text-amber-100 transition-all cursor-pointer disabled:opacity-50 text-left group"
                        >
                            <div className="flex items-center gap-2 font-bold text-xs text-amber-700 dark:text-amber-300">
                                <AlignLeft className="w-4 h-4 text-amber-500 group-hover:scale-110 transition-transform" />
                                <span>Detail Out Next Breakdown</span>
                            </div>
                            <p className="text-[11px] text-amber-800/80 dark:text-amber-300/70 mt-1 leading-snug">
                                Expand active plans into exhaustive multi-step execution specifications and review dockets.
                            </p>
                        </button>

                        <button
                            type="button"
                            onClick={() => void runAction('next')}
                            disabled={busy !== null || pendingCount === 0}
                            className="flex flex-col items-start p-3.5 rounded-xl border border-emerald-500/30 bg-emerald-50/60 dark:bg-emerald-950/20 hover:bg-emerald-100/70 dark:hover:bg-emerald-950/40 text-emerald-900 dark:text-emerald-100 transition-all cursor-pointer disabled:opacity-50 text-left group"
                        >
                            <div className="flex items-center gap-2 font-bold text-xs text-emerald-700 dark:text-emerald-300">
                                <Play className="w-4 h-4 text-emerald-500 group-hover:scale-110 transition-transform" />
                                <span>Implement Next Batch ({state?.next_batch || 1}/{totalSlots})</span>
                            </div>
                            <p className="text-[11px] text-emerald-800/80 dark:text-emerald-300/70 mt-1 leading-snug">
                                Claim the next pending batch of 10-20 items and enqueue into the active FIFO queue.
                            </p>
                        </button>
                    </div>

                    {/* Progress / Status Strip */}
                    <div className="p-3 rounded-xl border border-slate-200/80 dark:border-[#40285e] bg-slate-50/50 dark:bg-[#1a122a]/50 flex items-center justify-between flex-wrap gap-2">
                        <div className="flex items-center gap-3">
                            <div className="flex items-center gap-1.5 text-slate-700 dark:text-slate-300 font-semibold">
                                <Layers className="w-3.5 h-3.5 text-purple-400" />
                                <span>Slots: {completedCount} / {totalSlots}</span>
                            </div>
                            <span className="text-slate-300 dark:text-slate-700">|</span>
                            <div className="flex items-center gap-1 text-slate-500 dark:text-slate-400">
                                <Clock className="w-3.5 h-3.5 text-amber-500" />
                                <span>{pendingCount} Pending</span>
                            </div>
                        </div>

                        {/* Category Selector & Dry Run Toggle */}
                        <div className="flex items-center gap-3">
                            <label className="flex items-center gap-1.5 text-[11px] text-slate-500 dark:text-slate-400 cursor-pointer select-none">
                                <input
                                    type="checkbox"
                                    checked={dryRun}
                                    onChange={(e) => setDryRun(e.target.checked)}
                                    className="rounded border-slate-300 dark:border-slate-700 text-amber-500 focus:ring-0"
                                />
                                <span>Dry Run</span>
                            </label>
                            <div className="flex items-center gap-1.5">
                                <span className="text-[11px] text-slate-500 dark:text-slate-400">Category:</span>
                                <div className="inline-flex rounded-lg bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-0.5">
                                    {KINDS.map((k) => (
                                        <button
                                            key={k.id}
                                            type="button"
                                            onClick={() => setKind(k.id)}
                                            className={cn(
                                                "px-2 py-0.5 rounded-md text-[10.5px] font-medium transition-colors cursor-pointer",
                                                kind === k.id
                                                    ? "bg-amber-500/15 text-amber-700 dark:text-amber-300 font-bold"
                                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                            )}
                                        >
                                            {k.label}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Batch Slots Pill List */}
                    <div className="space-y-1.5">
                        <div className="flex items-center justify-between text-[11px] font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wide">
                            <span>100 Improvements Execution Slots</span>
                            <span className="font-mono text-[10px] text-slate-400">Wave {state?.wave || 1}</span>
                        </div>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                            {state?.batches?.map((b) => {
                                const isDone = b.status === 'done' || b.status === 'dispatched';
                                return (
                                    <div
                                        key={b.index}
                                        className={cn(
                                            "flex items-center justify-between px-3 py-2 rounded-lg border text-xs transition-colors",
                                            isDone
                                                ? "border-emerald-500/30 bg-emerald-50/30 dark:bg-emerald-950/15 text-emerald-800 dark:text-emerald-200"
                                                : "border-slate-200 dark:border-slate-800 bg-white/60 dark:bg-slate-900/60 text-slate-700 dark:text-slate-300"
                                        )}
                                    >
                                        <div className="flex items-center gap-2 truncate">
                                            {isDone ? (
                                                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                                            ) : (
                                                <span className="w-2 h-2 rounded-full bg-amber-400 shrink-0" />
                                            )}
                                            <span className="font-medium truncate">{b.title}</span>
                                        </div>
                                        <span className="text-[10px] font-mono text-slate-400 shrink-0 ml-1">
                                            {b.item_start}–{b.item_end}
                                        </span>
                                    </div>
                                );
                            })}
                        </div>
                    </div>

                    {/* Task Hints */}
                    {state?.task_hints && state.task_hints.length > 0 && (
                        <div className="space-y-1">
                            <span className="text-[10px] uppercase font-bold text-slate-400">
                                Active Workspace Tasks Detected ({state.task_hints.length})
                            </span>
                            <div className="max-h-24 overflow-y-auto rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/40 p-2 space-y-1 font-mono text-[10.5px] text-slate-600 dark:text-slate-400">
                                {state.task_hints.map((hint, idx) => (
                                    <div key={idx} className="truncate">
                                        • {hint}
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}
                </div>

                {/* Footer Controls */}
                <div className="flex items-center justify-between px-5 py-3 border-t border-slate-200 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#1f1533]/80">
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => void runAction('plan')}
                            disabled={busy !== null}
                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300 hover:bg-amber-500/20 text-xs font-semibold transition-colors cursor-pointer disabled:opacity-50"
                            title="Plan 100 new improvements for this workspace"
                        >
                            <Sparkles className="w-3.5 h-3.5 text-amber-500" />
                            <span>Plan 100 ({kind})</span>
                        </button>
                        <button
                            type="button"
                            onClick={exportMarkdown}
                            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 text-xs font-medium transition-colors cursor-pointer"
                            title="Export markdown roadmap"
                        >
                            <FileDown className="w-3.5 h-3.5" />
                            <span>Export .md</span>
                        </button>
                    </div>

                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => void loadStatus()}
                            disabled={busy !== null}
                            className="p-1.5 rounded-lg border border-slate-200 dark:border-slate-800 text-slate-500 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
                            title="Refresh status"
                        >
                            <RefreshCw className={cn("w-3.5 h-3.5", busy === 'status' && 'animate-spin')} />
                        </button>
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-3.5 py-1.5 rounded-lg bg-slate-200 dark:bg-slate-800 hover:bg-slate-300 dark:hover:bg-slate-700 text-slate-800 dark:text-slate-200 text-xs font-semibold transition-colors cursor-pointer"
                        >
                            Done
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );

    return typeof document !== 'undefined' ? createPortal(modalContent, document.body) : modalContent;
}
