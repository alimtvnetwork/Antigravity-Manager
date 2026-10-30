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

export function flywheelActionToast(
    action: 'plan' | 'next' | 'check' | 'detail',
    dryRun: boolean,
    next: { next_batch: number; completed_batches: number },
): string {
    if (action === 'plan') {
        return dryRun
            ? 'Planned 10 batches of 10 (dry-run, no prompt dispatched)'
            : 'Planned 100 improvements and queued the discovery prompt';
    }
    if (action === 'next') {
        return dryRun
            ? `Next batch ${next.next_batch} previewed (dry-run)`
            : `Dispatched next 10 (batch ${Math.max(1, next.completed_batches)})`;
    }
    if (action === 'detail') {
        return dryRun
            ? 'Detail-out previewed (dry-run, no prompt dispatched)'
            : 'Queued 100-step breakdown for the active workspace';
    }
    return 'Ledger refreshed';
}
