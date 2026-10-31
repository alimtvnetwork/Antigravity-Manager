import React from 'react';
import { cn } from '../../utils/cn';

export type StatusPillVariant = 'success' | 'info' | 'warning' | 'error' | 'neutral' | 'purple';
export type StatusPillSize = 'xs' | 'sm' | 'md';

export interface StatusPillProps {
    variant?: StatusPillVariant;
    label: React.ReactNode;
    icon?: React.ReactNode;
    dot?: boolean;
    size?: StatusPillSize;
    className?: string;
}

const variantStyles: Record<StatusPillVariant, { pill: string; dot: string }> = {
    success: {
        pill: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/30',
        dot: 'bg-emerald-500',
    },
    info: {
        pill: 'bg-sky-500/15 text-sky-700 dark:text-sky-300 border-sky-500/30',
        dot: 'bg-sky-500',
    },
    warning: {
        pill: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30',
        dot: 'bg-amber-500',
    },
    error: {
        pill: 'bg-rose-500/15 text-rose-700 dark:text-rose-300 border-rose-500/30',
        dot: 'bg-rose-500',
    },
    neutral: {
        pill: 'bg-slate-500/15 text-slate-700 dark:text-slate-300 border-slate-500/30',
        dot: 'bg-slate-400',
    },
    purple: {
        pill: 'bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-500/30',
        dot: 'bg-purple-500',
    },
};

const sizeStyles: Record<StatusPillSize, string> = {
    xs: 'text-[9px] px-1.5 py-0.2 gap-1 rounded font-medium',
    sm: 'text-[10px] px-2 py-0.5 gap-1.5 rounded-full font-semibold',
    md: 'text-xs px-2.5 py-0.5 gap-1.5 rounded-full font-semibold',
};

export function StatusPill({
    variant = 'neutral',
    label,
    icon,
    dot = false,
    size = 'sm',
    className,
}: StatusPillProps) {
    const style = variantStyles[variant];

    return (
        <span
            className={cn(
                'inline-flex items-center border select-none transition-colors duration-150 backdrop-blur-xs',
                style.pill,
                sizeStyles[size],
                className
            )}
        >
            {dot && <span className={cn('w-1.5 h-1.5 rounded-full shrink-0', style.dot)} />}
            {icon && <span className="shrink-0 flex items-center">{icon}</span>}
            <span className="truncate">{label}</span>
        </span>
    );
}

export default StatusPill;
