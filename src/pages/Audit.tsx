import { useEffect, useState, useMemo } from 'react';
import { Copy, Info } from 'lucide-react';
import { request } from '../utils/request';
import { maskEmail } from '../utils/maskEmail';
import type {
    TaskRecord,
    TaskHistoryItem,
    TaskDetail,
    TaskHistoryPage,
    TaskPayload,
    AuditFilterType,
} from '../types/audit';

export type { TaskRecord, TaskHistoryItem, TaskDetail, TaskHistoryPage, TaskPayload, AuditFilterType };

const MONTH_NAMES = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];

export function formatStrictTime(unix: number | undefined | null): string {
    if (!unix) return '—';
    const d = new Date(unix * 1000);
    if (isNaN(d.getTime())) return '—';
    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'JAN';
    const year = d.getFullYear();
    const hours = String(d.getHours()).padStart(2, '0');
    const mins = String(d.getMinutes()).padStart(2, '0');
    const secs = String(d.getSeconds()).padStart(2, '0');
    return `${day}-${month}-${year} ${hours}:${mins}:${secs}`;
}

export function formatTime(unix: number) {
    return formatStrictTime(unix);
}

export function subjectText(subject: string, revealed: boolean) {
    if (!subject || !subject.includes('@')) return subject || '—';
    return revealed ? subject : maskEmail(subject);
}

export function moveText(item: TaskRecord, revealed: boolean) {
    const from = (item.from_email || '').trim();
    const to = (item.to_email || '').trim();
    if (!from && !to) return subjectText(item.subject, revealed);
    const fromLabel = from ? subjectText(from, revealed) : '—';
    const toLabel = to ? subjectText(to, revealed) : subjectText(item.subject, revealed);
    return `${fromLabel} → ${toLabel}`;
}

export function recorded(value: string | undefined, fallback: string) {
    const text = (value || '').trim();
    return text || fallback;
}

export function getActionLabel(item: TaskRecord): string {
    if (item.action_label && item.action_label.trim()) {
        return item.action_label;
    }
    const code = item.action_code;
    const action = (item.action || '').trim();
    if (code === 4 || action === 'SchedulePrompt' || action === 'schedule_prompt') {
        return 'Schedule Prompt';
    }
    if (code === 5 || action === 'RequeueConversation' || action === 'requeue_conversation') {
        return 'Requeue Conversation';
    }
    if (code === 3 || action.toLowerCase().includes('switch')) {
        return 'Switch Account';
    }
    if (code === 1 || action.toLowerCase().includes('add')) {
        return 'Add Account';
    }
    if (code === 2 || action.toLowerCase().includes('update')) {
        return 'Update Account';
    }
    return action || '—';
}

export function isSchedulerAction(item: TaskRecord): boolean {
    return (
        item.action_code === 4 ||
        item.action_code === 5 ||
        item.action === 'SchedulePrompt' ||
        item.action === 'RequeueConversation' ||
        item.action === 'schedule_prompt' ||
        item.action === 'requeue_conversation'
    );
}

export function isSwitchAction(item: TaskRecord): boolean {
    return (
        item.action_code === 3 ||
        item.action === 'SwitchAccount' ||
        item.action === 'switch_account' ||
        item.action.toLowerCase().includes('switch')
    );
}

export function renderActionBadge(item: TaskRecord) {
    const label = getActionLabel(item);
    const code = item.action_code;
    const action = item.action || '';

    let badgeClass = 'bg-slate-100 text-slate-700 dark:bg-slate-800/60 dark:text-slate-300 border-slate-200 dark:border-slate-700';

    if (code === 4 || action === 'SchedulePrompt' || action === 'schedule_prompt') {
        badgeClass = 'bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300 border-purple-200/50 dark:border-purple-800/50';
    } else if (code === 5 || action === 'RequeueConversation' || action === 'requeue_conversation') {
        badgeClass = 'bg-cyan-100 text-cyan-700 dark:bg-cyan-900/40 dark:text-cyan-300 border-cyan-200/50 dark:border-cyan-800/50';
    } else if (code === 3 || action.toLowerCase().includes('switch')) {
        badgeClass = 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300 border-amber-200/50 dark:border-amber-800/50';
    } else if (code === 1 || action.toLowerCase().includes('add')) {
        badgeClass = 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300 border-emerald-200/50 dark:border-emerald-800/50';
    } else if (code === 2 || action.toLowerCase().includes('update')) {
        badgeClass = 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300 border-blue-200/50 dark:border-blue-800/50';
    }

    return (
        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${badgeClass}`}>
            {label}
        </span>
    );
}

export interface DetailRow {
    label: string;
    value: React.ReactNode;
    copyValue?: string;
}

export function detailRows(detail: TaskDetail, payload: TaskPayload | null, revealed: boolean): DetailRow[] {
    const when = formatTime(payload?.timestamp || payload?.moved_at || detail.created_at);
    const actionLabel = getActionLabel(detail);

    // Check if detail represents a scheduler event
    const isScheduler =
        detail.action_code === 4 ||
        detail.action_code === 5 ||
        detail.action === 'SchedulePrompt' ||
        detail.action === 'RequeueConversation' ||
        detail.action === 'schedule_prompt' ||
        detail.action === 'requeue_conversation' ||
        Boolean(payload?.scheduler_run_id) ||
        typeof payload?.idle_check_passed === 'boolean';

    if (isScheduler) {
        const actionTaken = payload?.action_taken || detail.status || '—';
        const isActionPositive = /dispatch|push|requeue|ok|success/i.test(actionTaken);
        const idlePassed = payload?.idle_check_passed;
        const idleLabel = typeof idlePassed === 'boolean'
            ? (idlePassed ? 'Idle Verified (Passed)' : 'Busy')
            : 'Not recorded';
        const promptText = payload?.prompt_preview || payload?.prompt_text;
        const reasonText = payload?.reason || detail.detail || '—';
        const convId = payload?.conversation_id || '—';
        const projectName = payload?.project_name || detail.subject || '—';
        const repoPath = payload?.repo_path;

        const rows: DetailRow[] = [
            {
                label: 'When',
                value: when,
                copyValue: when,
            },
            {
                label: 'Action',
                value: renderActionBadge(detail),
                copyValue: actionLabel,
            },
            {
                label: 'Scheduler Run',
                value: payload?.scheduler_run_id ? (
                    <span className="font-mono text-xs font-semibold text-purple-700 dark:text-purple-300">
                        {payload.scheduler_run_id}
                    </span>
                ) : (
                    '—'
                ),
                copyValue: payload?.scheduler_run_id || '—',
            },
            {
                label: 'Project',
                value: (
                    <div>
                        <span className="font-medium text-slate-900 dark:text-slate-100">{projectName}</span>
                        {repoPath && repoPath !== projectName && (
                            <div className="text-[11px] font-mono text-slate-500 dark:text-slate-400 break-all select-all mt-0.5">
                                {repoPath}
                            </div>
                        )}
                    </div>
                ),
                copyValue: repoPath && projectName !== '—' ? `${projectName} (${repoPath})` : projectName,
            },
            {
                label: 'Action Taken',
                value: payload?.action_taken ? (
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${
                        isActionPositive
                            ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950/70 dark:text-emerald-300 border-emerald-300/70 dark:border-emerald-800'
                            : 'bg-blue-100 text-blue-800 dark:bg-blue-950/70 dark:text-blue-300 border-blue-300/70 dark:border-blue-800'
                    }`}>
                        {payload.action_taken}
                    </span>
                ) : (
                    <span className="text-slate-500 dark:text-slate-400">—</span>
                ),
                copyValue: actionTaken,
            },
            {
                label: 'Idle Check',
                value: typeof idlePassed === 'boolean' ? (
                    idlePassed ? (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800 dark:bg-emerald-950/70 dark:text-emerald-300 border border-emerald-300/70 dark:border-emerald-800">
                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                            Idle Verified (Passed)
                        </span>
                    ) : (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-950/70 dark:text-amber-300 border border-amber-300/70 dark:border-amber-800">
                            <span className="w-1.5 h-1.5 rounded-full bg-amber-500"></span>
                            Busy
                        </span>
                    )
                ) : (
                    <span className="text-slate-500 dark:text-slate-400">Not recorded</span>
                ),
                copyValue: idleLabel,
            },
            {
                label: 'Reason / Trail',
                value: <span className="text-slate-800 dark:text-slate-200">{reasonText}</span>,
                copyValue: reasonText,
            },
            {
                label: 'Prompt Preview',
                value: promptText ? (
                    <div className="rounded-lg border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-2.5 max-h-48 overflow-y-auto font-mono text-xs text-slate-800 dark:text-slate-200 select-text whitespace-pre-wrap break-words">
                        {promptText}
                    </div>
                ) : (
                    <span className="text-slate-500 dark:text-slate-400">No prompt preview recorded</span>
                ),
                copyValue: promptText || 'No prompt preview recorded',
            },
            {
                label: 'Conversation ID',
                value: payload?.conversation_id ? (
                    <span className="font-mono text-xs text-slate-700 dark:text-cyan-300 select-all">
                        {payload.conversation_id}
                    </span>
                ) : (
                    <span className="text-slate-500 dark:text-slate-400">Not recorded on this row</span>
                ),
                copyValue: convId,
            },
        ];

        if (detail.instance_id || payload?.instance_id) {
            const inst = payload?.instance_id || detail.instance_id;
            rows.splice(4, 0, {
                label: 'Instance',
                value: <span className="font-mono text-xs text-slate-700 dark:text-slate-300">{inst}</span>,
                copyValue: inst,
            });
        }

        if (payload?.prompt_id) {
            rows.push({
                label: 'Prompt ID',
                value: <span className="font-mono text-xs text-slate-700 dark:text-slate-300 select-all">{payload.prompt_id}</span>,
                copyValue: payload.prompt_id,
            });
        }

        return rows;
    }

    const from = payload?.from_email || detail.from_email || '';
    const to = payload?.to_email || detail.to_email || detail.subject;
    const isSwitch =
        detail.action_code === 3 ||
        (detail.action || '').toLowerCase().includes('switch') ||
        Boolean(payload?.from_email || payload?.to_email || payload?.reason);

    if (!isSwitch) {
        return [
            { label: 'When', value: when, copyValue: when },
            { label: 'Action', value: renderActionBadge(detail), copyValue: actionLabel },
            { label: 'Account', value: subjectText(detail.subject, revealed), copyValue: subjectText(detail.subject, revealed) },
            { label: 'What happened', value: detail.detail || '—', copyValue: detail.detail || '—' },
        ];
    }

    const reinject = payload && typeof payload.prompt_reinjected === 'boolean'
        ? (payload.prompt_reinjected ? 'Yes' : 'No')
        : 'Not recorded on this row';

    const rows: DetailRow[] = [
        { label: 'From', value: subjectText(from, revealed), copyValue: subjectText(from, revealed) },
        { label: 'To', value: subjectText(to, revealed), copyValue: subjectText(to, revealed) },
        { label: 'When', value: when, copyValue: when },
        { label: 'Action', value: renderActionBadge(detail), copyValue: actionLabel },
    ];

    const instId = payload?.instance_id || detail.instance_id;
    if (instId) {
        rows.push({
            label: 'Instance ID',
            value: <span className="font-mono text-xs text-slate-800 dark:text-cyan-300 select-all">{instId}</span>,
            copyValue: instId,
        });
    }

    if (payload?.ide_type) {
        rows.push({
            label: 'IDE Type',
            value: <span className="font-medium text-xs text-slate-800 dark:text-slate-200">{payload.ide_type}</span>,
            copyValue: payload.ide_type,
        });
    }

    if (payload?.idc_machine_alias) {
        rows.push({
            label: 'IDC Machine Alias',
            value: <span className="font-medium text-xs text-slate-800 dark:text-slate-200">{payload.idc_machine_alias}</span>,
            copyValue: payload.idc_machine_alias,
        });
    }

    if (payload?.ide_path) {
        rows.push({
            label: 'IDE Path',
            value: <span className="font-mono text-xs text-slate-600 dark:text-slate-400 break-all select-all">{payload.ide_path}</span>,
            copyValue: payload.ide_path,
        });
    }

    const switchReason = payload?.switch_reason || payload?.reason || detail.detail;
    if (switchReason) {
        rows.push({
            label: 'Switch Reason',
            value: <span className="text-slate-800 dark:text-slate-200">{switchReason}</span>,
            copyValue: switchReason,
        });
    }

    if (payload?.how) {
        rows.push({
            label: 'How',
            value: recorded(payload.how, 'Not recorded on this row'),
            copyValue: recorded(payload.how, 'Not recorded on this row'),
        });
    }

    if (payload?.conversation_id) {
        rows.push({
            label: 'Conversation',
            value: recorded(payload.conversation_id, 'Not recorded on this row'),
            copyValue: recorded(payload.conversation_id, 'Not recorded on this row'),
        });
    }

    if (payload?.prompt_text) {
        rows.push({
            label: 'Prompt running',
            value: recorded(payload.prompt_text, 'No running prompt was stored'),
            copyValue: recorded(payload.prompt_text, 'No running prompt was stored'),
        });
    }

    rows.push({
        label: 'Prompt injected again',
        value: reinject,
        copyValue: reinject,
    });

    return rows;
}

export default function Audit() {
    const [page, setPage] = useState(0);
    const [pageSize, setPageSize] = useState<number>(() => {
        const saved = localStorage.getItem('audit_page_size');
        return saved === '200' ? 200 : 100;
    });
    const [filter, setFilter] = useState<AuditFilterType>('all');
    const [data, setData] = useState<TaskHistoryPage | null>(null);
    const [error, setError] = useState('');
    const [revealed, setRevealed] = useState<Record<string, boolean>>({});
    const [openId, setOpenId] = useState('');
    const [detail, setDetail] = useState<TaskDetail | null>(null);
    const [detailError, setDetailError] = useState('');
    const [copied, setCopied] = useState(false);

    const handlePageSizeChange = (newSize: number) => {
        setPageSize(newSize);
        localStorage.setItem('audit_page_size', String(newSize));
        setPage(0);
    };

    useEffect(() => {
        let alive = true;
        const fetchHistory = async () => {
            try {
                if (pageSize === 200) {
                    const [p1, p2] = await Promise.all([
                        request<TaskHistoryPage>('list_task_history', { offset: page * 200, limit: 100 }),
                        request<TaskHistoryPage>('list_task_history', { offset: page * 200 + 100, limit: 100 }).catch(() => null),
                    ]);
                    if (alive) {
                        const items = p2 ? [...p1.items, ...p2.items] : p1.items;
                        setData({
                            total: p1.total,
                            offset: page * 200,
                            limit: 200,
                            items,
                            splits: p1.splits,
                        });
                        setError('');
                    }
                } else {
                    const res = await request<TaskHistoryPage>('list_task_history', { offset: page * 100, limit: 100 });
                    if (alive) {
                        setData(res);
                        setError('');
                    }
                }
            } catch (err: unknown) {
                if (alive) setError(err instanceof Error ? err.message : String(err));
            }
        };
        fetchHistory();
        return () => {
            alive = false;
        };
    }, [page, pageSize]);

    const loadDetail = (id: string) => {
        setOpenId(id);
        setDetail(null);
        setDetailError('');
        setCopied(false);
        request<TaskDetail>('get_task_history_detail', { id })
            .then((result) => setDetail(result))
            .catch((err: unknown) => setDetailError(err instanceof Error ? err.message : String(err)));
    };

    const allItems = useMemo(() => data?.items || [], [data?.items]);

    const { switchItems, schedulerItems } = useMemo(() => {
        const sw: TaskRecord[] = [];
        const sc: TaskRecord[] = [];
        for (const item of allItems) {
            if (isSwitchAction(item)) sw.push(item);
            if (isSchedulerAction(item)) sc.push(item);
        }
        return { switchItems: sw, schedulerItems: sc };
    }, [allItems]);

    const filterCounts = useMemo(() => ({
        all: allItems.length,
        switch: switchItems.length,
        scheduler: schedulerItems.length,
    }), [allItems.length, switchItems.length, schedulerItems.length]);

    const displayItems = useMemo(() => {
        if (filter === 'switch') return switchItems;
        if (filter === 'scheduler') return schedulerItems;
        return allItems;
    }, [filter, switchItems, schedulerItems, allItems]);

    const total = data?.total ?? 0;
    const pageCount = Math.max(1, Math.ceil(total / pageSize));

    let payload: TaskPayload | null = null;
    if (detail?.payload_json) {
        try {
            payload = JSON.parse(detail.payload_json) as TaskPayload;
        } catch {
            payload = null;
        }
    }
    const detailRevealed = detail ? Boolean(revealed[detail.id]) : false;

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

            {/* Audit Table */}
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
                                        <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium border ${
                                            /dispatch|ok|success|complete/i.test(item.status)
                                                ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300 border-emerald-200/60 dark:border-emerald-800/60'
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
                                            className="inline-flex h-7 w-7 items-center justify-center rounded-full border border-slate-300 dark:border-[#15334d] bg-white dark:bg-[#071a27] text-slate-800 dark:text-gray-200 hover:bg-slate-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
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

            {/* Audit Detail Modal Dialog */}
            {openId && (
                <div
                    className="fixed inset-0 z-[80] flex items-center justify-center bg-black/50 p-4"
                    onClick={() => { setOpenId(''); setDetail(null); }}
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
                                    onClick={() => { setOpenId(''); setDetail(null); }}
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
            )}

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
