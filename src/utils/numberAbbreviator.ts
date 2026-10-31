/**
 * Formats a numeric value into a concise human-readable metric string
 * (e.g. 1,234 -> 1.2K, 5,432,100 -> 5.4M, 1,200,000,000 -> 1.2B).
 */
export function abbreviateNumber(num: number, decimals: number = 1): string {
  if (!Number.isFinite(num)) {
    return '0';
  }

  const isNegative = num < 0;
  const absVal = Math.abs(num);

  if (absVal < 1000) {
    return num.toString();
  }

  const tiers = [
    { suffix: 'T', threshold: 1e12 },
    { suffix: 'B', threshold: 1e9 },
    { suffix: 'M', threshold: 1e6 },
    { suffix: 'K', threshold: 1e3 },
  ];

  for (const { suffix, threshold } of tiers) {
    if (absVal >= threshold) {
      const scaled = absVal / threshold;
      const formatted = scaled.toFixed(decimals).replace(/\.0+$/, '');
      return `${isNegative ? '-' : ''}${formatted}${suffix}`;
    }
  }

  return num.toString();
}
