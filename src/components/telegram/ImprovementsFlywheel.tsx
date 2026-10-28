import { useCallback, useEffect, useState } from 'react';
import { ListChecks, Play, RefreshCw, Sparkles } from 'lucide-react';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import {
    improvementsService,
    type FlywheelState,
    type ImprovementKind,
} from '../../services/improvementsService';

const KINDS: { id: ImprovementKind; label: string }[] = [
    { id: 'system', label: 'System' },
    { id: 'ui', label: 'UI' },
    { id: 'code', label: 'Code' },
    { id: 'ideas', label: 'Ideas' },
];

export default function ImprovementsFlywheel() {
    const [kind, setKind] = useState<ImprovementKind>('system');
    const [dryRun, setDryRun] = useState(true);
    const [busy, setBusy] = useState<string | null>(null);
    const [state, setState] = useState<FlywheelState | null>(null);

    const loadStatus = useCallback(async () => {
        setBusy('status');
        try {
            const next = await improvementsService.status();
            setState(next);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ImprovementsFlywheel.loadStatus',
                triggerAction: 'improvements_status',
            });
        } finally {
            setBusy(null);
        }
    }, []);

    useEffect(() => {
        void loadStatus();
    }, [loadStatus]);

    const run = async (action: 'plan' | 'next' | 'check') => {
        setBusy(action);
        try {
            const next =
                action === 'plan'
                    ? await improvementsService.plan(kind, dryRun)
                    : action === 'next'
                      ? await improvementsService.next(dryRun)
                      : await improvementsService.check();
            setState(next);
            showToast(
                action === 'plan'
                    ? dryRun
                        ? 'Planned 10 batches of 10 (dry-run, no prompt dispatched)'
                        : 'Planned 100 improvements and queued the discovery prompt'
                    : action === 'next'
                      ? dryRun
                          ? `Next batch ${next.next_batch} previewed (dry-run)`
                          : `Dispatched next 10 (batch ${Math.max(1, next.completed_batches)})`
                      : 'Ledger refreshed',
                'success',
                2200,
            );
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'ImprovementsFlywheel.run',
                triggerAction: `improvements_${action}`,
            });
        } finally {
            setBusy(null);
        }
    };

    const pending = state?.batches.filter((b) => b.status === 'pending').length ?? 10;
    const done = state?.completed_batches ?? 0;

    return (
        <div className="agm-hover-shade rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-xs space-y-4">
            <div className="flex items-start justify-between gap-3 pb-3 border-b border-slate-200 dark:border-[#40285e]">
                <div>
                    <h2 className="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                        <Sparkles className="w-4 h-4 text-amber-500" />
                        100 Improvements Flywheel
                    </h2>
                    <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                        Plan 100 items, then implement 10 at a time. Same path as{' '}
                        <code className="font-mono">agm improvements</code> and{' '}
                        <code className="font-mono">/improvements</code>.
                    </p>
                </div>
                <span className="text-[10px] uppercase font-bold tracking-wide px-2 py-1 rounded-full bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border border-amber-200/70 dark:border-amber-800/60">
                    {done}/10 batches
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
                        Next 10
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
                ledger <span className="font-mono">.ai-memory/improvements/flywheel.json</span>
            </p>

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
