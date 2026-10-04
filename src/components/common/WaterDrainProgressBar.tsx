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

    // Map track gradient according to clamped percentage
    const getTrackGradient = (pct: number) => {
        if (pct >= 75) return 'bg-gradient-to-r from-[#1af18d] to-[#059669]';
        if (pct >= 50) return 'bg-gradient-to-r from-[#059669] to-[#eab308]';
        if (pct >= 25) return 'bg-gradient-to-r from-[#eab308] to-[#f97316]';
        return 'bg-gradient-to-r from-[#f97316] to-[#ef4444]';
    };

    // Specific checkpoint styling logic:
    // Glow is strictly isolated to the 1st bubble (idx === 0). Secondary bubbles retain 1.5px border and shadow-none.
    const getNodeStyle = (idx: number, isFilled: boolean) => {
        if (!isFilled) {
            return "bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none";
        }
        switch (idx) {
            case 0: // 1st bubble (100%): vibrant green with active glow
                return "bg-[#1af18d] border-[1.5px] border-[#12b27d] shadow-[0_0_8px_rgba(26,241,141,0.6)]";
            case 1: // 2nd bubble (75%): deep green, no glow
                return "bg-[#059669] border-[1.5px] border-[#047857] shadow-none";
            case 2: // 3rd bubble (50%): orangey yellow, no glow
                return "bg-[#eab308] border-[1.5px] border-[#ca8a04] shadow-none";
            case 3: // 4th bubble (25%): orange, no glow
                return "bg-[#f97316] border-[1.5px] border-[#ea580c] shadow-none";
            default:
                return "bg-[#ef4444] border-[1.5px] border-[#dc2626] shadow-none";
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
            <div className="relative w-full h-2 rounded-full overflow-hidden bg-slate-200 dark:bg-[#071a27] border border-slate-300 dark:border-[#15334d]">
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
