import { useEffect, useState } from 'react';
import { request } from '../utils/request';

const PAGE_SIZE = 100;

interface TaskRecord {
    id: string;
    action: string;
    status: string;
    subject: string;
    detail: string;
    instance_id: string;
    split_path: string;
    created_at: number;
    finished_at?: number | null;
}

interface SplitInfo {
    id: string;
    file_path: string;
    row_count: number;
    is_current: boolean;
}

interface TaskHistoryPage {
    total: number;
    offset: number;
    limit: number;
    items: TaskRecord[];
    splits: SplitInfo[];
}

function formatTime(unix: number) {
    if (!unix) return '';
    return new Date(unix * 1000).toLocaleString();
}

export default function Audit() {
    const [page, setPage] = useState(0);
    const [data, setData] = useState<TaskHistoryPage | null>(null);
    const [error, setError] = useState('');

    useEffect(() => {
        let alive = true;
        request<TaskHistoryPage>('list_task_history', { offset: page * PAGE_SIZE, limit: PAGE_SIZE })
            .then((result) => {
                if (alive) {
                    setData(result);
                    setError('');
                }
            })
            .catch((err: unknown) => {
                if (alive) setError(err instanceof Error ? err.message : String(err));
            });
        return () => {
            alive = false;
        };
    }, [page]);

    const total = data?.total ?? 0;
    const pageCount = Math.max(1, Math.ceil(total / PAGE_SIZE));

    return (
        <div className="h-full overflow-auto px-4 sm:px-6 pt-3 pb-6 max-w-[1400px] mx-auto w-full">
            <div className="mb-3">
                <h1 className="text-lg font-semibold text-slate-950 dark:text-white">Audit</h1>
                <p className="text-xs text-slate-600 dark:text-slate-300">
                    Account adds and switches are stored in split SQLite files. The root index is task_index.db.
                </p>
            </div>
            {data && data.splits.length > 0 && (
                <div className="mb-3 rounded-xl border border-slate-200 bg-white p-3 text-xs text-slate-950 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100">
                    {data.splits.map((split) => (
                        <div key={split.id} className="truncate">
                            {split.is_current ? 'Current' : 'Closed'} · {split.row_count} rows · {split.file_path}
                        </div>
                    ))}
                </div>
            )}
            {error && (
                <div className="mb-3 rounded-lg border border-rose-200 bg-white px-3 py-2 text-sm text-rose-700">
                    {error}
                </div>
            )}
            <div className="overflow-hidden rounded-xl border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900">
                <table className="w-full text-left text-xs">
                    <thead className="bg-slate-50 text-slate-700 dark:bg-slate-800 dark:text-slate-100">
                        <tr>
                            <th className="px-3 py-2 font-semibold">When</th>
                            <th className="px-3 py-2 font-semibold">Status</th>
                            <th className="px-3 py-2 font-semibold">Action</th>
                            <th className="px-3 py-2 font-semibold">Subject</th>
                            <th className="px-3 py-2 font-semibold">Detail</th>
                        </tr>
                    </thead>
                    <tbody>
                        {(data?.items || []).map((item) => (
                            <tr key={item.id} className="border-t border-slate-100 text-slate-950 dark:border-slate-800 dark:text-slate-100">
                                <td className="px-3 py-2 whitespace-nowrap">{formatTime(item.created_at)}</td>
                                <td className="px-3 py-2">{item.status}</td>
                                <td className="px-3 py-2">{item.action}</td>
                                <td className="px-3 py-2">{item.subject}</td>
                                <td className="px-3 py-2">{item.detail}</td>
                            </tr>
                        ))}
                        {data && data.items.length === 0 && (
                            <tr>
                                <td colSpan={5} className="px-3 py-6 text-slate-500">
                                    No history yet. Add or switch an account and it will show up here.
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
            <div className="mt-3 flex items-center justify-between text-xs text-slate-700 dark:text-slate-200">
                <span>{total} events · page {page + 1} of {pageCount}</span>
                <div className="flex gap-2">
                    <button
                        type="button"
                        className="rounded-lg border border-slate-300 bg-white px-3 py-1 text-slate-950 disabled:opacity-40"
                        disabled={page <= 0}
                        onClick={() => setPage((current) => Math.max(0, current - 1))}
                    >
                        Previous
                    </button>
                    <button
                        type="button"
                        className="rounded-lg border border-slate-300 bg-white px-3 py-1 text-slate-950 disabled:opacity-40"
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
