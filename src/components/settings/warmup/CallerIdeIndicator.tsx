import { useState, useEffect } from 'react';
import { PauseCircle, Laptop } from 'lucide-react';
import { callerIdePreservation } from '../../../services/callerIdePreservation';

export interface CallerIdeIndicatorProps {
  showWhenIdle?: boolean;
  className?: string;
}

/**
 * CallerIdeIndicator displays active IDE preservation status.
 * When the developer's working IDE is streaming or in active cooldown,
 * displays a sky-blue pause pill to reassure the user that background warmup is suspended.
 */
export function CallerIdeIndicator({
  showWhenIdle = false,
  className = '',
}: CallerIdeIndicatorProps) {
  const [isBusy, setIsBusy] = useState<boolean>(() => callerIdePreservation.isCallerIdeBusy());
  const [remainingSecs, setRemainingSecs] = useState<number>(() =>
    callerIdePreservation.getIdeBusyRemainingSecs()
  );

  useEffect(() => {
    const timer = setInterval(() => {
      const busy = callerIdePreservation.isCallerIdeBusy();
      setIsBusy(busy);
      setRemainingSecs(callerIdePreservation.getIdeBusyRemainingSecs());
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  if (!isBusy) {
    if (!showWhenIdle) return null;
    return (
      <span
        className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-mono font-medium bg-zinc-800 text-zinc-400 border border-zinc-700/60 ${className}`}
      >
        <Laptop className="w-3 h-3 text-zinc-400" />
        <span>IDE Idle</span>
      </span>
    );
  }

  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-mono font-medium bg-sky-500/10 text-sky-400 border border-sky-500/30 animate-pulse ${className}`}
      title="Background warmup is paused while you are actively prompting in your IDE"
    >
      <PauseCircle className="w-3 h-3 text-sky-400" />
      <span>Paused (IDE Busy {remainingSecs > 0 ? `${remainingSecs}s` : ''})</span>
    </span>
  );
}
