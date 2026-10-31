import React, { useState, useCallback } from 'react';
import { Copy, Check } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';

export type CodeBadgeVariant = 'default' | 'emerald' | 'amber' | 'purple' | 'rose' | 'sky';
export type CodeBadgeSize = 'xs' | 'sm' | 'md';

export interface InlineCodeBadgeProps {
    code: string;
    variant?: CodeBadgeVariant;
    size?: CodeBadgeSize;
    copyable?: boolean;
    className?: string;
}

const VARIANT_STYLES: Record<CodeBadgeVariant, string> = {
    default: 'bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700/60',
    emerald: 'bg-emerald-500/10 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border-emerald-300/40 dark:border-emerald-700/50',
    amber: 'bg-amber-500/10 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border-amber-300/40 dark:border-amber-700/50',
    purple: 'bg-purple-500/10 dark:bg-purple-950/40 text-purple-700 dark:text-purple-300 border-purple-300/40 dark:border-purple-700/50',
    rose: 'bg-rose-500/10 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300 border-rose-300/40 dark:border-rose-700/50',
    sky: 'bg-sky-500/10 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 border-sky-300/40 dark:border-sky-700/50',
};

export const InlineCodeBadge: React.FC<InlineCodeBadgeProps> = ({
    code,
    variant = 'default',
    size = 'sm',
    copyable = false,
    className = '',
}) => {
    const [copied, setCopied] = useState(false);

    const handleCopy = useCallback(async (e: React.MouseEvent) => {
        if (!copyable) return;
        e.stopPropagation();

        try {
            if (navigator?.clipboard?.writeText) {
                await navigator.clipboard.writeText(code);
            }
            setCopied(true);
            setTimeout(() => setCopied(false), 1600);
        } catch (err) {
            useErrorStore.getState().trackWarning('InlineCodeBadge clipboard copy failed', {
                source: 'InlineCodeBadge.handleCopy',
                triggerAction: 'copy_code',
                context: { preview: code.slice(0, 50) },
            });
        }
    }, [code, copyable]);

    const sizeClasses = {
        xs: 'px-1 py-0.2 text-[9px] gap-1',
        sm: 'px-1.5 py-0.5 text-[10px] gap-1.5',
        md: 'px-2 py-0.5 text-xs gap-2',
    }[size];

    return (
        <code
            onClick={copyable ? handleCopy : undefined}
            className={`inline-flex items-center font-mono font-medium rounded border shadow-2xs backdrop-blur-xs select-all transition-all ${
                copyable ? 'cursor-pointer hover:border-slate-400 dark:hover:border-slate-500/80 active:scale-95' : ''
            } ${sizeClasses} ${VARIANT_STYLES[variant]} ${className}`}
            title={copyable ? (copied ? 'Copied!' : 'Click to copy') : undefined}
        >
            <span>{code}</span>
            {copyable && (
                <span className="shrink-0 text-slate-400 dark:text-slate-500">
                    {copied ? (
                        <Check size={size === 'xs' ? 9 : 11} className="text-emerald-400" />
                    ) : (
                        <Copy size={size === 'xs' ? 9 : 11} />
                    )}
                </span>
            )}
        </code>
    );
};

export default InlineCodeBadge;
