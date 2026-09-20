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
 * - ≥ 480px: standalone buttons (mini view, theme, language, window controls)
 * - < 480px: more dropdown menu
 */
export function NavSettings({
    theme,
    currentLanguage,
    onThemeToggle,
    onLanguageChange
}: NavSettingsProps) {
    const { t } = useTranslation();
    const { setMiniView } = useViewStore();

    const handleMinimize = async () => {
        try {
            await getCurrentWindow().minimize();
        } catch (e) {
            console.error('Failed to minimize window:', e);
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
        <>
            {/* Standalone buttons (≥ 480px) */}
            <div className="hidden min-[480px]:flex items-center gap-1.5 md:gap-2">
                {/* Mini view toggle button */}
                <button
                    onClick={() => setMiniView(true)}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-colors duration-150 ease-out shadow-xs cursor-pointer"
                    title={t('nav.mini_view', 'Mini View')}
                >
                    <Minimize2 className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300" />
                </button>

                {/* Theme toggle button */}
                <button
                    onClick={onThemeToggle}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-colors duration-150 ease-out shadow-xs cursor-pointer"
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

                {/* Window controls (Minimize, Close) - Tauri only */}
                {isTauri() && (
                    <>
                        <button
                            type="button"
                            onClick={handleMinimize}
                            className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-colors duration-150 ease-out shadow-xs cursor-pointer"
                            title={t('common.minimize', 'Minimize')}
                            aria-label="Minimize"
                        >
                            <Minus className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300" />
                        </button>

                        <button
                            type="button"
                            onClick={handleClose}
                            className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white flex items-center justify-center transition-colors duration-150 ease-out shadow-xs cursor-pointer group"
                            title={t('common.close', 'Close')}
                            aria-label="Close"
                        >
                            <X className="w-4 h-4 md:w-5 md:h-5 text-gray-700 dark:text-gray-300 group-hover:text-white transition-colors" />
                        </button>
                    </>
                )}

                {/* Logout button - Web mode only */}
                {!isTauri() && (
                    <button
                        onClick={handleLogout}
                        className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/40 flex items-center justify-center transition-colors duration-150 ease-out shadow-xs cursor-pointer"
                        title={t('nav.logout', 'Logout')}
                    >
                        <LogOut className="w-4 h-4 md:w-5 md:h-5 text-red-600 dark:text-red-400" />
                    </button>
                )}
            </div>

            {/* More menu (< 480px) */}
            <div className="min-[480px]:hidden">
                <MoreDropdown
                    theme={theme}
                    currentLanguage={currentLanguage}
                    languages={LANGUAGES}
                    onThemeToggle={onThemeToggle}
                    onLanguageChange={onLanguageChange}
                />
            </div>
        </>
    );
}
