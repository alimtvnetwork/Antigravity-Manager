/**
 * Time Ago Dynamic Relative Formatter
 * Converts Unix epoch timestamps into human relative strings and calculates optimal polling intervals.
 */

export interface RelativeTimeResult {
  formatted: string;
  nextRefreshSeconds: number;
}

export function formatRelativeTime(epochSec: number, nowSec = Math.floor(Date.now() / 1000)): RelativeTimeResult {
  if (!epochSec || epochSec <= 0) {
    return { formatted: 'never', nextRefreshSeconds: 3600 };
  }

  const elapsed = Math.max(0, nowSec - epochSec);

  if (elapsed < 10) {
    return { formatted: 'just now', nextRefreshSeconds: 5 };
  }

  if (elapsed < 60) {
    return { formatted: `${elapsed}s ago`, nextRefreshSeconds: 5 };
  }

  const minutes = Math.floor(elapsed / 60);
  if (minutes < 60) {
    return { formatted: `${minutes}m ago`, nextRefreshSeconds: 30 };
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return { formatted: `${hours}h ago`, nextRefreshSeconds: 300 };
  }

  const days = Math.floor(hours / 24);
  if (days < 30) {
    return { formatted: `${days}d ago`, nextRefreshSeconds: 3600 };
  }

  // Format date
  const date = new Date(epochSec * 1000);
  const formatted = date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });

  return { formatted, nextRefreshSeconds: 86400 };
}
