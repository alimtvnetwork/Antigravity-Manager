/**
 * High-Precision Duration Formatter Utility
 *
 * Converts milliseconds or seconds into concise, human-readable duration strings
 * (e.g. 850ms, 14s, 3m 20s, 2h 15m, 1d 4h).
 */

export function formatDurationMs(ms: number): string {
  if (ms < 1000) {
    return `${Math.round(ms)}ms`;
  }

  const totalSeconds = Math.floor(ms / 1000);
  return formatDurationSeconds(totalSeconds);
}

export function formatDurationSeconds(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }

  const minutes = Math.floor(seconds / 60);
  const remainingSecs = seconds % 60;

  if (minutes < 60) {
    return remainingSecs > 0 ? `${minutes}m ${remainingSecs}s` : `${minutes}m`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMins = minutes % 60;

  if (hours < 24) {
    return remainingMins > 0 ? `${hours}h ${remainingMins}m` : `${hours}h`;
  }

  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;

  return remainingHours > 0 ? `${days}d ${remainingHours}h` : `${days}d`;
}
