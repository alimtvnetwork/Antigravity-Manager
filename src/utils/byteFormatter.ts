/**
 * Byte Size Formatter Utility.
 * Formats raw byte counts into human-readable strings (`12.4 KB`, `1.8 MB`, `2.4 GB`).
 */

export function formatByteSize(bytes: number, decimals: number = 1): string {
    if (bytes <= 0 || isNaN(bytes)) {
        return '0 B';
    }

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const index = Math.min(i, sizes.length - 1);

    const val = bytes / Math.pow(k, index);
    return `${val.toFixed(dm)} ${sizes[index]}`;
}

/**
 * Parses a byte size string with units (e.g. "16MB", "1.5 GB") into raw byte count.
 */
export function parseByteSizeString(input: string): number {
    const trimmed = input.trim().toUpperCase();
    const match = trimmed.match(/^([\d.]+)\s*([KMGT]?B?)$/);
    if (!match) return 0;

    const val = parseFloat(match[1]);
    if (isNaN(val)) return 0;

    const unit = match[2];
    if (unit.startsWith('K')) return Math.round(val * 1024);
    if (unit.startsWith('M')) return Math.round(val * 1024 * 1024);
    if (unit.startsWith('G')) return Math.round(val * 1024 * 1024 * 1024);
    if (unit.startsWith('T')) return Math.round(val * 1024 * 1024 * 1024 * 1024);

    return Math.round(val);
}
