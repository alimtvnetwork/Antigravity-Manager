/**
 * High-Efficiency Number, Token, and Byte Formatter
 * Formats token counts, byte tallies, and request totals with compact SI prefixes.
 */

/**
 * Format numbers with compact metric prefixes (e.g. 1.2k, 45.8M, 3.1B).
 */
export function formatCompactNumber(value: number, decimals = 1): string {
  if (!Number.isFinite(value) || isNaN(value)) {
    return '0';
  }

  const abs = Math.abs(value);
  const sign = value < 0 ? '-' : '';

  if (abs < 1000) {
    return `${sign}${Math.round(abs * 10 ** decimals) / 10 ** decimals}`;
  }

  const units = ['k', 'M', 'B', 'T'];
  let unitIndex = -1;
  let scaled = abs;

  while (scaled >= 1000 && unitIndex < units.length - 1) {
    scaled /= 1000;
    unitIndex++;
  }

  const factor = 10 ** decimals;
  const rounded = Math.round(scaled * factor) / factor;
  return `${sign}${rounded}${units[unitIndex]}`;
}

/**
 * Format token counts with specific formatting for prompt token counts.
 * E.g., 250 -> 250, 4820 -> 4.8k, 1250000 -> 1.25M
 */
export function formatTokenCount(tokens: number): string {
  if (tokens >= 1_000_000) {
    return formatCompactNumber(tokens, 2);
  }
  if (tokens >= 1_000) {
    return formatCompactNumber(tokens, 1);
  }
  return tokens.toLocaleString();
}

/**
 * Format raw byte counts to human-readable binary units (B, KB, MB, GB, TB).
 */
export function formatByteSize(bytes: number, decimals = 1): string {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B';
  }

  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const safeIndex = Math.min(i, units.length - 1);
  const scaled = bytes / 1024 ** safeIndex;

  const factor = 10 ** decimals;
  const rounded = Math.round(scaled * factor) / factor;
  return `${rounded} ${units[safeIndex]}`;
}

/**
 * Format float ratio into percentage string (e.g. 0.854 -> "85.4%").
 */
export function formatPercentage(ratio: number, decimals = 1): string {
  if (!Number.isFinite(ratio) || isNaN(ratio)) {
    return '0%';
  }
  const pct = Math.min(100, Math.max(0, ratio * 100));
  return `${pct.toFixed(decimals)}%`;
}

/**
 * Format millisecond latency into clean readable string (e.g. 240ms, 1.45s, 2m 15s).
 */
export function formatDurationMs(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return '0ms';
  if (ms < 1000) return `${Math.round(ms)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  return `${minutes}m ${seconds}s`;
}

/**
 * Parse human compact numbers back to numeric values (e.g. "1.5k" -> 1500).
 */
export function parseCompactNumber(input: string): number {
  if (!input) return 0;
  const clean = input.trim();
  const match = clean.match(/^([+-]?\d+(?:\.\d+)?)\s*([kmbgt])?$/i);
  if (!match) {
    const raw = parseFloat(clean);
    return isNaN(raw) ? 0 : raw;
  }

  const val = parseFloat(match[1]);
  const unit = (match[2] || '').toLowerCase();

  switch (unit) {
    case 'k':
      return val * 1_000;
    case 'm':
      return val * 1_000_000;
    case 'b':
      return val * 1_000_000_000;
    case 't':
      return val * 1_000_000_000_000;
    default:
      return val;
  }
}
