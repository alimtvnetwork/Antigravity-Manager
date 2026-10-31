import React from 'react';

export interface Milestone {
  label: string;
  percentage: number;
}

export interface ProgressBarCapsuleProps {
  value: number;
  max?: number;
  milestones?: Milestone[];
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  labelPrefix?: string;
  customThresholds?: {
    warning: number; // default 70
    danger: number;  // default 90
  };
  animated?: boolean;
  className?: string;
  formatValue?: (val: number, max: number) => string;
}

export const ProgressBarCapsule: React.FC<ProgressBarCapsuleProps> = ({
  value,
  max = 100,
  milestones,
  size = 'md',
  showLabel = false,
  labelPrefix,
  customThresholds = { warning: 70, danger: 90 },
  animated = true,
  className = '',
  formatValue,
}) => {
  const safeMax = max > 0 ? max : 100;
  const percentage = Math.min(100, Math.max(0, (value / safeMax) * 100));

  const getColorClasses = (pct: number): string => {
    if (pct >= customThresholds.danger) {
      return 'bg-gradient-to-r from-rose-500 to-red-600 shadow-[0_0_12px_rgba(244,63,94,0.4)]';
    }
    if (pct >= customThresholds.warning) {
      return 'bg-gradient-to-r from-amber-500 to-amber-600 shadow-[0_0_12px_rgba(245,158,11,0.4)]';
    }
    return 'bg-gradient-to-r from-emerald-500 to-teal-500 shadow-[0_0_12px_rgba(16,185,129,0.4)]';
  };

  const getHeightClass = (): string => {
    switch (size) {
      case 'sm':
        return 'h-1.5';
      case 'lg':
        return 'h-3.5';
      case 'md':
      default:
        return 'h-2.5';
    }
  };

  const displayText = formatValue
    ? formatValue(value, safeMax)
    : `${labelPrefix ? `${labelPrefix} ` : ''}${Math.round(percentage)}%`;

  return (
    <div className={`w-full flex flex-col gap-1.5 ${className}`}>
      {showLabel && (
        <div className="flex items-center justify-between text-xs font-mono font-medium text-slate-300 dark:text-slate-300 px-0.5">
          <span>{labelPrefix || 'Progress'}</span>
          <span>{displayText}</span>
        </div>
      )}

      <div className={`relative w-full ${getHeightClass()} bg-slate-800/80 dark:bg-slate-900/90 border border-slate-700/60 rounded-full overflow-hidden backdrop-blur-sm`}>
        {/* Animated Track Fill */}
        <div
          className={`h-full rounded-full ${getColorClasses(percentage)} ${
            animated ? 'transition-all duration-500 ease-out' : ''
          }`}
          style={{ width: `${percentage}%` }}
        />

        {/* Milestone Tick Nodes */}
        {milestones &&
          milestones.map((milestone, idx) => {
            const milestonePct = Math.min(100, Math.max(0, milestone.percentage));
            const isReached = percentage >= milestonePct;

            return (
              <div
                key={`milestone-${idx}-${milestone.label}`}
                className="absolute top-0 bottom-0 flex items-center justify-center -translate-x-1/2 pointer-events-none"
                style={{ left: `${milestonePct}%` }}
                title={`${milestone.label} (${milestonePct}%)`}
              >
                <div
                  className={`w-1 h-full ${
                    isReached ? 'bg-white/80' : 'bg-slate-500/50'
                  } rounded-full`}
                />
              </div>
            );
          })}
      </div>
    </div>
  );
};
