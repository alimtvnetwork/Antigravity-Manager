import { useState, useEffect } from 'react';
import { AlertOctagon, RotateCcw } from 'lucide-react';
import { warmupCircuitBreaker, type CircuitBreakerStatus } from '../../../services/warmupCircuitBreaker';

export interface CircuitBreakerBannerProps {
  className?: string;
}

/**
 * Formats seconds into MM:SS.
 */
function formatRemaining(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s.toString().padStart(2, '0')}s`;
}

/**
 * CircuitBreakerBanner displays a dark-glass amber/rose alert when 3 consecutive
 * 5xx upstream failures trip the safety circuit breaker.
 */
export function CircuitBreakerBanner({ className = '' }: CircuitBreakerBannerProps) {
  const [status, setStatus] = useState<CircuitBreakerStatus>(() => warmupCircuitBreaker.getStatus());

  useEffect(() => {
    const timer = setInterval(() => {
      setStatus(warmupCircuitBreaker.getStatus());
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  if (status.state === 'closed') {
    return null;
  }

  const isHalfOpen = status.state === 'half-open';

  const handleManualReset = () => {
    warmupCircuitBreaker.reset();
    setStatus(warmupCircuitBreaker.getStatus());
  };

  return (
    <div
      className={`rounded-xl border border-rose-500/30 bg-rose-950/40 p-3.5 text-rose-300 flex items-center justify-between gap-3 text-xs shadow-lg backdrop-blur-md ${className}`}
      role="alert"
    >
      <div className="flex items-center gap-2.5">
        <div className="w-7 h-7 rounded-full bg-rose-900/60 flex items-center justify-center border border-rose-500/40 text-rose-400 shrink-0">
          <AlertOctagon className="w-4 h-4" />
        </div>
        <div>
          <div className="font-semibold text-rose-200 flex items-center gap-2">
            <span>Circuit Breaker Tripped (Upstream Outage)</span>
            <span className="font-mono text-[10px] bg-rose-500/20 border border-rose-500/30 text-rose-300 px-1.5 py-0.2 rounded-full">
              {isHalfOpen ? 'Canary Testing' : `Paused for ${formatRemaining(status.remainingCooldownSecs)}`}
            </span>
          </div>
          <p className="text-[11px] text-rose-300/80 mt-0.5">
            {isHalfOpen
              ? 'Attempting single probe to confirm cloud recovery before restoring full schedule.'
              : 'Repeated 5xx upstream server errors detected. Automated warmup paused to avoid spamming.'}
          </p>
        </div>
      </div>

      <button
        type="button"
        onClick={handleManualReset}
        className="shrink-0 bg-rose-900/40 hover:bg-rose-900/70 border border-rose-500/30 text-rose-200 text-xs px-2.5 py-1.5 rounded-lg flex items-center gap-1.5 font-medium transition-colors"
        title="Manually close circuit breaker and resume probes"
      >
        <RotateCcw className="w-3.5 h-3.5" />
        <span>Reset Breaker</span>
      </button>
    </div>
  );
}
