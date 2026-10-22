import { Zap } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface TokenCounterPillProps {
    promptTokens?: number;
    completionTokens?: number;
    totalTokens?: number;
    maxTokens?: number;
    className?: string;
}

export function formatTokenNumber(num: number): string {
    if (num >= 1_000_000) {
        return `${(num / 1_000_000).toFixed(1)}M`;
    }
    if (num >= 1_000) {
        return `${(num / 1_000).toFixed(1)}k`;
    }
    return num.toString();
}

export function TokenCounterPill({
    promptTokens = 0,
    completionTokens = 0,
    totalTokens,
    maxTokens,
    className,
}: TokenCounterPillProps) {
    const total = totalTokens !== undefined ? totalTokens : promptTokens + completionTokens;
    const utilization = maxTokens && maxTokens > 0 ? (total / maxTokens) * 100 : null;

    const isWarning = utilization !== null && utilization >= 80 && utilization < 95;
    const isCritical = utilization !== null && utilization >= 95;

    return (
        <span
            title={`Prompt: ${promptTokens.toLocaleString()} | Completion: ${completionTokens.toLocaleString()} | Total: ${total.toLocaleString()}${
                maxTokens ? ` / ${maxTokens.toLocaleString()}` : ''
            }`}
            className={cn(
                'inline-flex items-center gap-1 px-2 py-0.5 rounded-full font-mono text-[10px] font-semibold border select-none transition-colors',
                !isWarning && !isCritical && 'bg-slate-500/10 border-slate-500/20 text-slate-600 dark:text-slate-300',
                isWarning && 'bg-amber-500/15 border-amber-500/30 text-amber-700 dark:text-amber-300',
                isCritical && 'bg-rose-500/15 border-rose-500/30 text-rose-700 dark:text-rose-300 animate-pulse',
                className
            )}
        >
            <Zap className="w-2.5 h-2.5 shrink-0 opacity-70" />
            <span>{formatTokenNumber(total)}</span>
            {maxTokens && <span className="opacity-50">/ {formatTokenNumber(maxTokens)}</span>}
        </span>
    );
}

export default TokenCounterPill;
