import { cn } from '../../utils/cn';

export interface WaterDrainProgressBarProps {
    percentage: number;
    checkpoints?: number[];
    className?: string;
    showCheckpoints?: boolean;
}

export function WaterDrainProgressBar({
    percentage,
    checkpoints = [100, 75, 50, 25],
    className,
    showCheckpoints = true,
}: WaterDrainProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));
    const isCritical = clamped < 25;
    const hasCheckpoints = showCheckpoints;

    // Piecewise gradient matching specification:
    // < 25%: deep dark red transition
    // >= 75%: neon emerald
    // 50-74%: emerald to teal
    // 25-49%: amber to orange
    const getTrackGradient = (pct: number) => {
        if (pct < 25) {
            return 'bg-gradient-to-r from-[#7f1d1d] via-[#991b1b] to-[#dc2626]';
        }
        if (pct >= 75) {
            return 'bg-gradient-to-r from-emerald-400 to-[#1af18d]';
        }
        if (pct >= 50) {
            return 'bg-gradient-to-r from-emerald-500 via-teal-400 to-[#1af18d]';
        }
        return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
    };

    const getTrackGlow = (pct: number) => {
        if (pct >= 75) return 'shadow-[0_0_10px_rgba(26,241,141,0.75)]';
        if (pct < 25) return 'shadow-[0_0_8px_rgba(220,38,38,0.75)]';
        return '';
    };

    // Specific checkpoint styling blending seamlessly with track gradient
    const getNodeStyle = (checkpoint: number, isFilled: boolean, isCriticalState: boolean) => {
        if (isCriticalState && checkpoint <= 25) {
            return 'bg-[#991b1b] border-[1.5px] border-[#dc2626] shadow-[0_0_8px_rgba(220,38,38,0.75)]';
        }
        if (isFilled) {
            if (checkpoint >= 100) {
                return 'bg-[#1af18d] border-[1.5px] border-[#1af18d] shadow-[0_0_8px_rgba(26,241,141,0.85)]';
            }
            if (checkpoint >= 75) {
                return 'bg-[#34d399] border-[1.5px] border-[#34d399] shadow-[0_0_6px_rgba(52,211,153,0.6)]';
            }
            if (checkpoint >= 50) {
                return 'bg-[#f59e0b] border-[1.5px] border-[#f59e0b] shadow-none';
            }
            if (checkpoint >= 25) {
                return 'bg-[#f97316] border-[1.5px] border-[#f97316] shadow-none';
            }
            return 'bg-rose-500 border-[1.5px] border-rose-400 shadow-none';
        }
        return 'bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none';
    };

    // Sort checkpoints descending and cap strictly to max 5 nodes
    const sortedCheckpoints = [...checkpoints]
        .filter((cp) => cp > 0)
        .sort((a, b) => b - a)
        .slice(0, 5);

    return (
        <div className={cn("relative w-full h-[18px] flex items-center min-w-[40px]", className)}>
            <style>{`
                @keyframes agm-water-shimmer {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(200%); }
                }
            `}</style>

            {/* Base Progress Track */}
            <div className={cn(
                "relative w-full h-2 rounded-full overflow-hidden bg-slate-200 dark:bg-[#071a27] border border-slate-300 dark:border-[#15334d]",
                getTrackGlow(clamped)
            )}>
                {/* Active Water Fill with Gradient & Subtle Shimmer */}
                <div
                    className={cn(
                        "h-full rounded-full relative overflow-hidden transition-all duration-500 ease-out",
                        getTrackGradient(clamped)
                    )}
                    style={{ width: `${clamped}%` }}
                >
                    <div
                        className="absolute inset-0 bg-gradient-to-r from-transparent via-white/35 to-transparent pointer-events-none"
                        style={{ animation: 'agm-water-shimmer 2s infinite linear' }}
                    />
                </div>
            </div>

            {/* Checkpoint Nodes (Milestone dots with miniature checkmarks or critical percentage numbers) */}
            {hasCheckpoints && sortedCheckpoints.map((cp) => {
                const isFilled = clamped >= cp || (isCritical && cp <= 25);
                const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';

                return (
                    <div
                        key={cp}
                        className={cn(
                            "absolute top-1/2 w-3.5 h-3.5 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none",
                            isCritical && cp <= 25 && "w-[15px] h-[15px]",
                            getNodeStyle(cp, isFilled, isCritical)
                        )}
                        style={{ left: leftPos, transform }}
                        title={`Checkpoint ${cp}%`}
                    >
                        {isCritical && cp <= 25 ? (
                            <span className="text-[7.5px] font-mono font-black text-white leading-none tracking-tight">
                                {clamped}%
                            </span>
                        ) : isCritical ? null : (
                            <svg
                                className={cn(
                                    "w-2 h-2 fill-none stroke-current transition-colors",
                                    isFilled ? "text-white stroke-[2.5]" : "text-gray-400/50 dark:text-white/30 stroke-[2.2]"
                                )}
                                viewBox="0 0 12 12"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                            >
                                <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                            </svg>
                        )}
                    </div>
                );
            })}
        </div>
    );
}

export default WaterDrainProgressBar;
