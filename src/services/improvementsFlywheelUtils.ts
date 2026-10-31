export function resolveImprovementsInstanceId(activeId?: string | null): string {
    const trimmed = (activeId ?? '').trim();
    if (!trimmed || trimmed === '__default__') {
        return 'default';
    }
    return trimmed;
}

export function workspaceLabel(root?: string): string {
    const trimmed = (root ?? '').trim();
    if (!trimmed) {
        return 'active instance workspace';
    }
    const parts = trimmed.replace(/\\/g, '/').split('/').filter(Boolean);
    return parts.slice(-2).join('/') || trimmed;
}

const TASK_HINT_CAP = 16;
const SKIP_STEMS = new Set(['readme', 'index', 'changelog', 'license']);
const SKIP_DIRS = new Set(['completed', 'archived', 'closed', 'changelog']);

function skipStem(stem: string): boolean {
    return SKIP_STEMS.has(stem.toLowerCase());
}

function skipDir(name: string): boolean {
    return SKIP_DIRS.has(name.toLowerCase());
}

/** Mirrors `collect_workspace_task_hints` so Next breakdown lists any pending tasks. */
export function collectWorkspaceTaskHints(
    files: string[],
    cap: number = TASK_HINT_CAP,
): string[] {
    const buckets: Record<string, string[]> = {
        pending: [],
        issues: [],
        plans: [],
        questions: [],
        subtasks: [],
    };
    const seen = new Set<string>();
    const normalized = files
        .map((raw) => raw.replace(/\\/g, '/'))
        .sort((a, b) => a.localeCompare(b));
    for (const path of normalized) {
        if (!path.toLowerCase().endsWith('.md')) {
            continue;
        }
        const parts = path.split('/').filter(Boolean);
        const file = parts[parts.length - 1] ?? '';
        const stem = file.replace(/\.md$/i, '');
        if (skipStem(stem) || parts.some((part) => skipDir(part))) {
            continue;
        }
        const joined = parts.join('/').toLowerCase();
        let bucket: keyof typeof buckets | null = null;
        if (joined.includes('.ai-memory/plans/pending/')) {
            bucket = 'pending';
        } else if (joined.includes('.ai-memory/issues/')) {
            bucket = 'issues';
        } else if (joined.includes('.ai-memory/ambiguous-questions/')) {
            bucket = 'questions';
        } else if (joined.includes('.ai-memory/plans/subtasks/')) {
            bucket = 'subtasks';
        } else if (joined.includes('.ai-memory/plans/') && parts[parts.length - 2] === 'plans') {
            bucket = 'plans';
        }
        if (!bucket) {
            continue;
        }
        const key = stem.toLowerCase();
        if (seen.has(key)) {
            continue;
        }
        seen.add(key);
        buckets[bucket].push(stem);
    }
    return [
        ...buckets.pending,
        ...buckets.issues,
        ...buckets.plans,
        ...buckets.questions,
        ...buckets.subtasks,
    ].slice(0, cap);
}

export function flywheelActionToast(
    action: 'plan' | 'next' | 'check' | 'detail',
    dryRun: boolean,
    next: { next_batch: number; completed_batches: number; task_hints?: string[] },
): string {
    const taskN = next.task_hints?.length ?? 0;
    const taskLabel = taskN === 1 ? '1 pending task' : `${taskN} pending tasks`;
    if (action === 'plan') {
        return dryRun
            ? 'Planned 10 batches of 10 (dry-run, no prompt dispatched)'
            : 'Planned 100 improvements and queued the discovery prompt';
    }
    if (action === 'next') {
        return dryRun
            ? `Next batch ${next.next_batch} previewed (${taskLabel})`
            : `Dispatched next 10 (batch ${Math.max(1, next.completed_batches)})`;
    }
    if (action === 'detail') {
        return dryRun
            ? `Detail-out previewed (${taskLabel} in this workspace)`
            : `Queued 100-step breakdown for ${taskLabel} in the active workspace`;
    }
    return 'Ledger refreshed';
}
