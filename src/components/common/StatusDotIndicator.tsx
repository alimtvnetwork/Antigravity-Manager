import React from 'react';

export type StatusVariant = 'green' | 'amber' | 'rose' | 'slate' | 'sky';

interface StatusDotIndicatorProps {
  status: StatusVariant;
  pulse?: boolean;
  size?: 'xs' | 'sm' | 'md';
  title?: string;
  label?: string;
  className?: string;
}

const colorMap: Record<StatusVariant, { dot: string; ping: string }> = {
  green: {
    dot: 'bg-emerald-400 border-emerald-300/60 shadow-[0_0_8px_rgba(52,211,153,0.5)]',
    ping: 'bg-emerald-400',
  },
  amber: {
    dot: 'bg-amber-400 border-amber-300/60 shadow-[0_0_8px_rgba(251,191,36,0.5)]',
    ping: 'bg-amber-400',
  },
  rose: {
    dot: 'bg-rose-500 border-rose-300/60 shadow-[0_0_8px_rgba(244,63,94,0.5)]',
    ping: 'bg-rose-500',
  },
  slate: {
    dot: 'bg-slate-400 border-slate-300/60',
    ping: 'bg-slate-400',
  },
  sky: {
    dot: 'bg-sky-400 border-sky-300/60 shadow-[0_0_8px_rgba(56,189,248,0.5)]',
    ping: 'bg-sky-400',
  },
};

const sizeMap: Record<'xs' | 'sm' | 'md', string> = {
  xs: 'w-1.5 h-1.5',
  sm: 'w-2 h-2',
  md: 'w-2.5 h-2.5',
};

export const StatusDotIndicator: React.FC<StatusDotIndicatorProps> = ({
  status,
  pulse = false,
  size = 'sm',
  title,
  label,
  className = '',
}) => {
  const { dot, ping } = colorMap[status] || colorMap.slate;
  const sizeClasses = sizeMap[size] || sizeMap.sm;

  return (
    <span
      className={`inline-flex items-center gap-1.5 ${className}`}
      title={title || label || status}
      role="status"
    >
      <span className="relative flex items-center justify-center">
        {pulse && (
          <span
            className={`absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping ${ping}`}
          />
        )}
        <span
          className={`relative inline-flex rounded-full border ${sizeClasses} ${dot}`}
        />
      </span>
      {label && <span className="text-xs text-slate-300 font-medium">{label}</span>}
    </span>
  );
};
