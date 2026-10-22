import { ChevronDown } from 'lucide-react';

export interface TruncatedContextCalloutProps {
    omittedBytes?: number;
    omittedLines?: number;
    fullText?: string;
    onExpandFull?: () => void;
}

export function TruncatedContextCallout({
    omittedBytes,
    omittedLines,
    onExpandFull,
}: TruncatedContextCalloutProps) {
    const formattedSize = omittedBytes 
        ? omittedBytes >= 1024 * 1024 
            ? `${(omittedBytes / (1024 * 1024)).toFixed(1)} MB` 
            : `${(omittedBytes / 1024).toFixed(1)} KB`
        : omittedLines 
            ? `${omittedLines} lines`
            : 'transcript context';

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onExpandFull?.();
        }
    };

    return (
        <div
            role="button"
            tabIndex={0}
            onClick={onExpandFull}
            onKeyDown={handleKeyDown}
            className="my-3 flex items-center justify-between px-3.5 py-2 rounded-xl bg-amber-500/10 dark:bg-amber-950/40 border border-amber-500/30 dark:border-amber-600/40 text-amber-900 dark:text-amber-100 text-xs shadow-sm backdrop-blur-md cursor-pointer hover:bg-amber-500/20 dark:hover:bg-amber-900/40 focus:outline-none focus:ring-2 focus:ring-amber-500/50 transition-all select-none"
            title="Click to inspect/expand full un-truncated context"
            aria-label={`Omitted ${formattedSize} transcript context. Click to expand full context.`}
        >
            <div className="flex items-center gap-2 font-mono font-medium">
                <span className="text-amber-500 text-sm">⚡</span>
                <span>Omitted {formattedSize} transcript context · Click to expand</span>
            </div>
            {onExpandFull && (
                <button
                    type="button"
                    tabIndex={-1}
                    onClick={(e) => {
                        e.stopPropagation();
                        onExpandFull();
                    }}
                    aria-label="Expand full transcript context"
                    className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-amber-500 text-white dark:text-slate-900 font-bold hover:bg-amber-600 transition-colors text-[10px] cursor-pointer focus:outline-none focus:ring-2 focus:ring-amber-400"
                >
                    <span>Expand Full</span>
                    <ChevronDown className="w-3 h-3" />
                </button>
            )}
        </div>
    );
}

// Helper to standardize and format clean sequence codes (#P001, C001)
