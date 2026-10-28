import { useErrorStore } from '../stores/error-store';

export type CircuitBreakerState = 'closed' | 'open' | 'half-open';

export interface CircuitBreakerStatus {
  state: CircuitBreakerState;
  consecutive5xxCount: number;
  trippedAtEpochMs: number | null;
  tripDurationMs: number;
  remainingCooldownSecs: number;
}

const DEFAULT_TRIP_DURATION_MS = 15 * 60 * 1000; // 15 minutes
const SECONDARY_TRIP_DURATION_MS = 30 * 60 * 1000; // 30 minutes on failed canary
const CONSECUTIVE_5XX_THRESHOLD = 3;

/**
 * 3-Strike Circuit Breaker protecting against upstream Google Gemini / Anthropic cloud outages.
 * Skips automated warmup probes to prevent error log pollution and false health degradation.
 */
class WarmupCircuitBreaker {
  private state: CircuitBreakerState = 'closed';
  private consecutive5xxCount = 0;
  private trippedAtEpochMs: number | null = null;
  private currentTripDurationMs = DEFAULT_TRIP_DURATION_MS;

  /**
   * Reports a successful probe. Resets error count and transitions half-open back to closed.
   */
  public reportSuccess(): void {
    this.consecutive5xxCount = 0;
    if (this.state === 'half-open' || this.state === 'open') {
      this.state = 'closed';
      this.trippedAtEpochMs = null;
      this.currentTripDurationMs = DEFAULT_TRIP_DURATION_MS;
    }
  }

  /**
   * Reports an HTTP 5xx server error.
   */
  public report5xxError(httpStatus = 500, detail?: string): CircuitBreakerStatus {
    this.consecutive5xxCount += 1;

    try {
      useErrorStore.getState().trackWarning(
        new Error(`Warmup probe 5xx error (${httpStatus}): ${detail ?? 'Upstream outage'}`),
        {
          source: 'WarmupCircuitBreaker.report5xxError',
          triggerAction: 'circuit_breaker_5xx',
        }
      );
    } catch {
      // Justification: Warning tracking is non-fatal best effort.
    }

    if (this.state === 'half-open') {
      // Canary failed! Re-open for 30 minutes
      this.state = 'open';
      this.trippedAtEpochMs = Date.now();
      this.currentTripDurationMs = SECONDARY_TRIP_DURATION_MS;
    } else if (this.state === 'closed' && this.consecutive5xxCount >= CONSECUTIVE_5XX_THRESHOLD) {
      // 3 consecutive 5xx errors reached! Trip breaker
      this.state = 'open';
      this.trippedAtEpochMs = Date.now();
      this.currentTripDurationMs = DEFAULT_TRIP_DURATION_MS;
    }

    return this.getStatus();
  }

  /**
   * Evaluates if a probe can be executed given the current circuit breaker state.
   */
  public canExecuteProbe(): boolean {
    this.evaluateState();
    return this.state === 'closed' || this.state === 'half-open';
  }

  /**
   * Checks expiration timers and advances state from open to half-open when duration expires.
   */
  private evaluateState(): void {
    if (this.state === 'open' && this.trippedAtEpochMs) {
      const elapsed = Date.now() - this.trippedAtEpochMs;
      if (elapsed >= this.currentTripDurationMs) {
        this.state = 'half-open';
      }
    }
  }

  /**
   * Returns current status snapshot.
   */
  public getStatus(): CircuitBreakerStatus {
    this.evaluateState();
    const now = Date.now();
    let remainingSecs = 0;

    if (this.state === 'open' && this.trippedAtEpochMs) {
      const expiresAt = this.trippedAtEpochMs + this.currentTripDurationMs;
      remainingSecs = Math.max(0, Math.ceil((expiresAt - now) / 1000));
    }

    return {
      state: this.state,
      consecutive5xxCount: this.consecutive5xxCount,
      trippedAtEpochMs: this.trippedAtEpochMs,
      tripDurationMs: this.currentTripDurationMs,
      remainingCooldownSecs: remainingSecs,
    };
  }

  /**
   * Manually resets circuit breaker to closed state.
   */
  public reset(): void {
    this.state = 'closed';
    this.consecutive5xxCount = 0;
    this.trippedAtEpochMs = null;
    this.currentTripDurationMs = DEFAULT_TRIP_DURATION_MS;
  }
}

export const warmupCircuitBreaker = new WarmupCircuitBreaker();
