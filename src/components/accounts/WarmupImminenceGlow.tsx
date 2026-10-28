import type { ReactNode } from 'react';
import { Flame } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface WarmupImminenceGlowProps {
    isImminent?: boolean;
    children: ReactNode;
    className?: string;
    showBadge?: boolean;
    badgeLabel?: string;
}

export function isImminentWindow(
    remainingSeconds: number | null | undefined,
    thresholdSeconds: number = 1800
): boolean {
    if (remainingSeconds === null || remainingSeconds === undefined) {
        return false;
    }
    return remainingSeconds > 0 && remainingSeconds <= thresholdSeconds;
}

function renderImminentBadge(showBadge: boolean, isImminent: boolean, label: string) {
    if (!isImminent || !showBadge) return null;
    return (
        <div className="absolute -top-2.5 right-3 z-10 pointer-events-none">
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-orange-500/15 text-orange-600 dark:text-orange-400 border border-orange-500/30 shadow-sm backdrop-blur-sm animate-pulse">
                <Flame className="w-3 h-3 text-orange-500" />
                {label}
            </span>
        </div>
    );
}

export function WarmupImminenceGlow({
    isImminent = false,
    children,
    className,
    showBadge = true,
    badgeLabel = 'Imminent Reset',
}: WarmupImminenceGlowProps) {
    const activeGlowStyles = isImminent
        ? 'border-orange-500/60 dark:border-orange-500/80 shadow-[0_0_15px_rgba(249,115,22,0.25)] ring-1 ring-orange-500/30'
        : 'border-transparent shadow-none ring-0';

    return (
        <div
            className={cn(
                'relative rounded-xl border transition-all duration-700 ease-in-out',
                activeGlowStyles,
                className
            )}
        >
            {renderImminentBadge(showBadge, isImminent, badgeLabel)}
            {children}
        </div>
    );
}

export default WarmupImminenceGlow;
