import React, { useEffect, useState } from 'react';
import { X, CheckCircle, AlertTriangle, AlertCircle, Info } from 'lucide-react';

export type NotificationSeverity = 'success' | 'warning' | 'error' | 'info';

export interface FloatingNotificationCardProps {
  id: string;
  title: string;
  message?: string;
  severity?: NotificationSeverity;
  durationMs?: number;
  onDismiss: (id: string) => void;
  className?: string;
}

export const FloatingNotificationCard: React.FC<FloatingNotificationCardProps> = ({
  id,
  title,
  message,
  severity = 'info',
  durationMs = 4000,
  onDismiss,
  className = '',
}) => {
  const [progress, setProgress] = useState(100);

  useEffect(() => {
    if (durationMs <= 0) return;
    const interval = 50;
    const step = (interval / durationMs) * 100;

    const timer = setInterval(() => {
      setProgress((prev) => {
        if (prev <= step) {
          clearInterval(timer);
          onDismiss(id);
          return 0;
        }
        return prev - step;
      });
    }, interval);

    return () => clearInterval(timer);
  }, [id, durationMs, onDismiss]);

  const icons: Record<NotificationSeverity, React.ReactNode> = {
    success: <CheckCircle className="w-4 h-4 text-emerald-400 shrink-0" />,
    warning: <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />,
    error: <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />,
    info: <Info className="w-4 h-4 text-sky-400 shrink-0" />,
  };

  const borderStyles: Record<NotificationSeverity, string> = {
    success: 'border-emerald-500/30',
    warning: 'border-amber-500/30',
    error: 'border-rose-500/30',
    info: 'border-sky-500/30',
  };

  const barColors: Record<NotificationSeverity, string> = {
    success: 'bg-emerald-500',
    warning: 'bg-amber-500',
    error: 'bg-rose-500',
    info: 'bg-sky-500',
  };

  return (
    <div
      role="status"
      className={`relative overflow-hidden rounded-xl bg-slate-900/95 border backdrop-blur-md shadow-2xl p-3 min-w-[280px] max-w-sm transition-all duration-200 ${borderStyles[severity]} ${className}`}
    >
      <div className="flex items-start gap-2.5">
        <span className="mt-0.5">{icons[severity]}</span>
        <div className="flex-1 min-w-0">
          <p className="text-xs font-semibold text-slate-200 truncate">{title}</p>
          {message && <p className="text-xs text-slate-400 mt-0.5 leading-relaxed break-words">{message}</p>}
        </div>
        <button
          onClick={() => onDismiss(id)}
          className="text-slate-400 hover:text-slate-200 p-0.5 rounded-lg hover:bg-slate-800/60 transition-colors focus:outline-none"
          aria-label="Dismiss notification"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {durationMs > 0 && (
        <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-slate-800">
          <div
            className={`h-full transition-all duration-75 ${barColors[severity]}`}
            style={{ width: `${progress}%` }}
          />
        </div>
      )}
    </div>
  );
};
