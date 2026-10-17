/**
 * AuditPanel — modal listing recorded UI interactions.
 *
 * Filterable by actor and result (actor badges follow the house 4-tier
 * origin visual language: sky/user, zinc/cli, purple/telegram-bot,
 * amber/system-wrench). Exports the visible entries as JSON or JSONL
 * for bug reports.
 */
import { useMemo, useState } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import {
    AlertTriangle,
    Bot,
    Check,
    ChevronDown,
    Copy,
    FileJson,
    FileText,
    RefreshCw,
    Terminal,
    Trash2,
    User,
    Wrench,
    X,
} from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';
import { copyToClipboard } from '../../utils/clipboard';
import { cn } from '../../utils/cn';
import { showToast } from '../common/ToastContainer';
import { useAuditStore, type AuditActorFilter, type AuditResultFilter } from './audit-store';
import type { AuditActor, AuditEntry, AuditStackFrame } from './audit-types';

const SOURCE = 'audit/AuditPanel';

function parseParams(params: AuditEntry['params']): Record<string, unknown> {
    if (typeof params === 'object' && params !== null) return params as Record<string, unknown>;
    try {
        const parsed: unknown = JSON.parse(params || '{}');
        return typeof parsed === 'object' && parsed !== null
            ? (parsed as Record<string, unknown>)
            : { value: parsed };
    } catch {
        return { raw: params };
    }
}

type CallerFrame = AuditStackFrame;

function frameFromUnknown(value: unknown): CallerFrame | null {
    if (!value || typeof value !== 'object') return null;
    const record = value as Record<string, unknown>;
    const fnName = String(record.function || record.functionName || '').trim();
    const location = String(record.location || record.file || '').trim();
    if (!fnName && !location) return null;
    return { function: fnName || 'unknown', location: location || 'unknown' };
}

/** Parse the dedicated `stackTrace` column (raw JSON string or decoded array). */
function framesFromDedicatedTrace(value: unknown): CallerFrame[] {
    let decoded: unknown = value;
    if (typeof value === 'string') {
        try {
            decoded = JSON.parse(value);
        } catch {
            // Malformed trace: the panel renders the "unavailable" fallback below.
            return [];
        }
    }
    if (!Array.isArray(decoded)) return [];
    return decoded
        .map(frameFromUnknown)
        .filter((frame): frame is CallerFrame => frame !== null);
}

function framesFromStackText(stackText: string): CallerFrame[] {
    const frames: CallerFrame[] = [];
    for (const line of stackText.split('\n')) {
        const trimmed = line.trim();
        const matched = trimmed.match(/^at\s+(.+?)\s+\((.+)\)$/) || trimmed.match(/^at\s+(.+)$/);
        if (!matched) continue;
        const fnName = matched[2] ? matched[1] : 'anonymous';
        const location = matched[2] || matched[1];
        if (!location.includes('/') && !location.includes('\\') && !location.includes('.ts')) continue;
        frames.push({ function: fnName, location });
        if (frames.length === 2) break;
    }
    return frames;
}

function lastTwoCallerFrames(
    params: Record<string, unknown>,
    stackTrace: unknown,
): CallerFrame[] {
    const stored = params.stack_frames ?? params.stackFrames;
    if (Array.isArray(stored) && stored.length > 0) {
        const parsed = stored.map(frameFromUnknown).filter((frame): frame is CallerFrame => frame !== null).slice(0, 2);
        if (parsed.length > 0) return parsed;
    }
    const fromText = framesFromStackText(String(stackTrace || ''));
    if (fromText.length > 0) return fromText;
    const caller = String(params.callerInfo || params.caller || (params.metadata as Record<string, unknown> | undefined)?.callerInfo || '').trim();
    if (caller) {
        const parts = caller.split(/\s+/);
        if (parts.length >= 2) {
            return [{ function: parts[0], location: parts.slice(1).join(' ') }];
        }
        return [{ function: caller, location: 'caller' }];
    }
    return [];
}

function formatTimestamp(timestamp: AuditEntry['timestamp']): string {
    const ms = typeof timestamp === 'number' ? timestamp : Date.parse(timestamp);
    if (Number.isNaN(ms)) return String(timestamp);
    return new Date(ms).toLocaleTimeString();
}

function ActorBadge({ actor }: { actor: AuditActor }) {
    const config: Record<AuditActor, { icon: typeof User; cls: string; label: string }> = {
        user: { icon: User, cls: 'bg-sky-500/15 text-sky-400 border-sky-500/30', label: 'USER' },
        cli: { icon: Terminal, cls: 'bg-zinc-500/15 text-zinc-400 border-zinc-500/30', label: 'CLI' },
        telegram: { icon: Bot, cls: 'bg-purple-500/15 text-purple-400 border-purple-500/30', label: 'TELEGRAM' },
        system: { icon: Wrench, cls: 'bg-amber-500/15 text-amber-400 border-amber-500/30', label: 'SYSTEM' },
    };
    const { icon: Icon, cls, label } = config[actor] ?? config.system;
    return (
        <span className={`inline-flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-[9px] font-bold ${cls}`}>
            <Icon className="w-2.5 h-2.5 shrink-0" />
            {label}
        </span>
    );
}

const METHOD_LABELS: Record<string, string> = {
    send_prompt_now: 'Send Prompt Now',
    enqueue_prompt: 'Enqueue Prompt',
    'instance-row-prompts-btn': 'Prompts & History',
    'instance-row-switch-btn': 'Switch Profile',
    'instance-row-stop-btn': 'Stop Instance',
    'instance-row-restart-btn': 'Restart Instance',
    'instance-row-more-menu-btn': 'More Options Menu',
    'instance-menu-delete-cache': 'Delete Cache & Re-inject',
    'instance-menu-fast-forward': 'Fast Forward Shortcut',
    'instance-menu-open-folder': 'Open Data Directory',
    'instance-menu-copy-token': 'Copy Auth Token',
    'prompt-tree-refresh-btn': 'Refresh Prompt Tree',
    'prompt-tree-copy-all-btn': 'Copy All With Images',
    'prompt-tree-send-now-btn': 'Send Prompt Now',
    'prompt-tree-enqueue-btn': 'Enqueue Prompt',
    'prompt-tree-instructions-tab': 'Prompt Instructions View',
    'prompt-tree-results-tab': 'AI Results & Outputs View',
    'quota-progress-bar': 'Quota Progress Bar',
    'quota-progress-track': 'Quota Progress Track',
    'quota-milestone-node': 'Quota Milestone Node',
};

function humanMethodLabel(methodName: string, uiName?: unknown): string {
    if (typeof uiName === 'string' && uiName) {
        if (METHOD_LABELS[uiName]) return METHOD_LABELS[uiName];
        return uiName
            .replace(/-(btn|tab|menu|card|item|view)$/, '')
            .replace(/-/g, ' ')
            .replace(/\b\w/g, (c) => c.toUpperCase());
    }
    if (METHOD_LABELS[methodName]) {
        return METHOD_LABELS[methodName];
    }
    if (methodName.startsWith('click:')) {
        const sub = methodName.slice(6);
        if (METHOD_LABELS[sub]) return `Click: ${METHOD_LABELS[sub]}`;
        if (sub.includes('-')) {
            const formatted = sub
                .replace(/-(btn|tab|menu|card|item|view)$/, '')
                .replace(/-/g, ' ')
                .replace(/\b\w/g, (c) => c.toUpperCase());
            return `Click: ${formatted}`;
        }
        return `Click: ${sub}`;
    }
    if (methodName.startsWith('input:')) {
        const sub = methodName.slice(6);
        return `Input: ${sub}`;
    }
    return methodName;
}

function EntryRow({ entry }: { entry: AuditEntry }) {
    const [expanded, setExpanded] = useState(false);
    const params = useMemo(() => parseParams(entry.params), [entry.params]);
    const methodName = entry.methodName || entry.method_name || 'CLICK';
    const methodLabel = humanMethodLabel(methodName, params.uiName);
    const uiTarget = entry.uiTarget || entry.ui_target || '-';

    const stackTrace = params.stackTrace || params.stack || (params.metadata as Record<string, unknown> | undefined)?.stackTrace;
    const callerInfo = params.callerInfo || params.caller || (params.metadata as Record<string, unknown> | undefined)?.callerInfo;
    // Dedicated `stackTrace` column wins; legacy params shapes are the fallback.
    const dedicatedFrames = useMemo(() => framesFromDedicatedTrace(entry.stackTrace), [entry.stackTrace]);
    const traceFrames = dedicatedFrames.length > 0 ? dedicatedFrames : lastTwoCallerFrames(params, stackTrace);

    return (
        <>
            <tr
                onClick={() => setExpanded((v) => !v)}
                className="border-b border-slate-700/40 hover:bg-slate-700/30 cursor-pointer transition-colors"
            >
                <td className="px-2 py-1.5 font-mono text-[10px] text-slate-400 whitespace-nowrap">
                    {formatTimestamp(entry.timestamp)}
                </td>
                <td className="px-2 py-1.5"><ActorBadge actor={entry.actor} /></td>
                <td className="px-2 py-1.5">
                    <span className="inline-block rounded bg-slate-700/60 px-1.5 py-0.5 text-[10px] text-slate-300">
                        {entry.trigger}
                    </span>
                </td>
                <td className="px-2 py-1.5 text-[10px] text-slate-200 max-w-[220px] truncate" title={methodName}>
                    {methodLabel}
                </td>
                <td className="px-2 py-1.5 font-mono text-[10px] text-slate-400 max-w-[260px] truncate" title={uiTarget}>
                    {uiTarget}
                </td>
                <td className="px-2 py-1.5">
                    {entry.result === 'ok' ? (
                        <span className="text-[10px] font-bold text-emerald-400">OK</span>
                    ) : (
                        <span className="text-[10px] font-bold text-red-400" title={entry.message}>ERROR</span>
                    )}
                </td>
                <td className="px-2 py-1.5">
                    <ChevronDown className={`w-3.5 h-3.5 text-slate-500 transition-transform ${expanded ? 'rotate-180' : ''}`} />
                </td>
            </tr>
            {expanded && (
                <tr className="border-b border-slate-700/40 bg-slate-900/60">
                    <td colSpan={7} className="px-3 py-3 space-y-2">
                        {Boolean(params.machine_name || params.node_alias || params.agm_version) && (
                            <div className="flex flex-wrap items-center gap-2 text-[10px] font-mono text-cyan-300 bg-cyan-950/40 px-2.5 py-1.5 rounded-md border border-cyan-500/20">
                                <Terminal className="w-3.5 h-3.5 text-cyan-400 shrink-0" />
                                <span className="font-semibold text-cyan-400">Node Telemetry:</span>
                                {Boolean(params.machine_name) && <span>🖥️ {String(params.machine_name)}</span>}
                                {Boolean(params.local_ip) && <span className="text-slate-400">({String(params.local_ip)})</span>}
                                {Boolean(params.node_alias) && <span>🏷️ {String(params.node_alias)}</span>}
                                {Boolean(params.agm_version) && <span className="text-slate-400">📦 {String(params.agm_version)}</span>}
                                {Boolean(params.switch_mode) && <span className="text-emerald-400 uppercase font-bold">[{String(params.switch_mode)}]</span>}
                                {Boolean(params.condition) && <span className="text-amber-400 font-semibold">{String(params.condition)}</span>}
                            </div>
                        )}
                        {Boolean(callerInfo) && (
                            <div className="flex items-center gap-2 text-[11px] font-mono text-sky-300 bg-sky-950/40 px-2.5 py-1.5 rounded-md border border-sky-500/20">
                                <Terminal className="w-3.5 h-3.5 text-sky-400 shrink-0" />
                                <span className="font-semibold text-sky-400">Caller:</span>
                                <span className="truncate">{String(callerInfo)}</span>
                            </div>
                        )}
                        <div>
                            <h5 className="text-[10px] font-semibold text-sky-400 uppercase tracking-wider mb-1 flex items-center gap-1.5">
                                <Terminal className="w-3 h-3" />
                                Stack trace{traceFrames.length > 0 ? ` (${traceFrames.length} frame${traceFrames.length === 1 ? '' : 's'})` : ''}
                            </h5>
                            {traceFrames.length === 0 ? (
                                <p className="font-mono text-[10px] text-slate-500">Stack trace unavailable for this entry.</p>
                            ) : (
                                <ol className="font-mono text-[10px] text-sky-200/90 bg-slate-950/80 p-2.5 rounded-lg border border-sky-500/20 space-y-1">
                                    {traceFrames.map((frame, index) => (
                                        <li key={`${frame.function}-${frame.location}-${index}`}>
                                            <span className="text-slate-500 mr-1.5">{index + 1}.</span>
                                            <span className="text-sky-100">{frame.function}</span>
                                            <span className="text-slate-500"> — </span>
                                            <span className="text-emerald-300">{frame.location}</span>
                                        </li>
                                    ))}
                                </ol>
                            )}
                        </div>
                        {Boolean(stackTrace) && (
                            <div>
                                <h5 className="text-[10px] font-semibold text-sky-400 uppercase tracking-wider mb-1 flex items-center gap-1.5">
                                    <Terminal className="w-3 h-3" />
                                    Call Stack &amp; Source Location
                                </h5>
                                <pre className="font-mono text-[10px] text-sky-200/90 bg-slate-950/80 p-2.5 rounded-lg border border-sky-500/20 whitespace-pre overflow-x-auto max-h-36">
                                    {String(stackTrace)}
                                </pre>
                            </div>
                        )}
                        <div>
                            <h5 className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">
                                Interaction Parameters
                            </h5>
                            <pre className="font-mono text-[10px] text-slate-300 bg-slate-950/60 p-2.5 rounded-lg border border-slate-800 whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
                                {JSON.stringify(params, null, 2)}
                            </pre>
                        </div>
                        {entry.message && (
                            <p className="mt-1 text-[11px] font-medium text-red-400 flex items-center gap-1.5">
                                <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
                                {entry.message}
                            </p>
                        )}
                    </td>
                </tr>
            )}
        </>
    );
}

export function AuditPanel() {
    const { t } = useTranslation();
    const isOpen = useAuditStore((s) => s.isPanelOpen);
    const setPanelOpen = useAuditStore((s) => s.setPanelOpen);
    const entries = useAuditStore((s) => s.entries);
    const isLoading = useAuditStore((s) => s.isLoading);
    const isRecording = useAuditStore((s) => s.isRecording);
    const refreshEntries = useAuditStore((s) => s.refreshEntries);
    const clearEntries = useAuditStore((s) => s.clearEntries);
    const actorFilter = useAuditStore((s) => s.actorFilter);
    const resultFilter = useAuditStore((s) => s.resultFilter);
    const setActorFilter = useAuditStore((s) => s.setActorFilter);
    const setResultFilter = useAuditStore((s) => s.setResultFilter);

    const filtered = useMemo(
        () =>
            entries.filter(
                (e) =>
                    (actorFilter === 'all' || e.actor === actorFilter) &&
                    (resultFilter === 'all' || e.result === resultFilter)
            ),
        [entries, actorFilter, resultFilter]
    );

    const [copiedType, setCopiedType] = useState<'json' | 'jsonl' | 'summary' | null>(null);

    if (!isOpen) return null;

    const handleCopySummary = async () => {
        try {
            const total = filtered.length;
            const actorCounts: Record<string, number> = {};
            const resultCounts: Record<string, number> = {};
            const triggerCounts: Record<string, number> = {};

            filtered.forEach((e) => {
                actorCounts[e.actor] = (actorCounts[e.actor] || 0) + 1;
                resultCounts[e.result] = (resultCounts[e.result] || 0) + 1;
                triggerCounts[e.trigger] = (triggerCounts[e.trigger] || 0) + 1;
            });

            const recentEntries = filtered.slice(0, 10).map((e, idx) => {
                const time = formatTimestamp(e.timestamp);
                const p = parseParams(e.params);
                const mName = e.methodName || e.method_name || 'CLICK';
                const label = humanMethodLabel(mName, p.uiName);
                const target = e.uiTarget || e.ui_target || '-';
                return `  ${idx + 1}. [${time}] [${e.actor.toUpperCase()}] [${e.result.toUpperCase()}] ${label} (${target})`;
            });

            const summaryLines = [
                `========================================`,
                `INTERACTION AUDIT LOG SUMMARY`,
                `Generated: ${new Date().toISOString()}`,
                `Total Filtered Entries: ${total} (out of ${entries.length} total)`,
                `========================================`,
                ``,
                `[BREAKDOWN BY ACTOR]`,
                ...Object.entries(actorCounts).map(([actor, count]) => `  - ${actor}: ${count}`),
                ``,
                `[BREAKDOWN BY RESULT]`,
                ...Object.entries(resultCounts).map(([res, count]) => `  - ${res}: ${count}`),
                ``,
                `[BREAKDOWN BY TRIGGER]`,
                ...Object.entries(triggerCounts).map(([trig, count]) => `  - ${trig}: ${count}`),
                ``,
                `[RECENT INTERACTIONS (Up to 10)]`,
                ...(recentEntries.length > 0 ? recentEntries : ['  (None)']),
                ``,
                `========================================`,
            ];

            const summaryText = summaryLines.join('\n');
            const success = await copyToClipboard(summaryText);
            if (success) {
                setCopiedType('summary');
                showToast({ message: t('audit.copied_summary', 'Audit summary copied to clipboard'), type: 'success' });
                window.setTimeout(() => setCopiedType(null), 2000);
            } else {
                showToast(t('audit.copy_failed', 'Failed to copy audit log to clipboard'), 'error');
            }
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: `${SOURCE}.handleCopySummary`,
                triggerAction: 'copyToClipboard',
            });
        }
    };

    const handleCopyJson = async () => {
        try {
            const payload = JSON.stringify(filtered, null, 2);
            const success = await copyToClipboard(payload);
            if (success) {
                setCopiedType('json');
                showToast({ message: t('audit.copied_json', 'JSON copied to clipboard'), type: 'success' });
                window.setTimeout(() => setCopiedType(null), 2000);
            } else {
                showToast(t('audit.copy_failed', 'Failed to copy audit log to clipboard'), 'error');
            }
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: `${SOURCE}.handleCopyJson`,
                triggerAction: 'copyToClipboard',
            });
        }
    };

    const handleCopyJsonl = async () => {
        try {
            const lines = filtered
                .map((e) => JSON.stringify({ ...e, params: parseParams(e.params) }))
                .join('\n');
            const success = await copyToClipboard(`${lines}\n`);
            if (success) {
                setCopiedType('jsonl');
                showToast({ message: t('audit.copied_jsonl', 'JSONL copied to clipboard'), type: 'success' });
                window.setTimeout(() => setCopiedType(null), 2000);
            } else {
                showToast(t('audit.copy_failed', 'Failed to copy audit log to clipboard'), 'error');
            }
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: `${SOURCE}.handleCopyJsonl`,
                triggerAction: 'copyToClipboard',
            });
        }
    };

    const handleClear = () => {
        if (window.confirm(t('audit.confirm_clear', 'Delete all recorded audit entries?'))) {
            void clearEntries();
        }
    };

    const selectCls =
        'bg-transparent border-0 focus:ring-0 focus:outline-none text-[11px] text-slate-300 cursor-pointer';

    return createPortal(
        <div className="fixed inset-0 z-[99999] flex items-center justify-center p-4 overflow-y-auto bg-black/60 backdrop-blur-xs">
            <div className="relative w-full max-w-4xl max-h-[90vh] flex flex-col my-auto bg-slate-900 border border-slate-700/60 rounded-2xl shadow-2xl overflow-hidden">
                {/* Header */}
                <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700/60 shrink-0">
                    <div className="flex items-center gap-2">
                        <span
                            className={`inline-block w-2 h-2 rounded-full ${isRecording ? 'bg-red-500 animate-pulse' : 'bg-slate-500'}`}
                        />
                        <h2 className="text-sm font-semibold text-slate-100">
                            {t('audit.panel_title', 'Interaction Audit Log')}
                        </h2>
                        <span className="text-[10px] font-mono text-slate-500">
                            {filtered.length}/{entries.length}
                        </span>
                    </div>
                    <button
                        type="button"
                        data-ui-name="audit-close-btn"
                        data-ui-id="audit-modal-close"
                        onClick={() => setPanelOpen(false)}
                        className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-700/60 transition-colors cursor-pointer"
                        aria-label={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Toolbar: filters + actions */}
                <div className="flex flex-wrap items-center gap-2 px-4 py-2 border-b border-slate-700/60 shrink-0 text-[11px]">
                    <div className="inline-flex items-center rounded-full border border-slate-700/60 bg-slate-800/60 px-2 py-0.5">
                        <span className="text-slate-500 mr-1">{t('audit.actor', 'Actor')}</span>
                        <select
                            className={selectCls}
                            value={actorFilter}
                            onChange={(e) => setActorFilter(e.target.value as AuditActorFilter)}
                            aria-label={t('audit.filter_actor', 'Filter by actor')}
                        >
                            <option value="all">{t('audit.all', 'All')}</option>
                            <option value="user">user</option>
                            <option value="cli">cli</option>
                            <option value="telegram">telegram</option>
                            <option value="system">system</option>
                        </select>
                    </div>
                    <div className="inline-flex items-center rounded-full border border-slate-700/60 bg-slate-800/60 px-2 py-0.5">
                        <span className="text-slate-500 mr-1">{t('audit.result', 'Result')}</span>
                        <select
                            className={selectCls}
                            value={resultFilter}
                            onChange={(e) => setResultFilter(e.target.value as AuditResultFilter)}
                            aria-label={t('audit.filter_result', 'Filter by result')}
                        >
                            <option value="all">{t('audit.all', 'All')}</option>
                            <option value="ok">ok</option>
                            <option value="error">error</option>
                        </select>
                    </div>
                    <div className="flex-1" />
                    <button
                        type="button"
                        data-ui-name="audit-refresh-btn"
                        data-ui-id="audit-refresh"
                        onClick={() => void refreshEntries()}
                        disabled={isLoading}
                        className="inline-flex items-center gap-1 rounded-full border border-slate-700/60 bg-slate-800/60 px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors cursor-pointer disabled:opacity-50"
                    >
                        <RefreshCw className={`w-3 h-3 ${isLoading ? 'animate-spin' : ''}`} />
                        {t('audit.refresh', 'Refresh')}
                    </button>
                    {/* Contiguous Segmented Export Capsule */}
                    <div className="inline-flex items-center rounded-full border border-slate-700/60 bg-slate-800/60 divide-x divide-slate-700/60 overflow-hidden shadow-2xs">
                        <button
                            type="button"
                            data-ui-name="audit-copy-summary-btn"
                            data-ui-id="audit-copy-summary"
                            onClick={handleCopySummary}
                            className={cn(
                                "inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium transition-colors cursor-pointer",
                                copiedType === 'summary'
                                    ? "bg-emerald-500/20 text-emerald-300 font-semibold"
                                    : "text-slate-300 hover:text-white hover:bg-slate-700/50"
                            )}
                            title={t('audit.copy_summary_tooltip', 'Copy human-readable audit summary to clipboard')}
                        >
                            {copiedType === 'summary' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                            {copiedType === 'summary' ? t('common.copied', 'Copied!') : t('audit.summary', 'Copy Summary')}
                        </button>
                        <button
                            type="button"
                            data-ui-name="audit-copy-json-btn"
                            data-ui-id="audit-copy-json"
                            onClick={handleCopyJson}
                            className={cn(
                                "inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium transition-colors cursor-pointer",
                                copiedType === 'json'
                                    ? "bg-emerald-500/20 text-emerald-300 font-semibold"
                                    : "text-slate-300 hover:text-white hover:bg-slate-700/50"
                            )}
                            title={t('audit.copy_json_tooltip', 'Copy filtered entries as JSON to clipboard')}
                        >
                            {copiedType === 'json' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <FileJson className="w-3.5 h-3.5" />}
                            {copiedType === 'json' ? t('common.copied', 'Copied!') : 'JSON'}
                        </button>
                        <button
                            type="button"
                            data-ui-name="audit-copy-jsonl-btn"
                            data-ui-id="audit-copy-jsonl"
                            onClick={handleCopyJsonl}
                            className={cn(
                                "inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium transition-colors cursor-pointer",
                                copiedType === 'jsonl'
                                    ? "bg-emerald-500/20 text-emerald-300 font-semibold"
                                    : "text-slate-300 hover:text-white hover:bg-slate-700/50"
                            )}
                            title={t('audit.copy_jsonl_tooltip', 'Copy filtered entries as JSONL (newline-delimited) to clipboard')}
                        >
                            {copiedType === 'jsonl' ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <FileText className="w-3.5 h-3.5" />}
                            {copiedType === 'jsonl' ? t('common.copied', 'Copied!') : 'JSONL'}
                        </button>
                    </div>
                    <button
                        type="button"
                        data-ui-name="audit-clear-btn"
                        data-ui-id="audit-clear"
                        onClick={handleClear}
                        className="inline-flex items-center gap-1 rounded-full border border-red-500/40 bg-red-500/10 px-2.5 py-1 text-red-300 hover:text-red-200 hover:bg-red-500/20 transition-colors cursor-pointer"
                    >
                        <Trash2 className="w-3 h-3" />
                        {t('audit.clear', 'Clear')}
                    </button>
                </div>

                {/* Entries */}
                <div className="flex-1 min-h-0 overflow-y-auto px-4 py-2">
                    {isLoading && entries.length === 0 ? (
                        <p className="py-8 text-center text-xs text-slate-500">
                            {t('audit.loading', 'Loading entries…')}
                        </p>
                    ) : filtered.length === 0 ? (
                        <p className="py-8 text-center text-xs text-slate-500">
                            {t('audit.empty', 'No audit entries yet. Start recording to capture interactions.')}
                        </p>
                    ) : (
                        <table className="w-full text-left border-collapse">
                            <thead className="sticky top-0 bg-slate-900">
                                <tr className="text-[10px] uppercase tracking-wide text-slate-500">
                                    <th className="px-2 py-1.5 font-medium">{t('audit.time', 'Time')}</th>
                                    <th className="px-2 py-1.5 font-medium">{t('audit.actor', 'Actor')}</th>
                                    <th className="px-2 py-1.5 font-medium">{t('audit.trigger', 'Trigger')}</th>
                                    <th className="px-2 py-1.5 font-medium">{t('audit.method', 'Method')}</th>
                                    <th className="px-2 py-1.5 font-medium">{t('audit.target', 'Target')}</th>
                                    <th className="px-2 py-1.5 font-medium">{t('audit.result', 'Result')}</th>
                                    <th className="px-2 py-1.5 w-8" />
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.map((entry, i) => (
                                    <EntryRow key={entry.id ?? `${entry.timestamp}-${i}`} entry={entry} />
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>

                {/* Footer */}
                <div className="flex items-center gap-2 px-4 py-2 border-t border-slate-700/60 shrink-0 text-[10px] text-slate-500">
                    <Copy className="w-3 h-3" />
                    {t('audit.export_hint', 'Copy currently filtered entries as JSON or JSONL to clipboard for bug reports.')}
                </div>
            </div>
            <div className="absolute inset-0 z-[-1]" onClick={() => setPanelOpen(false)} />
        </div>,
        document.body
    );
}
