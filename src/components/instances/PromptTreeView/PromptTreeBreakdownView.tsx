import { useCallback, useEffect, useState } from 'react';
import {
    Sparkles,
    RefreshCw,
    CheckCircle2,
    Clock,
    Layers,
    Terminal,
    ArrowRight,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';
import {
    emptyFlywheelState,
    improvementsService,
    type FlywheelState,
    type ImprovementKind,
    type ImprovementBatch,
} from '../../../services/improvementsService';
import {
    flywheelActionToast,
    resolveImprovementsInstanceId,
} from '../../../services/improvementsFlywheelUtils';

export interface PromptTreeBreakdownViewProps {
    instanceId?: string;
    conversationTitle?: string;
    repoPath?: string;
    onPromptDispatched?: (promptId?: string) => void;
}

const CATEGORIES: { id: ImprovementKind; label: string; icon: string }[] = [
    { id: 'code', label: 'Code Quality', icon: '💻' },
    { id: 'ui', label: 'UI / UX Design', icon: '🎨' },
    { id: 'system', label: 'Whole System', icon: '⚙️' },
    { id: 'ideas', label: 'Innovations', icon: '💡' },
];

const STEP_COUNTS = [10, 20, 50, 100];

export function PromptTreeBreakdownView({
    instanceId: propInstanceId,
    conversationTitle: _conversationTitle,
    repoPath: _repoPath,
    onPromptDispatched,
}: PromptTreeBreakdownViewProps) {
    const instanceId = resolveImprovementsInstanceId(propInstanceId || 'default');
    const [kind, setKind] = useState<ImprovementKind>('code');
    const [stepCount, setStepCount] = useState<number>(100);
    const [dryRun, setDryRun] = useState(false);
    const [busy, setBusy] = useState<string | null>(null);
    const [state, setState] = useState<FlywheelState | null>(null);

    const loadStatus = useCallback(async () => {
        setBusy('status');
        try {
            const res = await improvementsService.status(instanceId);
            setState(res);
        } catch (e) {
            useErrorStore.getState().trackWarning(e, {
                source: 'PromptTreeBreakdownView.loadStatus',
                triggerAction: 'improvements_status',
            });
            setState(emptyFlywheelState());
        } finally {
            setBusy(null);
        }
    }, [instanceId]);

    useEffect(() => {
        void loadStatus();
    }, [loadStatus]);

    const runAction = async (action: 'plan' | 'next' | 'detail' | 'check') => {
        setBusy(action);
        try {
            let res: FlywheelState;
            if (action === 'plan') {
                res = await improvementsService.plan(kind, dryRun, instanceId);
            } else if (action === 'next') {
                res = await improvementsService.next(dryRun, instanceId);
            } else if (action === 'detail') {
                res = await improvementsService.detail(dryRun, instanceId);
            } else {
                res = await improvementsService.check(instanceId);
            }
            setState(res);
            const msg = flywheelActionToast(action, dryRun, res);
            showToast(msg, 'success');
            if (res.dispatched_prompt_id) {
                onPromptDispatched?.(res.dispatched_prompt_id);
            }
        } catch (e) {
            useErrorStore.getState().trackWarning(e, {
                source: `PromptTreeBreakdownView.${action}`,
                triggerAction: `improvements_${action}`,
            });
            showToast(`Improvements ${action} failed`, 'error');
        } finally {
            setBusy(null);
        }
    };

    const slots: ImprovementBatch[] = state?.batches || [];
    const completedCount = slots.filter((s) => s.status === 'completed').length;
    const progressPercent = slots.length > 0 ? Math.round((completedCount / slots.length) * 100) : 0;

    return (
        <div className="flex flex-col h-full overflow-y-auto space-y-4 p-4 text-slate-200">
            {/* Header / Summary Bar */}
            <div className="rounded-xl border border-amber-500/20 bg-gradient-to-r from-amber-950/20 via-slate-900/60 to-purple-950/20 p-4 backdrop-blur-md shadow-sm">
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-400">
                            <Sparkles className="w-5 h-5" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2">
                                <h3 className="text-sm font-bold text-white tracking-wide">
                                    100 Improvements Flywheel
                                </h3>
                                <span className="rounded-md border border-amber-500/30 bg-amber-500/10 px-2 py-0.5 text-[10px] font-mono font-semibold text-amber-300">
                                    Wave {state?.wave || 1}
                                </span>
                                {dryRun && (
                                    <span className="rounded-md border border-cyan-500/30 bg-cyan-500/10 px-1.5 py-0.5 text-[10px] font-mono text-cyan-300">
                                        Dry Run
                                    </span>
                                )}
                            </div>
                            <p className="text-xs text-slate-400 mt-0.5">
                                Plan 100 steps, execute the next 10, or expand multi-step subtasks inside this active workspace.
                            </p>
                        </div>
                    </div>

                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => void loadStatus()}
                            disabled={busy !== null}
                            className="inline-flex items-center gap-1.5 rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1.5 text-xs font-semibold text-slate-300 hover:bg-slate-700 hover:text-white transition-colors cursor-pointer"
                            title="Refresh Flywheel Status"
                        >
                            <RefreshCw className={cn("w-3.5 h-3.5", busy === 'status' && "animate-spin text-amber-400")} />
                            <span>Refresh</span>
                        </button>
                    </div>
                </div>

                {/* Progress bar */}
                <div className="mt-3">
                    <div className="flex items-center justify-between text-xs mb-1 font-mono text-slate-400">
                        <span>Wave Progress ({completedCount}/{slots.length || 10} batches)</span>
                        <span className="text-amber-300 font-bold">{progressPercent}%</span>
                    </div>
                    <div className="h-1.5 w-full rounded-full bg-slate-800 overflow-hidden">
                        <div
                            className="h-full bg-gradient-to-r from-amber-500 to-emerald-400 transition-all duration-300"
                            style={{ width: `${progressPercent}%` }}
                        />
                    </div>
                </div>
            </div>

            {/* Controls Card: Category + Steps + Action Buttons */}
            <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-4 space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {/* Category Selection */}
                    <div>
                        <label className="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">
                            Improvement Category
                        </label>
                        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                            {CATEGORIES.map((cat) => (
                                <button
                                    key={cat.id}
                                    type="button"
                                    onClick={() => setKind(cat.id)}
                                    className={cn(
                                        "flex flex-col items-center justify-center p-2.5 rounded-lg border text-xs font-semibold transition-all cursor-pointer",
                                        kind === cat.id
                                            ? "border-amber-500/60 bg-amber-500/15 text-amber-300 shadow-sm"
                                            : "border-slate-800 bg-slate-950/40 text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
                                    )}
                                >
                                    <span className="text-base mb-1">{cat.icon}</span>
                                    <span>{cat.label}</span>
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Step Batch Sizing */}
                    <div>
                        <div className="flex items-center justify-between mb-2">
                            <label className="text-xs font-bold text-slate-400 uppercase tracking-wider">
                                Task Batch Count
                            </label>
                            <label className="flex items-center gap-1.5 text-xs text-slate-400 cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={dryRun}
                                    onChange={(e) => setDryRun(e.target.checked)}
                                    className="rounded border-slate-700 bg-slate-800 text-amber-500 focus:ring-0 cursor-pointer"
                                />
                                <span>Dry Run</span>
                            </label>
                        </div>
                        <div className="grid grid-cols-4 gap-2">
                            {STEP_COUNTS.map((cnt) => (
                                <button
                                    key={cnt}
                                    type="button"
                                    onClick={() => setStepCount(cnt)}
                                    className={cn(
                                        "py-2.5 px-3 rounded-lg border text-center font-mono text-xs font-bold transition-all cursor-pointer",
                                        stepCount === cnt
                                            ? "border-amber-500/60 bg-amber-500/15 text-amber-300 shadow-sm"
                                            : "border-slate-800 bg-slate-950/40 text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
                                    )}
                                >
                                    {cnt} Steps
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Primary Action Buttons */}
                <div className="flex flex-wrap items-center gap-2.5 pt-2 border-t border-slate-800/80">
                    <button
                        type="button"
                        onClick={() => void runAction('next')}
                        disabled={busy !== null}
                        className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-amber-500 to-amber-600 px-4 py-2 text-xs font-bold text-slate-950 hover:from-amber-400 hover:to-amber-500 transition-all shadow-md active:scale-[0.98] cursor-pointer disabled:opacity-50"
                    >
                        <Sparkles className="w-4 h-4 text-slate-950" />
                        <span>Auto-Enqueue Next Batch ({stepCount})</span>
                    </button>

                    <button
                        type="button"
                        onClick={() => void runAction('detail')}
                        disabled={busy !== null}
                        className="inline-flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800/90 px-3.5 py-2 text-xs font-semibold text-slate-200 hover:bg-slate-700 hover:text-white transition-all cursor-pointer disabled:opacity-50"
                    >
                        <Layers className="w-4 h-4 text-cyan-400" />
                        <span>Detail Out Tasks</span>
                    </button>

                    <button
                        type="button"
                        onClick={() => void runAction('plan')}
                        disabled={busy !== null}
                        className="inline-flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800/90 px-3.5 py-2 text-xs font-semibold text-slate-200 hover:bg-slate-700 hover:text-white transition-all cursor-pointer disabled:opacity-50"
                    >
                        <Sparkles className="w-4 h-4 text-purple-400" />
                        <span>Plan 100 ({kind.toUpperCase()})</span>
                    </button>

                    <button
                        type="button"
                        onClick={() => void runAction('check')}
                        disabled={busy !== null}
                        className="inline-flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800/90 px-3.5 py-2 text-xs font-semibold text-slate-200 hover:bg-slate-700 hover:text-white transition-all cursor-pointer disabled:opacity-50"
                    >
                        <RefreshCw className="w-4 h-4 text-emerald-400" />
                        <span>Check Backlog</span>
                    </button>
                </div>
            </div>

            {/* Execution Slots Grid */}
            <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-4">
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <Layers className="w-4 h-4 text-amber-400" />
                        <h4 className="text-xs font-bold text-white uppercase tracking-wider">
                            Execution Slots (Wave {state?.wave || 1})
                        </h4>
                    </div>
                    <button
                        type="button"
                        onClick={() => void runAction('plan')}
                        disabled={busy !== null}
                        className="text-xs font-semibold text-purple-400 hover:text-purple-300 transition-colors inline-flex items-center gap-1 cursor-pointer"
                    >
                        <span>Re-plan 100</span>
                        <ArrowRight className="w-3 h-3" />
                    </button>
                </div>

                {slots.length === 0 ? (
                    <div className="text-center py-6 border border-dashed border-slate-800 rounded-md text-xs text-slate-500">
                        No active slots in this wave yet. Click "Plan 100 (CODE)" or "Detail Out Tasks" above.
                    </div>
                ) : (
                    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-5 gap-2">
                        {slots.map((slot) => {
                            const isCompleted = slot.status === 'completed';
                            const isRunning = slot.status === 'running';
                            return (
                                <div
                                    key={slot.index}
                                    className={cn(
                                        "rounded-md border p-2.5 flex flex-col justify-between transition-all",
                                        isCompleted
                                            ? "border-emerald-500/30 bg-emerald-950/20"
                                            : isRunning
                                            ? "border-amber-500/40 bg-amber-950/20 animate-pulse"
                                            : "border-slate-800 bg-slate-950/40"
                                    )}
                                >
                                    <div className="flex items-center justify-between mb-1.5">
                                        <span className="font-mono text-[11px] font-bold text-slate-300">
                                            Slot {String(slot.index).padStart(2, '0')}
                                        </span>
                                        {isCompleted ? (
                                            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                                        ) : isRunning ? (
                                            <Clock className="w-3.5 h-3.5 text-amber-400" />
                                        ) : (
                                            <span className="w-2 h-2 rounded-full bg-slate-700" />
                                        )}
                                    </div>
                                    <div className="text-[10px] text-slate-400 truncate mb-2">
                                        Steps {slot.item_start}–{slot.item_end}
                                    </div>
                                    <span
                                        className={cn(
                                            "inline-block rounded px-1.5 py-0.5 text-[9px] font-mono text-center font-semibold capitalize",
                                            isCompleted
                                                ? "bg-emerald-500/20 text-emerald-300"
                                                : isRunning
                                                ? "bg-amber-500/20 text-amber-300"
                                                : "bg-slate-800 text-slate-400"
                                        )}
                                    >
                                        {slot.status}
                                    </span>
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>

            {/* Active Workspace Tasks */}
            {state?.task_hints && state.task_hints.length > 0 && (
                <div className="rounded-lg border border-slate-800 bg-slate-900/60 p-3">
                    <h4 className="text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">
                        Pending Workspace Plans ({state.task_hints.length})
                    </h4>
                    <div className="flex flex-wrap gap-1.5">
                        {state.task_hints.map((task, idx) => (
                            <span
                                key={idx}
                                className="rounded-md border border-slate-800 bg-slate-950/60 px-2 py-1 text-[11px] font-mono text-slate-300 hover:border-slate-700 transition-colors"
                            >
                                {task}
                            </span>
                        ))}
                    </div>
                </div>
            )}

            {/* CLI Parity Hint */}
            <div className="rounded-md border border-slate-800/80 bg-slate-950/80 p-2.5 flex items-center justify-between text-xs text-slate-400">
                <div className="flex items-center gap-2">
                    <Terminal className="w-4 h-4 text-cyan-400 shrink-0" />
                    <span className="font-mono text-[11px] text-slate-300">
                        agm improvements next --instance {instanceId}
                    </span>
                </div>
                <span className="text-[10px] text-slate-500">CLI & UI Parity Guaranteed</span>
            </div>
        </div>
    );
}
