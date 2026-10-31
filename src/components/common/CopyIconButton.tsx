import React, { useState, useCallback } from 'react';
import { Copy, Check } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';

export interface CopyIconButtonProps {
    text: string;
    label?: string;
    size?: 'xs' | 'sm' | 'md';
    className?: string;
    onCopySuccess?: () => void;
}

export const CopyIconButton: React.FC<CopyIconButtonProps> = ({
    text,
    label = 'Copy to clipboard',
    size = 'sm',
    className = '',
    onCopySuccess,
}) => {
    const [copied, setCopied] = useState(false);

    const handleCopy = useCallback(async (e: React.MouseEvent) => {
        e.stopPropagation();
        e.preventDefault();

        try {
            if (navigator?.clipboard?.writeText) {
                await navigator.clipboard.writeText(text);
            } else {
                // Fallback for older environments
                const textarea = document.createElement('textarea');
                textarea.value = text;
                textarea.style.position = 'fixed';
                textarea.style.opacity = '0';
                document.body.appendChild(textarea);
                textarea.select();
                document.execCommand('copy');
                document.body.removeChild(textarea);
            }

            setCopied(true);
            onCopySuccess?.();
            setTimeout(() => setCopied(false), 1800);
        } catch (err) {
            useErrorStore.getState().trackWarning('Clipboard copy failed in CopyIconButton', {
                source: 'CopyIconButton.handleCopy',
                triggerAction: 'copy_text',
                context: { preview: text.slice(0, 100) },
            });
        }
    }, [text, onCopySuccess]);

    const iconSizes = {
        xs: 12,
        sm: 14,
        md: 16,
    }[size];

    const buttonSizes = {
        xs: 'p-1',
        sm: 'p-1.5',
        md: 'p-2',
    }[size];

    return (
        <button
            type="button"
            onClick={handleCopy}
            className={`inline-flex items-center justify-center rounded-md text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 dark:hover:bg-slate-700/50 transition-colors focus:outline-none focus:ring-1 focus:ring-slate-500/40 ${buttonSizes} ${className}`}
            title={copied ? 'Copied!' : label}
            aria-label={copied ? 'Copied!' : label}
        >
            {copied ? (
                <Check size={iconSizes} className="text-emerald-400 animate-in fade-in zoom-in duration-200" />
            ) : (
                <Copy size={iconSizes} className="transition-transform active:scale-90" />
            )}
        </button>
    );
};

export default CopyIconButton;
