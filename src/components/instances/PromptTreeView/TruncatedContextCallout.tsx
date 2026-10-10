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

    return (
        <div
            onClick={onExpandFull}
            className="my-3 flex items-center justify-between px-3.5 py-2 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-200 text-xs shadow-xs backdrop-blur-xs cursor-pointer hover:bg-amber-500/20 transition-all"
            title="Click to inspect/expand full un-truncated context"
        >
            <div className="flex items-center gap-2 font-mono font-medium">
                <span className="text-amber-500 text-sm">⚡</span>
                <span>Omitted {formattedSize} transcript context · Click to expand</span>
            </div>
            {onExpandFull && (
                <button
                    type="button"
                    onClick={(e) => {
                        e.stopPropagation();
                        onExpandFull();
                    }}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-full bg-amber-500 text-white dark:text-slate-900 font-bold hover:bg-amber-600 transition-colors text-[10px] cursor-pointer"
                >
                    <span>Expand Full</span>
                    <ChevronDown className="w-3 h-3" />
                </button>
            )}
        </div>
    );
}

// Helper to standardize and format clean sequence codes (#P001, C001)
