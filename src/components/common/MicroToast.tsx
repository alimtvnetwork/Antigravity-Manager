import React, { useEffect, ReactNode, useRef } from 'react';
import { CheckCircle2, AlertTriangle, AlertCircle, Info, X } from 'lucide-react';

export type ToastVariant = 'success' | 'warning' | 'error' | 'info';

export interface MicroToastProps {
  message: string;
  variant?: ToastVariant;
  icon?: ReactNode;
  durationMs?: number;
  onDismiss: () => void;
  className?: string;
}

export const MicroToast: React.FC<MicroToastProps> = ({
  message,
  variant = 'info',
  icon,
  durationMs = 3500,
  onDismiss,
  className = '',
}) => {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const remainingTimeRef = useRef<number>(durationMs);
  const startTimeRef = useRef<number>(Date.now());

  const startTimer = (ms: number) => {
    if (ms <= 0) return;
    startTimeRef.current = Date.now();
    timerRef.current = setTimeout(() => {
      onDismiss();
    }, ms);
  };

  const clearTimer = () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const handleMouseEnter = () => {
    if (durationMs > 0 && timerRef.current) {
      const elapsed = Date.now() - startTimeRef.current;
      remainingTimeRef.current = Math.max(0, remainingTimeRef.current - elapsed);
      clearTimer();
    }
  };

  const handleMouseLeave = () => {
    if (durationMs > 0 && remainingTimeRef.current > 0) {
      startTimer(remainingTimeRef.current);
    }
  };

  useEffect(() => {
    startTimer(durationMs);
    return () => clearTimer();
  }, [durationMs]);

  const getVariantStyles = () => {
    switch (variant) {
      case 'success':
        return {
          border: 'border-emerald-500/40',
          bg: 'bg-emerald-950/85 backdrop-blur-md',
          text: 'text-emerald-200',
          icon: <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" aria-hidden="true" />,
        };
      case 'warning':
        return {
          border: 'border-amber-500/40',
          bg: 'bg-amber-950/85 backdrop-blur-md',
          text: 'text-amber-200',
          icon: <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" aria-hidden="true" />,
        };
      case 'error':
        return {
          border: 'border-rose-500/40',
          bg: 'bg-rose-950/85 backdrop-blur-md',
          text: 'text-rose-200',
          icon: <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" aria-hidden="true" />,
        };
      case 'info':
      default:
        return {
          border: 'border-sky-500/40',
          bg: 'bg-slate-900/90 backdrop-blur-md',
          text: 'text-sky-200',
          icon: <Info className="w-4 h-4 text-sky-400 shrink-0" aria-hidden="true" />,
        };
    }
  };

  const styles = getVariantStyles();
  const isAlert = variant === 'error';

  return (
    <div
      role={isAlert ? 'alert' : 'status'}
      aria-live={isAlert ? 'assertive' : 'polite'}
      aria-atomic="true"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onFocus={handleMouseEnter}
      onBlur={handleMouseLeave}
      className={`flex items-center gap-2.5 px-3.5 py-2 rounded-xl border shadow-xl animate-in fade-in-0 slide-in-from-top-2 duration-200 select-none ${styles.border} ${styles.bg} ${styles.text} ${className}`}
    >
      {icon ? icon : styles.icon}
      <span className="text-xs md:text-sm font-medium">{message}</span>

      <button
        type="button"
        onClick={onDismiss}
        aria-label="Dismiss notification"
        title="Dismiss notification"
        className="p-1 ml-1 text-slate-400 hover:text-slate-200 rounded-lg hover:bg-white/10 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 transition-colors"
      >
        <X className="w-3.5 h-3.5" aria-hidden="true" />
      </button>
    </div>
  );
};
