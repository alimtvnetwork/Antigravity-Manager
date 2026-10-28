import { Moon, BellOff, ShieldAlert } from 'lucide-react';
import { useWarmupStore, type QuietHoursConfig } from '../../../stores/useWarmupStore';

export interface QuietHoursSchedulerProps {
  config?: QuietHoursConfig;
  onChange?: (config: QuietHoursConfig) => void;
  disabled?: boolean;
  className?: string;
}

/**
 * Determines whether a given hour falls inside the configured quiet hours window,
 * accounting for windows that wrap past midnight (e.g. 23:00 to 07:00).
 */
export function isWithinQuietHours(currentHour: number, startHour: number, endHour: number): boolean {
  if (startHour === endHour) return false;
  if (startHour > endHour) {
    // Spans midnight (e.g. 23 to 7: >= 23 OR < 7)
    return currentHour >= startHour || currentHour < endHour;
  }
  // Same day window (e.g. 1 to 5: >= 1 AND < 5)
  return currentHour >= startHour && currentHour < endHour;
}

/**
 * Formats a 24-hour integer into a standard HH:00 time string.
 */
function formatHour(hour: number): string {
  const pad = hour.toString().padStart(2, '0');
  return `${pad}:00`;
}

const HOURS_LIST = Array.from({ length: 24 }, (_, i) => i);

/**
 * QuietHoursScheduler configures quiet hours where automated background warmup
 * is paused to save battery, avoid roaming data usage, or silence alerts.
 */
export function QuietHoursScheduler({
  config,
  onChange,
  disabled = false,
  className = '',
}: QuietHoursSchedulerProps) {
  const storeQuietHours = useWarmupStore((state) => state.config.quietHours);
  const setStoreQuietHours = useWarmupStore((state) => state.setQuietHoursConfig);

  const currentConfig = config ?? storeQuietHours;

  const handleToggle = (enabled: boolean) => {
    const next = { ...currentConfig, enabled };
    if (onChange) {
      onChange(next);
    } else {
      setStoreQuietHours(next);
    }
  };

  const handleStartHourChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const startHour = Number(e.target.value);
    const next = { ...currentConfig, startHour };
    if (onChange) {
      onChange(next);
    } else {
      setStoreQuietHours(next);
    }
  };

  const handleEndHourChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const endHour = Number(e.target.value);
    const next = { ...currentConfig, endHour };
    if (onChange) {
      onChange(next);
    } else {
      setStoreQuietHours(next);
    }
  };

  const handleCriticalResetToggle = (allowCriticalReset: boolean) => {
    const next = { ...currentConfig, allowCriticalReset };
    if (onChange) {
      onChange(next);
    } else {
      setStoreQuietHours(next);
    }
  };

  const nowHour = new Date().getHours();
  const isCurrentlyInQuietHours = currentConfig.enabled && isWithinQuietHours(
    nowHour,
    currentConfig.startHour,
    currentConfig.endHour
  );

  return (
    <div
      className={`rounded-xl border border-zinc-800 bg-zinc-900/60 p-4 transition-all duration-200 hover:border-zinc-700/80 ${className}`}
    >
      <div className="flex items-center justify-between gap-2 mb-2">
        <div className="flex items-center gap-2">
          <Moon className="w-4 h-4 text-indigo-400" />
          <span className="text-xs font-semibold text-zinc-200">Quiet Hours & DND Window</span>
        </div>
        <div className="flex items-center gap-2">
          {isCurrentlyInQuietHours && (
            <span className="font-mono text-[10px] text-indigo-300 bg-indigo-500/10 px-2 py-0.5 rounded-full border border-indigo-500/20 flex items-center gap-1">
              <BellOff className="w-3 h-3" />
              Active Now
            </span>
          )}
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={currentConfig.enabled}
              onChange={(e) => handleToggle(e.target.checked)}
              disabled={disabled}
              className="sr-only peer"
              aria-label="Toggle quiet hours"
            />
            <div className="w-9 h-5 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-indigo-600"></div>
          </label>
        </div>
      </div>

      <p className="text-[11px] text-zinc-400 mb-3 leading-relaxed">
        Suppresses automated background network probes and chime notifications during selected sleep hours.
      </p>

      {/* Time Window Selectors */}
      <div className={`space-y-3 transition-opacity ${currentConfig.enabled ? 'opacity-100' : 'opacity-40 pointer-events-none'}`}>
        <div className="flex items-center gap-2 flex-wrap text-xs text-zinc-300">
          <span>Pause from</span>
          <select
            value={currentConfig.startHour}
            onChange={handleStartHourChange}
            disabled={disabled || !currentConfig.enabled}
            aria-label="Quiet hours start time"
            className="bg-zinc-800 border border-zinc-700 rounded-lg px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
          >
            {HOURS_LIST.map((h) => (
              <option key={h} value={h}>
                {formatHour(h)}
              </option>
            ))}
          </select>
          <span>until</span>
          <select
            value={currentConfig.endHour}
            onChange={handleEndHourChange}
            disabled={disabled || !currentConfig.enabled}
            aria-label="Quiet hours end time"
            className="bg-zinc-800 border border-zinc-700 rounded-lg px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
          >
            {HOURS_LIST.map((h) => (
              <option key={h} value={h}>
                {formatHour(h)}
              </option>
            ))}
          </select>
          <span className="text-zinc-500 text-[11px]">(Local Time)</span>
        </div>

        {/* Critical Reset Override */}
        <label className="flex items-center gap-2 cursor-pointer pt-1">
          <input
            type="checkbox"
            checked={currentConfig.allowCriticalReset}
            onChange={(e) => handleCriticalResetToggle(e.target.checked)}
            disabled={disabled || !currentConfig.enabled}
            className="rounded border-zinc-700 bg-zinc-800 text-indigo-500 focus:ring-0 focus:ring-offset-0"
          />
          <span className="text-[11px] text-zinc-300 flex items-center gap-1.5">
            <ShieldAlert className="w-3.5 h-3.5 text-amber-400" />
            Allow probe override if 7-day quota window resets during quiet hours
          </span>
        </label>
      </div>
    </div>
  );
}
