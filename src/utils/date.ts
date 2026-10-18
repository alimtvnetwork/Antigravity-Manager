/**
 * Polite Date & Time formatting utility
 * Standard format: DD-MMM-YYYY HH:mm (e.g., 18-Oct-2026 21:40) in 24-hour clock
 */

export type DateFormatMode = '24h' | '12h' | 'iso';

const MONTH_NAMES = [
    'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
    'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
];

/**
 * Normalizes input value to timestamp ms
 */
function parseToMs(value?: number | string | Date | null): number | null {
    if (value === undefined || value === null) return null;

    let ms: number;
    if (typeof value === 'number') {
        if (!Number.isFinite(value) || value <= 0) return null;
        ms = value < 10000000000 ? value * 1000 : value;
    } else if (typeof value === 'string') {
        const trimmed = value.trim();
        if (!trimmed) return null;
        const parsed = Date.parse(trimmed);
        if (isNaN(parsed) || parsed <= 0) return null;
        ms = parsed;
    } else if (value instanceof Date) {
        ms = value.getTime();
        if (isNaN(ms) || ms <= 0) return null;
    } else {
        return null;
    }

    return ms;
}

/**
 * Formats a Unix timestamp (seconds or milliseconds) or Date into polite string.
 * Default format: DD-MMM-YYYY HH:mm (24-hour clock, e.g., 18-Oct-2026 21:40)
 */
export function formatDateTime(
    value?: number | string | Date | null,
    formatMode: DateFormatMode = '24h'
): string {
    const ms = parseToMs(value);
    if (ms === null) return '-';

    const d = new Date(ms);
    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'Jan';
    const year = d.getFullYear();

    if (formatMode === 'iso') {
        const monthNum = String(d.getMonth() + 1).padStart(2, '0');
        const dayNum = day;
        const hours = String(d.getHours()).padStart(2, '0');
        const minutes = String(d.getMinutes()).padStart(2, '0');
        const seconds = String(d.getSeconds()).padStart(2, '0');
        return `${year}-${monthNum}-${dayNum} ${hours}:${minutes}:${seconds}`;
    }

    if (formatMode === '12h') {
        let hours = d.getHours();
        const ampm = hours >= 12 ? 'PM' : 'AM';
        hours = hours % 12;
        hours = hours ? hours : 12;
        const hoursStr = String(hours).padStart(2, '0');
        const minutesStr = String(d.getMinutes()).padStart(2, '0');
        return `${day}-${month}-${year} ${hoursStr}:${minutesStr} ${ampm}`;
    }

    // Default '24h' mode: DD-MMM-YYYY HH:mm
    const hours = String(d.getHours()).padStart(2, '0');
    const minutes = String(d.getMinutes()).padStart(2, '0');
    return `${day}-${month}-${year} ${hours}:${minutes}`;
}

/**
 * Formats just the date part: DD-MMM-YYYY (e.g. 18-Oct-2026)
 */
export function formatDateOnly(value?: number | string | Date | null): string {
    const ms = parseToMs(value);
    if (ms === null) return '-';

    const d = new Date(ms);
    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'Jan';
    const year = d.getFullYear();

    return `${day}-${month}-${year}`;
}

/**
 * Formats relative recency delta (e.g., "just now", "5 minutes ago", "1 hour ago", "1 day ago")
 */
export function formatRelativeTime(
    value?: number | string | Date | null,
    options?: { prefixRan?: boolean }
): string {
    const ms = parseToMs(value);
    if (ms === null) return '-';

    const deltaMs = Date.now() - ms;
    const prefix = options?.prefixRan ? 'ran ' : '';

    if (deltaMs < 45_000) {
        return `${prefix}just now`;
    }
    if (deltaMs < 90_000) {
        return `${prefix}1 minute ago`;
    }
    if (deltaMs < 45 * 60_000) {
        const mins = Math.max(1, Math.round(deltaMs / 60_000));
        return `${prefix}${mins} minutes ago`;
    }
    if (deltaMs < 90 * 60_000) {
        return `${prefix}1 hour ago`;
    }
    if (deltaMs < 24 * 3600_000) {
        const hrs = Math.max(1, Math.round(deltaMs / 3600_000));
        return `${prefix}${hrs} hours ago`;
    }
    if (deltaMs < 48 * 3600_000) {
        return `${prefix}1 day ago`;
    }
    if (deltaMs < 30 * 86400_000) {
        const days = Math.max(1, Math.round(deltaMs / 86400_000));
        return `${prefix}${days} days ago`;
    }

    const months = Math.max(1, Math.round(deltaMs / (30 * 86400_000)));
    return `${prefix}${months} months ago`;
}

/**
 * Formats duration with second suppression when hours or minutes exist.
 * - < 60s: `${secs}s`
 * - 60s <= secs < 3600s: `${mins}m` (seconds omitted)
 * - >= 3600s: `${hrs}h ${mins}m` (or `${hrs}h` if mins == 0, seconds omitted)
 */
export function formatDuration(secs: number): string {
    if (!Number.isFinite(secs) || secs <= 0) return '0s';

    const roundedSecs = Math.floor(secs);
    if (roundedSecs < 60) {
        return `${roundedSecs}s`;
    }

    if (roundedSecs < 3600) {
        const mins = Math.floor(roundedSecs / 60);
        return `${mins}m`;
    }

    const hrs = Math.floor(roundedSecs / 3600);
    const mins = Math.floor((roundedSecs % 3600) / 60);
    if (mins === 0) {
        return `${hrs}h`;
    }
    return `${hrs}h ${mins}m`;
}

/**
 * Alias for formatDuration with smart second suppression
 */
export const formatSmartDuration = formatDuration;
