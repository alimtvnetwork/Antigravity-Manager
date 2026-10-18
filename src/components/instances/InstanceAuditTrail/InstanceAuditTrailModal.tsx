import { useState, useEffect, useCallback } from 'react';
import {
    History,
    Activity,
    X,
    RotateCw,
    AlertCircle,
    Info,
    CheckCircle2,
    XCircle,
    Clock,
    ChevronDown,
    ChevronUp,
    Copy,
    Check,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { getInstanceSwitchHistory, getInstanceAuditTrail } from '../../../services/instanceService';
import { useErrorStore } from '../../../stores/error-store';
import type { InstanceSwitchHistoryResponse, TaskRecord } from '../../../types/audit';
import type { InstanceAuditTrailModalProps } from './types';
import { SwitchRecordCard, type SwitchRecord } from './SwitchRecordCard';
import { formatAuditTime } from './formatAuditTime';

const AUTO_EXPAND_COUNT = 2;
const HISTORY_LIMIT = 10;
const TRAIL_LIMIT = 50;

function getActionBadgeStyle(code: number, actionName: string): { bg: string; text: string; border: string } {
    const act = actionName.toLowerCase();
    if (code === 3 || act.includes('switch')) {
        return {
            bg: 'bg-amber-100 dark:bg-amber-950/60',
            text: 'text-amber-800 dark:text-amber-300',
            border: 'border-amber-300/50 dark:border-amber-700/50',
        };
    }
    if (code === 4 || act.includes('schedule')) {
        return {
            bg: 'bg-purple-100 dark:bg-purple-950/60',
            text: 'text-purple-800 dark:text-purple-300',
            border: 'border-purple-300/50 dark:border-purple-700/50',
        };
    }
    if (code === 5 || act.includes('requeue')) {
        return {
            bg: 'bg-cyan-100 dark:bg-cyan-950/60',
            text: 'text-cyan-800 dark:text-cyan-300',
            border: 'border-cyan-300/50 dark:border-cyan-700/50',
        };
    }
    if (code === 1 || act.includes('add')) {
        return {
            bg: 'bg-emerald-100 dark:bg-emerald-950/60',
            text: 'text-emerald-800 dark:text-emerald-300',
            border: 'border-emerald-300/50 dark:border-emerald-700/50',
        };
    }
    if (code === 2 || act.includes('update')) {
        return {
            bg: 'bg-blue-100 dark:bg-blue-950/60',
            text: 'text-blue-800 dark:text-blue-300',
            border: 'border-blue-300/50 dark:border-blue-700/50',
        };
    }
    // Action codes 6–13
    if (code === 6 || act.includes('launch') || act.includes('start')) {
        return {
            bg: 'bg-emerald-100 dark:bg-emerald-950/60',
            text: 'text-emerald-800 dark:text-emerald-300',
            border: 'border-emerald-300/50 dark:border-emerald-700/50',
        };
    }
    if (code === 7 || act.includes('stop') || act.includes('close')) {
        return {
            bg: 'bg-amber-100 dark:bg-amber-950/60',
            text: 'text-amber-800 dark:text-amber-300',
            border: 'border-amber-300/50 dark:border-amber-700/50',
        };
    }
    if (code === 8 || act.includes('restart')) {
        return {
            bg: 'bg-indigo-100 dark:bg-indigo-950/60',
            text: 'text-indigo-800 dark:text-indigo-300',
            border: 'border-indigo-300/50 dark:border-indigo-700/50',
        };
    }
    if (code === 9 || act.includes('kill')) {
        return {
            bg: 'bg-rose-100 dark:bg-rose-950/60',
            text: 'text-rose-800 dark:text-rose-300',
            border: 'border-rose-300/50 dark:border-rose-700/50',
        };
    }
    if (code === 10 || act.includes('create')) {
        return {
            bg: 'bg-teal-100 dark:bg-teal-950/60',
            text: 'text-teal-800 dark:text-teal-300',
            border: 'border-teal-300/50 dark:border-teal-700/50',
        };
    }
    if (code === 11 || act.includes('delete') || act.includes('remove')) {
        return {
            bg: 'bg-red-100 dark:bg-red-950/60',
            text: 'text-red-800 dark:text-red-300',
            border: 'border-red-300/50 dark:border-red-700/50',
        };
    }
    if (code === 12 || act.includes('clone') || act.includes('copy')) {
        return {
            bg: 'bg-sky-100 dark:bg-sky-950/60',
            text: 'text-sky-800 dark:text-sky-300',
            border: 'border-sky-300/50 dark:border-sky-700/50',
        };
    }
    if (code === 13 || act.includes('error') || act.includes('fail')) {
        return {
            bg: 'bg-rose-100 dark:bg-rose-950/60',
            text: 'text-rose-800 dark:text-rose-300',
            border: 'border-rose-300/50 dark:border-rose-700/50',
        };
    }
    if (act.includes('backup') || act.includes('restore') || act.includes('prompt')) {
        return {
            bg: 'bg-indigo-100 dark:bg-indigo-950/60',
            text: 'text-indigo-800 dark:text-indigo-300',
            border: 'border-indigo-300/50 dark:border-indigo-700/50',
        };
    }
    return {
        bg: 'bg-slate-100 dark:bg-slate-800/60',
        text: 'text-slate-700 dark:text-slate-300',
        border: 'border-slate-300/50 dark:border-slate-700/50',
    };
}

export default function InstanceAuditTrailModal({
    instance,
    isOpen,
    onClose,
}: InstanceAuditTrailModalProps) {
    const [history, setHistory] = useState<InstanceSwitchHistoryResponse | null>(null);
    const [auditTrail, setAuditTrail] = useState<TaskRecord[] | null>(null);
    const [activeTab, setActiveTab] = useState<'switches' | 'operations'>('switches');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [unmaskedEmails, setUnmaskedEmails] = useState<Record<string, boolean>>({});
    const [expandedCards, setExpandedCards] = useState<Record<string, boolean>>({});
    const [expandedRawFacts, setExpandedRawFacts] = useState<Record<string, boolean>>({});
    const [expandedOps, setExpandedOps] = useState<Record<string, boolean>>({});
    const [copiedId, setCopiedId] = useState<string | null>(null);

    const loadAuditHistory = useCallback(async () => {
        if (!instance?.id) return;
        setIsLoading(true);
        setError(null);
        try {
            const [switchRes, trailRes] = await Promise.all([
                getInstanceSwitchHistory(instance.id, HISTORY_LIMIT),
                getInstanceAuditTrail(instance.id, TRAIL_LIMIT),
            ]);
            setHistory(switchRes);
            setAuditTrail(trailRes);

            // Auto-expand the latest switches (first 2)
            const initialExpanded: Record<string, boolean> = {};
            const loaded = switchRes.recent_switches?.length
                ? switchRes.recent_switches
                : ((switchRes as { records?: typeof switchRes.recent_switches }).records
                    || (switchRes as { items?: typeof switchRes.recent_switches }).items
                    || []);
            loaded.slice(0, AUTO_EXPAND_COUNT).forEach((sw) => {
                initialExpanded[sw.id] = true;
            });
            setExpandedCards(initialExpanded);

            // If recentSwitches.length === 0, automatically default to the 'operations' tab so users see real events
            if (loaded.length === 0) {
                setActiveTab('operations');
            } else {
                setActiveTab('switches');
            }
        } catch (err: unknown) {
            console.error('[InstanceAuditTrailModal] Failed to fetch audit history:', err);
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceAuditTrailModal.fetchHistory',
                triggerAction: 'fetch_audit_history',
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
            setAuditTrail(null);
            setError(null);
            setActiveTab('switches');
        }
    }, [isOpen, instance?.id, loadAuditHistory]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) {
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

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

    const toggleOpExpand = (opId: string) => {
        setExpandedOps((prev) => ({ ...prev, [opId]: !prev[opId] }));
    };

    const handleCopy = (text: string, id: string) => {
        navigator.clipboard.writeText(text);
        setCopiedId(id);
        setTimeout(() => setCopiedId(null), 1500);
    };

    const historyBag = history as (InstanceSwitchHistoryResponse & {
        records?: InstanceSwitchHistoryResponse['recent_switches'];
        items?: InstanceSwitchHistoryResponse['recent_switches'];
        total?: number;
    }) | null;
    const recentSwitches = historyBag?.recent_switches?.length
        ? historyBag.recent_switches
        : historyBag?.records || historyBag?.items || [];
    const totalSwitches = historyBag?.total_switches || historyBag?.total || recentSwitches.length;
    const totalOperations = auditTrail?.length || 0;

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="audit-trail-dialog-title"
            className="fixed inset-0 z-[99999] flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 sm:p-6 overflow-y-auto animate-in fade-in duration-200"
            onClick={(e) => {
                if (e.target === e.currentTarget) {
                    onClose();
                }
            }}
        >
            <div
                className="flex h-[90vh] max-h-[860px] w-full max-w-4xl flex-col my-auto overflow-hidden rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-200/80 dark:border-[#40285e] px-6 py-4 bg-slate-50/80 dark:bg-[#090611]">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 shadow-2xs">
                            <History className="h-5 w-5 text-amber-500" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2.5">
                                <h2 id="audit-trail-dialog-title" className="text-base font-bold text-slate-900 dark:text-white">
                                    Audit Trail: {instance.name}
                                </h2>
                                <span className="rounded-full bg-amber-100 px-2.5 py-0.5 text-[11px] font-semibold text-amber-800 dark:bg-amber-950/60 dark:text-amber-300 border border-amber-300/40 shadow-2xs">
                                    Switches: {totalSwitches}
                                </span>
                                {totalOperations > 0 && (
                                    <span className="rounded-full bg-slate-100 px-2.5 py-0.5 text-[11px] font-semibold text-slate-700 dark:bg-[#40285e]/60 dark:text-slate-300 border border-slate-300/40 shadow-2xs">
                                        Operations: {totalOperations}
                                    </span>
                                )}
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

                    {/* Header Controls: Segmented View Switcher & Actions */}
                    <div className="flex items-center gap-2.5">
                        {/* Segmented Capsule Toggle: Switch Cycles vs All Operations */}
                        <div className="inline-flex items-center rounded-full bg-slate-200/70 dark:bg-[#1a1228] border border-slate-300/80 dark:border-[#40285e] p-0.5 shadow-2xs">
                            <button
                                type="button"
                                onClick={() => setActiveTab('switches')}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-full transition-all cursor-pointer",
                                    activeTab === 'switches'
                                        ? "bg-amber-500 text-white shadow-xs"
                                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                                )}
                            >
                                <History className="w-3.5 h-3.5" />
                                <span>Switch Cycles</span>
                                {recentSwitches.length > 0 && (
                                    <span className={cn(
                                        "text-[10px] px-1.5 py-0.2 rounded-full font-mono",
                                        activeTab === 'switches'
                                            ? "bg-amber-600 text-white"
                                            : "bg-slate-300/80 dark:bg-[#40285e] text-slate-700 dark:text-slate-300"
                                    )}>
                                        {totalSwitches}
                                    </span>
                                )}
                            </button>
                            <button
                                type="button"
                                onClick={() => setActiveTab('operations')}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-full transition-all cursor-pointer",
                                    activeTab === 'operations'
                                        ? "bg-amber-500 text-white shadow-xs"
                                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                                )}
                            >
                                <Activity className="w-3.5 h-3.5" />
                                <span>All Operations</span>
                                {totalOperations > 0 && (
                                    <span className={cn(
                                        "text-[10px] px-1.5 py-0.2 rounded-full font-mono",
                                        activeTab === 'operations'
                                            ? "bg-amber-600 text-white"
                                            : "bg-slate-300/80 dark:bg-[#40285e] text-slate-700 dark:text-slate-300"
                                    )}>
                                        {totalOperations}
                                    </span>
                                )}
                            </button>
                        </div>

                        {/* Action Buttons: Contiguous Segmented Capsule */}
                        <div className="inline-flex items-center rounded-full bg-slate-100 dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] p-0.5 shadow-2xs">
                            <button
                                type="button"
                                onClick={loadAuditHistory}
                                disabled={isLoading}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded-full transition-colors cursor-pointer"
                                title="Refresh Audit Trail"
                            >
                                <RotateCw className={cn("w-3.5 h-3.5 text-amber-500", isLoading && "animate-spin")} />
                                <span>Refresh</span>
                            </button>
                            <span className="w-px h-4 bg-slate-300 dark:bg-[#40285e] my-auto mx-0.5" />
                            <button
                                type="button"
                                onClick={onClose}
                                className="p-1.5 rounded-full text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                                title="Close modal (Esc)"
                                aria-label="Close modal"
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>
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

                    {isLoading && !history && !auditTrail ? (
                        <div className="flex flex-col items-center justify-center py-20 text-slate-400">
                            <RotateCw className="w-8 h-8 animate-spin text-amber-500 mb-3" />
                            <p className="text-xs font-medium text-slate-500 dark:text-slate-400">Loading audit trail records...</p>
                        </div>
                    ) : activeTab === 'switches' ? (
                        /* TAB 1: Switch Cycles */
                        recentSwitches.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-16 px-4 text-center rounded-2xl border border-dashed border-slate-200 dark:border-[#40285e] bg-white/60 dark:bg-[#171025]/60">
                                <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-500/10 text-amber-500 border border-amber-500/20 mb-3">
                                    <History className="w-6 h-6" />
                                </div>
                                <h3 className="text-sm font-bold text-slate-800 dark:text-slate-200 mb-1">
                                    No Account Switches Recorded
                                </h3>
                                <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mb-3 leading-relaxed">
                                    This instance has not undergone any automated or manual account switch cycles yet. When rotation or quota fallback occurs, the full 4-step backup and verification trail will be logged here.
                                </p>
                                {totalOperations > 0 && (
                                    <button
                                        type="button"
                                        onClick={() => setActiveTab('operations')}
                                        className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold bg-amber-500 text-white hover:bg-amber-600 transition-colors shadow-2xs cursor-pointer mb-3"
                                    >
                                        <Activity className="w-3.5 h-3.5" />
                                        <span>View All Operations ({totalOperations} recorded)</span>
                                    </button>
                                )}
                                <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-100 dark:bg-[#090611] text-slate-600 dark:text-slate-400 text-xs border border-slate-200 dark:border-[#40285e]">
                                    <Info className="w-3.5 h-3.5 text-blue-500" />
                                    <span>Monitors: Prompts Backup • Reset • Restore • Worker PIDs</span>
                                </div>
                            </div>
                        ) : (
                            <div className="space-y-4">
                                <div className="flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 px-1">
                                    <span className="font-semibold uppercase tracking-wider text-[11px]">
                                        Recent Switch Cycles ({recentSwitches.length})
                                    </span>
                                    <span>Click card to expand 4-step execution details</span>
                                </div>

                                {recentSwitches.map((record) => (
                                    <SwitchRecordCard
                                        key={record.id}
                                        record={{
                                            ...(record as SwitchRecord),
                                            raw_payload:
                                                (record as SwitchRecord).raw_payload
                                                || (record as { payload_json?: string }).payload_json,
                                            how:
                                                (record as SwitchRecord).how
                                                || (record as { switch_reason?: string }).switch_reason,
                                        }}
                                        isExpanded={Boolean(expandedCards[record.id])}
                                        isUnmasked={Boolean(unmaskedEmails[record.id])}
                                        isRawOpen={Boolean(expandedRawFacts[record.id])}
                                        onToggleExpand={toggleCardExpand}
                                        onToggleEmailMask={toggleEmailMask}
                                        onToggleRawFacts={toggleRawFacts}
                                    />
                                ))}
                            </div>
                        )
                    ) : (
                        /* TAB 2: All Operations */
                        !auditTrail || auditTrail.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-16 px-4 text-center rounded-2xl border border-dashed border-slate-200 dark:border-[#40285e] bg-white/60 dark:bg-[#171025]/60">
                                <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-500/10 text-amber-500 border border-amber-500/20 mb-3">
                                    <Activity className="w-6 h-6" />
                                </div>
                                <h3 className="text-sm font-bold text-slate-800 dark:text-slate-200 mb-1">
                                    No Operations Logged
                                </h3>
                                <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mb-4 leading-relaxed">
                                    No operations, tasks, prompt backups, or scheduler actions have been logged for this instance yet.
                                </p>
                            </div>
                        ) : (
                            <div className="space-y-3">
                                <div className="flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 px-1">
                                    <span className="font-semibold uppercase tracking-wider text-[11px]">
                                        Instance Operations & Events ({auditTrail.length})
                                    </span>
                                    <span>Click row to view details & execution payload</span>
                                </div>

                                {auditTrail.map((record) => {
                                    const isExpanded = Boolean(expandedOps[record.id]);
                                    const badgeStyle = getActionBadgeStyle(record.action_code, record.action_label || record.action);
                                    const isSuccess = record.status.toLowerCase() === 'ok' || record.status.toLowerCase() === 'success';
                                    const isFailed = record.status.toLowerCase() === 'fail' || record.status.toLowerCase() === 'error';
                                    const durationSec = record.finished_at && record.finished_at >= record.created_at
                                        ? record.finished_at - record.created_at
                                        : null;

                                    return (
                                        <div
                                            key={record.id}
                                            className="rounded-xl border border-slate-200/90 dark:border-[#40285e]/80 bg-white dark:bg-[#171025] hover:border-slate-300 dark:hover:border-[#5a3884] transition-all overflow-hidden shadow-2xs"
                                        >
                                            {/* Summary Header */}
                                            <div
                                                onClick={() => toggleOpExpand(record.id)}
                                                className="p-3.5 flex items-center justify-between gap-3 cursor-pointer select-none bg-slate-50/50 hover:bg-slate-100/50 dark:bg-[#171025] dark:hover:bg-[#1f1533] transition-colors"
                                            >
                                                <div className="flex items-center gap-3 min-w-0 flex-1">
                                                    {/* Status Icon */}
                                                    <div className="shrink-0">
                                                        {isSuccess ? (
                                                            <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                                                        ) : isFailed ? (
                                                            <XCircle className="w-4 h-4 text-rose-500" />
                                                        ) : (
                                                            <Clock className="w-4 h-4 text-amber-500" />
                                                        )}
                                                    </div>

                                                    {/* Action Badge */}
                                                    <span className={cn(
                                                        "px-2 py-0.5 rounded-md text-[11px] font-semibold border shrink-0",
                                                        badgeStyle.bg,
                                                        badgeStyle.text,
                                                        badgeStyle.border
                                                    )}>
                                                        {record.action_label || record.action}
                                                    </span>

                                                    {/* Subject / Detail preview */}
                                                    <div className="min-w-0 flex-1 flex items-center gap-2 text-xs truncate">
                                                        <span className="font-semibold text-slate-800 dark:text-slate-200 truncate">
                                                            {record.subject || '—'}
                                                        </span>
                                                        {record.detail && (
                                                            <>
                                                                <span className="text-slate-400">•</span>
                                                                <span className="text-slate-500 dark:text-slate-400 font-mono text-[11px] truncate">
                                                                    {record.detail}
                                                                </span>
                                                            </>
                                                        )}
                                                    </div>
                                                </div>

                                                {/* Right: Timestamp & Expand Indicator */}
                                                <div className="flex items-center gap-3 shrink-0">
                                                    <span className="text-[11px] font-mono text-slate-400">
                                                        {formatAuditTime(record.created_at)}
                                                    </span>
                                                    <button
                                                        type="button"
                                                        className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 p-0.5"
                                                    >
                                                        {isExpanded ? (
                                                            <ChevronUp className="w-4 h-4" />
                                                        ) : (
                                                            <ChevronDown className="w-4 h-4" />
                                                        )}
                                                    </button>
                                                </div>
                                            </div>

                                            {/* Expandable Details */}
                                            {isExpanded && (
                                                <div className="p-4 border-t border-slate-200/80 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#0c0817] space-y-3">
                                                    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3 text-xs">
                                                        <div>
                                                            <span className="block text-[10px] uppercase font-bold text-slate-400 mb-0.5">Task ID</span>
                                                            <div className="flex items-center gap-1.5 font-mono text-[11px] text-slate-700 dark:text-slate-300">
                                                                <span className="truncate">{record.id}</span>
                                                                <button
                                                                    type="button"
                                                                    onClick={(e) => {
                                                                        e.stopPropagation();
                                                                        handleCopy(record.id, record.id);
                                                                    }}
                                                                    className="p-1 hover:bg-slate-200 dark:hover:bg-[#40285e] rounded transition-colors text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                                                                    title="Copy Task ID"
                                                                >
                                                                    {copiedId === record.id ? (
                                                                        <Check className="w-3 h-3 text-emerald-500" />
                                                                    ) : (
                                                                        <Copy className="w-3 h-3" />
                                                                    )}
                                                                </button>
                                                            </div>
                                                        </div>

                                                        <div>
                                                            <span className="block text-[10px] uppercase font-bold text-slate-400 mb-0.5">Status</span>
                                                            <span className={cn(
                                                                "inline-flex items-center gap-1 font-semibold uppercase text-[11px]",
                                                                isSuccess ? "text-emerald-600 dark:text-emerald-400" : isFailed ? "text-rose-600 dark:text-rose-400" : "text-amber-600 dark:text-amber-400"
                                                            )}>
                                                                {record.status}
                                                                {durationSec !== null && ` (${durationSec}s)`}
                                                            </span>
                                                        </div>

                                                        <div>
                                                            <span className="block text-[10px] uppercase font-bold text-slate-400 mb-0.5">Instance ID</span>
                                                            <span className="font-mono text-slate-700 dark:text-slate-300">
                                                                {record.instance_id || 'default'}
                                                            </span>
                                                        </div>

                                                        <div>
                                                            <span className="block text-[10px] uppercase font-bold text-slate-400 mb-0.5">Storage Split</span>
                                                            <span className="font-mono text-[11px] text-slate-500 dark:text-slate-400 truncate block" title={record.split_path}>
                                                                {record.split_path ? record.split_path.split(/[/\\]/).pop() : 'primary_db'}
                                                            </span>
                                                        </div>
                                                    </div>

                                                    {(record.from_email || record.to_email) && (
                                                        <div className="text-xs pt-1 border-t border-slate-200 dark:border-[#40285e]/60 flex items-center gap-2">
                                                            <span className="text-slate-400 font-medium">Account Rotation:</span>
                                                            <span className="font-mono text-slate-600 dark:text-slate-300">
                                                                {record.from_email || '—'} → {record.to_email || '—'}
                                                            </span>
                                                        </div>
                                                    )}

                                                    {record.detail && (
                                                        <div>
                                                            <span className="block text-[10px] uppercase font-bold text-slate-400 mb-1">Execution Detail</span>
                                                            <div className="bg-white dark:bg-[#090611] p-3 rounded-lg border border-slate-200 dark:border-[#40285e] font-mono text-xs text-slate-700 dark:text-slate-300 whitespace-pre-wrap break-all leading-relaxed">
                                                                {record.detail}
                                                            </div>
                                                        </div>
                                                    )}
                                                </div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        )
                    )}
                </div>

                {/* Footer */}
                <div className="border-t border-slate-200/80 dark:border-[#40285e] px-6 py-3.5 bg-slate-50/80 dark:bg-[#090611] flex items-center justify-between text-xs text-slate-500 dark:text-slate-400">
                    <span className="flex items-center gap-1.5">
                        <History className="w-3.5 h-3.5 text-amber-500" />
                        <span>All account rotation cycles, process restarts, and PID verifications are audited.</span>
                    </span>
                    <div className="inline-flex rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#120d1d] p-0.5 backdrop-blur-xs shadow-2xs">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-1.5 rounded-full text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-white dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                        >
                            Close
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
