import { cn } from '../../utils/cn';

export interface WaterDrainProgressBarProps {
    percentage: number;
    checkpoints?: number[];
    className?: string;
    showCheckpoints?: boolean;
}

export function WaterDrainProgressBar({
    percentage,
    checkpoints = [100, 75, 50, 25, 0],
    className,
    showCheckpoints = true,
}: WaterDrainProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));

    // Tier thresholds: Critical(deep red) → Warning(amber/orange) → Healthy(green) → Excellent(neon green)
    const getTrackGradient = (pct: number) => {
        if (pct >= 75) return 'bg-gradient-to-r from-emerald-400 to-[#1af18d]';
        if (pct >= 50) return 'bg-gradient-to-r from-emerald-500 to-[#1af18d]';
        if (pct >= 25) return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
        return 'bg-gradient-to-r from-[#520808] via-rose-600 to-rose-700';
    };

    // Specific checkpoint styling logic:
    // Glow is strictly isolated to the top 2 bubbles. Secondary bubbles retain 1.5px border and shadow-none.
    const getNodeStyle = (idx: number, isFilled: boolean) => {
        if (!isFilled) {
            return "bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none";
        }
        switch (idx) {
            case 0: // 100% node — neon green glow
                return 'bg-[#1af18d] border-[1.5px] border-emerald-300 shadow-[0_0_8px_rgba(26,241,141,0.85)]';
            case 1: // 75% node — emerald with glow
                return 'bg-emerald-400 border-[1.5px] border-emerald-300 shadow-[0_0_6px_rgba(26,241,141,0.6)]';
            case 2: // 50% node — amber, no glow
                return 'bg-amber-400 dark:bg-amber-500 border-[1.5px] border-amber-300 dark:border-amber-400 shadow-none';
            case 3: // 25% node — orange, no glow
                return 'bg-orange-500 border-[1.5px] border-orange-400 shadow-none';
            default: // 0% node or critical
                return 'bg-rose-600 border-[1.5px] border-rose-400 shadow-none';
        }
    };

    // Defensively clamp custom checkpoints to <= 5 balls and sort descending
    const activeCheckpoints = (checkpoints && checkpoints.length > 0 ? checkpoints : [100, 75, 50, 25, 0]).slice(0, 5);
    const sortedCheckpoints = [...activeCheckpoints].sort((a, b) => b - a);

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
                clamped >= 50 ? 'shadow-[0_0_10px_rgba(26,241,141,0.75)]' : ''
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

            {/* Checkpoint Nodes (Milestone dots with miniature checkmarks or critical percentage) */}
            {showCheckpoints && sortedCheckpoints.map((cp, idx) => {
                const isFilled = clamped >= cp;
                const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';
                const isCritical = clamped < 25;

                return (
                    <div
                        key={cp}
                        className={cn(
                            "absolute top-1/2 w-3.5 h-3.5 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none",
                            getNodeStyle(idx, isFilled)
                        )}
                        style={{ left: leftPos, transform }}
                        title={`Checkpoint ${cp}%`}
                    >
                        {isFilled ? (
                            isCritical && cp <= 25 ? (
                                <span className="text-[7.5px] font-black font-mono text-white leading-none tracking-tighter select-none">
                                    {Math.round(clamped)}%
                                </span>
                            ) : (
                                <svg
                                    className="w-2 h-2 fill-none stroke-current text-white stroke-[2.5]"
                                    viewBox="0 0 12 12"
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                >
                                    <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                                </svg>
                            )
                        ) : (
                            <span className="w-1 h-1 rounded-full bg-slate-400/40 dark:bg-white/20" />
                        )}
                    </div>
                );
            })}
        </div>
    );
}

export default WaterDrainProgressBar;
