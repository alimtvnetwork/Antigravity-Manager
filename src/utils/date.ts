/**
 * Polite Date & Time formatting utility
 * Standard format: DD-MMM-YY - hh:mm A (e.g., 22-Sep-26 - 10:29 AM)
 */

const MONTH_NAMES = [
    'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
    'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
];

/**
 * Formats a Unix timestamp (seconds or milliseconds) or Date into polite string: DD-MMM-YY - hh:mm A
 */
export function formatDateTime(value?: number | string | Date | null): string {
    if (value === undefined || value === null) return '-';

    let ms: number;
    if (typeof value === 'number') {
        if (value <= 0) return '-';
        ms = value < 10000000000 ? value * 1000 : value;
    } else if (typeof value === 'string') {
        const parsed = Date.parse(value);
        if (isNaN(parsed)) return '-';
        if (parsed <= 0) return '-';
        ms = parsed;
    } else if (value instanceof Date) {
        ms = value.getTime();
        if (isNaN(ms)) return '-';
        if (ms <= 0) return '-';
    } else {
        return '-';
    }

    const d = new Date(ms);
    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'Jan';
    const year = String(d.getFullYear()).slice(-2);
    const time = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

    return `${day}-${month}-${year} - ${time}`;
}

/**
 * Formats just the date part: DD-MMM-YY
 */
export function formatDateOnly(value?: number | string | Date | null): string {
    if (value === undefined || value === null) return '-';

    let ms: number;
    if (typeof value === 'number') {
        if (value <= 0) return '-';
        ms = value < 10000000000 ? value * 1000 : value;
    } else if (typeof value === 'string') {
        const parsed = Date.parse(value);
        if (isNaN(parsed)) return '-';
        if (parsed <= 0) return '-';
        ms = parsed;
    } else if (value instanceof Date) {
        ms = value.getTime();
        if (isNaN(ms)) return '-';
        if (ms <= 0) return '-';
    } else {
        return '-';
    }

    const d = new Date(ms);
    const day = String(d.getDate()).padStart(2, '0');
    const month = MONTH_NAMES[d.getMonth()] || 'Jan';
    const year = String(d.getFullYear()).slice(-2);

    return `${day}-${month}-${year}`;
}
