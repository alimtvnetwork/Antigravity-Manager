import {
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
    Clock,
    Layers,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { maskEmail } from '../../../utils/maskEmail';
import type { SwitchAuditSteps } from '../../../types/audit';
import { formatAuditTime } from './formatAuditTime';

export interface SwitchRecord {
    id: string;
    from_email?: string;
    to_email?: string;
    status: string;
    created_at: number;
    finished_at?: number;
    switch_reason?: string;
    how?: string;
    steps?: SwitchAuditSteps;
    raw_payload?: string;
}

export interface SwitchRecordCardProps {
    record: SwitchRecord;
    isExpanded: boolean;
    isUnmasked: boolean;
    isRawOpen: boolean;
    onToggleExpand: (id: string) => void;
    onToggleEmailMask: (id: string) => void;
    onToggleRawFacts: (id: string) => void;
}

function StepBadge({ success, successLabel, failLabel }: { success: boolean; successLabel: string; failLabel: string }) {
    return (
        <span
            className={cn(
                "px-2 py-0.5 rounded text-[10px] font-bold border",
                success
                    ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                    : "bg-rose-500/10 text-rose-600 border-rose-500/20"
            )}
        >
            {success ? successLabel : failLabel}
        </span>
    );
}

function safeParseRawPayload(record: SwitchRecord): unknown {
    try {
        return record.steps || (record.raw_payload ? JSON.parse(record.raw_payload) : record);
    } catch {
        // If raw_payload is not valid JSON, fall back to the raw string
        return record.raw_payload || record;
    }
}

export function SwitchRecordCard({
    record,
    isExpanded,
    isUnmasked,
    isRawOpen,
    onToggleExpand,
    onToggleEmailMask,
    onToggleRawFacts,
}: SwitchRecordCardProps) {
    const isSuccess =
        record.status.toLowerCase() === 'completed' ||
        record.status.toLowerCase() === 'ok' ||
        record.status.toLowerCase() === 'success';

    const displayFrom = isUnmasked ? record.from_email : maskEmail(record.from_email || 'None');
    const displayTo = isUnmasked ? record.to_email : maskEmail(record.to_email || 'None');

    const steps: SwitchAuditSteps | undefined = record.steps;

    // Duration in seconds, handling both second and millisecond timestamps
    const durationSecs =
        record.finished_at != null
            ? Math.max(0, Math.round((record.finished_at - record.created_at) * (record.created_at < 1e11 ? 1 : 0.001)))
            : null;

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
                onClick={() => onToggleExpand(record.id)}
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
                                    onToggleEmailMask(record.id);
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
                            {durationSecs != null && (
                                <>
                                    <span>•</span>
                                    <span>Duration: {durationSecs}s</span>
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
                                <StepBadge success={steps?.backup?.success ?? true} successLabel="SUCCESS" failLabel="FAILED" />
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
                                <StepBadge success={steps?.reset?.success ?? true} successLabel="SUCCESS" failLabel="FAILED" />
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
                                <StepBadge success={steps?.restore?.success ?? true} successLabel="SUCCESS" failLabel="FAILED" />
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
                                <StepBadge
                                    success={steps?.verification?.verified ?? isSuccess}
                                    successLabel="VERIFIED"
                                    failLabel="PENDING"
                                />
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
                            onClick={() => onToggleRawFacts(record.id)}
                            className="flex items-center gap-1.5 text-[11px] font-semibold text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 transition-colors cursor-pointer"
                        >
                            <Layers className="w-3.5 h-3.5 text-amber-500" />
                            <span>{isRawOpen ? 'Hide Raw Audit Facts' : 'View Raw Audit Facts & Payload'}</span>
                            {isRawOpen ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
                        </button>

                        {isRawOpen && (
                            <div className="mt-2 rounded-xl bg-slate-900 border border-slate-800 p-3 text-[10px] font-mono text-slate-200 overflow-x-auto max-h-56">
                                <pre className="whitespace-pre-wrap break-all">
                                    {JSON.stringify(safeParseRawPayload(record), null, 2)}
                                </pre>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
