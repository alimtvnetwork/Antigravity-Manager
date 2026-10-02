/** Hide the domain and keep the start of the name so two accounts stay distinct. */
export function maskEmail(email: string): string {
    const local = (email.split('@')[0] || email).trim();
    if (!local) return '***';
    if (local.length <= 2) return `${local[0]}***`;
    const shown = Math.min(4, local.length - 1);
    return `${local.slice(0, shown)}***`;
}
