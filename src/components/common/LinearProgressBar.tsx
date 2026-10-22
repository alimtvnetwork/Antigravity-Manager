import React from 'react';

interface LinearProgressBarProps {
  value: number; // 0 to 100
  height?: 'xs' | 'sm' | 'md';
  variant?: 'sky' | 'emerald' | 'amber' | 'rose';
  showLabel?: boolean;
  striped?: boolean;
  className?: string;
}

const variantGradients: Record<string, string> = {
  sky: 'bg-gradient-to-r from-sky-500 to-cyan-400',
  emerald: 'bg-gradient-to-r from-emerald-500 to-teal-400',
  amber: 'bg-gradient-to-r from-amber-500 to-yellow-400',
  rose: 'bg-gradient-to-r from-rose-500 to-pink-500',
};

const heightStyles: Record<string, string> = {
  xs: 'h-1.5',
  sm: 'h-2.5',
  md: 'h-3.5',
};

export const LinearProgressBar: React.FC<LinearProgressBarProps> = ({
  value,
  height = 'sm',
  variant = 'sky',
  showLabel = false,
  striped = false,
  className = '',
}) => {
  const clampedValue = Math.min(100, Math.max(0, value));
  const gradient = variantGradients[variant] || variantGradients.sky;
  const heightClass = heightStyles[height] || heightStyles.sm;

  return (
    <div className={`w-full flex items-center gap-2.5 ${className}`}>
      <div
        role="progressbar"
        aria-valuenow={clampedValue}
        aria-valuemin={0}
        aria-valuemax={100}
        className={`flex-1 relative overflow-hidden rounded-full bg-slate-800/90 border border-slate-700/50 shadow-inner ${heightClass}`}
      >
        <div
          className={`h-full rounded-full transition-all duration-300 ease-out ${gradient} ${
            striped ? 'bg-[length:20px_20px] animate-[pulse_2s_infinite]' : ''
          }`}
          style={{ width: `${clampedValue}%` }}
        />
      </div>
      {showLabel && (
        <span className="text-xs font-semibold text-slate-300 min-w-[36px] text-right font-mono">
          {Math.round(clampedValue)}%
        </span>
      )}
    </div>
  );
};
