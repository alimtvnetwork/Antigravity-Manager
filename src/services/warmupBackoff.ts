import { useErrorStore } from '../stores/error-store';

export interface BackoffOptions {
  baseDelaySecs?: number;
  maxDelaySecs?: number;
  maxRetries?: number;
}

export const DEFAULT_BACKOFF_OPTIONS: Required<BackoffOptions> = {
  baseDelaySecs: 5,
  maxDelaySecs: 120,
  maxRetries: 4,
};

/**
 * Calculates adaptive exponential backoff delay in seconds with uniform full jitter (0.8 to 1.2).
 */
export function calculateBackoffDelay(
  attempt: number,
  options: BackoffOptions = {}
): number {
  const base = options.baseDelaySecs ?? DEFAULT_BACKOFF_OPTIONS.baseDelaySecs;
  const max = options.maxDelaySecs ?? DEFAULT_BACKOFF_OPTIONS.maxDelaySecs;

  const exponential = base * Math.pow(2, Math.max(0, attempt));
  const bounded = Math.min(exponential, max);
  // Full uniform jitter between 0.8 and 1.2
  const jitterFactor = 0.8 + Math.random() * 0.4;
  return Number((bounded * jitterFactor).toFixed(2));
}

/**
 * Parses HTTP Retry-After header into delay in seconds.
 * Supports both integer seconds ("120") and HTTP-date strings ("Wed, 21 Oct 2026 07:28:00 GMT").
 */
export function parseRetryAfterHeader(headerValue: string | null | undefined): number | null {
  if (!headerValue) return null;
  const trimmed = headerValue.trim();

  // Try parsing as integer seconds
  const seconds = Number(trimmed);
  if (!isNaN(seconds) && seconds >= 0) {
    return seconds;
  }

  // Try parsing as HTTP-Date
  const dateEpoch = Date.parse(trimmed);
  if (!isNaN(dateEpoch)) {
    const diffSecs = Math.max(0, Math.ceil((dateEpoch - Date.now()) / 1000));
    return diffSecs;
  }

  return null;
}

export interface FleetRateLimitStatus {
  isFleetPaused: boolean;
  consecutive429Count: number;
  pauseExpiresEpochMs: number | null;
  remainingPauseSecs: number;
}

/**
 * Singleton tracker monitoring consecutive upstream 429/503 errors across the account fleet.
 * Automatically trips a temporary fleet pause after 3 consecutive rate limit rejections.
 */
class WarmupBackoffTracker {
  private consecutive429Count = 0;
  private pauseUntilEpochMs: number | null = null;
  private readonly threshold = 3;
  private readonly pauseDurationMs = 60_000; // 60s fleet pause

  /**
   * Reports a successful probe, resetting consecutive rate limit counters.
   */
  public reportSuccess(): void {
    this.consecutive429Count = 0;
    if (this.pauseUntilEpochMs && Date.now() >= this.pauseUntilEpochMs) {
      this.pauseUntilEpochMs = null;
    }
  }

  /**
   * Reports an HTTP 429 or 503 error. If the threshold is reached, trips fleet pause.
   */
  public reportRateLimit(statusCode: number, accountId?: string): FleetRateLimitStatus {
    this.consecutive429Count += 1;

    try {
      useErrorStore.getState().trackWarning(
        new Error(`Warmup probe rate-limited with HTTP ${statusCode} for account ${accountId ?? 'unknown'}`),
        {
          source: 'WarmupBackoffTracker.reportRateLimit',
          triggerAction: 'warmup_rate_limited',
        }
      );
    } catch {
      // Justification: Warning tracking is non-fatal best effort.
    }

    if (this.consecutive429Count >= this.threshold) {
      this.pauseUntilEpochMs = Date.now() + this.pauseDurationMs;
    }

    return this.getStatus();
  }

  /**
   * Returns current fleet pause status and remaining duration.
   */
  public getStatus(): FleetRateLimitStatus {
    const now = Date.now();
    const isPaused = Boolean(this.pauseUntilEpochMs && now < this.pauseUntilEpochMs);
    const remainingSecs = this.pauseUntilEpochMs && now < this.pauseUntilEpochMs
      ? Math.max(0, Math.ceil((this.pauseUntilEpochMs - now) / 1000))
      : 0;

    if (!isPaused && this.pauseUntilEpochMs) {
      this.pauseUntilEpochMs = null;
    }

    return {
      isFleetPaused: isPaused,
      consecutive429Count: this.consecutive429Count,
      pauseExpiresEpochMs: this.pauseUntilEpochMs,
      remainingPauseSecs: remainingSecs,
    };
  }

  /**
   * Manually clears the rate limit pause state.
   */
  public clear(): void {
    this.consecutive429Count = 0;
    this.pauseUntilEpochMs = null;
  }
}

export const warmupBackoffTracker = new WarmupBackoffTracker();
