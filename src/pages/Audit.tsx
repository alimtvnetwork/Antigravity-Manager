import { useAuditData } from './audit/useAuditData';
import { AuditTable } from './audit/AuditTable';
import { AuditDetailModal } from './audit/AuditDetailModal';

// Re-export helpers for backward compatibility
export {
    formatStrictTime,
    formatTime,
    subjectText,
    moveText,
    recorded,
    getActionLabel,
    isSchedulerAction,
    isSwitchAction,
    renderActionBadge,
    detailRows,
} from './audit/audit-helpers';
export type { DetailRow } from './audit/audit-helpers';
export type {
    TaskRecord,
    TaskHistoryItem,
    TaskDetail,
    TaskHistoryPage,
    TaskPayload,
    AuditFilterType,
} from '../types/audit';

export default function Audit() {
    const {
        page, setPage,
        pageSize, handlePageSizeChange,
        filter, setFilter,
        data, error,
        revealed, setRevealed,
        openId, setOpenId,
        detail, setDetail, detailError,
        copied, setCopied,
        loadDetail,
        allItems, displayItems, filterCounts,
        total, pageCount,
        payload, detailRevealed,
    } = useAuditData();

    return (
        <div className="h-full overflow-auto bg-white dark:bg-[#0c2438] px-4 sm:px-6 pt-3 pb-6 max-w-[1400px] mx-auto w-full text-slate-950 dark:text-base-content">
            <div className="mb-3">
                <h1 className="text-lg font-semibold text-gray-900 dark:text-base-content">Audit Log & History</h1>
                <p className="text-xs text-slate-600 dark:text-slate-400">
                    Audit trail for account rotations, prompt scheduler dispatches, and IDE reconnect events. Addresses hide the domain, and detailed facts are loaded on demand. Double-click any row to view details.
                </p>
            </div>

            {error && (
                <div className="mb-3 rounded-lg border border-rose-200 dark:border-rose-900/50 bg-white dark:bg-rose-950/20 px-3 py-2 text-sm text-rose-700 dark:text-rose-400">
                    {error}
                </div>
            )}

        {/* Filter Tabs / Chips */}
        <div className="mb-3 flex flex-wrap items-center justify-between gap-3">
            <div className="inline-flex items-center gap-1.5 p-1 rounded-xl bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d]">
                <button
                    type="button"
                    onClick={() => setFilter('all')}
                    className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                        filter === 'all'
                            ? 'bg-white dark:bg-[#15334d] text-slate-900 dark:text-white shadow-sm border border-slate-200/80 dark:border-cyan-500/40'
                            : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                    }`}
                >
                    <span>All Activity</span>
                    {data && (
                        <span className={`text-[11px] px-1.5 py-0.5 rounded-full font-semibold ${
                            filter === 'all'
                                ? 'bg-slate-100 text-slate-700 dark:bg-cyan-950/80 dark:text-cyan-300'
                                : 'bg-slate-200/80 text-slate-600 dark:bg-[#0c2438] dark:text-slate-400'
                        }`}>
                            {filterCounts.all}
                        </span>
                    )}
                </button>

                <button
                    type="button"
                    onClick={() => setFilter('switch')}
                    className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                        filter === 'switch'
                            ? 'bg-white dark:bg-[#15334d] text-slate-900 dark:text-white shadow-sm border border-slate-200/80 dark:border-amber-500/40'
                            : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                    }`}
                >
                    <span>Account Switch</span>
                    {data && (
                        <span className={`text-[11px] px-1.5 py-0.5 rounded-full font-semibold ${
                            filter === 'switch'
                                ? 'bg-amber-100 text-amber-800 dark:bg-amber-950/80 dark:text-amber-300'
                                : 'bg-slate-200/80 text-slate-600 dark:bg-[#0c2438] dark:text-slate-400'
                        }`}>
                            {filterCounts.switch}
                        </span>
                    )}
                </button>

                <button
                    type="button"
                    onClick={() => setFilter('scheduler')}
                    className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                        filter === 'scheduler'
                            ? 'bg-white dark:bg-[#15334d] text-slate-900 dark:text-white shadow-sm border border-slate-200/80 dark:border-purple-500/40'
                            : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                    }`}
                >
                    <span>Prompt Scheduler</span>
                    {data && (
                        <span className={`text-[11px] px-1.5 py-0.5 rounded-full font-semibold ${
                            filter === 'scheduler'
                                ? 'bg-purple-100 text-purple-800 dark:bg-purple-950/80 dark:text-purple-300'
                                : 'bg-slate-200/80 text-slate-600 dark:bg-[#0c2438] dark:text-slate-400'
                        }`}>
                            {filterCounts.scheduler}
                        </span>
                    )}
                </button>
            </div>

            {filter !== 'all' && (
                <span className="text-xs text-slate-500 dark:text-slate-400">
                    Showing {displayItems.length} of {allItems.length} events on this page
                </span>
            )}
        </div>

            <AuditTable
                displayItems={displayItems}
                filter={filter}
                revealed={revealed}
                setRevealed={setRevealed}
                loadDetail={loadDetail}
            />

            <AuditDetailModal
                openId={openId}
                detail={detail}
                detailError={detailError}
                payload={payload}
                detailRevealed={detailRevealed}
                copied={copied}
                setCopied={setCopied}
                onClose={() => { setOpenId(''); setDetail(null); }}
            />

        {/* Pagination Controls */}
        <div className="mt-3 flex flex-wrap items-center justify-between gap-3 text-xs text-slate-700 dark:text-gray-300">
            <div className="flex items-center gap-3">
                <span>
                    {filter === 'all'
                        ? `${total} events · page ${page + 1} of ${pageCount}`
                        : `Showing ${displayItems.length} of ${allItems.length} on this page (${total} total events)`
                    }
                </span>
                <label className="inline-flex items-center gap-1.5 text-xs text-slate-600 dark:text-slate-400">
                    <span>Items per page:</span>
                    <select
                        value={pageSize}
                        onChange={(e) => handlePageSizeChange(Number(e.target.value))}
                        className="h-7 px-2 py-0.5 rounded-lg border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#0c2438] text-slate-900 dark:text-gray-100 font-medium cursor-pointer"
                    >
                        <option value={100}>100</option>
                        <option value={200}>200</option>
                    </select>
                </label>
            </div>
            <div className="flex gap-2">
                <button
                    type="button"
                    className="rounded-lg border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#0c2438] px-3 py-1 text-slate-950 dark:text-gray-200 hover:bg-slate-50 dark:hover:bg-[#15334d] disabled:opacity-40 cursor-pointer"
                    disabled={page <= 0}
                    onClick={() => setPage((current) => Math.max(0, current - 1))}
                >
                    Previous
                </button>
                <button
                    type="button"
                    className="rounded-lg border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#0c2438] px-3 py-1 text-slate-950 dark:text-gray-200 hover:bg-slate-50 dark:hover:bg-[#15334d] disabled:opacity-40 cursor-pointer"
                    disabled={page + 1 >= pageCount}
                    onClick={() => setPage((current) => current + 1)}
                >
                    Next
                </button>
            </div>
        </div>
        </div>
    );
}
