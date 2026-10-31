import { useCallback } from 'react';
import { showToast, ShowToastOptions } from '../components/common/ToastContainer';
import { ToastType } from '../components/common/Toast';

export interface ToastTriggerOptions {
    duration?: number;
}

export interface UseToastNotificationReturn {
    success: (message: string, duration?: number) => void;
    error: (message: string | Error, duration?: number) => void;
    warning: (message: string, duration?: number) => void;
    info: (message: string, duration?: number) => void;
    show: (options: ShowToastOptions | string, type?: ToastType, duration?: number) => void;
}

/**
 * useToastNotification - Ergonomic, typed React hook for dispatching toast notifications.
 *
 * Provides pre-bound methods for common toast severities (success, error, warning, info)
 * and auto-formats Error instances safely.
 */
export function useToastNotification(): UseToastNotificationReturn {
    const success = useCallback((message: string, duration?: number) => {
        showToast(message, 'success', duration ?? 3000);
    }, []);

    const error = useCallback((messageOrError: string | Error, duration?: number) => {
        const text = messageOrError instanceof Error ? messageOrError.message : String(messageOrError);
        showToast(text, 'error', duration ?? 4500);
    }, []);

    const warning = useCallback((message: string, duration?: number) => {
        showToast(message, 'warning', duration ?? 3500);
    }, []);

    const info = useCallback((message: string, duration?: number) => {
        showToast(message, 'info', duration ?? 3000);
    }, []);

    const show = useCallback((options: ShowToastOptions | string, type: ToastType = 'info', duration: number = 3000) => {
        showToast(options, type, duration);
    }, []);

    return {
        success,
        error,
        warning,
        info,
        show,
    };
}

export default useToastNotification;
