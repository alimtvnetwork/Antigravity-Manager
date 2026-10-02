function hashLocal(local: string): number {
    let hash = 0;
    for (let index = 0; index < local.length; index += 1) {
        hash = (hash * 33 + local.charCodeAt(index)) >>> 0;
    }
    return hash;
}

export function maskEmail(email: string): string {
    const local = email.split('@')[0] || email;
    if (local.length <= 1) {
        return '***';
    }
    const midpoint = Math.max(1, Math.floor(local.length / 2));
    const hideSecondHalf = hashLocal(local) % 2 === 0;
    if (hideSecondHalf) {
        return `${local.slice(0, midpoint)}***`;
    }
    return `***${local.slice(midpoint)}`;
}
