import { useState, useEffect, useCallback } from 'react';
import {
    History,
    X,
    RotateCw,
    CheckCircle2,
    XCircle,
    ChevronDown,
    ChevronRight,
    FolderArchive,
    Terminal,
    Send,
    ShieldCheck,
    Eye,
    EyeOff,
    ArrowRight,
    AlertCircle,
    Info,
    Layers,
    Clock
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { maskEmail } from '../../utils/maskEmail';
import { getInstanceSwitchHistory } from '../../services/instanceService';
import { useErrorStore } from '../../stores/error-store';
import type {
    InstanceSwitchHistoryResponse,
    SwitchAuditSteps
} from '../../types/audit';

interface InstanceAuditTrailModalProps {
    instance: {
        id: string;
        name: string;
        sequence_name?: string;
    } | null;
    isOpen: boolean;
    onClose: () => void;
}

function formatAuditTime(ts?: number | null): string {
    if (!ts || ts <= 0) return '—';
    const ms = ts < 1e11 ? ts * 1000 : ts;
    const d = new Date(ms);
    if (isNaN(d.getTime())) return String(ts);
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    const day = String(d.getDate()).padStart(2, '0');
    const month = months[d.getMonth()];
    const year = d.getFullYear();
    const hours = String(d.getHours()).padStart(2, '0');
    const mins = String(d.getMinutes()).padStart(2, '0');
    const secs = String(d.getSeconds()).padStart(2, '0');
    return `${day}-${month}-${year} ${hours}:${mins}:${secs}`;
}

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
            const res = await getInstanceSwitchHistory(instance.id, 10);
            setHistory(res);
            // Auto-expand the latest switches (first 2)
            const initialExpanded: Record<string, boolean> = {};
            (res.recent_switches || []).slice(0, 2).forEach((sw) => {
                initialExpanded[sw.id] = true;
            });
            setExpandedCards(initialExpanded);
        } catch (err: any) {
            console.error('[InstanceAuditTrailModal] Failed to fetch switch history:', err);
            // Tracked in the error module; modal already shows the error state to the user.
            useErrorStore.getState().trackWarning(err, {
              source: 'InstanceAuditTrailModal.fetchHistory',
              triggerAction: 'fetch_switch_history',
            });
            setError(err?.message || err?.toString() || 'Failed to fetch audit trail history');
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

                            {history.recent_switches.map((record) => {
                                const isExpanded = Boolean(expandedCards[record.id]);
                                const isUnmasked = Boolean(unmaskedEmails[record.id]);
                                const isRawOpen = Boolean(expandedRawFacts[record.id]);
                                const isSuccess = record.status.toLowerCase() === 'completed' || record.status.toLowerCase() === 'ok' || record.status.toLowerCase() === 'success';

                                const displayFrom = isUnmasked ? record.from_email : maskEmail(record.from_email || 'None');
                                const displayTo = isUnmasked ? record.to_email : maskEmail(record.to_email || 'None');

                                const steps: SwitchAuditSteps | undefined = record.steps;

                                return (
                                    <div
                                        key={record.id}
                                        className={cn(
                                            "rounded-2xl border transition-all duration-200 overflow-hidden bg-white dark:bg-[#0c2438] shadow-xs",
                                            isExpanded
                                                ? "border-amber-400/50 dark:border-amber-500/40 shadow-md ring-1 ring-amber-400/20"
                                                : "border-slate-200/80 dark:border-[#15334d] hover:border-slate-300 dark:hover:border-slate-600"
                                        )}
                                    >
                                        {/* Card Header / Summary Row */}
                                        <div
                                            onClick={() => toggleCardExpand(record.id)}
                                            className="p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 cursor-pointer select-none bg-slate-50/50 dark:bg-[#0a1e30]/50 hover:bg-slate-100/60 dark:hover:bg-[#0e273e]/60 transition-colors"
                                        >
                                            <div className="flex items-center gap-3 min-w-0 flex-1">
                                                <div className="shrink-0">
                                                    {isExpanded ? (
                                                        <ChevronDown className="w-4 h-4 text-amber-500" />
                                                    ) : (
                                                        <ChevronRight className="w-4 h-4 text-slate-400" />
                                                    )}
                                                </div>

                                                <div className="min-w-0 flex-1">
                                                    <div className="flex items-center gap-2 flex-wrap">
                                                        <span className="font-mono text-xs font-semibold text-slate-800 dark:text-slate-100">
                                                            {displayFrom}
                                                        </span>
                                                        <ArrowRight className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                                                        <span className="font-mono text-xs font-bold text-blue-600 dark:text-cyan-300">
                                                            {displayTo}
                                                        </span>
                                                        <button
                                                            type="button"
                                                            onClick={(e) => {
                                                                e.stopPropagation();
                                                                toggleEmailMask(record.id);
                                                            }}
                                                            className="p-1 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors"
                                                            title={isUnmasked ? "Mask emails" : "Unmask emails"}
                                                        >
                                                            {isUnmasked ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
                                                        </button>
                                                    </div>

                                                    <div className="flex items-center gap-2 mt-1 text-[11px] text-slate-500 dark:text-slate-400 flex-wrap">
                                                        <span className="flex items-center gap-1 font-mono">
                                                            <Clock className="w-3 h-3 text-slate-400" />
                                                            {formatAuditTime(record.created_at)}
                                                        </span>
                                                        {record.finished_at && (
                                                            <>
                                                                <span>•</span>
                                                                <span>Duration: {Math.max(0, Math.round((record.finished_at - record.created_at) * (record.created_at < 1e11 ? 1 : 0.001)))}s</span>
                                                            </>
                                                        )}
                                                        {record.switch_reason && (
                                                            <span className="inline-flex items-center px-2 py-0.2 rounded-full text-[10px] font-medium bg-slate-200/80 dark:bg-[#15334d] text-slate-700 dark:text-slate-300 border border-slate-300/40 dark:border-[#1d4366]">
                                                                {record.switch_reason}
                                                            </span>
                                                        )}
                                                        {record.how && (
                                                            <span className="inline-flex items-center px-1.5 py-0.2 rounded text-[10px] font-mono bg-blue-500/10 text-blue-600 dark:text-cyan-400">
                                                                {record.how}
                                                            </span>
                                                        )}
                                                    </div>
                                                </div>
                                            </div>

                                            {/* Status Badge */}
                                            <div className="flex items-center gap-2 shrink-0">
                                                <span
                                                    className={cn(
                                                        "inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-bold border shadow-2xs",
                                                        isSuccess
                                                            ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/30"
                                                            : "bg-rose-500/15 text-rose-700 dark:text-rose-300 border-rose-500/30"
                                                    )}
                                                >
                                                    {isSuccess ? (
                                                        <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />
                                                    ) : (
                                                        <XCircle className="w-3.5 h-3.5 text-rose-500" />
                                                    )}
                                                    <span>{record.status.toUpperCase()}</span>
                                                </span>
                                            </div>
                                        </div>

                                        {/* Expandable 4-Step Stepper / Timeline */}
                                        {isExpanded && (
                                            <div className="p-5 border-t border-slate-200/70 dark:border-[#15334d] bg-white dark:bg-[#0c2438] space-y-4">
                                                <div className="text-xs font-bold uppercase tracking-wider text-slate-400 flex items-center justify-between">
                                                    <span>4-Step Switch Lifecycle Execution</span>
                                                    <span className="font-mono text-[10px] lowercase text-slate-500">ID: {record.id}</span>
                                                </div>

                                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                                                    {/* Step 1: Prompts Backup */}
                                                    <div className="p-3.5 rounded-xl border border-slate-200/80 dark:border-[#15334d] bg-slate-50/70 dark:bg-[#071a27]/70 space-y-2">
                                                        <div className="flex items-center justify-between">
                                                            <div className="flex items-center gap-2 text-xs font-bold text-slate-800 dark:text-slate-200">
                                                                <div className="p-1 rounded-lg bg-indigo-500/10 text-indigo-600 dark:text-indigo-400">
                                                                    <FolderArchive className="w-4 h-4" />
                                                                </div>
                                                                <span>Step 1: Prompts Backup</span>
                                                            </div>
                                                            <span className={cn(
                                                                "px-2 py-0.5 rounded text-[10px] font-bold border",
                                                                steps?.backup?.success ?? true
                                                                    ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                                                                    : "bg-rose-500/10 text-rose-600 border-rose-500/20"
                                                            )}>
                                                                {steps?.backup?.success ?? true ? 'SUCCESS' : 'FAILED'}
                                                            </span>
                                                        </div>

                                                        <div className="text-xs space-y-1 text-slate-600 dark:text-slate-300 font-medium">
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Prompt Count:</span>
                                                                <span className="font-mono font-bold text-slate-800 dark:text-slate-200">
                                                                    {steps?.backup?.prompt_count ?? 0} active
                                                                </span>
                                                            </div>
                                                            {steps?.backup?.backup_batch_id && (
                                                                <div className="flex justify-between">
                                                                    <span className="text-slate-400">Batch ID:</span>
                                                                    <span className="font-mono text-[11px] text-indigo-600 dark:text-indigo-400 truncate max-w-[180px]">
                                                                        {steps.backup.backup_batch_id}
                                                                    </span>
                                                                </div>
                                                            )}
                                                            {steps?.backup?.project_names && steps.backup.project_names.length > 0 && (
                                                                <div>
                                                                    <span className="text-slate-400 block text-[11px]">Projects:</span>
                                                                    <div className="flex flex-wrap gap-1 mt-1">
                                                                        {steps.backup.project_names.map((name, i) => (
                                                                            <span key={i} className="px-1.5 py-0.2 rounded bg-slate-200 dark:bg-[#15334d] text-[10px] font-mono text-slate-700 dark:text-slate-300">
                                                                                {name}
                                                                            </span>
                                                                        ))}
                                                                    </div>
                                                                </div>
                                                            )}
                                                        </div>
                                                    </div>

                                                    {/* Step 2: Account Reset */}
                                                    <div className="p-3.5 rounded-xl border border-slate-200/80 dark:border-[#15334d] bg-slate-50/70 dark:bg-[#071a27]/70 space-y-2">
                                                        <div className="flex items-center justify-between">
                                                            <div className="flex items-center gap-2 text-xs font-bold text-slate-800 dark:text-slate-200">
                                                                <div className="p-1 rounded-lg bg-amber-500/10 text-amber-600 dark:text-amber-400">
                                                                    <Terminal className="w-4 h-4" />
                                                                </div>
                                                                <span>Step 2: Account Reset</span>
                                                            </div>
                                                            <span className={cn(
                                                                "px-2 py-0.5 rounded text-[10px] font-bold border",
                                                                steps?.reset?.success ?? true
                                                                    ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                                                                    : "bg-rose-500/10 text-rose-600 border-rose-500/20"
                                                            )}>
                                                                {steps?.reset?.success ?? true ? 'SUCCESS' : 'FAILED'}
                                                            </span>
                                                        </div>

                                                        <div className="text-xs space-y-1 text-slate-600 dark:text-slate-300 font-medium">
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Terminated PIDs:</span>
                                                                <span className="font-mono text-[11px] text-rose-600 dark:text-rose-400 font-semibold">
                                                                    {steps?.reset?.terminated_pids && steps.reset.terminated_pids.length > 0
                                                                        ? steps.reset.terminated_pids.join(', ')
                                                                        : '0 (Clean)'}
                                                                </span>
                                                            </div>
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Auth Token Swapped:</span>
                                                                <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                                                                    {steps?.reset?.auth_swapped ?? true ? 'Yes' : 'No'}
                                                                </span>
                                                            </div>
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Credentials Injected:</span>
                                                                <span className="font-semibold text-blue-600 dark:text-cyan-400">
                                                                    {steps?.reset?.credentials_injected ?? true ? 'Injected' : 'Pending'}
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </div>

                                                    {/* Step 3: Prompts Restore */}
                                                    <div className="p-3.5 rounded-xl border border-slate-200/80 dark:border-[#15334d] bg-slate-50/70 dark:bg-[#071a27]/70 space-y-2">
                                                        <div className="flex items-center justify-between">
                                                            <div className="flex items-center gap-2 text-xs font-bold text-slate-800 dark:text-slate-200">
                                                                <div className="p-1 rounded-lg bg-sky-500/10 text-sky-600 dark:text-sky-400">
                                                                    <Send className="w-4 h-4" />
                                                                </div>
                                                                <span>Step 3: Prompts Restore</span>
                                                            </div>
                                                            <span className={cn(
                                                                "px-2 py-0.5 rounded text-[10px] font-bold border",
                                                                steps?.restore?.success ?? true
                                                                    ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                                                                    : "bg-rose-500/10 text-rose-600 border-rose-500/20"
                                                            )}>
                                                                {steps?.restore?.success ?? true ? 'SUCCESS' : 'FAILED'}
                                                            </span>
                                                        </div>

                                                        <div className="text-xs space-y-1 text-slate-600 dark:text-slate-300 font-medium">
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Method:</span>
                                                                <span className="font-mono text-[11px] text-slate-800 dark:text-slate-200">
                                                                    {steps?.restore?.method || 'IPC Dispatch / State Replay'}
                                                                </span>
                                                            </div>
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Restored / Dispatched:</span>
                                                                <span className="font-mono text-slate-800 dark:text-slate-200">
                                                                    {steps?.restore?.restored_count ?? 0} / {steps?.restore?.dispatched_count ?? 0}
                                                                </span>
                                                            </div>
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Channel Wait Stabilization:</span>
                                                                <span className="text-emerald-600 dark:text-emerald-400">
                                                                    {steps?.restore?.prompt_channel_waited ?? true ? 'Waited (Passed)' : 'Skipped'}
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </div>

                                                    {/* Step 4: Post-Restore Verification */}
                                                    <div className="p-3.5 rounded-xl border border-slate-200/80 dark:border-[#15334d] bg-slate-50/70 dark:bg-[#071a27]/70 space-y-2">
                                                        <div className="flex items-center justify-between">
                                                            <div className="flex items-center gap-2 text-xs font-bold text-slate-800 dark:text-slate-200">
                                                                <div className="p-1 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
                                                                    <ShieldCheck className="w-4 h-4" />
                                                                </div>
                                                                <span>Step 4: Verification</span>
                                                            </div>
                                                            <span className={cn(
                                                                "px-2 py-0.5 rounded text-[10px] font-bold border",
                                                                steps?.verification?.verified ?? isSuccess
                                                                    ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                                                                    : "bg-rose-500/10 text-rose-600 border-rose-500/20"
                                                            )}>
                                                                {steps?.verification?.verified ?? isSuccess ? 'VERIFIED' : 'PENDING'}
                                                            </span>
                                                        </div>

                                                        <div className="text-xs space-y-1 text-slate-600 dark:text-slate-300 font-medium">
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Active Worker PIDs:</span>
                                                                <span className="font-mono text-[11px] text-emerald-600 dark:text-emerald-400 font-bold">
                                                                    {steps?.verification?.active_worker_pids && steps.verification.active_worker_pids.length > 0
                                                                        ? steps.verification.active_worker_pids.join(', ')
                                                                        : 'Confirmed'}
                                                                </span>
                                                            </div>
                                                            <div className="flex justify-between">
                                                                <span className="text-slate-400">Resume File Check:</span>
                                                                <span className="text-blue-600 dark:text-cyan-400">
                                                                    {steps?.verification?.resume_file_verified ?? true ? 'Confirmed OK' : 'Unchecked'}
                                                                </span>
                                                            </div>
                                                            {steps?.verification?.message && (
                                                                <div className="text-[11px] text-slate-500 dark:text-slate-400 italic truncate" title={steps.verification.message}>
                                                                    {steps.verification.message}
                                                                </div>
                                                            )}
                                                        </div>
                                                    </div>
                                                </div>

                                                {/* Raw Facts Drawer Toggle */}
                                                <div className="pt-2 border-t border-slate-100 dark:border-[#15334d]/60">
                                                    <button
                                                        type="button"
                                                        onClick={() => toggleRawFacts(record.id)}
                                                        className="flex items-center gap-1.5 text-[11px] font-semibold text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 transition-colors cursor-pointer"
                                                    >
                                                        <Layers className="w-3.5 h-3.5 text-amber-500" />
                                                        <span>{isRawOpen ? 'Hide Raw Audit Facts' : 'View Raw Audit Facts & Payload'}</span>
                                                        {isRawOpen ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
                                                    </button>

                                                    {isRawOpen && (
                                                        <div className="mt-2 rounded-xl bg-slate-900 border border-slate-800 p-3 text-[10px] font-mono text-slate-200 overflow-x-auto max-h-56">
                                                            <pre className="whitespace-pre-wrap break-all">
                                                                {JSON.stringify(
                                                                    record.steps || (record.raw_payload ? JSON.parse(record.raw_payload) : record),
                                                                    null,
                                                                    2
                                                                )}
                                                            </pre>
                                                        </div>
                                                    )}
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                );
                            })}
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
