/**
 * durationFormatter - Formats duration values (milliseconds) into concise, readable strings.
 */

export interface DurationFormatOptions {
    compact?: boolean;
    decimals?: number;
    includeMs?: boolean;
}

/**
 * Formats a duration in milliseconds to a human-readable string.
 *
 * Examples:
 * - formatDuration(350) => "350ms"
 * - formatDuration(4250) => "4.3s"
 * - formatDuration(75000) => "1m 15s"
 * - formatDuration(75000, { compact: true }) => "1m"
 * - formatDuration(7200000) => "2h 0m"
 * - formatDuration(90000000) => "1d 1h"
 */
export function formatDuration(ms: number, options: DurationFormatOptions = {}): string {
    if (!Number.isFinite(ms) || ms < 0) {
        return '0ms';
    }

    const {
        compact = false,
        decimals = 1,
        includeMs = true,
    } = options;

    if (ms < 1000) {
        return includeMs ? `${Math.round(ms)}ms` : '<1s';
    }

    const totalSeconds = ms / 1000;
    if (totalSeconds < 60) {
        const factor = Math.pow(10, decimals);
        const rounded = Math.round(totalSeconds * factor) / factor;
        return `${rounded}s`;
    }

    const totalMinutes = Math.floor(totalSeconds / 60);
    const remSeconds = Math.round(totalSeconds % 60);

    if (totalMinutes < 60) {
        if (compact || remSeconds === 0) {
            return `${totalMinutes}m`;
        }
        return `${totalMinutes}m ${remSeconds}s`;
    }

    const totalHours = Math.floor(totalMinutes / 60);
    const remMinutes = totalMinutes % 60;

    if (totalHours < 24) {
        if (compact || remMinutes === 0) {
            return `${totalHours}h`;
        }
        return `${totalHours}h ${remMinutes}m`;
    }

    const totalDays = Math.floor(totalHours / 24);
    const remHours = totalHours % 24;

    if (compact || remHours === 0) {
        return `${totalDays}d`;
    }
    return `${totalDays}d ${remHours}h`;
}

export default formatDuration;
