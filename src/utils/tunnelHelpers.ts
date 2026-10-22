/**
 * Interactive Cloudflared Tunnel URL Capsule Helper
 * Parses, validates, and formats tunnel URLs for UI display, protocol verification, and copy triggers.
 */

export interface TunnelUrlInfo {
  rawUrl: string;
  host: string;
  protocol: string;
  isHttps: boolean;
  isTryCloudflare: boolean;
  displayLabel: string;
}

export function parseTunnelUrl(url: string): TunnelUrlInfo | null {
  const trimmed = url.trim();
  if (!trimmed) return null;

  try {
    const fullUrl = trimmed.startsWith('http://') || trimmed.startsWith('https://')
      ? trimmed
      : `https://${trimmed}`;

    const parsed = new URL(fullUrl);
    const host = parsed.hostname;
    const isHttps = parsed.protocol === 'https:';
    const isTryCloudflare = host.endsWith('.trycloudflare.com');

    // Create a compact display label (e.g. "abc-123.trycloudflare.com")
    const displayLabel = host.length > 32
      ? `${host.slice(0, 14)}...${host.slice(-14)}`
      : host;

    return {
      rawUrl: parsed.origin,
      host,
      protocol: parsed.protocol.replace(':', ''),
      isHttps,
      isTryCloudflare,
      displayLabel,
    };
  } catch {
    return null;
  }
}

export function formatTunnelCapsuleText(info: TunnelUrlInfo): string {
  const badge = info.isTryCloudflare ? '[Quick Tunnel]' : '[Named Tunnel]';
  return `${badge} ${info.rawUrl}`;
}
