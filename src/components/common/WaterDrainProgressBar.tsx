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

    // Tier thresholds: Critical(red) → Warning(orange) → Healthy(green) → Excellent(neon green)
    const getTrackGradient = (pct: number) => {
        if (pct >= 75) return 'bg-gradient-to-r from-emerald-400 to-[#1af18d]';
        if (pct >= 50) return 'bg-gradient-to-r from-emerald-500 to-[#1af18d]';
        if (pct >= 25) return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
        return 'bg-gradient-to-r from-orange-500 via-rose-500 to-rose-600';
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
            default:
                return 'bg-rose-500 border-[1.5px] border-rose-400 shadow-none';
        }
    };

    // Sort checkpoints descending so index 0 corresponds to the 1st checkpoint (100%)
    const sortedCheckpoints = [...checkpoints].sort((a, b) => b - a);

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

            {/* Checkpoint Nodes (Milestone dots with miniature checkmarks) */}
            {showCheckpoints && sortedCheckpoints.map((cp, idx) => {
                const isFilled = clamped >= cp;
                const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';

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
                    </div>
                );
            })}
        </div>
    );
}

export default WaterDrainProgressBar;
