import { useCallback, useEffect, useState } from 'react';
import { AlignLeft, FileDown, ListChecks, Play, RefreshCw, Sparkles } from 'lucide-react';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import { useInstanceStore } from '../../stores/useInstanceStore';
import {
    emptyFlywheelState,
    improvementsService,
    type FlywheelState,
    type ImprovementKind,
} from '../../services/improvementsService';
import {
    flywheelActionToast,
    resolveImprovementsInstanceId,
    workspaceLabel,
} from '../../services/improvementsFlywheelUtils';

const KINDS: { id: ImprovementKind; label: string }[] = [
    { id: 'system', label: 'System' },
    { id: 'ui', label: 'UI' },
    { id: 'code', label: 'Code' },
    { id: 'ideas', label: 'Ideas' },
];

export default function ImprovementsFlywheel() {
    const activeInstanceId = useInstanceStore((s) => s.activeInstanceId);
    const instanceId = resolveImprovementsInstanceId(activeInstanceId);
    const [kind, setKind] = useState<ImprovementKind>('system');
    const [dryRun, setDryRun] = useState(true);
    const [busy, setBusy] = useState<string | null>(null);
    const [state, setState] = useState<FlywheelState | null>(null);

    const loadStatus = useCallback(async () => {
        setBusy('status');
        try {
            const next = await improvementsService.status(instanceId);
            setState(next);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ImprovementsFlywheel.loadStatus',
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

    const run = async (action: 'plan' | 'next' | 'check' | 'detail') => {
        setBusy(action);
        try {
            const next =
                action === 'plan'
                    ? await improvementsService.plan(kind, dryRun, instanceId)
                    : action === 'next'
                      ? await improvementsService.next(dryRun, instanceId)
                      : action === 'detail'
                        ? await improvementsService.detail(dryRun, instanceId)
                        : await improvementsService.check(instanceId);
            setState(next);
            showToast(flywheelActionToast(action, dryRun, next), 'success', 2200);
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'ImprovementsFlywheel.run',
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
        md += `- **Workspace:** \`${state.workspace_root || 'Current Workspace'}\`\n`;
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

    const pending = state?.batches.filter((b) => b.status === 'pending').length ?? 5;
    const done = state?.completed_batches ?? 0;
    const totalSlots = state?.batches.length ?? 5;
    const slotSize = state?.batch_size ?? 20;

    return (
        <div
            id="agm-improvements-flywheel"
            className="agm-hover-shade rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-xs space-y-4"
        >
            <div className="flex items-start justify-between gap-3 pb-3 border-b border-slate-200 dark:border-[#40285e]">
                <div>
                    <h2 className="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                        <Sparkles className="w-4 h-4 text-amber-500" />
                        100 Improvements Flywheel
                    </h2>
                    <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                        Plan 100, implement the next 10, or detail a 100-step breakdown
                        for any pending tasks in the active instance workspace (any project).
                        Same path as{' '}
                        <code className="font-mono">agm improvements</code> and{' '}
                        <code className="font-mono">/improvements</code>.
                    </p>
                </div>
                <span className="text-[10px] uppercase font-bold tracking-wide px-2 py-1 rounded-full bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border border-amber-200/70 dark:border-amber-800/60">
                    {done}/{totalSlots} slots
                </span>
            </div>

            <div className="flex flex-wrap items-center gap-2">
                <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#201538]/80 overflow-hidden">
                    <select
                        value={kind}
                        onChange={(e) => setKind(e.target.value as ImprovementKind)}
                        className="bg-transparent border-0 focus:ring-0 focus:outline-none text-xs font-semibold px-3 py-1.5 text-slate-700 dark:text-slate-200 cursor-pointer"
                        aria-label="Improvement kind"
                    >
                        {KINDS.map((item) => (
                            <option key={item.id} value={item.id}>
                                {item.label}
                            </option>
                        ))}
                    </select>
                    <span className="w-px self-stretch bg-slate-200 dark:bg-[#40285e]" />
                    <button
                        type="button"
                        onClick={() => void run('plan')}
                        disabled={busy !== null}
                        className="agm-hover-shade px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50"
                    >
                        {busy === 'plan' ? (
                            <RefreshCw className="w-3.5 h-3.5 animate-spin inline mr-1" />
                        ) : (
                            <ListChecks className="w-3.5 h-3.5 inline mr-1" />
                        )}
                        Plan 100
                    </button>
                    <span className="w-px self-stretch bg-slate-200 dark:bg-[#40285e]" />
                    <button
                        type="button"
                        onClick={() => void run('next')}
                        disabled={busy !== null}
                        className="agm-hover-shade px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50"
                    >
                        {busy === 'next' ? (
                            <RefreshCw className="w-3.5 h-3.5 animate-spin inline mr-1" />
                        ) : (
                            <Play className="w-3.5 h-3.5 inline mr-1" />
                        )}
                        Next {slotSize}
                    </button>
                    <span className="w-px self-stretch bg-slate-200 dark:bg-[#40285e]" />
                    <button
                        type="button"
                        onClick={() => void run('detail')}
                        disabled={busy !== null}
                        className="agm-hover-shade px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50"
                    >
                        {busy === 'detail' ? (
                            <RefreshCw className="w-3.5 h-3.5 animate-spin inline mr-1" />
                        ) : (
                            <AlignLeft className="w-3.5 h-3.5 inline mr-1" />
                        )}
                        Next breakdown
                    </button>
                    <span className="w-px self-stretch bg-slate-200 dark:bg-[#40285e]" />
                    <button
                        type="button"
                        onClick={() => void loadStatus()}
                        disabled={busy !== null}
                        className="agm-hover-shade px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50"
                    >
                        {busy === 'status' ? (
                            <RefreshCw className="w-3.5 h-3.5 animate-spin inline mr-1" />
                        ) : null}
                        Status
                    </button>
                    <span className="w-px self-stretch bg-slate-200 dark:bg-[#40285e]" />
                    <button
                        type="button"
                        onClick={exportMarkdown}
                        className="agm-hover-shade px-3 py-1.5 text-xs font-semibold text-sky-600 dark:text-sky-400 hover:text-sky-500 cursor-pointer"
                        title="Export current 100 improvements roadmap as Markdown (.md) for AI"
                    >
                        <FileDown className="w-3.5 h-3.5 inline mr-1" />
                        Export MD
                    </button>
                </div>

                <label className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 cursor-pointer select-none">
                    <input
                        type="checkbox"
                        checked={dryRun}
                        onChange={(e) => setDryRun(e.target.checked)}
                        className="checkbox checkbox-xs checkbox-primary"
                    />
                    Dry-run (no prompt dispatch)
                </label>
            </div>

            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                {pending} pending · action <span className="font-mono">{state?.last_action ?? 'unplanned'}</span> ·
                workspace <span className="font-mono">{workspaceLabel(state?.workspace_root)}</span> ·
                {state?.task_hints?.length ?? 0} tasks ·
                ledger <span className="font-mono">.ai-memory/improvements/flywheel.json</span>
            </p>
            {(state?.task_hints?.length ?? 0) > 0 ? (
                <ul className="flex flex-wrap gap-1.5 text-[10px] font-mono">
                    {state?.task_hints?.slice(0, 8).map((hint) => (
                        <li
                            key={hint}
                            className="px-2 py-0.5 rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#090611] text-slate-600 dark:text-slate-300"
                        >
                            {hint}
                        </li>
                    ))}
                </ul>
            ) : null}

            <ol className="grid grid-cols-1 sm:grid-cols-2 gap-1.5 text-[11px] font-mono">
                {(state?.batches ?? []).map((batch) => (
                    <li
                        key={batch.index}
                        className="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-lg bg-slate-50 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e]"
                    >
                        <span className="truncate text-slate-700 dark:text-slate-300">{batch.title}</span>
                        <span
                            className={
                                batch.status === 'pending'
                                    ? 'text-amber-600 dark:text-amber-400'
                                    : 'text-emerald-600 dark:text-emerald-400'
                            }
                        >
                            {batch.status}
                        </span>
                    </li>
                ))}
            </ol>
        </div>
    );
}
