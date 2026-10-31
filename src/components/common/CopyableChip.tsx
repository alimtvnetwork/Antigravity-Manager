import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface CopyableChipProps {
    text: string;
    displayValue?: string;
    maxLength?: number;
    className?: string;
    title?: string;
}

export function CopyableChip({
    text,
    displayValue,
    maxLength = 24,
    className,
    title,
}: CopyableChipProps) {
    const [copied, setCopied] = useState(false);

    const handleCopy = async (e: React.MouseEvent) => {
        e.stopPropagation();
        e.preventDefault();
        try {
            await navigator.clipboard.writeText(text);
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
        } catch {
            // Best effort copy
        }
    };

    const displayText = displayValue ?? text;
    const truncated =
        displayText.length > maxLength
            ? `${displayText.slice(0, Math.floor(maxLength / 2))}...${displayText.slice(-Math.floor(maxLength / 2))}`
            : displayText;

    return (
        <button
            type="button"
            onClick={handleCopy}
            title={title ?? text}
            className={cn(
                'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md font-mono text-[11px]',
                'bg-slate-100 dark:bg-slate-800/80 hover:bg-slate-200 dark:hover:bg-slate-700/80',
                'border border-slate-200 dark:border-slate-700/60 text-slate-700 dark:text-slate-300',
                'transition-all duration-150 cursor-pointer select-none group focus:outline-none focus-visible:ring-1 focus-visible:ring-purple-500',
                copied && 'border-emerald-500/40 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
                className
            )}
        >
            <span className="truncate">{truncated}</span>
            <span className="shrink-0 text-slate-400 group-hover:text-slate-600 dark:group-hover:text-slate-200 transition-colors">
                {copied ? (
                    <Check className="w-3 h-3 text-emerald-500 animate-in zoom-in-50 duration-150" />
                ) : (
                    <Copy className="w-3 h-3 opacity-60 group-hover:opacity-100" />
                )}
            </span>
        </button>
    );
}

export default CopyableChip;
