import React, { useRef, useEffect } from 'react';
import { Search, X } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface SearchInputProps {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    className?: string;
    shortcutKey?: string;
    autoFocus?: boolean;
}

export function SearchInput({
    value,
    onChange,
    placeholder = 'Search...',
    className,
    shortcutKey = '/',
    autoFocus = false,
}: SearchInputProps) {
    const inputRef = useRef<HTMLInputElement | null>(null);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === shortcutKey && document.activeElement !== inputRef.current) {
                // Focus search input unless user is typing in another input
                const tag = document.activeElement?.tagName.toLowerCase();
                if (tag !== 'input' && tag !== 'textarea') {
                    e.preventDefault();
                    inputRef.current?.focus();
                }
            }
        };

        if (shortcutKey) {
            window.addEventListener('keydown', handleKeyDown);
        }
        return () => {
            if (shortcutKey) {
                window.removeEventListener('keydown', handleKeyDown);
            }
        };
    }, [shortcutKey]);

    const handleClear = () => {
        onChange('');
        inputRef.current?.focus();
    };

    return (
        <div
            className={cn(
                'relative flex items-center rounded-xl border transition-all duration-150',
                'bg-white/80 dark:bg-slate-900/80 backdrop-blur-md',
                'border-slate-200 dark:border-slate-800 focus-within:border-purple-500/80 focus-within:ring-2 focus-within:ring-purple-500/20',
                className
            )}
        >
            <Search className="w-3.5 h-3.5 ml-3 text-slate-400 shrink-0 pointer-events-none" />

            <input
                ref={inputRef}
                type="text"
                value={value}
                autoFocus={autoFocus}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder}
                className="w-full pl-2 pr-7 py-1.5 text-xs bg-transparent border-0 focus:ring-0 focus:outline-none text-slate-800 dark:text-slate-100 placeholder:text-slate-400"
            />

            {value ? (
                <button
                    type="button"
                    onClick={handleClear}
                    className="absolute right-2 p-0.5 rounded-full text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors"
                >
                    <X className="w-3 h-3" />
                </button>
            ) : (
                shortcutKey && (
                    <kbd className="absolute right-2.5 px-1.5 py-0.2 font-mono text-[9px] font-semibold text-slate-400 dark:text-slate-500 bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700/60 rounded">
                        {shortcutKey}
                    </kbd>
                )
            )}
        </div>
    );
}

export default SearchInput;
