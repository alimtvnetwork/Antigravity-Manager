/**
 * Strict privacy mask for user email addresses.
 * Displays 4 characters from the email local part first,
 * and hides `@gmail` / `@googlemail` domains completely.
 *
 * Examples:
 *  - `david.smith@gmail.com` -> `davi***`
 *  - `kevin@gmail.com` -> `kevi***`
 *  - `dan@gmail.com` -> `dan***`
 *  - `ab@gmail.com` -> `ab***`
 *  - `x@gmail.com` -> `x***`
 *  - `worker.alpha@example.com` -> `work***@example.com`
 *  - `not-an-email` -> `not-an-email`
 *  - `""` -> `—`
 */
export function maskEmail(email: string): string {
    const trimmed = (email || '').trim();
    if (!trimmed) return '—';

    if (!trimmed.includes('@')) {
        return trimmed;
    }

    const [local, domain] = trimmed.split('@');
    const localPart = local || '';
    const shown = Math.min(4, Math.max(1, localPart.length));
    const maskedLocal = `${localPart.slice(0, shown)}***`;

    if (!domain) {
        return maskedLocal;
    }

    const lowerDomain = domain.toLowerCase();
    // Hide @gmail.com and @googlemail.com completely as requested
    if (lowerDomain.includes('gmail') || lowerDomain.includes('googlemail')) {
        return maskedLocal;
    }

    return `${maskedLocal}@${domain}`;
}
