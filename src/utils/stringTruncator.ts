/**
 * Middle-Ellipsis String Truncation Utility (Imp-509)
 * Formats long file paths, model identifiers, and hashes with readable middle-ellipsis.
 */

export function truncateMiddle(
  str: string | null | undefined,
  maxLength: number = 32,
  separator: string = '...'
): string {
  if (!str) return '';
  if (str.length <= maxLength) return str;

  const sepLen = separator.length;
  if (maxLength <= sepLen) return str.slice(0, maxLength);

  const available = maxLength - sepLen;
  const frontChars = Math.ceil(available / 2);
  const backChars = Math.floor(available / 2);

  return str.slice(0, frontChars) + separator + str.slice(str.length - backChars);
}

export function truncateEnd(
  str: string | null | undefined,
  maxLength: number = 32,
  suffix: string = '...'
): string {
  if (!str) return '';
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - suffix.length) + suffix;
}

export function truncatePath(
  path: string | null | undefined,
  maxSegments: number = 3,
  separator: string = '.../'
): string {
  if (!path) return '';
  const normalized = path.replace(/\\/g, '/');
  const segments = normalized.split('/').filter(Boolean);

  if (segments.length <= maxSegments) {
    return path;
  }

  const keptTail = segments.slice(-maxSegments).join('/');
  return separator + keptTail;
}
