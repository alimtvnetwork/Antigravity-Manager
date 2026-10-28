import { useId } from 'react';
import { ShieldCheck } from 'lucide-react';
import { useWarmupStore } from '../../../stores/useWarmupStore';

export interface QuotaReserveSliderProps {
  value?: number;
  onChange?: (value: number) => void;
  disabled?: boolean;
  className?: string;
}

/**
 * QuotaReserveSlider adjusts the minimum quota reserve floor (0% to 20%).
 * Ensures accounts with low remaining quota never have their remaining capacity
 * consumed by automated background probes.
 */
export function QuotaReserveSlider({
  value,
  onChange,
  disabled = false,
  className = '',
}: QuotaReserveSliderProps) {
  const inputId = useId();
  const storeReserve = useWarmupStore((state) => state.config.quotaReserveFloorPct);
  const setStoreReserve = useWarmupStore((state) => state.setQuotaReserveFloorPct);

  const currentReserve = value !== undefined ? value : storeReserve;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const nextVal = Number(e.target.value);
    if (onChange) {
      onChange(nextVal);
    } else {
      setStoreReserve(nextVal);
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
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          <span>Quota Reserve Floor Guard</span>
        </label>
        <span className="font-mono text-xs text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
          Guard Floor: {currentReserve}%
        </span>
      </div>

      <p className="text-[11px] text-zinc-400 mb-3 leading-relaxed">
        Suspends automated background probes if an account has less than {currentReserve}% rolling quota,
        guaranteeing reserve capacity for interactive coding sessions.
      </p>

      <div className="flex items-center gap-3">
        <span className="text-[10px] font-mono text-zinc-500">0% (Disabled)</span>
        <input
          id={inputId}
          type="range"
          min={0}
          max={20}
          step={1}
          value={currentReserve}
          onChange={handleChange}
          disabled={disabled}
          className="flex-1 h-1.5 bg-zinc-800 rounded-full appearance-none cursor-pointer accent-emerald-500 disabled:opacity-50"
          aria-label="Quota reserve floor percentage"
        />
        <span className="text-[10px] font-mono text-zinc-500">20%</span>
      </div>
    </div>
  );
}
