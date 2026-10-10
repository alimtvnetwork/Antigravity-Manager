export function maskEmailAddress(email: string): string {
    if (!email || !email.includes('@')) {
        return email || '—';
    }
    const [local, domain] = email.split('@');
    if (!local || local.length <= 2) {
        return `${local ? local[0] : '*'}***@${domain || ''}`;
    }
    const firstChar = local[0];
    const lastChar = local[local.length - 1];
    return `${firstChar}***${lastChar}@${domain}`;
}

export function formatRelativeHeartbeat(timestamp?: number | null): string {
    if (!timestamp || timestamp <= 0) {
        return 'Never';
    }
    const ms = timestamp < 10000000000 ? timestamp * 1000 : timestamp;
    const diffSec = Math.max(0, Math.floor((Date.now() - ms) / 1000));
    if (diffSec < 5) return 'Just now';
    if (diffSec < 60) return `${diffSec}s ago`;
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHour = Math.floor(diffMin / 60);
    if (diffHour < 24) return `${diffHour}h ago`;
    const diffDay = Math.floor(diffHour / 24);
    return `${diffDay}d ago`;
}

export function toDisplayTimestamp(timestamp: number): number {
    return timestamp < 10000000000 ? timestamp * 1000 : timestamp;
}
