import { useEffect, useState } from 'react';
import { RefreshCw } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface StaleQuotaWarningProps {
    fetchedAt?: number | string | null;
    onRefresh?: () => void;
    className?: string;
    isRefreshing?: boolean;
}

export function parseTimestampMs(value: number | string | null | undefined): number | null {
    if (value === null || value === undefined || value === '') return null;
    if (typeof value === 'number') {
        if (!Number.isFinite(value) || value <= 0) return null;
        return value < 10_000_000_000 ? value * 1000 : value;
    }
    const parsed = Date.parse(value);
    return Number.isFinite(parsed) ? parsed : null;
}

export function getElapsedMinutes(
    fetchedAt: number | string | null | undefined,
    currentTimestamp: number = Date.now()
): number | null {
    const timeMs = parseTimestampMs(fetchedAt);
    if (timeMs === null) return null;
    const diff = currentTimestamp - timeMs;
    return diff > 0 ? Math.floor(diff / 60_000) : 0;
}

export function isQuotaStale(
    fetchedAt: number | string | null | undefined,
    thresholdMs: number = 900_000,
    currentTimestamp: number = Date.now()
): boolean {
    const timeMs = parseTimestampMs(fetchedAt);
    if (timeMs === null) return false;
    return currentTimestamp - timeMs > thresholdMs;
}

export function StaleQuotaWarning({
    fetchedAt,
    onRefresh,
    className,
    isRefreshing = false,
}: StaleQuotaWarningProps) {
    const [currentTime, setCurrentTime] = useState(() => Date.now());

    useEffect(() => {
        const intervalId = setInterval(() => {
            setCurrentTime(Date.now());
        }, 60_000);
        return () => clearInterval(intervalId);
    }, []);

    useEffect(() => {
        if (!onRefresh) return;
        const handleFocus = () => {
            if (isQuotaStale(fetchedAt, 900_000, Date.now())) {
                onRefresh();
            }
        };
        window.addEventListener('focus', handleFocus);
        return () => window.removeEventListener('focus', handleFocus);
    }, [fetchedAt, onRefresh]);

    const isStale = isQuotaStale(fetchedAt, 900_000, currentTime);
    if (!isStale) return null;

    const elapsedMinutes = getElapsedMinutes(fetchedAt, currentTime) ?? 15;

    const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
        e.stopPropagation();
        onRefresh?.();
    };

    return (
        <button
            type="button"
            onClick={handleClick}
            disabled={isRefreshing}
            title="Quota data is older than 15 minutes. Click to refresh."
            className={cn(
                'group bg-amber-500/10 text-amber-400 border border-amber-500/30 text-[10px] px-2 py-0.5 rounded-full font-medium inline-flex items-center gap-1 cursor-pointer hover:bg-amber-500/20 transition-colors',
                className
            )}
        >
            <span aria-hidden="true">⚠️</span>
            <span>Quota data is stale (fetched {elapsedMinutes}m ago)</span>
            <RefreshCw
                className={cn(
                    'w-3 h-3 transition-transform duration-500',
                    isRefreshing ? 'animate-spin' : 'group-hover:rotate-180'
                )}
            />
        </button>
    );
}
