import { useEffect, useState, useRef } from 'react';
import { Sun, Moon, LogOut, Minus, X, RotateCcw, Globe, ChevronDown, Check, Palette, Sparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { useClickOutside } from './NavDropdowns';
import { LANGUAGES } from './constants';
import { isTauri } from '../../utils/env';
import { useErrorStore } from '../../stores/error-store';
import { useConfigStore } from '../../stores/useConfigStore';
import { THEME_PALETTES, findPalette } from '../common/themePalettes';
import { AgyCleanModal } from '../modals/agy-clean-modal';

interface NavSettingsProps {
    theme: 'light' | 'dark';
    currentLanguage: string;
    onThemeToggle: (event: React.MouseEvent<HTMLButtonElement>) => void;
    onLanguageChange: (langCode: string) => void;
}

export function NavSettings({
    theme,
    currentLanguage,
    onThemeToggle,
    onLanguageChange
}: NavSettingsProps) {
    const { t } = useTranslation();
    const [isMaximized, setIsMaximized] = useState(false);
    const [isCleanModalOpen, setIsCleanModalOpen] = useState(false);
    const [isPrefsOpen, setIsPrefsOpen] = useState(false);
    const [isThemeOpen, setIsThemeOpen] = useState(false);
    const prefsRef = useRef<HTMLDivElement>(null);
    const themeRef = useRef<HTMLDivElement>(null);
    const { config, updateTheme } = useConfigStore();

    useClickOutside(prefsRef, () => setIsPrefsOpen(false));
    useClickOutside(themeRef, () => setIsThemeOpen(false));

    useEffect(() => {
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'nav-settings') {
                setIsPrefsOpen(false);
            }
            if (customEvent.detail?.source !== 'theme-switcher') {
                setIsThemeOpen(false);
            }
            if (customEvent.detail?.source === 'instance-selector' || customEvent.detail?.source === 'theme-switcher') {
                setIsCleanModalOpen(false);
            }
        };
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsPrefsOpen(false);
                setIsCleanModalOpen(false);
                setIsThemeOpen(false);
            }
        };
        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        document.addEventListener('keydown', handleKeyDown);
        return () => {
            window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, []);

    useEffect(() => {
        if (!isTauri()) return;
        const win = getCurrentWindow();
        win.isMaximized().then(setIsMaximized).catch((e) => {
            console.warn('Initial window maximize check notice:', e);
        });

        let unlistenResize: (() => void) | undefined;
        win.onResized(async () => {
            try {
                const max = await win.isMaximized();
                setIsMaximized(max);
            } catch (err) {
                console.warn('Window resize polling notice:', err);
            }
        }).then((fn) => {
            unlistenResize = fn;
        }).catch((err) => {
            console.warn('Window resize listener setup notice:', err);
        });

        return () => {
            if (unlistenResize) {
                unlistenResize();
            }
        };
    }, []);

    const handleMinimize = async () => {
        try {
            await invoke('minimize_window');
        } catch (invokeErr) {
            try {
                await getCurrentWindow().minimize();
            } catch (e) {
                console.error('Failed to minimize window:', e);
                const captured = useErrorStore.getState().captureError(e, {
                    source: 'NavSettings.tsx',
                    triggerComponent: 'NavSettings.WindowControls',
                    triggerAction: 'handleMinimize',
                    targetId: 'btn-window-minimize',
                });
                useErrorStore.getState().openErrorModal(captured);
            }
        }
    };

    const handleToggleMaximize = async () => {
        try {
            const max = await invoke<boolean>('toggle_maximize_window');
            setIsMaximized(max);
        } catch (invokeErr) {
            try {
                const win = getCurrentWindow();
                await win.toggleMaximize();
                const max = await win.isMaximized();
                setIsMaximized(max);
            } catch (e) {
                console.error('Failed to toggle maximize window:', e);
                const captured = useErrorStore.getState().captureError(e, {
                    source: 'NavSettings.tsx',
                    triggerComponent: 'NavSettings.WindowControls',
                    triggerAction: 'handleToggleMaximize',
                    targetId: 'btn-window-maximize',
                });
                useErrorStore.getState().openErrorModal(captured);
            }
        }
    };

    const handleClose = async () => {
        try {
            await invoke('close_window');
        } catch (invokeErr) {
            try {
                await getCurrentWindow().close();
            } catch (e) {
                console.error('Failed to close window:', e);
                const captured = useErrorStore.getState().captureError(e, {
                    source: 'NavSettings.tsx',
                    triggerComponent: 'NavSettings.WindowControls',
                    triggerAction: 'handleClose',
                    targetId: 'btn-window-close',
                });
                useErrorStore.getState().openErrorModal(captured);
            }
        }
    };

    const handleLogout = () => {
        sessionStorage.removeItem('abv_admin_api_key');
        localStorage.removeItem('abv_admin_api_key');
        window.location.reload();
    };

    const currentLangItem = LANGUAGES.find(l => l.code === currentLanguage) || LANGUAGES[0];
    const currentThemeId = config?.theme || 'dark';
    const currentPalette = findPalette(currentThemeId);

    return (
        <div className="flex items-center gap-1 sm:gap-1.5 shrink-0">
            {/* 1. Quick Clean + Theme Switcher + Language Preferences Segmented Pill Capsule */}
            <div className="flex items-center rounded-full bg-gray-100 dark:bg-[#0c2438]/90 border border-gray-200/60 dark:border-[#15334d] p-0.5 shadow-xs">
                {/* Antigravity Quick Clean (Recycle) Icon Button */}
                <button
                    type="button"
                    onClick={() => {
                        setIsPrefsOpen(false);
                        setIsThemeOpen(false);
                        setIsCleanModalOpen(true);
                        window.dispatchEvent(new CustomEvent('agm:dropdown-open', { detail: { source: 'clean-modal' } }));
                    }}
                    className="w-7 h-7 rounded-l-full hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center transition-all duration-150 ease-out cursor-pointer text-gray-700 dark:text-gray-300 border-r border-gray-200/50 dark:border-slate-700/60"
                    title={t('nav.quick_clean', 'Antigravity Cache & Retention Clean')}
                    aria-label="Quick Clean"
                >
                    <RotateCcw className="w-3.5 h-3.5" />
                </button>

                {/* Theme Switcher (Palette icon + swatch dot + dropdown) */}
                <div className="relative" ref={themeRef}>
                    <button
                        type="button"
                        onClick={() => {
                            const next = !isThemeOpen;
                            setIsThemeOpen(next);
                            if (next) {
                                setIsPrefsOpen(false);
                                window.dispatchEvent(new CustomEvent('agm:dropdown-open', { detail: { source: 'theme-switcher' } }));
                            }
                        }}
                        className="h-7 px-2 hover:bg-gray-200 dark:hover:bg-[#15334d] flex items-center gap-1 transition-all duration-150 ease-out cursor-pointer text-[11px] font-semibold text-gray-700 dark:text-gray-300 border-r border-gray-200/50 dark:border-slate-700/60"
                        title={`Active Theme: ${currentPalette.label}`}
                        aria-expanded={isThemeOpen}
                    >
                        <div
                            className="w-2.5 h-2.5 rounded-full border border-black/15 dark:border-white/20 shrink-0"
                            style={{
                                background: `linear-gradient(135deg, ${currentPalette.bg} 50%, ${currentPalette.primary} 50%)`,
                            }}
                        />
                        <Palette className="w-3.5 h-3.5 text-blue-500 shrink-0" />
                        <ChevronDown className={`w-2.5 h-2.5 text-gray-400 transition-transform duration-150 ${isThemeOpen ? 'rotate-180' : ''}`} />
                    </button>

                    {/* Theme Catalogue Dropdown Popover */}
                    {isThemeOpen && (
                        <div
                            className="absolute right-0 mt-1.5 w-72 max-w-[calc(100vw-24px)] bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-2xl p-2.5 z-[9999] animate-in fade-in zoom-in-95 duration-150 origin-top"
                            role="menu"
                        >
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

                            <div className="grid grid-cols-2 gap-1.5 max-h-64 overflow-y-auto pr-1">
                                {THEME_PALETTES.map((palette) => {
                                    const isActive = currentThemeId === palette.id;
                                    return (
                                        <button
                                            key={palette.id}
                                            type="button"
                                            onClick={() => {
                                                updateTheme(palette.id);
                                                setIsThemeOpen(false);
                                            }}
                                            className={`w-full p-1.5 text-left flex items-center justify-between gap-1.5 rounded-[5px] border transition-all duration-150 cursor-pointer text-xs ${
                                                isActive
                                                    ? 'border-[var(--primary)] ring-1 ring-[var(--primary)] bg-blue-50/60 dark:bg-blue-900/20 font-semibold'
                                                    : 'border-gray-200/70 dark:border-slate-800 hover:border-gray-300 dark:hover:border-slate-700 hover:bg-gray-50/80 dark:hover:bg-slate-800/60'
                                            }`}
                                            title={palette.label}
                                        >
                                            <div className="flex items-center gap-1.5 min-w-0">
                                                <div
                                                    className="w-4 h-4 rounded-[3px] border border-black/10 dark:border-white/10 shrink-0"
                                                    style={{
                                                        background: `linear-gradient(135deg, ${palette.bg} 50%, ${palette.primary} 50%)`,
                                                    }}
                                                />
                                                <div className="flex flex-col min-w-0">
                                                    <span className="truncate text-[10px] leading-tight font-medium text-gray-800 dark:text-gray-200">
                                                        {palette.label}
                                                    </span>
                                                    <span className="text-[8px] text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                                                        {palette.dark ? 'Dark' : 'Light'}
                                                    </span>
                                                </div>
                                            </div>
                                            {isActive && (
                                                <Check className="w-3 h-3 text-blue-600 dark:text-blue-400 shrink-0" />
                                            )}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                </div>

                {/* Combined Theme & Language Dropdown Button */}
                <div className="relative" ref={prefsRef}>
                    <button
                        type="button"
                        onClick={() => {
                            const next = !isPrefsOpen;
                            setIsPrefsOpen(next);
                            if (next) {
                                setIsThemeOpen(false);
                                window.dispatchEvent(new CustomEvent('agm:dropdown-open', { detail: { source: 'nav-settings' } }));
                            }
                        }}
                        className="h-7 px-2 rounded-r-full hover:bg-gray-200 dark:hover:bg-[#15334d] flex items-center gap-1 transition-all duration-150 ease-out cursor-pointer text-[11px] font-semibold text-gray-700 dark:text-gray-300"
                        title="Theme & Language Preferences"
                    >
                        {theme === 'light' ? (
                            <Sun className="w-3 h-3 text-amber-500 shrink-0" />
                        ) : (
                            <Moon className="w-3 h-3 text-blue-400 shrink-0" />
                        )}
                        <span className="uppercase">{currentLangItem.short}</span>
                        <ChevronDown className={`w-2.5 h-2.5 text-gray-400 transition-transform duration-150 ${isPrefsOpen ? 'rotate-180' : ''}`} />
                    </button>

                    {isPrefsOpen && (
                        <div className="absolute right-0 mt-1.5 w-52 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-2xl py-1.5 z-[9999] text-xs animate-in fade-in zoom-in-95">
                            <div className="flex items-center justify-between px-3 pb-1.5 mb-1 border-b border-gray-100 dark:border-slate-800">
                                <span className="text-[10px] font-bold uppercase tracking-wider text-gray-400">
                                    {t('nav.preferences', 'Preferences')}
                                </span>
                                <button
                                    type="button"
                                    onClick={() => setIsPrefsOpen(false)}
                                    className="p-0.5 rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
                                    title={t('common.close', 'Close')}
                                >
                                    <X className="w-3.5 h-3.5" />
                                </button>
                            </div>
                            <div className="px-3 py-1 text-[10px] font-bold uppercase tracking-wider text-gray-400">
                                Appearance
                            </div>
                            <button
                                type="button"
                                onClick={(e) => {
                                    onThemeToggle(e);
                                    setIsPrefsOpen(false);
                                }}
                                className="w-full px-3 py-1.5 text-left flex items-center justify-between hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-gray-200 transition-colors cursor-pointer"
                            >
                                <span className="flex items-center gap-2">
                                    {theme === 'light' ? <Moon className="w-3.5 h-3.5 text-blue-500" /> : <Sun className="w-3.5 h-3.5 text-amber-500" />}
                                    <span>{theme === 'light' ? t('nav.theme_to_dark', 'Switch to Dark Mode') : t('nav.theme_to_light', 'Switch to Light Mode')}</span>
                                </span>
                            </button>

                            <div className="my-1 border-t border-gray-100 dark:border-slate-800" />
                            <div className="px-3 py-1 text-[10px] font-bold uppercase tracking-wider text-gray-400 flex items-center gap-1">
                                <Globe className="w-3 h-3" />
                                <span>Language</span>
                            </div>
                            <div className="max-h-56 overflow-y-auto">
                                {LANGUAGES.map((lang) => {
                                    const isCurrent = lang.code === currentLanguage;
                                    return (
                                        <button
                                            key={lang.code}
                                            type="button"
                                            onClick={() => {
                                                onLanguageChange(lang.code);
                                                setIsPrefsOpen(false);
                                            }}
                                            className={`w-full px-3 py-1.5 text-left flex items-center justify-between hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors cursor-pointer ${
                                                isCurrent ? 'text-blue-600 dark:text-blue-400 font-semibold bg-blue-50/50 dark:bg-blue-900/20' : 'text-gray-700 dark:text-gray-300'
                                            }`}
                                        >
                                            <span>{lang.label}</span>
                                            {isCurrent && <Check className="w-3.5 h-3.5 shrink-0" />}
                                        </button>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Window controls (Minimize, Maximize/Restore, Close) - Tauri only: Permanently visible, never collapsed */}
            {isTauri() && (
                <div className="flex items-center rounded-full bg-gray-100 dark:bg-[#0c2438]/90 border border-gray-200/60 dark:border-[#15334d] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs z-50 shrink-0">
                    <button
                        type="button"
                        id="btn-window-minimize"
                        name="window-minimize"
                        data-xpath="//*[@id='btn-window-minimize']"
                        onClick={handleMinimize}
                        className="w-7 h-7 rounded-l-full hover:bg-gray-200 dark:hover:bg-[#15334d] flex items-center justify-center transition-all duration-150 ease-out cursor-pointer"
                        title={t('common.minimize', 'Minimize')}
                        aria-label="Minimize"
                    >
                        <Minus className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300 pointer-events-none" />
                    </button>

                    <button
                        type="button"
                        id="btn-window-maximize"
                        name="window-maximize"
                        data-xpath="//*[@id='btn-window-maximize']"
                        onClick={handleToggleMaximize}
                        className="w-7 h-7 hover:bg-gray-200 dark:hover:bg-[#15334d] flex items-center justify-center transition-all duration-150 ease-out cursor-pointer"
                        title={isMaximized ? t('common.restore', 'Restore') : t('common.maximize', 'Maximize')}
                        aria-label={isMaximized ? "Restore" : "Maximize"}
                    >
                        {isMaximized ? (
                            <svg className="w-3 h-3 text-gray-700 dark:text-gray-300 pointer-events-none" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <rect x="4.5" y="1.5" width="10" height="10" rx="1" />
                                <path d="M1.5 5.5v9h9" />
                            </svg>
                        ) : (
                            <svg className="w-3 h-3 text-gray-700 dark:text-gray-300 pointer-events-none" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <rect x="2" y="2" width="12" height="12" rx="1.5" />
                            </svg>
                        )}
                    </button>

                    <button
                        type="button"
                        id="btn-window-close"
                        name="window-close"
                        data-xpath="//*[@id='btn-window-close']"
                        onClick={handleClose}
                        className="w-7 h-7 rounded-r-full hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white flex items-center justify-center transition-all duration-150 ease-out cursor-pointer group"
                        title={t('common.close', 'Close')}
                        aria-label="Close"
                    >
                        <X className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300 group-hover:text-white transition-colors duration-150 pointer-events-none" />
                    </button>
                </div>
            )}

            {/* Logout button - Web mode only */}
            {!isTauri() && (
                <button
                    onClick={handleLogout}
                    className="w-7 h-7 rounded-full bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/40 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                    title={t('nav.logout', 'Logout')}
                >
                    <LogOut className="w-3.5 h-3.5 text-red-600 dark:text-red-400" />
                </button>
            )}

            {/* Quick Clean & Retention Modal */}
            <AgyCleanModal
                isOpen={isCleanModalOpen}
                onClose={() => setIsCleanModalOpen(false)}
            />
        </div>
    );
}
