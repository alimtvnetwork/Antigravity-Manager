import { useState, useEffect } from 'react';
import { CheckCircle, XCircle, Info, AlertTriangle, X } from 'lucide-react';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface ToastProps {
    id: string;
    message: string;
    type: ToastType;
    duration?: number;
    onClose: (id: string) => void;
}

const Toast = ({ id, message, type, duration = 3000, onClose }: ToastProps) => {
    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        // Exciting entrance
        requestAnimationFrame(() => setIsVisible(true));

        if (duration > 0) {
            const timer = setTimeout(() => {
                setIsVisible(false);
                setTimeout(() => onClose(id), 300); // Wait for transition
            }, duration);
            return () => clearTimeout(timer);
        }
    }, [duration, id, onClose]);

    const getIcon = () => {
        switch (type) {
            case 'success': return <CheckCircle className="w-5 h-5 text-emerald-500 dark:text-emerald-400 shrink-0" aria-hidden="true" />;
            case 'error': return <XCircle className="w-5 h-5 text-rose-500 dark:text-rose-400 shrink-0" aria-hidden="true" />;
            case 'warning': return <AlertTriangle className="w-5 h-5 text-amber-500 dark:text-amber-400 shrink-0" aria-hidden="true" />;
            case 'info': default: return <Info className="w-5 h-5 text-sky-500 dark:text-sky-400 shrink-0" aria-hidden="true" />;
        }
    };

    const getStyles = () => {
        switch (type) {
            case 'success': return 'border-emerald-200 dark:border-emerald-800/40 bg-white/95 dark:bg-slate-900/90 text-slate-800 dark:text-slate-100 shadow-emerald-500/5';
            case 'error': return 'border-rose-200 dark:border-rose-800/40 bg-white/95 dark:bg-slate-900/90 text-slate-800 dark:text-slate-100 shadow-rose-500/5';
            case 'warning': return 'border-amber-200 dark:border-amber-800/40 bg-white/95 dark:bg-slate-900/90 text-slate-800 dark:text-slate-100 shadow-amber-500/5';
            case 'info': default: return 'border-sky-200 dark:border-sky-800/40 bg-white/95 dark:bg-slate-900/90 text-slate-800 dark:text-slate-100 shadow-sky-500/5';
        }
    };

    const isAlert = type === 'error';

    return (
        <div
            role={isAlert ? 'alert' : 'status'}
            aria-live={isAlert ? 'assertive' : 'polite'}
            aria-atomic="true"
            className={`flex items-center gap-3 px-4 py-3 rounded-xl shadow-lg border backdrop-blur-md transition-all duration-300 transform ${getStyles()} ${isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-2'}`}
            style={{ minWidth: '300px' }}
        >
            {getIcon()}
            <p className="flex-1 text-sm font-medium text-slate-700 dark:text-slate-200">{message}</p>
            <button
                type="button"
                onClick={() => { setIsVisible(false); setTimeout(() => onClose(id), 300); }}
                aria-label="Dismiss notification"
                title="Dismiss notification"
                className="p-1 rounded-lg text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 transition-colors"
            >
                <X className="w-4 h-4" aria-hidden="true" />
            </button>
        </div>
    );
};

export default Toast;
