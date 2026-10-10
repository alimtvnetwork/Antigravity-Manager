import { maskEmail } from '../../utils/maskEmail';
import type {
    TaskRecord,
    TaskDetail,
    TaskPayload,
} from '../../types/audit';

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
        badgeClass = 'bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300 border-cyan-200/50 dark:border-cyan-800/50';
    } else if (code === 2 || action.toLowerCase().includes('update')) {
        badgeClass = 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300 border-blue-200/50 dark:border-blue-800/50';
    }

    return (
        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-[5px] text-xs font-medium border ${badgeClass}`}>
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
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-[5px] text-xs font-medium border ${
                        isActionPositive
                            ? 'bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300 border-cyan-200/50 dark:border-cyan-800/50'
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
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-[5px] text-xs font-medium bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300 border border-cyan-200/50 dark:border-cyan-800/50">
                            <span className="w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse"></span>
                            Idle Verified (Passed)
                        </span>
                    ) : (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-[5px] text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-950/70 dark:text-amber-300 border border-amber-300/70 dark:border-amber-800">
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
