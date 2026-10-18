/**
 * Centralized Datetime & Duration Formatters
 *
 * Requirements:
 * - Standard 24-hour clock formatting: DD-MMM-YYYY HH:mm (e.g. 18-Oct-2026 21:40)
 *   based on the local Windows/system timezone.
 * - Meaningless seconds omitted when durations/ranges are large (hours/days).
 * - Human-readable relative time badges: "ran 5 minutes ago", "ran 1 day ago", "just now".
 * - Configurable via localStorage ('agm_time_format_preference') while backend
 *   enforces UTC0 storage invariant.
 */

const MONTH_NAMES = [
    'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
    'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
];

export interface FormatDateTimeOptions {
    includeSeconds?: boolean;
    useUtc?: boolean;
}

/**
 * Normalizes any timestamp representation (ISO string, unix epoch in sec/ms, Date) to a Date object.
 */
export function parseTimestampToDate(input?: string | number | Date | null): Date | null {
    if (!input) return null;
    if (input instanceof Date) return isNaN(input.getTime()) ? null : input;

    if (typeof input === 'number') {
        const ms = input < 1e11 ? input * 1000 : input;
        const d = new Date(ms);
        return isNaN(d.getTime()) ? null : d;
    }

    const trimmed = String(input).trim();
    if (/^\d+$/.test(trimmed)) {
        const num = Number(trimmed);
        const ms = num < 1e11 ? num * 1000 : num;
        const d = new Date(ms);
        return isNaN(d.getTime()) ? null : d;
    }

    const parsed = new Date(trimmed);
    return isNaN(parsed.getTime()) ? null : parsed;
}

/**
 * Formats date into DD-MMM-YYYY HH:mm (24-hour clock) according to local system timezone.
 * Example: "18-Oct-2026 21:40" or "18-Oct-2026 21:40:15" if includeSeconds is true.
 */
export function formatDateTime24h(
    input?: string | number | Date | null,
    options?: FormatDateTimeOptions
): string {
    const d = parseTimestampToDate(input);
    if (!d) return '—';

    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'Jan';
    const year = d.getFullYear();
    const hours = String(d.getHours()).padStart(2, '0');
    const mins = String(d.getMinutes()).padStart(2, '0');

    if (options?.includeSeconds) {
        const secs = String(d.getSeconds()).padStart(2, '0');
        return `${day}-${month}-${year} ${hours}:${mins}:${secs}`;
    }

    return `${day}-${month}-${year} ${hours}:${mins}`;
}

/**
 * Produces a human-friendly relative time label, e.g. "ran 5 minutes ago", "ran 1 day ago", "just now".
 */
export function getRelativeTimeBadge(input?: string | number | Date | null): string {
    const d = parseTimestampToDate(input);
    if (!d) return '';

    const diffMs = Date.now() - d.getTime();
    if (diffMs < 0 && Math.abs(diffMs) < 60000) {
        return 'running now';
    }

    const diffSeconds = Math.max(0, Math.floor(diffMs / 1000));
    if (diffSeconds < 45) {
        return 'just now';
    }
    if (diffSeconds < 90) {
        return 'ran 1 minute ago';
    }
    if (diffSeconds < 3600) {
        const mins = Math.floor(diffSeconds / 60);
        return `ran ${mins} minutes ago`;
    }
    if (diffSeconds < 7200) {
        return 'ran 1 hour ago';
    }
    if (diffSeconds < 86400) {
        const hrs = Math.floor(diffSeconds / 3600);
        return `ran ${hrs} hours ago`;
    }
    if (diffSeconds < 172800) {
        return 'ran 1 day ago';
    }
    const days = Math.floor(diffSeconds / 86400);
    return `ran ${days} days ago`;
}

/**
 * Human-friendly duration string.
 * Omits meaningless seconds when hours or large minutes are present.
 * Example: 24s, 5m, 14m 20s, 67h 33m.
 */
export function formatDurationHuman(diffSeconds: number): string {
    if (!Number.isFinite(diffSeconds) || diffSeconds < 0) return '0s';
    if (diffSeconds < 60) return `${diffSeconds}s`;

    const mins = Math.floor(diffSeconds / 60);
    const secs = diffSeconds % 60;

    if (diffSeconds < 600) {
        // Less than 10 mins: show minutes + seconds
        return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`;
    }

    if (diffSeconds < 3600) {
        // 10 to 60 mins: omit seconds
        return `${mins}m`;
    }

    const hrs = Math.floor(diffSeconds / 3600);
    const remMins = Math.floor((diffSeconds % 3600) / 60);
    return remMins > 0 ? `${hrs}h ${remMins}m` : `${hrs}h`;
}
