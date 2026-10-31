import React from 'react';

export type StatusType = 'running' | 'paused' | 'idle' | 'error' | 'queued';

export interface StatusPingBadgeProps {
  status: StatusType;
  label?: string;
  showPing?: boolean;
  size?: 'sm' | 'md';
  className?: string;
}

export const StatusPingBadge: React.FC<StatusPingBadgeProps> = ({
  status,
  label,
  showPing = true,
  size = 'md',
  className = '',
}) => {
  const getStatusStyles = () => {
    switch (status) {
      case 'running':
        return {
          bg: 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400',
          dot: 'bg-emerald-500',
          ping: 'bg-emerald-400',
          defaultLabel: 'Running',
        };
      case 'paused':
        return {
          bg: 'bg-amber-500/10 border-amber-500/30 text-amber-400',
          dot: 'bg-amber-500',
          ping: 'bg-amber-400',
          defaultLabel: 'Paused',
        };
      case 'error':
        return {
          bg: 'bg-rose-500/10 border-rose-500/30 text-rose-400',
          dot: 'bg-rose-500',
          ping: 'bg-rose-400',
          defaultLabel: 'Error',
        };
      case 'queued':
        return {
          bg: 'bg-sky-500/10 border-sky-500/30 text-sky-400',
          dot: 'bg-sky-500',
          ping: 'bg-sky-400',
          defaultLabel: 'Queued',
        };
      case 'idle':
      default:
        return {
          bg: 'bg-slate-800/60 border-slate-700/60 text-slate-400',
          dot: 'bg-slate-500',
          ping: 'bg-slate-400',
          defaultLabel: 'Idle',
        };
    }
  };

  const styles = getStatusStyles();
  const textLabel = label || styles.defaultLabel;

  const getSizeClasses = () => {
    switch (size) {
      case 'sm':
        return 'text-[10px] px-2 py-0.5 gap-1.5';
      case 'md':
      default:
        return 'text-xs px-2.5 py-1 gap-2';
    }
  };

  return (
    <span
      className={`inline-flex items-center font-mono font-medium rounded-full border backdrop-blur-sm select-none ${styles.bg} ${getSizeClasses()} ${className}`}
    >
      <span className="relative flex h-2 w-2">
        {showPing && status !== 'idle' && (
          <span
            className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${styles.ping}`}
          />
        )}
        <span
          className={`relative inline-flex rounded-full h-2 w-2 ${styles.dot}`}
        />
      </span>
      <span>{textLabel}</span>
    </span>
  );
};
