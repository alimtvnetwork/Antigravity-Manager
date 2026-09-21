import { useEffect, useState } from 'react';
import { Sun, Moon, LogOut, Minimize2, Minus, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LanguageDropdown, MoreDropdown } from './NavDropdowns';
import { LANGUAGES } from './constants';
import { isTauri } from '../../utils/env';
import { useViewStore } from '../../stores/useViewStore';

interface NavSettingsProps {
    theme: 'light' | 'dark';
    currentLanguage: string;
    onThemeToggle: (event: React.MouseEvent<HTMLButtonElement>) => void;
    onLanguageChange: (langCode: string) => void;
}

/**
 * Settings button component - handles responsiveness independently
 *
 * Responsive strategy:
 * - ≥ 1024px: standalone secondary buttons (mini view, theme, language)
 * - < 1024px: secondary tools collapse into responsive kebab/overflow menu
 * - Window controls (Minimize, Maximize/Restore, Close): always permanently visible and pinned
 */
export function NavSettings({
    theme,
    currentLanguage,
    onThemeToggle,
    onLanguageChange
}: NavSettingsProps) {
    const { t } = useTranslation();
    const { setMiniView } = useViewStore();
    const [isMaximized, setIsMaximized] = useState(false);

    useEffect(() => {
        if (!isTauri()) return;
        const win = getCurrentWindow();
        win.isMaximized().then(setIsMaximized).catch(() => {});

        let unlistenResize: (() => void) | undefined;
        win.onResized(async () => {
            try {
                const max = await win.isMaximized();
                setIsMaximized(max);
            } catch {
                // Ignore window polling error during destruction
            }
        }).then((fn) => {
            unlistenResize = fn;
        }).catch(() => {});

        return () => {
            if (unlistenResize) {
                unlistenResize();
            }
        };
    }, []);

    const handleMinimize = async () => {
        try {
            await getCurrentWindow().minimize();
        } catch (e) {
            console.error('Failed to minimize window:', e);
        }
    };

    const handleToggleMaximize = async () => {
        try {
            const win = getCurrentWindow();
            await win.toggleMaximize();
            const max = await win.isMaximized();
            setIsMaximized(max);
        } catch (e) {
            console.error('Failed to toggle maximize window:', e);
        }
    };

    const handleClose = async () => {
        try {
            await getCurrentWindow().close();
        } catch (e) {
            console.error('Failed to close window:', e);
        }
    };

    const handleLogout = () => {
        sessionStorage.removeItem('abv_admin_api_key');
        localStorage.removeItem('abv_admin_api_key');
        window.location.reload();
    };

    return (
        <div className="flex items-center gap-1.5 md:gap-2 shrink-0">
            {/* Collapsible Secondary Tools: Visible on >= 1024px */}
            <div className="hidden lg:flex items-center gap-1.5 md:gap-2 shrink-0">
                {/* Mini view toggle button */}
                <button
                    onClick={() => setMiniView(true)}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                    title={t('nav.mini_view', 'Mini View')}
                >
                    <Minimize2 className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300" />
                </button>

                {/* Theme toggle button */}
                <button
                    onClick={onThemeToggle}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                    title={theme === 'light' ? t('nav.theme_to_dark') : t('nav.theme_to_light')}
                >
                    {theme === 'light' ? (
                        <Moon className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300" />
                    ) : (
                        <Sun className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300" />
                    )}
                </button>

                {/* Language switch dropdown */}
                <LanguageDropdown
                    currentLanguage={currentLanguage}
                    languages={LANGUAGES}
                    onLanguageChange={onLanguageChange}
                />
            </div>

            {/* Kebab/Overflow menu for secondary tools on < 1024px */}
            <div className="flex lg:hidden shrink-0">
                <MoreDropdown
                    theme={theme}
                    currentLanguage={currentLanguage}
                    languages={LANGUAGES}
                    onThemeToggle={onThemeToggle}
                    onLanguageChange={onLanguageChange}
                />
            </div>

            {/* Window controls (Minimize, Maximize/Restore, Close) - Tauri only: Permanently visible, never collapsed */}
            {isTauri() && (
                <div className="flex items-center gap-1 md:gap-1.5 shrink-0 z-50">
                    <button
                        type="button"
                        onClick={handleMinimize}
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                        title={t('common.minimize', 'Minimize')}
                        aria-label="Minimize"
                    >
                        <Minus className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300" />
                    </button>

                    <button
                        type="button"
                        onClick={handleToggleMaximize}
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                        title={isMaximized ? t('common.restore', 'Restore') : t('common.maximize', 'Maximize')}
                        aria-label={isMaximized ? "Restore" : "Maximize"}
                    >
                        {isMaximized ? (
                            <svg className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <rect x="4.5" y="1.5" width="10" height="10" rx="1" />
                                <path d="M1.5 5.5v9h9" />
                            </svg>
                        ) : (
                            <svg className="w-3.5 h-3.5 text-gray-700 dark:text-gray-300" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <rect x="2" y="2" width="12" height="12" rx="1.5" />
                            </svg>
                        )}
                    </button>

                    <button
                        type="button"
                        onClick={handleClose}
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer group"
                        title={t('common.close', 'Close')}
                        aria-label="Close"
                    >
                        <X className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300 group-hover:text-white transition-colors duration-150" />
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
        </div>
    );
}
