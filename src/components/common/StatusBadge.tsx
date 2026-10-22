import React from 'react';

export type StatusType = 'running' | 'queued' | 'idle' | 'error' | 'success' | 'cooldown';
export type StatusBadgeSize = 'xs' | 'sm' | 'md';

export interface StatusBadgeProps {
    status: StatusType;
    label?: string;
    size?: StatusBadgeSize;
    showDot?: boolean;
    pulse?: boolean;
    className?: string;
}

const STATUS_CONFIGS: Record<StatusType, {
    bg: string;
    text: string;
    border: string;
    dotBg: string;
    defaultLabel: string;
}> = {
    running: {
        bg: 'bg-sky-500/10 dark:bg-sky-950/40',
        text: 'text-sky-700 dark:text-sky-300',
        border: 'border-sky-300/40 dark:border-sky-700/50',
        dotBg: 'bg-sky-500',
        defaultLabel: 'Running',
    },
    queued: {
        bg: 'bg-amber-500/10 dark:bg-amber-950/40',
        text: 'text-amber-700 dark:text-amber-300',
        border: 'border-amber-300/40 dark:border-amber-700/50',
        dotBg: 'bg-amber-500',
        defaultLabel: 'Queued',
    },
    idle: {
        bg: 'bg-slate-100 dark:bg-slate-800/80',
        text: 'text-slate-600 dark:text-slate-400',
        border: 'border-slate-200 dark:border-slate-700/60',
        dotBg: 'bg-slate-400',
        defaultLabel: 'Idle',
    },
    error: {
        bg: 'bg-rose-500/10 dark:bg-rose-950/40',
        text: 'text-rose-700 dark:text-rose-300',
        border: 'border-rose-300/40 dark:border-rose-700/50',
        dotBg: 'bg-rose-500',
        defaultLabel: 'Error',
    },
    success: {
        bg: 'bg-emerald-500/10 dark:bg-emerald-950/40',
        text: 'text-emerald-700 dark:text-emerald-300',
        border: 'border-emerald-300/40 dark:border-emerald-700/50',
        dotBg: 'bg-emerald-500',
        defaultLabel: 'Success',
    },
    cooldown: {
        bg: 'bg-purple-500/10 dark:bg-purple-950/40',
        text: 'text-purple-700 dark:text-purple-300',
        border: 'border-purple-300/40 dark:border-purple-700/50',
        dotBg: 'bg-purple-500',
        defaultLabel: 'Cooldown',
    },
};

export const StatusBadge: React.FC<StatusBadgeProps> = ({
    status,
    label,
    size = 'sm',
    showDot = true,
    pulse = status === 'running',
    className = '',
}) => {
    const config = STATUS_CONFIGS[status] || STATUS_CONFIGS.idle;
    const displayLabel = label ?? config.defaultLabel;

    const sizeClasses = {
        xs: 'px-1.5 py-0.5 text-[9px] gap-1',
        sm: 'px-2 py-0.5 text-[10px] gap-1.5',
        md: 'px-2.5 py-1 text-xs gap-2',
    }[size];

    const dotSizes = {
        xs: 'w-1.5 h-1.5',
        sm: 'w-2 h-2',
        md: 'w-2.5 h-2.5',
    }[size];

    return (
        <span
            className={`inline-flex items-center rounded-full font-mono font-medium border backdrop-blur-xs select-none transition-colors ${sizeClasses} ${config.bg} ${config.text} ${config.border} ${className}`}
        >
            {showDot && (
                <span className="relative flex items-center justify-center">
                    {pulse && (
                        <span
                            className={`absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping ${config.dotBg}`}
                        />
                    )}
                    <span className={`relative inline-flex rounded-full ${dotSizes} ${config.dotBg}`} />
                </span>
            )}
            <span>{displayLabel}</span>
        </span>
    );
};

export default StatusBadge;
