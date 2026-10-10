import type { RefObject } from 'react';
import type { TFunction } from 'i18next';
import {
    Search,
    ChevronUp,
    ChevronDown,
    X,
    WrapText,
    CaseSensitive,
} from 'lucide-react';

export interface ViewerToolbarProps {
    t: TFunction;
    searchTerm: string;
    setSearchTerm: (v: string) => void;
    caseSensitive: boolean;
    setCaseSensitive: (v: boolean) => void;
    isWrap: boolean;
    setIsWrap: (v: boolean) => void;
    matchesCount: number;
    currentMatchIndex: number;
    onNext: () => void;
    onPrev: () => void;
    searchInputRef: RefObject<HTMLInputElement | null>;
}

export function ViewerToolbar({
    t,
    searchTerm,
    setSearchTerm,
    caseSensitive,
    setCaseSensitive,
    isWrap,
    setIsWrap,
    matchesCount,
    currentMatchIndex,
    onNext,
    onPrev,
    searchInputRef,
}: ViewerToolbarProps) {
    return (
        <div className="px-2.5 py-1.5 bg-gray-100/70 dark:bg-base-300/40 border-b border-gray-200 dark:border-base-300 flex items-center gap-1.5 shrink-0">
            <div className="relative flex-1 min-w-0 flex items-center">
                <Search size={12} className="absolute left-2 text-gray-400 pointer-events-none" />
                <input
                    ref={searchInputRef}
                    type="text"
                    placeholder={t('monitor.details.search_placeholder', 'Search payload... (Enter: next, Shift+Enter: previous)')}
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                            e.preventDefault();
                            if (e.shiftKey) {
                                onPrev();
                            } else {
                                onNext();
                            }
                        } else if (e.key === 'Escape') {
                            setSearchTerm('');
                            searchInputRef.current?.blur();
                        } else if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F')) {
                            e.preventDefault();
                            e.stopPropagation();
                            searchInputRef.current?.select();
                        }
                    }}
                    className="input input-xs input-bordered w-full pl-6 pr-6 text-[11px] h-7 bg-white dark:bg-base-100 border-gray-200 dark:border-base-300 text-gray-800 dark:text-gray-200 rounded-md focus:border-blue-500 font-mono"
                />
                {searchTerm && (
                    <button
                        type="button"
                        onClick={() => setSearchTerm('')}
                        className="absolute right-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-0.5"
                        title="Clear search"
                    >
                        <X size={12} />
                    </button>
                )}
            </div>

            {/* Case-Sensitive Toggle */}
            <button
                type="button"
                onClick={() => setCaseSensitive(!caseSensitive)}
                className={`h-7 px-1.5 rounded-md border text-[10px] font-bold flex items-center gap-0.5 transition-colors cursor-pointer select-none ${
                    caseSensitive
                        ? 'bg-blue-100 text-blue-700 border-blue-300 dark:bg-blue-900/40 dark:text-blue-300 dark:border-blue-800'
                        : 'bg-white dark:bg-base-100 text-gray-400 border-gray-200 dark:border-base-300 hover:text-gray-600 dark:hover:text-gray-300'
                }`}
                title={caseSensitive ? 'Case sensitive enabled' : 'Toggle case sensitive'}
            >
                <CaseSensitive size={13} />
            </button>

            {/* Wrap / No-Wrap Toggle */}
            <button
                type="button"
                onClick={() => setIsWrap(!isWrap)}
                className={`h-7 px-1.5 rounded-md border text-[10px] font-medium flex items-center gap-1 transition-colors cursor-pointer select-none ${
                    isWrap
                        ? 'bg-gray-200/80 text-gray-800 border-gray-300 dark:bg-base-200 dark:text-gray-200 dark:border-base-300'
                        : 'bg-white dark:bg-base-100 text-gray-400 border-gray-200 dark:border-base-300 hover:text-gray-600 dark:hover:text-gray-300'
                }`}
                title={isWrap ? 'Word wrap enabled (click to toggle)' : 'Single line horizontal scroll (click to toggle wrap)'}
            >
                <WrapText size={12} />
                <span className="hidden sm:inline text-[9px]">{isWrap ? 'Wrap' : 'No Wrap'}</span>
            </button>

            {/* Match Counter & Prev/Next Controls */}
            {searchTerm.trim() && (
                <div className="flex items-center gap-1 shrink-0 bg-white dark:bg-base-100 border border-gray-200 dark:border-base-300 rounded-md px-1.5 py-0.5 h-7">
                    <span className={`text-[10px] font-mono font-bold ${
                        matchesCount > 0
                            ? 'text-amber-600 dark:text-amber-400'
                            : 'text-gray-400'
                    }`}>
                        {matchesCount > 0 ? `${currentMatchIndex + 1}/${matchesCount}` : 'No matches'}
                    </span>
                    <div className="flex items-center">
                        <button
                            type="button"
                            onClick={onPrev}
                            disabled={matchesCount <= 1}
                            className="btn btn-ghost btn-xs p-0.5 h-5 min-h-0 text-gray-500 dark:text-gray-400 disabled:opacity-30"
                            title="Previous match (Shift+Enter)"
                        >
                            <ChevronUp size={12} />
                        </button>
                        <button
                            type="button"
                            onClick={onNext}
                            disabled={matchesCount <= 1}
                            className="btn btn-ghost btn-xs p-0.5 h-5 min-h-0 text-gray-500 dark:text-gray-400 disabled:opacity-30"
                            title="Next match (Enter)"
                        >
                            <ChevronDown size={12} />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
