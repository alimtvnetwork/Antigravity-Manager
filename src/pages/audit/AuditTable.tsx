import { Info } from 'lucide-react';
import { maskEmail } from '../../utils/maskEmail';
import { isSchedulerAction, moveText, renderActionBadge, formatStrictTime } from './audit-helpers';
import type { TaskRecord, AuditFilterType } from '../../types/audit';

interface AuditTableProps {
    displayItems: TaskRecord[];
    filter: AuditFilterType;
    revealed: Record<string, boolean>;
    setRevealed: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    loadDetail: (id: string) => void;
}

export function AuditTable({ displayItems, filter, revealed, setRevealed, loadDetail }: AuditTableProps) {
    return (
        <div className="overflow-hidden rounded-xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438]">
            <table className="w-full text-left text-xs">
                <thead className="bg-slate-50 dark:bg-[#071a27] text-slate-700 dark:text-gray-300 border-b border-slate-200 dark:border-[#15334d]">
                    <tr>
                        <th className="px-3 py-2.5 font-semibold">From → To / Subject</th>
                        <th className="px-3 py-2.5 font-semibold">Action</th>
                        <th className="px-3 py-2.5 font-semibold">Status</th>
                        <th className="px-3 py-2.5 font-semibold">Time</th>
                        <th className="px-3 py-2.5 font-semibold text-right">Details</th>
                    </tr>
                </thead>
                <tbody>
                    {displayItems.map((item) => {
                        const isRevealed = Boolean(revealed[item.id]);
                        const hasEmail = Boolean(item.from_email || item.to_email || item.subject?.includes('@'));
                        const isSched = isSchedulerAction(item);

                        const fromEmailRaw = item.from_email || '';
                        const toEmailRaw = item.to_email || item.subject || '';
                        const fromEmailText = fromEmailRaw ? (isRevealed ? fromEmailRaw : maskEmail(fromEmailRaw)) : '—';
                        const toEmailText = toEmailRaw ? (isRevealed ? toEmailRaw : maskEmail(toEmailRaw)) : '—';

                        return (
                            <tr
                                key={item.id}
                                onDoubleClick={() => loadDetail(item.id)}
                                className="border-t border-slate-100 dark:border-[#15334d]/60 hover:bg-slate-50/60 dark:hover:bg-[#15334d]/40 transition-colors cursor-pointer select-text"
                                title="Double click row to view detail"
                            >
                                {/* 1. From -> To / Subject */}
                                <td className="px-3 py-2">
                                    {isSched ? (
                                        <div className="flex flex-col">
                                            <span className="font-medium text-slate-900 dark:text-slate-100">
                                                {item.subject || '—'}
                                            </span>
                                            {item.detail && item.detail !== item.subject && (
                                                <span className="text-[11px] text-slate-500 dark:text-slate-400 truncate max-w-md" title={item.detail}>
                                                    {item.detail}
                                                </span>
                                            )}
                                        </div>
                                    ) : hasEmail ? (
                                        <div className="flex items-center gap-1.5 font-mono text-xs flex-wrap">
                                            <span
                                                className="cursor-pointer hover:underline text-slate-800 dark:text-slate-200 select-text"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setRevealed((current) => ({ ...current, [item.id]: !current[item.id] }));
                                                }}
                                                title={isRevealed ? (item.from_email || '—') : 'Click to unmask email'}
                                            >
                                                {fromEmailText}
                                            </span>
                                            <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20">
                                                →
                                            </span>
                                            <span
                                                className="cursor-pointer hover:underline text-slate-800 dark:text-slate-200 select-text"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setRevealed((current) => ({ ...current, [item.id]: !current[item.id] }));
                                                }}
                                                title={isRevealed ? (item.to_email || item.subject) : 'Click to unmask email'}
                                            >
                                                {toEmailText}
                                            </span>
                                        </div>
                                    ) : (
                                        <span className="font-mono text-slate-800 dark:text-slate-200">
                                            {moveText(item, isRevealed)}
                                        </span>
                                    )}
                                </td>

                                {/* 2. Action */}
                                <td className="px-3 py-2 whitespace-nowrap">
                                    {renderActionBadge(item)}
                                </td>

                                {/* 3. Status */}
                                <td className="px-3 py-2 whitespace-nowrap">
                                    <span className={`inline-flex items-center px-2 py-0.5 rounded-[5px] text-[11px] font-medium border ${
                                        /dispatch|ok|success|complete/i.test(item.status)
                                            ? 'bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300 border-cyan-200/50 dark:border-cyan-800/50'
                                            : /fail|err/i.test(item.status)
                                            ? 'bg-rose-50 text-rose-700 dark:bg-rose-950/50 dark:text-rose-300 border-rose-200/60 dark:border-rose-800/60'
                                            : 'bg-slate-100 text-slate-700 dark:bg-[#071a27] dark:text-slate-300 border-slate-200 dark:border-[#15334d]'
                                    }`}>
                                        {item.status || 'unknown'}
                                    </span>
                                </td>

                                {/* 4. Time formatted strictly as DD-MMM-YYYY HH:MM:SS */}
                                <td className="px-3 py-2 whitespace-nowrap text-slate-600 dark:text-slate-400 font-mono text-[11px]">
                                    {formatStrictTime(item.created_at)}
                                </td>

                                {/* 5. Details */}
                                <td className="px-3 py-2 text-right whitespace-nowrap">
                                    {hasEmail && (
                                        <button
                                            type="button"
                                            className="mr-2 text-slate-700 dark:text-cyan-400 underline hover:text-slate-900 dark:hover:text-cyan-300 cursor-pointer text-xs"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                setRevealed((current) => ({ ...current, [item.id]: !current[item.id] }));
                                            }}
                                        >
                                            {revealed[item.id] ? 'Hide email' : 'Show email'}
                                        </button>
                                    )}
                                    <button
                                        type="button"
                                        aria-label="Show audit detail"
                                        title="Detail"
                                        className="inline-flex h-7 w-7 items-center justify-center rounded-[5px] border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#071a27] text-slate-800 dark:text-gray-200 hover:bg-slate-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            loadDetail(item.id);
                                        }}
                                    >
                                        <Info className="h-3.5 w-3.5" />
                                    </button>
                                </td>
                            </tr>
                        );
                    })}
                    {displayItems.length === 0 && (
                        <tr>
                            <td colSpan={5} className="px-3 py-8 text-center text-slate-500 dark:text-slate-400">
                                {filter === 'scheduler' ? (
                                    <div>
                                        <p className="font-medium text-slate-600 dark:text-slate-300">No prompt scheduler events on this page.</p>
                                        <p className="text-xs mt-1 text-slate-400 dark:text-slate-500">Scheduler dispatch logs and requeue events will appear here when triggered.</p>
                                    </div>
                                ) : filter === 'switch' ? (
                                    <div>
                                        <p className="font-medium text-slate-600 dark:text-slate-300">No account switch events on this page.</p>
                                        <p className="text-xs mt-1 text-slate-400 dark:text-slate-500">Account rotation and quota-based switch events will appear here.</p>
                                    </div>
                                ) : (
                                    <p>No history yet. Add or switch an account or run the prompt scheduler and it will show up here.</p>
                                )}
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    );
}
