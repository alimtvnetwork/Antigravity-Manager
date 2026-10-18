import { useState, useRef, useMemo } from 'react';
import { cn } from '../../../../utils/cn';
import type { Account } from '../../../../types/account';
import {
    computeAccountAlgorithmScore,
    type AccountAlgorithmScoreResult,
} from '../../../../utils/accountAlgorithmScore';
import { useInstanceStore } from '../../../../stores/useInstanceStore';

export interface AlgorithmScoreCellProps {
    account: Account;
    inUseAccountIds?: string[];
    currentAccountId?: string;
    targetModel?: string;
    className?: string;
}

/**
 * Algorithm Score & Health Progress Bar Cell
 * Replaces the static last_used date column with candidate algorithm score,
 * color-ramped progress bar, and interactive dark-glass calculation breakdown.
 */
export function AlgorithmScoreCell({
    account,
    inUseAccountIds,
    currentAccountId,
    targetModel,
    className,
}: AlgorithmScoreCellProps) {
    const [isHovered, setIsHovered] = useState(false);
    const [showAbove, setShowAbove] = useState(true);
    const cellRef = useRef<HTMLTableCellElement>(null);
    const instances = useInstanceStore((state) => state.instances);

    // Compute effective inUseAccountIds (provided or derived from instance store)
    const effectiveInUseAccountIds = useMemo(() => {
        if (inUseAccountIds && inUseAccountIds.length > 0) {
            return inUseAccountIds;
        }
        return instances
            .filter((inst) => inst.is_running && inst.config.bound_account_id)
            .map((inst) => inst.config.bound_account_id as string);
    }, [inUseAccountIds, instances]);

    const result: AccountAlgorithmScoreResult = useMemo(() => {
        return computeAccountAlgorithmScore(account, {
            inUseAccountIds: effectiveInUseAccountIds,
            currentAccountId,
            targetModel,
        });
    }, [account, effectiveInUseAccountIds, currentAccountId, targetModel]);

    const handleMouseEnter = () => {
        if (cellRef.current) {
            const rect = cellRef.current.getBoundingClientRect();
            // If less than 220px to top of viewport, show below instead of above
            setShowAbove(rect.top >= 220);
        }
        setIsHovered(true);
    };

    const handleMouseLeave = () => {
        setIsHovered(false);
    };

    return (
        <td
            ref={cellRef}
            className={cn(
                "px-2.5 py-0.5 align-middle whitespace-nowrap w-[110px] min-w-[100px] border-b border-slate-200/90 dark:border-slate-800/90 transition-colors relative group",
                className
            )}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
        >
            <div className="flex flex-col gap-1 w-full max-w-[95px] cursor-help select-none">
                {/* Top line: Percentage badge and score points */}
                <div className="flex items-center justify-between gap-1 leading-none">
                    <span className={cn("text-[11px] font-bold font-mono transition-colors", result.colorScale.text)}>
                        {result.percentage.toFixed(0)}%
                    </span>
                    <span className="text-[9px] font-mono text-slate-400 dark:text-slate-500">
                        {result.score.toFixed(4)}
                    </span>
                </div>

                {/* Bottom line: Miniature progress bar track */}
                <div className="w-full h-1.5 rounded-full bg-slate-200 dark:bg-slate-700/50 overflow-hidden relative">
                    <div
                        className={cn(
                            "h-full rounded-full transition-all duration-300 bg-gradient-to-r",
                            result.colorScale.gradient
                        )}
                        style={{ width: `${Math.min(100, Math.max(0, result.percentage))}%` }}
                    />
                </div>
            </div>

            {/* Interactive Dark-Glass Hover Tooltip */}
            {isHovered && (
                <div
                    className={cn(
                        "pointer-events-none absolute z-50 backdrop-blur-md bg-white/95 dark:bg-[#13111c]/95 border border-slate-200 dark:border-[#40285e] shadow-2xl rounded-xl p-3 text-xs w-72 right-0 transition-opacity animate-in fade-in duration-150",
                        showAbove ? "bottom-full mb-2" : "top-full mt-2"
                    )}
                >
                    {/* Header: Title, Tier Badge, Readiness % */}
                    <div className="flex items-center justify-between gap-2 border-b border-slate-200/80 dark:border-slate-800/80 pb-2 mb-2">
                        <div>
                            <div className="font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-1.5">
                                <span>Algorithm Candidate Score</span>
                            </div>
                            <div className="text-[10px] text-slate-500 dark:text-slate-400 mt-0.5">
                                Multi-factor Candidate Readiness
                            </div>
                        </div>
                        <div className="flex items-center gap-1.5 shrink-0">
                            <span className="px-1.5 py-0.5 rounded font-bold text-[10px] uppercase tracking-wider bg-purple-500/10 text-purple-600 dark:text-purple-400 border border-purple-500/20">
                                {result.tier.toUpperCase()} {result.tierMultiplier.toFixed(1)}x
                            </span>
                            <span className={cn("font-bold text-xs font-mono", result.colorScale.text)}>
                                {result.percentage.toFixed(0)}%
                            </span>
                        </div>
                    </div>

                    {/* Formula */}
                    <div className="font-mono text-[10px] px-2 py-1 rounded bg-slate-100 dark:bg-[#1a1626] border border-slate-200/80 dark:border-[#40285e]/60 text-slate-700 dark:text-slate-300 mb-2.5">
                        <span className="text-slate-400 dark:text-slate-500 select-none mr-1">Formula:</span>
                        <span className="font-semibold text-slate-900 dark:text-slate-100">{result.mathBreakdown.formulaString}</span>
                    </div>

                    {/* Mathematical calculation breakdown */}
                    <div className="space-y-1.5 font-mono text-[11px]">
                        {/* Status Factor */}
                        <div className="flex items-center justify-between text-slate-600 dark:text-slate-400">
                            <span className="text-slate-500 dark:text-slate-400">Status Factor (S_active):</span>
                            <span className={cn("font-semibold", result.activeFactor === 1 ? "text-emerald-600 dark:text-emerald-400" : "text-rose-500 dark:text-rose-400")}>
                                {result.activeFactor === 1 ? "1 (Active / Available)" : `0 (${result.disqualificationReason || "Disqualified"})`}
                            </span>
                        </div>

                        {/* Multipliers */}
                        <div className="flex items-center justify-between text-slate-600 dark:text-slate-400">
                            <span className="text-slate-500 dark:text-slate-400">Multipliers:</span>
                            <span className="text-slate-800 dark:text-slate-200 font-medium">
                                {result.mathBreakdown.multiplicationStep}
                            </span>
                        </div>

                        {/* Division */}
                        <div className="flex items-center justify-between text-slate-600 dark:text-slate-400">
                            <span className="text-slate-500 dark:text-slate-400">Division:</span>
                            <span className="text-slate-800 dark:text-slate-200 font-medium">
                                {result.mathBreakdown.divisionStep}
                            </span>
                        </div>

                        {/* Readiness */}
                        <div className="flex items-center justify-between text-slate-600 dark:text-slate-400">
                            <span className="text-slate-500 dark:text-slate-400">Readiness:</span>
                            <span className={cn("font-semibold", result.colorScale.text)}>
                                {result.mathBreakdown.percentageStep}
                            </span>
                        </div>

                        {/* Disqualification / Liveness Cooldown */}
                        {result.disqualificationReason ? (
                            <div className="mt-2 pt-1.5 border-t border-slate-200/80 dark:border-slate-800/80 flex items-center justify-between text-rose-500 dark:text-rose-400 text-[10px]">
                                <span className="font-medium">Disqualification:</span>
                                <span className="font-semibold px-1.5 py-0.5 rounded bg-rose-500/10 border border-rose-500/20">
                                    {result.disqualificationReason}
                                </span>
                            </div>
                        ) : (
                            <div className="mt-2 pt-1.5 border-t border-slate-200/80 dark:border-slate-800/80 flex items-center justify-between text-slate-500 dark:text-slate-400 text-[10px]">
                                <span>Liveness:</span>
                                <span className="font-medium text-slate-700 dark:text-slate-300">{result.mathBreakdown.cooldownSummary}</span>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </td>
    );
}
