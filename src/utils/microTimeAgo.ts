/**
 * Ultra-compact relative time formatter.
 * Produces minimalist timestamp strings (`now`, `12s`, `4m`, `2h`, `5d`, `1y`)
 * optimized for dense tables, status pills, and card metadata ribbons.
 */
export function formatMicroTimeAgo(
  input: Date | number | string | null | undefined,
  nowMs: number = Date.now()
): string {
  if (input === null || input === undefined) {
    return '-';
  }

  let timestampMs: number;
  if (typeof input === 'number') {
    // If under 10 billion, assume seconds
    timestampMs = input < 10_000_000_000 ? input * 1000 : input;
  } else if (input instanceof Date) {
    timestampMs = input.getTime();
  } else {
    const parsed = Date.parse(input);
    if (isNaN(parsed)) return '-';
    timestampMs = parsed;
  }

  const diffSec = Math.floor((nowMs - timestampMs) / 1000);

  if (diffSec < 0) {
    // Future timestamp
    const futureSec = Math.abs(diffSec);
    if (futureSec < 60) return `in ${futureSec}s`;
    if (futureSec < 3600) return `in ${Math.floor(futureSec / 60)}m`;
    if (futureSec < 86400) return `in ${Math.floor(futureSec / 3600)}h`;
    return `in ${Math.floor(futureSec / 86400)}d`;
  }

  if (diffSec < 5) return 'now';
  if (diffSec < 60) return `${diffSec}s`;

  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m`;

  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 30) return `${diffDays}d`;

  const diffMonths = Math.floor(diffDays / 30);
  if (diffMonths < 12) return `${diffMonths}mo`;

  const diffYears = Math.floor(diffDays / 365);
  return `${diffYears}y`;
}
