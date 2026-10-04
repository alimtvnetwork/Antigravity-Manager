import { useState, useRef, useEffect } from 'react';
import { Palette, Check, ChevronDown, Sparkles } from 'lucide-react';
import { THEME_PALETTES, findPalette } from '../common/themePalettes';
import { useConfigStore } from '../../stores/useConfigStore';

export function ThemeSwitcherDropdown() {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);
    const { config, updateTheme } = useConfigStore();

    const currentThemeId = config?.theme || 'dark';
    const currentPalette = findPalette(currentThemeId);

    // Event bus synchronization: close if another dropdown opens
    useEffect(() => {
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'theme-switcher') {
                setIsOpen(false);
            }
        };

        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        return () => {
            window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        };
    }, []);

    // Outside click and Escape key dismissal
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                setIsOpen(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, []);

    const toggleOpen = () => {
        const nextState = !isOpen;
        setIsOpen(nextState);
        if (nextState) {
            window.dispatchEvent(
                new CustomEvent('agm:dropdown-open', {
                    detail: { source: 'theme-switcher' },
                })
            );
        }
    };

    const handleSelectTheme = (themeId: string) => {
        updateTheme(themeId);
    };

    return (
        <div className="relative" ref={dropdownRef}>
            {/* Trigger Button */}
            <button
                type="button"
                onClick={toggleOpen}
                aria-expanded={isOpen}
                aria-haspopup="true"
                className="h-8 px-2.5 sm:px-3 rounded-full flex items-center gap-1.5 text-xs font-semibold bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 text-gray-800 dark:text-gray-200 transition-all border border-gray-200/60 dark:border-slate-700 shadow-xs cursor-pointer"
                title={`Active Theme: ${currentPalette.label}`}
            >
                <div
                    className="w-3.5 h-3.5 rounded-full border border-black/15 dark:border-white/20 shrink-0"
                    style={{
                        background: `linear-gradient(135deg, ${currentPalette.bg} 50%, ${currentPalette.primary} 50%)`,
                    }}
                />
                <Palette className="w-3.5 h-3.5 text-blue-500 shrink-0" />
                <span className="hidden sm:inline font-medium text-[11px] truncate max-w-[80px]">
                    {currentPalette.label}
                </span>
                <ChevronDown
                    className={`w-3 h-3 text-gray-400 transition-transform duration-150 ${
                        isOpen ? 'rotate-180' : ''
                    }`}
                />
            </button>

            {/* Popover */}
            {isOpen && (
                <div
                    className="absolute top-full left-1/2 -translate-x-1/2 mt-2 w-80 max-w-[calc(100vw-24px)] bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-2xl p-2.5 z-[9999] animate-in fade-in zoom-in-95 duration-150 origin-top"
                    role="menu"
                >
                    {/* Header bar */}
                    <div className="flex items-center justify-between pb-2 mb-2 border-b border-gray-100 dark:border-slate-800">
                        <div className="flex items-center gap-1.5 text-xs font-bold text-gray-800 dark:text-gray-200">
                            <Palette className="w-3.5 h-3.5 text-blue-500" />
                            <span>Theme Catalogue</span>
                            <Sparkles className="w-3 h-3 text-amber-500 ml-0.5" />
                        </div>
                        <span className="text-[10px] font-semibold px-1.5 py-0.5 rounded-full bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border border-blue-200/50 dark:border-blue-800/50">
                            {THEME_PALETTES.length}
                        </span>
                    </div>

                    {/* Grid layout */}
                    <div className="grid grid-cols-2 gap-1.5 max-h-72 overflow-y-auto pr-1">
                        {THEME_PALETTES.map((palette) => {
                            const isActive = currentThemeId === palette.id;
                            return (
                                <button
                                    key={palette.id}
                                    type="button"
                                    onClick={() => handleSelectTheme(palette.id)}
                                    className={`w-full p-1.5 text-left flex items-center justify-between gap-1.5 rounded-[5px] border transition-all duration-150 cursor-pointer text-xs ${
                                        isActive
                                            ? 'border-[var(--primary)] ring-1 ring-[var(--primary)] bg-blue-50/60 dark:bg-blue-900/20 font-semibold'
                                            : 'border-gray-200/70 dark:border-slate-800 hover:border-gray-300 dark:hover:border-slate-700 hover:bg-gray-50/80 dark:hover:bg-slate-800/60'
                                    }`}
                                    title={palette.label}
                                >
                                    <div className="flex items-center gap-1.5 min-w-0">
                                        <div
                                            className="w-5 h-5 rounded-[4px] border border-black/10 dark:border-white/10 shrink-0"
                                            style={{
                                                background: `linear-gradient(135deg, ${palette.bg} 50%, ${palette.primary} 50%)`,
                                            }}
                                        />
                                        <div className="flex flex-col min-w-0">
                                            <span className="truncate text-[11px] leading-tight font-medium text-gray-800 dark:text-gray-200">
                                                {palette.label}
                                            </span>
                                            <span className="text-[9px] text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                                                {palette.dark ? 'Dark' : 'Light'}
                                            </span>
                                        </div>
                                    </div>
                                    {isActive && (
                                        <Check className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400 shrink-0" />
                                    )}
                                </button>
                            );
                        })}
                    </div>
                </div>
            )}
        </div>
    );
}

export default ThemeSwitcherDropdown;
