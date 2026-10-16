/**
 * Strict privacy mask for user email addresses.
 * Preserves initial distinguishing characters while masking the remainder of the local part.
 * Example: `riseup.asia.team@gmail.com` -> `ris***@gmail.com`
 */
export function maskEmail(email: string): string {
    const trimmed = (email || '').trim();
    if (!trimmed) return '***';
    const parts = trimmed.split('@');
    const local = parts[0] || '';
    const domain = parts[1];

    const shown = local.length <= 2
        ? Math.min(1, local.length)
        : Math.min(3, Math.max(1, local.length - 2));

    const maskedLocal = `${local.slice(0, shown)}***`;

    if (!domain) {
        return maskedLocal;
    }

    return `${maskedLocal}@${domain}`;
}
