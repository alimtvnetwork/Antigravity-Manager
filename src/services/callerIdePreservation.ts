/**
 * Service to protect and preserve caller IDE active streaming sessions.
 * Background warmup probes are deferred whenever the developer is actively interacting
 * with their IDE assistant to prevent socket contention or quota exhaustion.
 */

const DEFAULT_STREAMING_COOLDOWN_MS = 25_000; // 25 seconds of preservation after prompt dispatch

class CallerIdePreservationService {
  private lastPromptDispatchEpochMs = 0;
  private isStreamingActive = false;
  private activeInstanceId: string | null = null;

  /**
   * Records a user prompt dispatch or streaming start in the caller IDE.
   */
  public recordIdePromptDispatch(instanceId = 'caller-ide', cooldownMs = DEFAULT_STREAMING_COOLDOWN_MS): void {
    this.lastPromptDispatchEpochMs = Date.now() + cooldownMs;
    this.activeInstanceId = instanceId;
  }

  /**
   * Sets real-time streaming activity flag.
   */
  public setStreamingActive(isActive: boolean, instanceId = 'caller-ide'): void {
    this.isStreamingActive = isActive;
    if (isActive) {
      this.activeInstanceId = instanceId;
      this.lastPromptDispatchEpochMs = Date.now() + DEFAULT_STREAMING_COOLDOWN_MS;
    }
  }

  /**
   * Returns true if the developer is actively generating or within the post-stream cooldown window.
   */
  public isCallerIdeBusy(): boolean {
    if (this.isStreamingActive) return true;
    return Date.now() < this.lastPromptDispatchEpochMs;
  }

  /**
   * Returns remaining seconds of caller IDE busy cooldown.
   */
  public getIdeBusyRemainingSecs(): number {
    if (this.isStreamingActive) return 30;
    const diffMs = this.lastPromptDispatchEpochMs - Date.now();
    return Math.max(0, Math.ceil(diffMs / 1000));
  }

  /**
   * Returns the identifier of the active caller IDE instance.
   */
  public getActiveInstanceId(): string | null {
    return this.activeInstanceId;
  }

  /**
   * Resets preservation cooldown immediately.
   */
  public reset(): void {
    this.lastPromptDispatchEpochMs = 0;
    this.isStreamingActive = false;
    this.activeInstanceId = null;
  }
}

export const callerIdePreservation = new CallerIdePreservationService();
