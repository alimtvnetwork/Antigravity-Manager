import { useState, useEffect } from 'react';
import { cn } from '../../utils/cn';
import { getRemainingSeconds } from '../../utils/timeFormat';

export interface HumanizedResetLabelProps {
    resetTime?: string | number | null;
    className?: string;
    refreshIntervalMs?: number;
}

export interface HumanizedResetState {
    text: string;
    colorClass: string;
    isReady: boolean;
    isNearReset: boolean;
}

function computeReadyOrImminentLabel(remainingSecs: number): HumanizedResetState | null {
    if (remainingSecs <= 0) {
        return {
            text: 'Reset ready',
            colorClass: 'text-emerald-400 font-bold',
            isReady: true,
            isNearReset: false,
        };
    }
    if (remainingSecs < 60) {
        return {
            text: 'Resets in a few seconds',
            colorClass: 'text-orange-400 font-semibold',
            isReady: false,
            isNearReset: true,
        };
    }
    if (remainingSecs < 3600) {
        const mins = Math.max(1, Math.floor(remainingSecs / 60));
        return {
            text: `Resets in ${mins}m`,
            colorClass: 'text-orange-400 font-semibold',
            isReady: false,
            isNearReset: true,
        };
    }
    return null;
}

function computeLongTermLabel(remainingSecs: number): HumanizedResetState {
    if (remainingSecs < 86400) {
        const hours = Math.floor(remainingSecs / 3600);
        const mins = Math.floor((remainingSecs % 3600) / 60);
        return {
            text: `Resets in ${hours}h ${mins}m`,
            colorClass: 'text-slate-600 dark:text-slate-400 font-medium',
            isReady: false,
            isNearReset: false,
        };
    }
    const days = Math.floor(remainingSecs / 86400);
    const hours = Math.floor((remainingSecs % 86400) / 3600);
    return {
        text: `Resets in ${days}d ${hours}h`,
        colorClass: 'text-slate-500 dark:text-slate-400',
        isReady: false,
        isNearReset: false,
    };
}

export function computeResetHumanLabel(remainingSeconds: number): HumanizedResetState {
    const shortLabel = computeReadyOrImminentLabel(remainingSeconds);
    if (shortLabel !== null) {
        return shortLabel;
    }
    return computeLongTermLabel(remainingSeconds);
}

function useRemainingSeconds(resetTime?: string | number | null, intervalMs: number = 10000): number {
    const [seconds, setSeconds] = useState(() => getRemainingSeconds(resetTime));

    useEffect(() => {
        setSeconds(getRemainingSeconds(resetTime));
        const timer = setInterval(() => {
            setSeconds(getRemainingSeconds(resetTime));
        }, intervalMs);
        return () => clearInterval(timer);
    }, [resetTime, intervalMs]);

    return seconds;
}

export function HumanizedResetLabel({
    resetTime,
    className,
    refreshIntervalMs = 10000,
}: HumanizedResetLabelProps) {
    const remainingSeconds = useRemainingSeconds(resetTime, refreshIntervalMs);
    if (!resetTime) {
        return <span className={cn('text-slate-400 dark:text-slate-500 text-xs', className)}>—</span>;
    }

    const { text, colorClass } = computeResetHumanLabel(remainingSeconds);

    return (
        <span className={cn('text-xs transition-colors duration-300', colorClass, className)}>
            {text}
        </span>
    );
}

export default HumanizedResetLabel;
