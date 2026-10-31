/**
 * Human-Readable Duration Formatter (Imp-539)
 * Formats duration milliseconds into compact, standardized string representations.
 */

export function formatCompactDuration(ms: number | null | undefined): string {
  if (ms === null || ms === undefined || isNaN(ms) || ms < 0) {
    return '0ms';
  }

  if (ms < 1000) {
    return `${Math.round(ms)}ms`;
  }

  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }

  const minutes = Math.floor(seconds / 60);
  const remSeconds = Math.round(seconds % 60);
  if (minutes < 60) {
    return remSeconds > 0 ? `${minutes}m ${remSeconds}s` : `${minutes}m`;
  }

  const hours = Math.floor(minutes / 60);
  const remMinutes = minutes % 60;
  return remMinutes > 0 ? `${hours}h ${remMinutes}m` : `${hours}h`;
}

export function formatSecondsToHuman(seconds: number | null | undefined): string {
  if (!seconds || seconds <= 0) return '0s';
  return formatCompactDuration(seconds * 1000);
}
