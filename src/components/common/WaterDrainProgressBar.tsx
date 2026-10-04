import { cn } from '../../utils/cn';

export interface WaterDrainProgressBarProps {
    percentage: number;
    checkpoints?: number[];
    className?: string;
    showCheckpoints?: boolean;
}

export function WaterDrainProgressBar({
    percentage,
    checkpoints = [25, 50, 75, 100],
    className,
    showCheckpoints = true,
}: WaterDrainProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));

    return (
        <div className={cn("relative w-full h-[18px] flex items-center min-w-[40px]", className)}>
            <style>{`
                @keyframes agm-water-shimmer {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(200%); }
                }
            `}</style>

            {/* Base Progress Track */}
            <div className="relative w-full h-2 rounded-full overflow-hidden bg-slate-200/80 dark:bg-[#071a27] border border-slate-300/40 dark:border-[#15334d]">
                {/* Active Water Fill with Gradient & Subtle Shimmer */}
                <div
                    className="h-full rounded-full bg-gradient-to-r from-[#1af18d] to-[#12b27d] relative overflow-hidden transition-all duration-500 ease-out"
                    style={{ width: `${clamped}%` }}
                >
                    <div
                        className="absolute inset-0 bg-gradient-to-r from-transparent via-white/35 to-transparent pointer-events-none"
                        style={{ animation: 'agm-water-shimmer 2s infinite linear' }}
                    />
                </div>
            </div>

            {/* Checkpoint Nodes (Milestone dots with miniature checkmarks) */}
            {showCheckpoints && checkpoints.map((cp) => {
                const isFilled = clamped >= cp;
                const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';

                return (
                    <div
                        key={cp}
                        className={cn(
                            "absolute top-1/2 w-3.5 h-3.5 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none",
                            isFilled
                                ? "bg-[#1af18d] border-[1.5px] border-[#12b27d] shadow-[0_0_8px_rgba(26,241,141,0.6)]"
                                : "bg-[rgba(18,178,125,0.2)] border border-[rgba(18,178,125,0.35)]"
                        )}
                        style={{ left: leftPos, transform }}
                        title={`Checkpoint ${cp}%`}
                    >
                        <svg
                            className={cn(
                                "w-2 h-2 fill-none stroke-current transition-colors",
                                isFilled ? "text-white stroke-[2.5]" : "text-white/40 stroke-[2.2]"
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
