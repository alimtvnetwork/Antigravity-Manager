import { useState, useRef, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { ChevronDown, MoreVertical, Sun, Moon, LogOut, Minimize2, Sparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { NavItem, Language } from './constants';
import { isTauri } from '../../utils/env';
import { useViewStore } from '../../stores/useViewStore';

// useClickOutside Hook
export function useClickOutside(
    ref: React.RefObject<HTMLElement | null>,
    handler: () => void
) {
    useEffect(() => {
        const listener = (event: MouseEvent) => {
            if (!ref.current || ref.current.contains(event.target as Node)) {
                return;
            }
            handler();
        };

        document.addEventListener('mousedown', listener);
        return () => document.removeEventListener('mousedown', listener);
    }, [ref, handler]);
}

// Language dropdown component
interface LanguageDropdownProps {
    currentLanguage: string;
    languages: Language[];
    onLanguageChange: (langCode: string) => void;
    className?: string;
}

export function LanguageDropdown({
    currentLanguage,
    languages,
    onLanguageChange,
    className = ''
}: LanguageDropdownProps) {
    const [isOpen, setIsOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);
    const { t } = useTranslation();

    useClickOutside(menuRef, () => setIsOpen(false));

    const handleLanguageChange = (langCode: string) => {
        onLanguageChange(langCode);
        setIsOpen(false);
    };

    return (
        <div className={`relative ${className}`} ref={menuRef}>
            <button
                type="button"
                onClick={() => setIsOpen(!isOpen)}
                className="w-10 h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-colors cursor-pointer"
                title={t('settings.general.language')}
                aria-label="Language selection"
            >
                <span className="text-sm font-bold text-gray-700 dark:text-gray-300">
                    {languages.find(l => l.code === currentLanguage)?.short || 'EN'}
                </span>
            </button>

            {/* Dropdown menu */}
            {isOpen && (
                <div className="absolute ltr:right-0 rtl:left-0 mt-2 w-32 max-w-[calc(100vw-32px)] z-50 bg-white dark:bg-slate-900 rounded-xl shadow-lg border border-gray-100 dark:border-slate-800 py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-200 ltr:origin-top-right rtl:origin-top-left">
                    {languages.map((lang) => (
                        <button
                            key={lang.code}
                            onClick={() => handleLanguageChange(lang.code)}
                            className={`w-full px-4 py-2 text-left text-sm flex items-center justify-between hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors ${currentLanguage === lang.code
                                ? 'text-blue-500 font-medium bg-blue-50 dark:bg-blue-900/10'
                                : 'text-gray-700 dark:text-gray-300'
                                }`}
                        >
                            <div className="flex items-center gap-3">
                                <span className="font-mono font-bold w-6">{lang.short}</span>
                                <span className="text-xs opacity-70">{lang.label}</span>
                            </div>
                            {currentLanguage === lang.code && (
                                <span className="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
                            )}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}

// Navigation dropdown component (< 375px)
interface NavigationDropdownProps {
    navItems: NavItem[];
    isActive: (path: string) => boolean;
    getCurrentNavItem: () => NavItem | undefined;
    onNavigate: () => void;
    showLabel?: boolean; // Whether to display label
}

export function NavigationDropdown({
    navItems,
    isActive,
    getCurrentNavItem,
    onNavigate,
    showLabel = true // Default true
}: NavigationDropdownProps) {
    const [isOpen, setIsOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    useClickOutside(menuRef, () => setIsOpen(false));

    const handleNavItemClick = () => {
        setIsOpen(false);
        onNavigate();
    };

    const currentItem = getCurrentNavItem();
    const CurrentIcon = currentItem?.icon;

    // Return null if no active item
    if (!currentItem || !CurrentIcon) return null;

    return (
        <div className="relative" ref={menuRef}>
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="flex items-center gap-2 px-3 py-2 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors"
            >
                <CurrentIcon className="w-4 h-4 text-gray-700 dark:text-gray-300" />
                {/* Control text display according to showLabel */}
                {showLabel && (
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {currentItem.label}
                    </span>
                )}
                <ChevronDown className={`w-3 h-3 text-gray-700 dark:text-gray-300 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Dropdown menu */}
            {isOpen && (
                <div className="absolute left-1/2 -translate-x-1/2 mt-2 w-48 bg-white dark:bg-slate-900 rounded-xl shadow-xl border-2 border-gray-200 dark:border-slate-800 py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-200 origin-top">
                    {navItems.map((item) => (
                        <Link
                            key={item.path}
                            to={item.path}
                            draggable="false"
                            onClick={handleNavItemClick}
                            className={`w-full px-4 py-2.5 text-left text-sm flex items-center gap-3 hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors ${isActive(item.path)
                                ? 'text-blue-500 font-medium bg-blue-50 dark:bg-blue-900/10'
                                : 'text-gray-700 dark:text-gray-300'
                                }`}
                        >
                            <item.icon className="w-4 h-4" />
                            <span>{item.label}</span>
                        </Link>
                    ))}
                </div>
            )}
        </div>
    );
}

// More menu component (< 480px)
interface MoreDropdownProps {
    theme: 'light' | 'dark';
    currentLanguage: string;
    languages: Language[];
    onThemeToggle: (event: React.MouseEvent<HTMLButtonElement>) => void;
    onLanguageChange: (langCode: string) => void;
    onOpenCleanModal?: () => void;
}

export function MoreDropdown({
    theme,
    currentLanguage,
    languages,
    onThemeToggle,
    onLanguageChange,
    onOpenCleanModal
}: MoreDropdownProps) {
    const [isOpen, setIsOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);
    const { t } = useTranslation();
    const { setMiniView } = useViewStore();

    useClickOutside(menuRef, () => setIsOpen(false));

    const handleThemeToggle = (event: React.MouseEvent<HTMLButtonElement>) => {
        onThemeToggle(event);
        setIsOpen(false);
    };

    const handleLanguageChange = (langCode: string) => {
        onLanguageChange(langCode);
        setIsOpen(false);
    };

    const handleLogout = () => {
        sessionStorage.removeItem('abv_admin_api_key');
        localStorage.removeItem('abv_admin_api_key');
        window.location.reload();
    };

    return (
        <div className="relative" ref={menuRef}>
            <button
                type="button"
                onClick={() => setIsOpen(!isOpen)}
                className="w-10 h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-colors cursor-pointer"
                title={t('nav.more', 'More')}
                aria-label="More options"
            >
                <MoreVertical className="w-5 h-5 text-gray-700 dark:text-gray-300" />
            </button>

            {/* Dropdown menu */}
            {isOpen && (
                <div className="absolute ltr:right-0 rtl:left-0 mt-2 w-44 max-w-[calc(100vw-32px)] z-50 bg-white dark:bg-slate-900 rounded-xl shadow-lg border border-gray-100 dark:border-slate-800 py-1 overflow-hidden animate-in fade-in zoom-in-95 duration-200 ltr:origin-top-right rtl:origin-top-left">
                    {/* Quick Clean */}
                    {onOpenCleanModal && (
                        <button
                            onClick={() => {
                                onOpenCleanModal();
                                setIsOpen(false);
                            }}
                            className="w-full px-4 py-2.5 text-left text-sm flex items-center gap-3 hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors text-gray-700 dark:text-gray-300"
                        >
                            <Sparkles className="w-4 h-4 text-blue-500" />
                            <span>{t('nav.quick_clean', 'Quick Clean')}</span>
                        </button>
                    )}

                    {/* Mini view */}
                    <button
                        onClick={() => {
                            setMiniView(true);
                            setIsOpen(false);
                        }}
                        className="w-full px-4 py-2.5 text-left text-sm flex items-center gap-3 hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors text-gray-700 dark:text-gray-300"
                    >
                        <Minimize2 className="w-4 h-4" />
                        <span>{t('nav.mini_view', 'Mini View')}</span>
                    </button>

                    {/* Theme switch */}
                    <button
                        onClick={handleThemeToggle}
                        className="w-full px-4 py-2.5 text-left text-sm flex items-center gap-3 hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors text-gray-700 dark:text-gray-300"
                    >
                        {theme === 'light' ? (
                            <Moon className="w-4 h-4" />
                        ) : (
                            <Sun className="w-4 h-4" />
                        )}
                        <span>{theme === 'light' ? t('nav.theme_to_dark') : t('nav.theme_to_light')}</span>
                    </button>

                    {/* Divider */}
                    <div className="my-1 border-t border-gray-100 dark:border-slate-800"></div>

                    {/* Language selection */}
                    {languages.map((lang) => (
                        <button
                            key={lang.code}
                            onClick={() => handleLanguageChange(lang.code)}
                            className={`w-full px-4 py-2 text-left text-sm flex items-center justify-between hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors ${currentLanguage === lang.code
                                ? 'text-blue-500 font-medium bg-blue-50 dark:bg-blue-900/10'
                                : 'text-gray-700 dark:text-gray-300'
                                }`}
                        >
                            <div className="flex items-center gap-2">
                                <span className="font-mono font-bold text-xs">{lang.short}</span>
                                <span className="text-xs opacity-70">{lang.label}</span>
                            </div>
                            {currentLanguage === lang.code && (
                                <span className="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
                            )}
                        </button>
                    ))}

                    {/* Logout button - Web mode only */}
                    {!isTauri() && (
                        <>
                            <div className="my-1 border-t border-gray-100 dark:border-slate-800"></div>
                            <button
                                onClick={handleLogout}
                                className="w-full px-4 py-2.5 text-left text-sm flex items-center gap-3 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors text-red-600 dark:text-red-400"
                            >
                                <LogOut className="w-4 h-4" />
                                <span>{t('nav.logout', 'Logout')}</span>
                            </button>
                        </>
                    )}
                </div>
            )}
        </div>
    );
}
