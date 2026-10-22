/**
 * Formats a raw byte count into human-readable representation (e.g. "4.2 MB", "850 KB").
 */
export function formatByteSize(bytes: number | null | undefined, decimals: number = 1): string {
  if (bytes === null || bytes === undefined || isNaN(bytes) || bytes < 0) {
    return '0 B';
  }

  if (bytes === 0) return '0 B';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const safeIndex = Math.min(i, sizes.length - 1);

  const value = bytes / Math.pow(k, safeIndex);
  return `${parseFloat(value.toFixed(dm))} ${sizes[safeIndex]}`;
}
