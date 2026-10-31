import React from 'react';

export type HealthStatus = 'healthy' | 'warning' | 'critical' | 'idle';

export interface HealthPulseIndicatorProps {
  status: HealthStatus;
  label?: string;
  showPingRing?: boolean;
  size?: 'sm' | 'md';
  className?: string;
}

export const HealthPulseIndicator: React.FC<HealthPulseIndicatorProps> = ({
  status,
  label,
  showPingRing = true,
  size = 'md',
  className = '',
}) => {
  const dotSizes = size === 'sm' ? 'w-2 h-2' : 'w-2.5 h-2.5';

  const statusColors: Record<HealthStatus, { dot: string; ping: string; text: string }> = {
    healthy: {
      dot: 'bg-emerald-400',
      ping: 'bg-emerald-400/40',
      text: 'text-emerald-400',
    },
    warning: {
      dot: 'bg-amber-400',
      ping: 'bg-amber-400/40',
      text: 'text-amber-400',
    },
    critical: {
      dot: 'bg-rose-500',
      ping: 'bg-rose-500/40',
      text: 'text-rose-400',
    },
    idle: {
      dot: 'bg-slate-500',
      ping: 'bg-slate-500/30',
      text: 'text-slate-400',
    },
  };

  const current = statusColors[status];

  return (
    <div className={`inline-flex items-center gap-2 select-none ${className}`}>
      <span className="relative flex items-center justify-center">
        {showPingRing && status !== 'idle' && (
          <span
            className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${current.ping}`}
          />
        )}
        <span className={`relative inline-flex rounded-full ${dotSizes} ${current.dot}`} />
      </span>

      {label && <span className={`text-xs font-medium ${current.text}`}>{label}</span>}
    </div>
  );
};
