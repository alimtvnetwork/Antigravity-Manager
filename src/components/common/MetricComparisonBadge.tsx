import React from 'react';
import { ArrowUp, ArrowDown, Minus } from 'lucide-react';

export type MetricDirection = 'higher-is-better' | 'lower-is-better' | 'neutral';

export interface MetricComparisonBadgeProps {
  currentValue: number;
  previousValue: number;
  formatFn?: (val: number) => string;
  direction?: MetricDirection;
  unit?: string;
  className?: string;
}

export const MetricComparisonBadge: React.FC<MetricComparisonBadgeProps> = ({
  currentValue,
  previousValue,
  formatFn,
  direction = 'higher-is-better',
  unit = '',
  className = '',
}) => {
  const delta = currentValue - previousValue;
  const isZero = Math.abs(delta) < 0.0001;

  let isPositiveImpact = false;
  if (!isZero) {
    if (direction === 'higher-is-better') {
      isPositiveImpact = delta > 0;
    } else if (direction === 'lower-is-better') {
      isPositiveImpact = delta < 0;
    }
  }

  const formattedDelta = formatFn
    ? formatFn(Math.abs(delta))
    : `${delta > 0 ? '+' : ''}${delta.toFixed(1)}${unit}`;

  const colorClasses = isZero
    ? 'bg-slate-800/80 text-slate-400 border-slate-700/60'
    : isPositiveImpact
    ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
    : 'bg-rose-500/10 text-rose-400 border-rose-500/30';

  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-mono font-medium border backdrop-blur-sm ${colorClasses} ${className}`}
    >
      {isZero ? (
        <Minus className="w-3 h-3 text-slate-400" />
      ) : delta > 0 ? (
        <ArrowUp className="w-3 h-3 shrink-0" />
      ) : (
        <ArrowDown className="w-3 h-3 shrink-0" />
      )}
      <span>{formattedDelta}</span>
    </span>
  );
};
