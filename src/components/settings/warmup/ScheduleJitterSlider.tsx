import { useId } from 'react';
import { Shuffle } from 'lucide-react';
import { useWarmupStore } from '../../../stores/useWarmupStore';

export interface ScheduleJitterSliderProps {
  value?: number;
  onChange?: (value: number) => void;
  disabled?: boolean;
  className?: string;
}

/**
 * ScheduleJitterSlider allows tuning Gaussian dispatch jitter (0s to 180s).
 * Prevents anti-scraping heuristics and synchronized request spikes.
 */
export function ScheduleJitterSlider({
  value,
  onChange,
  disabled = false,
  className = '',
}: ScheduleJitterSliderProps) {
  const inputId = useId();
  const storeJitter = useWarmupStore((state) => state.config.jitterRangeSecs);
  const setStoreJitter = useWarmupStore((state) => state.setJitterRangeSecs);

  const currentJitter = value !== undefined ? value : storeJitter;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const nextVal = Number(e.target.value);
    if (onChange) {
      onChange(nextVal);
    } else {
      setStoreJitter(nextVal);
    }
  };

  return (
    <div
      className={`rounded-xl border border-zinc-800 bg-zinc-900/60 p-4 transition-all duration-200 hover:border-zinc-700/80 ${className}`}
    >
      <div className="flex items-center justify-between gap-2 mb-2">
        <label
          htmlFor={inputId}
          className="flex items-center gap-2 text-xs font-semibold text-zinc-200 cursor-pointer"
        >
          <Shuffle className="w-4 h-4 text-orange-400" />
          <span>Dispatch Schedule Jitter</span>
        </label>
        <span className="font-mono text-xs text-orange-400 bg-orange-500/10 px-2 py-0.5 rounded-full border border-orange-500/20">
          ±{currentJitter}s (Humanized)
        </span>
      </div>

      <p className="text-[11px] text-zinc-400 mb-3 leading-relaxed">
        Randomizes automated probe dispatches with Gaussian jitter to prevent synchronized burst spikes
        and anti-scraping blocks.
      </p>

      <div className="flex items-center gap-3">
        <span className="text-[10px] font-mono text-zinc-500">0s (Exact)</span>
        <input
          id={inputId}
          type="range"
          min={0}
          max={180}
          step={5}
          value={currentJitter}
          onChange={handleChange}
          disabled={disabled}
          className="flex-1 h-1.5 bg-zinc-800 rounded-full appearance-none cursor-pointer accent-orange-500 disabled:opacity-50 disabled:cursor-not-allowed"
          aria-label="Gaussian schedule jitter range in seconds"
        />
        <span className="text-[10px] font-mono text-zinc-500">±180s</span>
      </div>
    </div>
  );
}
