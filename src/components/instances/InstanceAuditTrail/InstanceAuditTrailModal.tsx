import { useState, useEffect, useCallback } from 'react';
import {
    History,
    X,
    RotateCw,
    AlertCircle,
    Info,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { getInstanceSwitchHistory } from '../../../services/instanceService';
import { useErrorStore } from '../../../stores/error-store';
import type { InstanceSwitchHistoryResponse } from '../../../types/audit';
import type { InstanceAuditTrailModalProps } from './types';
import { SwitchRecordCard, type SwitchRecord } from './SwitchRecordCard';

const AUTO_EXPAND_COUNT = 2;
const HISTORY_LIMIT = 10;

export default function InstanceAuditTrailModal({
    instance,
    isOpen,
    onClose,
}: InstanceAuditTrailModalProps) {
    const [history, setHistory] = useState<InstanceSwitchHistoryResponse | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [unmaskedEmails, setUnmaskedEmails] = useState<Record<string, boolean>>({});
    const [expandedCards, setExpandedCards] = useState<Record<string, boolean>>({});
    const [expandedRawFacts, setExpandedRawFacts] = useState<Record<string, boolean>>({});

    const loadAuditHistory = useCallback(async () => {
        if (!instance?.id) return;
        setIsLoading(true);
        setError(null);
        try {
            const res = await getInstanceSwitchHistory(instance.id, HISTORY_LIMIT);
            setHistory(res);
            // Auto-expand the latest switches (first 2)
            const initialExpanded: Record<string, boolean> = {};
            (res.recent_switches || []).slice(0, AUTO_EXPAND_COUNT).forEach((sw) => {
                initialExpanded[sw.id] = true;
            });
            setExpandedCards(initialExpanded);
        } catch (err: unknown) {
            console.error('[InstanceAuditTrailModal] Failed to fetch switch history:', err);
            // Tracked in the error module; modal already shows the error state to the user.
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceAuditTrailModal.fetchHistory',
                triggerAction: 'fetch_switch_history',
            });
            const msg = err instanceof Error ? err.message : String(err);
            setError(msg || 'Failed to fetch audit trail history');
        } finally {
            setIsLoading(false);
        }
    }, [instance?.id]);

    useEffect(() => {
        if (isOpen && instance?.id) {
            loadAuditHistory();
        } else {
            setHistory(null);
            setError(null);
        }
    }, [isOpen, instance?.id, loadAuditHistory]);

    if (!isOpen || !instance) return null;

    const toggleEmailMask = (switchId: string) => {
        setUnmaskedEmails((prev) => ({ ...prev, [switchId]: !prev[switchId] }));
    };

    const toggleCardExpand = (switchId: string) => {
        setExpandedCards((prev) => ({ ...prev, [switchId]: !prev[switchId] }));
    };

    const toggleRawFacts = (switchId: string) => {
        setExpandedRawFacts((prev) => ({ ...prev, [switchId]: !prev[switchId] }));
    };

    const totalSwitches = history?.total_switches ?? (history?.recent_switches?.length || 0);

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 sm:p-6 animate-in fade-in duration-200">
            <div className="flex h-[90vh] max-h-[860px] w-full max-w-4xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl">
                {/* Header */}
                <div className="flex items-center justify-between border-b border-slate-200/80 dark:border-[#15334d] px-6 py-4 bg-slate-50/80 dark:bg-[#071a27]">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 shadow-2xs">
                            <History className="h-5 w-5 text-amber-500" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2.5">
                                <h2 className="text-base font-bold text-slate-900 dark:text-white">
                                    Audit Trail: {instance.name}
                                </h2>
                                <span className="rounded-full bg-amber-100 px-2.5 py-0.5 text-[11px] font-semibold text-amber-800 dark:bg-amber-950/60 dark:text-amber-300 border border-amber-300/40 shadow-2xs">
                                    Total Switches: {totalSwitches}
                                </span>
                            </div>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5 flex items-center gap-1.5">
                                <span>Instance ID: <span className="font-mono text-slate-700 dark:text-slate-300">{instance.id}</span></span>
                                {instance.sequence_name && (
                                    <>
                                        <span>•</span>
                                        <span>Sequence: {instance.sequence_name}</span>
                                    </>
                                )}
                            </p>
                        </div>
                    </div>

                    {/* Header Action Capsule */}
                    <div className="flex items-center gap-2">
                        <div className="flex items-center rounded-full bg-slate-100 dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] p-0.5 shadow-2xs">
                            <button
                                type="button"
                                onClick={loadAuditHistory}
                                disabled={isLoading}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-full transition-colors cursor-pointer"
                                title="Refresh Switch History"
                            >
                                <RotateCw className={cn("w-3.5 h-3.5 text-amber-500", isLoading && "animate-spin")} />
                                <span>Refresh</span>
                            </button>
                        </div>

                        <button
                            type="button"
                            onClick={onClose}
                            className="p-1.5 rounded-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                            title="Close modal"
                        >
                            <X className="w-5 h-5" />
                        </button>
                    </div>
                </div>

                {/* Content Body */}
                <div className="flex-1 overflow-y-auto p-6 space-y-4 bg-slate-50/40 dark:bg-[#081a2b]">
                    {error && (
                        <div className="p-3.5 rounded-xl bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center gap-2.5">
                            <AlertCircle className="w-4 h-4 shrink-0 text-rose-500" />
                            <span className="flex-1">{error}</span>
                            <button
                                type="button"
                                onClick={loadAuditHistory}
                                className="px-2 py-0.5 rounded text-[11px] font-semibold bg-rose-200 dark:bg-rose-900/40 hover:bg-rose-300 transition-colors"
                            >
                                Retry
                            </button>
                        </div>
                    )}

                    {isLoading && !history ? (
                        <div className="flex flex-col items-center justify-center py-20 text-slate-400">
                            <RotateCw className="w-8 h-8 animate-spin text-amber-500 mb-3" />
                            <p className="text-xs font-medium text-slate-500 dark:text-slate-400">Loading audit trail records...</p>
                        </div>
                    ) : !history?.recent_switches || history.recent_switches.length === 0 ? (
                        /* Clean Empty State */
                        <div className="flex flex-col items-center justify-center py-16 px-4 text-center rounded-2xl border border-dashed border-slate-200 dark:border-[#15334d] bg-white/60 dark:bg-[#0c2438]/60">
                            <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-500/10 text-amber-500 border border-amber-500/20 mb-3">
                                <History className="w-6 h-6" />
                            </div>
                            <h3 className="text-sm font-bold text-slate-800 dark:text-slate-200 mb-1">
                                No Account Switches Recorded
                            </h3>
                            <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mb-4 leading-relaxed">
                                This instance has not undergone any automated or manual account switch cycles yet. When rotation or quota fallback occurs, the full 4-step backup and verification trail will be logged here.
                            </p>
                            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-100 dark:bg-[#071a27] text-slate-600 dark:text-slate-400 text-xs border border-slate-200 dark:border-[#15334d]">
                                <Info className="w-3.5 h-3.5 text-blue-500" />
                                <span>Monitors: Prompts Backup • Reset • Restore • Worker PIDs</span>
                            </div>
                        </div>
                    ) : (
                        /* Recent Switches Cards */
                        <div className="space-y-4">
                            <div className="flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 px-1">
                                <span className="font-semibold uppercase tracking-wider text-[11px]">
                                    Recent Switch Cycles ({history.recent_switches.length})
                                </span>
                                <span>Click card to expand 4-step execution details</span>
                            </div>

                            {history.recent_switches.map((record) => (
                                <SwitchRecordCard
                                    key={record.id}
                                    record={record as SwitchRecord}
                                    isExpanded={Boolean(expandedCards[record.id])}
                                    isUnmasked={Boolean(unmaskedEmails[record.id])}
                                    isRawOpen={Boolean(expandedRawFacts[record.id])}
                                    onToggleExpand={toggleCardExpand}
                                    onToggleEmailMask={toggleEmailMask}
                                    onToggleRawFacts={toggleRawFacts}
                                />
                            ))}
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="border-t border-slate-200/80 dark:border-[#15334d] px-6 py-3.5 bg-slate-50/80 dark:bg-[#071a27] flex items-center justify-between text-xs text-slate-500 dark:text-slate-400">
                    <span className="flex items-center gap-1.5">
                        <History className="w-3.5 h-3.5 text-amber-500" />
                        <span>All account rotation cycles, process restarts, and PID verifications are audited.</span>
                    </span>
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-1.5 rounded-full bg-slate-200 hover:bg-slate-300 dark:bg-[#15334d] dark:hover:bg-[#1d4366] text-slate-700 dark:text-slate-200 font-semibold transition-colors cursor-pointer"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
}
