import { useEffect, useState } from 'react';
import { request } from '../utils/request';
import { maskEmail } from '../utils/maskEmail';

const PAGE_SIZE = 100;

interface TaskRecord {
    id: string;
    action_code: number;
    action: string;
    action_label: string;
    status: string;
    subject: string;
    detail: string;
    instance_id: string;
    split_path: string;
    from_email?: string;
    to_email?: string;
    created_at: number;
    finished_at?: number | null;
}

interface TaskDetail extends TaskRecord {
    payload_json: string;
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

interface SwitchPayload {
    from_email?: string;
    to_email?: string;
    reason?: string;
    how?: string;
    prompt_id?: string;
    prompt_text?: string;
    prompt_reinjected?: boolean;
    moved_at?: number;
    switch_ok?: boolean;
}

function formatTime(unix: number) {
    if (!unix) return '';
    return new Date(unix * 1000).toLocaleString();
}

function subjectText(subject: string, revealed: boolean) {
    if (!subject || !subject.includes('@')) return subject || '—';
    return revealed ? subject : maskEmail(subject);
}

function moveText(item: TaskRecord, revealed: boolean) {
    const from = (item.from_email || '').trim();
    const to = (item.to_email || '').trim();
    if (!from && !to) return subjectText(item.subject, revealed);
    const fromLabel = from ? subjectText(from, revealed) : '—';
    const toLabel = to ? subjectText(to, revealed) : subjectText(item.subject, revealed);
    return `${fromLabel} → ${toLabel}`;
}

function recorded(value: string | undefined, fallback: string) {
    const text = (value || '').trim();
    return text || fallback;
}

function detailRows(detail: TaskDetail, payload: SwitchPayload | null, revealed: boolean) {
    const when = formatTime(payload?.moved_at || detail.created_at);
    const from = payload?.from_email || detail.from_email || '';
    const to = payload?.to_email || detail.to_email || detail.subject;
    const isSwitch = (detail.action || '').toLowerCase().includes('switch')
        || Boolean(payload?.from_email || payload?.to_email || payload?.reason);
    if (!isSwitch) {
        return [
            { label: 'When', value: when },
            { label: 'Action', value: detail.action_label || detail.action },
            { label: 'Account', value: subjectText(detail.subject, revealed) },
            { label: 'What happened', value: detail.detail || '—' },
        ];
    }
    const reinject = payload && typeof payload.prompt_reinjected === 'boolean'
        ? (payload.prompt_reinjected ? 'Yes' : 'No')
        : 'Not recorded on this row';
    return [
        { label: 'From', value: subjectText(from, revealed) },
        { label: 'To', value: subjectText(to, revealed) },
        { label: 'When', value: when },
        { label: 'Reason', value: recorded(payload?.reason, 'Not recorded on this row') },
        { label: 'How', value: recorded(payload?.how, 'Not recorded on this row') },
        { label: 'Prompt running', value: recorded(payload?.prompt_text, 'No running prompt was stored') },
        { label: 'Prompt injected again', value: reinject },
    ];
}

export default function Audit() {
    const [page, setPage] = useState(0);
    const [data, setData] = useState<TaskHistoryPage | null>(null);
    const [error, setError] = useState('');
    const [revealed, setRevealed] = useState<Record<string, boolean>>({});
    const [openId, setOpenId] = useState('');
    const [detail, setDetail] = useState<TaskDetail | null>(null);
    const [detailError, setDetailError] = useState('');

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

    const loadDetail = (id: string) => {
        setOpenId(id);
        setDetail(null);
        setDetailError('');
        request<TaskDetail>('get_task_history_detail', { id })
            .then((result) => setDetail(result))
            .catch((err: unknown) => setDetailError(err instanceof Error ? err.message : String(err)));
    };

    const total = data?.total ?? 0;
    const pageCount = Math.max(1, Math.ceil(total / PAGE_SIZE));
    let payload: SwitchPayload | null = null;
    if (detail?.payload_json) {
        try {
            payload = JSON.parse(detail.payload_json) as SwitchPayload;
        } catch {
            payload = null;
        }
    }
    const detailRevealed = detail ? Boolean(revealed[detail.id]) : false;

    return (
        <div className="h-full overflow-auto bg-white px-4 sm:px-6 pt-3 pb-6 max-w-[1400px] mx-auto w-full text-slate-950">
            <div className="mb-3">
                <h1 className="text-lg font-semibold">Audit</h1>
                <p className="text-xs text-slate-600 dark:text-slate-400">
                    Addresses hide the domain, such as gmail.com, and keep the start of the name. Detail loads that row only when you open it.
                </p>
            </div>
            {error && (
                <div className="mb-3 rounded-lg border border-rose-200 bg-white px-3 py-2 text-sm text-rose-700">
                    {error}
                </div>
            )}
            <div className="overflow-hidden rounded-xl border border-slate-200 bg-white">
                <table className="w-full text-left text-xs">
                    <thead className="bg-slate-50 text-slate-700">
                        <tr>
                            <th className="px-3 py-2 font-semibold">When</th>
                            <th className="px-3 py-2 font-semibold">Status</th>
                            <th className="px-3 py-2 font-semibold">Action</th>
                            <th className="px-3 py-2 font-semibold">From → To</th>
                            <th className="px-3 py-2 font-semibold"></th>
                        </tr>
                    </thead>
                    <tbody>
                        {(data?.items || []).map((item) => (
                            <tr key={item.id} className="border-t border-slate-100">
                                <td className="px-3 py-2 whitespace-nowrap">{formatTime(item.created_at)}</td>
                                <td className="px-3 py-2">{item.status}</td>
                                <td className="px-3 py-2" title={item.action}>{item.action_label || item.action}</td>
                                <td className="px-3 py-2">{moveText(item, Boolean(revealed[item.id]))}</td>
                                <td className="px-3 py-2 text-right">
                                    <button
                                        type="button"
                                        className="mr-2 text-slate-700 underline"
                                        onClick={() => setRevealed((current) => ({ ...current, [item.id]: !current[item.id] }))}
                                    >
                                        {revealed[item.id] ? 'Hide email' : 'Show email'}
                                    </button>
                                    <button
                                        type="button"
                                        className="rounded-lg border border-slate-300 bg-white px-2 py-1"
                                        onClick={() => loadDetail(item.id)}
                                    >
                                        Detail
                                    </button>
                                </td>
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
            {openId && (
                <div className="mt-3 rounded-xl border border-slate-200 bg-white p-4 text-sm">
                    <div className="mb-2 flex items-center justify-between">
                        <h2 className="font-semibold text-[#f5d76e] bg-[#070b10] inline-block rounded px-2 py-1">Detail</h2>
                        <button type="button" className="text-xs underline" onClick={() => { setOpenId(''); setDetail(null); }}>
                            Close
                        </button>
                    </div>
                    {detailError && <p className="text-rose-700">{detailError}</p>}
                    {!detail && !detailError && <p className="text-slate-500">Loading this row from its split file...</p>}
                    {detail && (
                        <table className="w-full text-left text-xs border border-slate-200 dark:border-white/10">
                            <tbody>
                                {detailRows(detail, payload, detailRevealed).map((row) => (
                                    <tr key={row.label} className="border-t border-slate-100 dark:border-white/10 first:border-t-0">
                                        <th className="w-40 px-3 py-2 font-semibold align-top bg-slate-50 dark:bg-white/5">{row.label}</th>
                                        <td className="px-3 py-2 whitespace-pre-wrap">{row.value}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>
            )}
            <div className="mt-3 flex items-center justify-between text-xs text-slate-700">
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
