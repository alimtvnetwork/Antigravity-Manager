/**
 * Time-Zone Normalized Clocks & UTC Formatting Utilities
 *
 * Provides normalized multi-format timestamps for local system timezone,
 * UTC, and Unix epoch seconds.
 */

export interface NormalizedResetTimeResult {
    localFormatted: string;
    utcFormatted: string;
    epochSecs: number;
}

const localDateTimeFormatter = new Intl.DateTimeFormat(undefined, {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
});

const utcDateTimeFormatter = new Intl.DateTimeFormat('en-GB', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
    timeZone: 'UTC',
    timeZoneName: 'short',
});

function parseNumericTime(num: number): Date | null {
    const ms = num < 1e11 ? num * 1000 : num;
    const resolved = new Date(ms);
    return isNaN(resolved.getTime()) ? null : resolved;
}

export function parseDateInput(input: string | number | Date | null | undefined): Date | null {
    if (!input) return null;
    if (input instanceof Date) return isNaN(input.getTime()) ? null : input;
    if (typeof input === 'number') return parseNumericTime(input);
    const trimmed = String(input).trim();
    if (/^\d+$/.test(trimmed)) return parseNumericTime(Number(trimmed));
    const parsed = new Date(trimmed);
    return isNaN(parsed.getTime()) ? null : parsed;
}

export function formatToLocal(date: Date): string {
    try {
        return localDateTimeFormatter.format(date);
    } catch {
        return date.toLocaleString();
    }
}

export function formatToUtc(date: Date): string {
    try {
        return utcDateTimeFormatter.format(date);
    } catch {
        return date.toUTCString();
    }
}

export function getRemainingSeconds(targetTime: string | number | Date | null | undefined): number {
    const parsedDate = parseDateInput(targetTime);
    if (!parsedDate) return 0;
    const diffMs = parsedDate.getTime() - Date.now();
    return Math.floor(diffMs / 1000);
}

export function formatNormalizedResetTime(
    resetTime: string | number | Date | null | undefined
): NormalizedResetTimeResult {
    const parsedDate = parseDateInput(resetTime);
    if (!parsedDate) {
        return {
            localFormatted: '—',
            utcFormatted: '—',
            epochSecs: 0,
        };
    }

    return {
        localFormatted: formatToLocal(parsedDate),
        utcFormatted: formatToUtc(parsedDate),
        epochSecs: Math.floor(parsedDate.getTime() / 1000),
    };
}
