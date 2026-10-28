import { useId } from 'react';
import { RefreshCw } from 'lucide-react';
import { useWarmupStore } from '../../../stores/useWarmupStore';

export interface ScanIntervalSliderProps {
  valueSecs?: number;
  onChange?: (valueSecs: number) => void;
  disabled?: boolean;
  className?: string;
}

const INTERVAL_PRESETS = [
  { label: '1m', secs: 60 },
  { label: '3m', secs: 180 },
  { label: '5m (Default)', secs: 300 },
  { label: '10m', secs: 600 },
  { label: '15m', secs: 900 },
];

/**
 * Formats seconds into human-readable duration label.
 */
function formatInterval(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const remainingSecs = secs % 60;
  return remainingSecs > 0 ? `${mins}m ${remainingSecs}s` : `${mins}m`;
}

/**
 * ScanIntervalSlider configures the background scheduler scan loop frequency.
 * Renders discrete presets and a smooth slider for granular resolution.
 */
export function ScanIntervalSlider({
  valueSecs,
  onChange,
  disabled = false,
  className = '',
}: ScanIntervalSliderProps) {
  const inputId = useId();
  const storeScanInterval = useWarmupStore((state) => state.config.scanIntervalSecs);
  const setStoreScanInterval = useWarmupStore((state) => state.setScanIntervalSecs);

  const currentSecs = valueSecs !== undefined ? valueSecs : storeScanInterval;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const nextVal = Number(e.target.value);
    if (onChange) {
      onChange(nextVal);
    } else {
      setStoreScanInterval(nextVal);
    }
  };

  const handlePresetClick = (secs: number) => {
    if (disabled) return;
    if (onChange) {
      onChange(secs);
    } else {
      setStoreScanInterval(secs);
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
          <RefreshCw className="w-4 h-4 text-emerald-400" />
          <span>Background Scan Interval</span>
        </label>
        <span className="font-mono text-xs text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
          Every {formatInterval(currentSecs)}
        </span>
      </div>

      <p className="text-[11px] text-zinc-400 mb-3 leading-relaxed">
        Specifies how frequently the background daemon scans quota reset timetables across accounts.
      </p>

      {/* Discrete Presets */}
      <div className="flex flex-wrap gap-1.5 mb-3">
        {INTERVAL_PRESETS.map((preset) => {
          const isSelected = currentSecs === preset.secs;
          return (
            <button
              key={preset.secs}
              type="button"
              onClick={() => handlePresetClick(preset.secs)}
              disabled={disabled}
              className={`text-[10px] font-mono px-2 py-1 rounded-full border transition-all ${
                isSelected
                  ? 'bg-emerald-500/20 border-emerald-500/40 text-emerald-300 font-semibold'
                  : 'bg-zinc-800/80 border-zinc-700/60 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800'
              } disabled:opacity-50`}
            >
              {preset.label}
            </button>
          );
        })}
      </div>

      <div className="flex items-center gap-3">
        <span className="text-[10px] font-mono text-zinc-500">1m</span>
        <input
          id={inputId}
          type="range"
          min={60}
          max={900}
          step={30}
          value={currentSecs}
          onChange={handleChange}
          disabled={disabled}
          className="flex-1 h-1.5 bg-zinc-800 rounded-full appearance-none cursor-pointer accent-emerald-500 disabled:opacity-50"
          aria-label="Background scan interval in seconds"
        />
        <span className="text-[10px] font-mono text-zinc-500">15m</span>
      </div>
    </div>
  );
}
