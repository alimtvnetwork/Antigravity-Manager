import React, { useEffect, ReactNode } from 'react';
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
  useEffect(() => {
    if (durationMs <= 0) return;
    const timer = setTimeout(() => {
      onDismiss();
    }, durationMs);

    return () => clearTimeout(timer);
  }, [durationMs, onDismiss]);

  const getVariantStyles = () => {
    switch (variant) {
      case 'success':
        return {
          border: 'border-emerald-500/40',
          bg: 'bg-emerald-950/80',
          text: 'text-emerald-200',
          icon: <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />,
        };
      case 'warning':
        return {
          border: 'border-amber-500/40',
          bg: 'bg-amber-950/80',
          text: 'text-amber-200',
          icon: <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />,
        };
      case 'error':
        return {
          border: 'border-rose-500/40',
          bg: 'bg-rose-950/80',
          text: 'text-rose-200',
          icon: <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />,
        };
      case 'info':
      default:
        return {
          border: 'border-sky-500/40',
          bg: 'bg-slate-900/90',
          text: 'text-sky-200',
          icon: <Info className="w-4 h-4 text-sky-400 shrink-0" />,
        };
    }
  };

  const styles = getVariantStyles();

  return (
    <div
      role="status"
      className={`flex items-center gap-2.5 px-3.5 py-2 rounded-xl border shadow-xl backdrop-blur-md animate-in fade-in-0 slide-in-from-top-2 duration-200 select-none ${styles.border} ${styles.bg} ${styles.text} ${className}`}
    >
      {icon ? icon : styles.icon}
      <span className="text-xs md:text-sm font-medium">{message}</span>

      <button
        type="button"
        onClick={onDismiss}
        className="p-0.5 ml-1 text-slate-400 hover:text-slate-200 rounded hover:bg-white/10 transition-colors"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
};
