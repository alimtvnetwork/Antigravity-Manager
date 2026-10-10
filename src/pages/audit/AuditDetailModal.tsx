import { Copy } from 'lucide-react';
import { detailRows, renderActionBadge } from './audit-helpers';
import type { TaskDetail, TaskPayload } from '../../types/audit';

interface AuditDetailModalProps {
    openId: string;
    detail: TaskDetail | null;
    detailError: string;
    payload: TaskPayload | null;
    detailRevealed: boolean;
    copied: boolean;
    setCopied: (v: boolean) => void;
    onClose: () => void;
}

export function AuditDetailModal({
    openId,
    detail,
    detailError,
    payload,
    detailRevealed,
    copied,
    setCopied,
    onClose,
}: AuditDetailModalProps) {
    if (!openId) return null;
    return (
        <div
            className="fixed inset-0 z-[80] flex items-center justify-center bg-black/50 p-4"
            onClick={onClose}
        >
            <div
                className="flex max-h-[85vh] w-full max-w-3xl flex-col rounded-xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] text-slate-950 dark:text-gray-100 shadow-xl"
                onClick={(event) => event.stopPropagation()}
                role="dialog"
                aria-label="Audit detail"
            >
                <div className="flex items-center justify-between gap-3 border-b border-slate-200 dark:border-[#15334d] px-4 py-3">
                    <div className="flex items-center gap-2">
                        <h2 className="font-semibold text-gray-900 dark:text-gray-100">Audit Detail</h2>
                        {detail && renderActionBadge(detail)}
                    </div>
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            className="inline-flex items-center gap-1 rounded-lg border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#071a27] text-slate-800 dark:text-gray-200 hover:bg-slate-50 dark:hover:bg-[#15334d] px-3 py-1 text-xs disabled:opacity-40 cursor-pointer"
                            disabled={!detail}
                            onClick={() => {
                                if (!detail) return;
                                const rows = detailRows(detail, payload, detailRevealed);
                                const lines = rows.map((row) => `${row.label}: ${row.copyValue ?? (typeof row.value === 'string' ? row.value : '')}`);
                                const raw = detail.payload_json?.trim();
                                const body = raw ? `${lines.join('\n')}\n\n${raw}` : lines.join('\n');
                                void navigator.clipboard.writeText(body).then(() => {
                                    setCopied(true);
                                    window.setTimeout(() => setCopied(false), 1500);
                                });
                            }}
                        >
                            <Copy className="h-3.5 w-3.5" />
                            {copied ? 'Copied' : 'Copy'}
                        </button>
                        <button
                            type="button"
                            className="rounded-lg border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#071a27] text-slate-800 dark:text-gray-200 hover:bg-slate-50 dark:hover:bg-[#15334d] px-3 py-1 text-xs cursor-pointer"
                            onClick={onClose}
                        >
                            Cancel
                        </button>
                    </div>
                </div>
                <div className="overflow-y-auto p-4 text-sm">
                    {detailError && <p className="text-rose-700 dark:text-rose-400">{detailError}</p>}
                    {!detail && !detailError && <p className="text-slate-500 dark:text-slate-400">Loading this row from its split file...</p>}
                    {detail && (
                        <table className="w-full text-left text-xs border border-slate-200 dark:border-[#15334d] rounded-lg overflow-hidden">
                            <tbody>
                                {detailRows(detail, payload, detailRevealed).map((row) => (
                                    <tr key={row.label} className="border-t border-slate-100 dark:border-[#15334d]/60 first:border-t-0">
                                        <th className="w-44 px-3.5 py-2.5 font-semibold align-top bg-slate-50 dark:bg-[#071a27] text-slate-700 dark:text-gray-300">{row.label}</th>
                                        <td className="px-3.5 py-2.5 whitespace-pre-wrap text-slate-900 dark:text-gray-100 bg-white dark:bg-[#0c2438]">{row.value}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>
            </div>
        </div>
    );
}
