import React, { useEffect } from 'react';
import { AlertTriangle, Info, X } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface ConfirmModalProps {
    isOpen: boolean;
    title: string;
    message: React.ReactNode;
    confirmLabel?: string;
    cancelLabel?: string;
    isDestructive?: boolean;
    isLoading?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
}

export function ConfirmModal({
    isOpen,
    title,
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    isDestructive = false,
    isLoading = false,
    onConfirm,
    onCancel,
}: ConfirmModalProps) {
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen && !isLoading) {
                onCancel();
            }
        };

        if (isOpen) {
            document.addEventListener('keydown', handleKeyDown);
        }
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [isOpen, isLoading, onCancel]);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[99999] flex items-center justify-center p-4">
            {/* Backdrop */}
            <div
                onClick={() => !isLoading && onCancel()}
                className="fixed inset-0 bg-black/60 backdrop-blur-sm animate-in fade-in duration-150"
            />

            {/* Modal Shell */}
            <div
                role="dialog"
                aria-modal="true"
                className={cn(
                    'relative w-full max-w-md rounded-2xl border shadow-2xl p-5',
                    'bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl',
                    'border-slate-200 dark:border-slate-800 text-slate-800 dark:text-slate-100',
                    'animate-in zoom-in-95 duration-150'
                )}
            >
                <div className="flex items-start justify-between gap-3 mb-3">
                    <div className="flex items-center gap-2.5">
                        <div
                            className={cn(
                                'w-8 h-8 rounded-full flex items-center justify-center shrink-0',
                                isDestructive
                                    ? 'bg-rose-500/15 text-rose-600 dark:text-rose-400'
                                    : 'bg-purple-500/15 text-purple-600 dark:text-purple-400'
                            )}
                        >
                            {isDestructive ? <AlertTriangle className="w-4 h-4" /> : <Info className="w-4 h-4" />}
                        </div>
                        <h3 className="text-sm font-semibold truncate">{title}</h3>
                    </div>

                    <button
                        type="button"
                        onClick={onCancel}
                        disabled={isLoading}
                        className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors p-1 rounded-lg"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <div className="text-xs text-slate-600 dark:text-slate-300 mb-5 leading-relaxed pl-10">
                    {message}
                </div>

                <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 dark:border-slate-800/80">
                    <button
                        type="button"
                        onClick={onCancel}
                        disabled={isLoading}
                        className="px-3 py-1.5 rounded-lg text-xs font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors disabled:opacity-50"
                    >
                        {cancelLabel}
                    </button>

                    <button
                        type="button"
                        onClick={onConfirm}
                        disabled={isLoading}
                        className={cn(
                            'px-4 py-1.5 rounded-lg text-xs font-semibold text-white shadow-xs transition-all disabled:opacity-50',
                            isDestructive
                                ? 'bg-rose-600 hover:bg-rose-500 active:bg-rose-700'
                                : 'bg-purple-600 hover:bg-purple-500 active:bg-purple-700'
                        )}
                    >
                        {isLoading ? 'Processing...' : confirmLabel}
                    </button>
                </div>
            </div>
        </div>
    );
}

export default ConfirmModal;
