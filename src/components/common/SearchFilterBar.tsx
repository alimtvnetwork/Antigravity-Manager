import React, { useRef, useEffect } from 'react';
import { Search, X, SlidersHorizontal } from 'lucide-react';

export interface SearchFilterBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  onClear?: () => void;
  onFilterClick?: () => void;
  isFilterActive?: boolean;
  filterCount?: number;
  shortcutKey?: string;
  enableShortcut?: boolean;
  autoFocus?: boolean;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export const SearchFilterBar: React.FC<SearchFilterBarProps> = ({
  value,
  onChange,
  placeholder = 'Search...',
  onClear,
  onFilterClick,
  isFilterActive = false,
  filterCount = 0,
  shortcutKey = 'k',
  enableShortcut = true,
  autoFocus = false,
  className = '',
  size = 'md',
}) => {
  const inputRef = useRef<HTMLInputElement | null>(null);

  const isMac =
    typeof navigator !== 'undefined' &&
    navigator.platform &&
    navigator.platform.toUpperCase().indexOf('MAC') >= 0;

  const shortcutBadge = isMac ? `⌘${shortcutKey.toUpperCase()}` : `Ctrl+${shortcutKey.toUpperCase()}`;

  useEffect(() => {
    if (!enableShortcut) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === shortcutKey.toLowerCase()) {
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [enableShortcut, shortcutKey]);

  const handleClear = () => {
    onChange('');
    if (onClear) onClear();
    inputRef.current?.focus();
  };

  const getHeightClass = () => {
    switch (size) {
      case 'sm':
        return 'h-8 text-xs';
      case 'lg':
        return 'h-11 text-sm';
      case 'md':
      default:
        return 'h-9 text-xs md:text-sm';
    }
  };

  return (
    <div
      className={`relative flex items-center w-full bg-slate-900/60 dark:bg-slate-950/70 border border-slate-700/60 rounded-xl px-2.5 py-1 backdrop-blur-md transition-all focus-within:border-sky-500/80 focus-within:ring-1 focus-within:ring-sky-500/50 ${getHeightClass()} ${className}`}
    >
      {/* Search Icon */}
      <Search className="w-4 h-4 text-slate-400 dark:text-slate-400 shrink-0 mr-2" />

      {/* Input */}
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        autoFocus={autoFocus}
        className="w-full bg-transparent border-0 p-0 text-slate-200 dark:text-slate-200 placeholder-slate-400 dark:placeholder-slate-400 focus:outline-none focus:ring-0"
      />

      <div className="flex items-center gap-1.5 shrink-0 ml-2">
        {/* Clear Button */}
        {value.length > 0 && (
          <button
            type="button"
            onClick={handleClear}
            className="p-1 text-slate-400 hover:text-slate-200 rounded-md hover:bg-slate-800/60 transition-colors"
            title="Clear search"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}

        {/* Keyboard Shortcut Indicator */}
        {enableShortcut && value.length === 0 && (
          <kbd className="hidden sm:inline-flex items-center px-1.5 py-0.5 text-[10px] font-mono text-slate-400 dark:text-slate-400 bg-slate-800/80 dark:bg-slate-900/80 border border-slate-700/80 rounded">
            {shortcutBadge}
          </kbd>
        )}

        {/* Filter Toggle Button */}
        {onFilterClick && (
          <button
            type="button"
            onClick={onFilterClick}
            className={`flex items-center gap-1 px-2 py-1 text-xs rounded-lg transition-colors border ${
              isFilterActive
                ? 'bg-sky-500/20 text-sky-300 border-sky-500/50'
                : 'text-slate-400 dark:text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 border-slate-700/60'
            }`}
            title="Filter options"
          >
            <SlidersHorizontal className="w-3.5 h-3.5" />
            {filterCount > 0 && (
              <span className="flex items-center justify-center w-4 h-4 text-[10px] font-bold text-white bg-sky-500 rounded-full">
                {filterCount}
              </span>
            )}
          </button>
        )}
      </div>
    </div>
  );
};
