import { useId } from 'react';
import { Layers, Timer } from 'lucide-react';
import { useWarmupStore } from '../../../stores/useWarmupStore';

export interface StaggerBatchControlsProps {
  batchSize?: number;
  staggerDelaySecs?: number;
  onChange?: (batchSize: number, staggerDelaySecs: number) => void;
  disabled?: boolean;
  className?: string;
}

/**
 * StaggerBatchControls provides dual controls for concurrent probe batch size
 * and inter-account stagger delay to prevent socket congestion and upstream 429 bursts.
 */
export function StaggerBatchControls({
  batchSize,
  staggerDelaySecs,
  onChange,
  disabled = false,
  className = '',
}: StaggerBatchControlsProps) {
  const batchId = useId();
  const staggerId = useId();

  const storeBatchSize = useWarmupStore((state) => state.config.batchSize);
  const storeStaggerDelay = useWarmupStore((state) => state.config.staggerDelaySecs);
  const setStoreStaggerConfig = useWarmupStore((state) => state.setStaggerBatchConfig);

  const currentBatchSize = batchSize !== undefined ? batchSize : storeBatchSize;
  const currentStaggerDelay = staggerDelaySecs !== undefined ? staggerDelaySecs : storeStaggerDelay;

  const handleBatchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const nextBatch = Number(e.target.value);
    if (onChange) {
      onChange(nextBatch, currentStaggerDelay);
    } else {
      setStoreStaggerConfig(nextBatch, currentStaggerDelay);
    }
  };

  const handleStaggerChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const nextStagger = Number(e.target.value);
    if (onChange) {
      onChange(currentBatchSize, nextStagger);
    } else {
      setStoreStaggerConfig(currentBatchSize, nextStagger);
    }
  };

  return (
    <div className={`grid grid-cols-1 md:grid-cols-2 gap-3 ${className}`}>
      {/* Batch Size Control */}
      <div className="bg-zinc-900/60 border border-zinc-800 rounded-xl p-3.5 flex flex-col gap-2 transition-all duration-200 hover:border-zinc-700/80">
        <div className="flex items-center justify-between gap-2">
          <label
            htmlFor={batchId}
            className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-zinc-300 cursor-pointer"
          >
            <Layers className="w-3.5 h-3.5 text-orange-400" />
            <span>Batch Concurrency</span>
          </label>
          <span className="font-mono text-xs text-orange-400 bg-orange-500/10 px-2 py-0.5 rounded-full border border-orange-500/20">
            {currentBatchSize} {currentBatchSize === 1 ? 'Account' : 'Accounts'}
          </span>
        </div>
        <p className="text-[11px] text-zinc-400">
          Number of accounts probed simultaneously per dispatch cycle.
        </p>
        <div className="flex items-center gap-2 pt-1">
          <span className="text-[10px] font-mono text-zinc-500">1</span>
          <input
            id={batchId}
            type="range"
            min={1}
            max={10}
            step={1}
            value={currentBatchSize}
            onChange={handleBatchChange}
            disabled={disabled}
            className="flex-1 h-1.5 bg-zinc-800 rounded-full appearance-none cursor-pointer accent-orange-500 disabled:opacity-50"
            aria-label="Account batch size concurrency"
          />
          <span className="text-[10px] font-mono text-zinc-500">10</span>
        </div>
      </div>

      {/* Stagger Delay Control */}
      <div className="bg-zinc-900/60 border border-zinc-800 rounded-xl p-3.5 flex flex-col gap-2 transition-all duration-200 hover:border-zinc-700/80">
        <div className="flex items-center justify-between gap-2">
          <label
            htmlFor={staggerId}
            className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-zinc-300 cursor-pointer"
          >
            <Timer className="w-3.5 h-3.5 text-sky-400" />
            <span>Stagger Delay</span>
          </label>
          <span className="font-mono text-xs text-sky-400 bg-sky-500/10 px-2 py-0.5 rounded-full border border-sky-500/20">
            {currentStaggerDelay}s Delay
          </span>
        </div>
        <p className="text-[11px] text-zinc-400">
          Cooldown sleep inserted between consecutive batch dispatches.
        </p>
        <div className="flex items-center gap-2 pt-1">
          <span className="text-[10px] font-mono text-zinc-500">1s</span>
          <input
            id={staggerId}
            type="range"
            min={1}
            max={10}
            step={1}
            value={currentStaggerDelay}
            onChange={handleStaggerChange}
            disabled={disabled}
            className="flex-1 h-1.5 bg-zinc-800 rounded-full appearance-none cursor-pointer accent-sky-500 disabled:opacity-50"
            aria-label="Inter-batch stagger delay in seconds"
          />
          <span className="text-[10px] font-mono text-zinc-500">10s</span>
        </div>
      </div>
    </div>
  );
}
