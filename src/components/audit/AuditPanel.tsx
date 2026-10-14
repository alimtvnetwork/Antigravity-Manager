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
    Bot,
    ChevronDown,
    Download,
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
import { useAuditStore, type AuditActorFilter, type AuditResultFilter } from './audit-store';
import type { AuditActor, AuditEntry } from './audit-types';

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

function downloadFile(filename: string, content: string, mime: string): void {
    try {
        const blob = new Blob([content], { type: mime });
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = filename;
        document.body.appendChild(anchor);
        anchor.click();
        anchor.remove();
        // Revoke on next tick so the download has started.
        window.setTimeout(() => URL.revokeObjectURL(url), 1000);
    } catch (error) {
        useErrorStore.getState().trackWarning(error, {
            source: SOURCE,
            triggerComponent: 'AuditPanel',
            triggerAction: 'downloadFile',
            context: { filename },
        });
    }
}

function EntryRow({ entry }: { entry: AuditEntry }) {
    const [expanded, setExpanded] = useState(false);
    const params = useMemo(() => parseParams(entry.params), [entry.params]);

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
                <td className="px-2 py-1.5 font-mono text-[10px] text-slate-200 max-w-[220px] truncate" title={entry.methodName}>
                    {entry.methodName}
                </td>
                <td className="px-2 py-1.5 font-mono text-[10px] text-slate-400 max-w-[260px] truncate" title={entry.uiTarget}>
                    {entry.uiTarget}
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
                    <td colSpan={7} className="px-3 py-2">
                        <pre className="font-mono text-[10px] text-slate-300 whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
                            {JSON.stringify(params, null, 2)}
                        </pre>
                        {entry.message && (
                            <p className="mt-1 text-[10px] text-red-400">{entry.message}</p>
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

    if (!isOpen) return null;

    const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);

    const handleExportJson = () => {
        downloadFile(
            `audit-log-${stamp}.json`,
            JSON.stringify(filtered, null, 2),
            'application/json'
        );
    };

    const handleExportJsonl = () => {
        const lines = filtered
            .map((e) => JSON.stringify({ ...e, params: parseParams(e.params) }))
            .join('\n');
        downloadFile(`audit-log-${stamp}.jsonl`, `${lines}\n`, 'application/x-ndjson');
    };

    const handleClear = () => {
        if (window.confirm(t('audit.confirm_clear', 'Delete all recorded audit entries?'))) {
            void clearEntries();
        }
    };

    const selectCls =
        'bg-transparent border-0 focus:ring-0 focus:outline-none text-[11px] text-slate-300 cursor-pointer';

    return createPortal(
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
            <div className="relative w-full max-w-4xl max-h-[90vh] flex flex-col bg-slate-900 border border-slate-700/60 rounded-2xl shadow-2xl overflow-hidden">
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
                        onClick={() => void refreshEntries()}
                        disabled={isLoading}
                        className="inline-flex items-center gap-1 rounded-full border border-slate-700/60 bg-slate-800/60 px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors cursor-pointer disabled:opacity-50"
                    >
                        <RefreshCw className={`w-3 h-3 ${isLoading ? 'animate-spin' : ''}`} />
                        {t('audit.refresh', 'Refresh')}
                    </button>
                    <button
                        type="button"
                        onClick={handleExportJson}
                        className="inline-flex items-center gap-1 rounded-full border border-slate-700/60 bg-slate-800/60 px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors cursor-pointer"
                    >
                        <FileJson className="w-3 h-3" />
                        JSON
                    </button>
                    <button
                        type="button"
                        onClick={handleExportJsonl}
                        className="inline-flex items-center gap-1 rounded-full border border-slate-700/60 bg-slate-800/60 px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors cursor-pointer"
                    >
                        <FileText className="w-3 h-3" />
                        JSONL
                    </button>
                    <button
                        type="button"
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
                    <Download className="w-3 h-3" />
                    {t('audit.export_hint', 'Exports download the currently filtered entries — attach them to bug reports.')}
                </div>
            </div>
            <div className="absolute inset-0 z-[-1]" onClick={() => setPanelOpen(false)} />
        </div>,
        document.body
    );
}
