import React from 'react';

interface QuotaHealthStatusCapsuleProps {
  remainingPercent: number;
  label?: string;
  size?: 'xs' | 'sm';
  showDot?: boolean;
  className?: string;
}

export const QuotaHealthStatusCapsule: React.FC<QuotaHealthStatusCapsuleProps> = ({
  remainingPercent,
  label,
  size = 'xs',
  showDot = true,
  className = '',
}) => {
  const clamped = Math.max(0, Math.min(100, remainingPercent));

  let colorClasses = 'bg-emerald-950/70 border-emerald-600/50 text-emerald-300';
  let dotClasses = 'bg-emerald-400 shadow-emerald-500/50';

  if (clamped < 15) {
    colorClasses = 'bg-rose-950/70 border-rose-600/50 text-rose-300';
    dotClasses = 'bg-rose-400 shadow-rose-500/50 animate-pulse';
  } else if (clamped <= 50) {
    colorClasses = 'bg-amber-950/70 border-amber-600/50 text-amber-300';
    dotClasses = 'bg-amber-400 shadow-amber-500/50';
  }

  const padClasses = size === 'xs' ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-xs';

  return (
    <div
      role="status"
      title={`Remaining Quota: ${clamped.toFixed(1)}%`}
      className={`inline-flex items-center gap-1.5 rounded-full border font-mono font-medium backdrop-blur-md shadow-sm ${colorClasses} ${padClasses} ${className}`}
    >
      {showDot && (
        <span
          className={`w-1.5 h-1.5 rounded-full shadow-[0_0_6px] ${dotClasses}`}
          aria-hidden="true"
        />
      )}
      {label && <span className="font-sans font-normal text-slate-400 uppercase text-[9px] tracking-wider">{label}</span>}
      <span>{clamped.toFixed(0)}%</span>
    </div>
  );
};
