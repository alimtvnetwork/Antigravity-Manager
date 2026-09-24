import { useEffect, useState, useRef } from 'react';
import { Sun, Moon, LogOut, Minus, X, RotateCcw, Globe, ChevronDown, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { useClickOutside } from './NavDropdowns';
import { LANGUAGES } from './constants';
import { isTauri } from '../../utils/env';
import { useErrorStore } from '../../stores/error-store';
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
    const prefsRef = useRef<HTMLDivElement>(null);

    useClickOutside(prefsRef, () => setIsPrefsOpen(false));

    useEffect(() => {
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'nav-settings') {
                setIsPrefsOpen(false);
            }
        };
        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        return () => window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
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

    return (
        <div className="flex items-center gap-1.5 md:gap-2 shrink-0">
            {/* 1. Antigravity Quick Clean (Recycle) Icon Button */}
            <button
                type="button"
                onClick={() => setIsCleanModalOpen(true)}
                className="w-9 h-9 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer text-gray-700 dark:text-gray-300"
                title={t('nav.quick_clean', 'Antigravity Cache & Retention Clean')}
                aria-label="Quick Clean"
            >
                <RotateCcw className="w-4 h-4" />
            </button>

            {/* 2. Combined Theme & Language Dropdown Button */}
            <div className="relative" ref={prefsRef}>
                <button
                    type="button"
                    onClick={() => {
                        const next = !isPrefsOpen;
                        setIsPrefsOpen(next);
                        if (next) {
                            window.dispatchEvent(new CustomEvent('agm:dropdown-open', { detail: { source: 'nav-settings' } }));
                        }
                    }}
                    className="h-9 px-2.5 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center gap-1.5 transition-all duration-150 ease-out shadow-xs cursor-pointer text-xs font-semibold text-gray-700 dark:text-gray-300 border border-gray-200/60 dark:border-slate-700"
                    title="Theme & Language Preferences"
                >
                    {theme === 'light' ? (
                        <Sun className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                    ) : (
                        <Moon className="w-3.5 h-3.5 text-blue-400 shrink-0" />
                    )}
                    <span className="uppercase">{currentLangItem.short}</span>
                    <ChevronDown className={`w-3 h-3 text-gray-400 transition-transform duration-150 ${isPrefsOpen ? 'rotate-180' : ''}`} />
                </button>

                {isPrefsOpen && (
                    <div className="absolute right-0 mt-1.5 w-48 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-2xl py-1.5 z-[9999] text-xs animate-in fade-in zoom-in-95">
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

            {/* Window controls (Minimize, Maximize/Restore, Close) - Tauri only: Permanently visible, never collapsed */}
            {isTauri() && (
                <div className="flex items-center gap-1 md:gap-1.5 shrink-0 z-50">
                    <button
                        type="button"
                        id="btn-window-minimize"
                        name="window-minimize"
                        data-xpath="//*[@id='btn-window-minimize']"
                        onClick={handleMinimize}
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                        title={t('common.minimize', 'Minimize')}
                        aria-label="Minimize"
                    >
                        <Minus className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300 pointer-events-none" />
                    </button>

                    <button
                        type="button"
                        id="btn-window-maximize"
                        name="window-maximize"
                        data-xpath="//*[@id='btn-window-maximize']"
                        onClick={handleToggleMaximize}
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                        title={isMaximized ? t('common.restore', 'Restore') : t('common.maximize', 'Maximize')}
                        aria-label={isMaximized ? "Restore" : "Maximize"}
                    >
                        {isMaximized ? (
                            <svg className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300 pointer-events-none" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <rect x="4.5" y="1.5" width="10" height="10" rx="1" />
                                <path d="M1.5 5.5v9h9" />
                            </svg>
                        ) : (
                            <svg className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300 pointer-events-none" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
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
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer group"
                        title={t('common.close', 'Close')}
                        aria-label="Close"
                    >
                        <X className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300 group-hover:text-white transition-colors duration-150 pointer-events-none" />
                    </button>
                </div>
            )}

            {/* Logout button - Web mode only */}
            {!isTauri() && (
                <button
                    onClick={handleLogout}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/40 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                    title={t('nav.logout', 'Logout')}
                >
                    <LogOut className="w-4 h-4 md:w-5 md:h-5 text-red-600 dark:text-red-400" />
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
