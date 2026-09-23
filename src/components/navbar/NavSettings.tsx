import { useEffect, useState } from 'react';
import { Sun, Moon, LogOut, Minus, X, RotateCcw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { LanguageDropdown, MoreDropdown } from './NavDropdowns';
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

/**
 * Settings button component - handles responsiveness independently
 *
 * Responsive strategy:
 * - ≥ 1024px: standalone secondary buttons (quick clean, theme, language)
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
    const [isMaximized, setIsMaximized] = useState(false);
    const [isCleanModalOpen, setIsCleanModalOpen] = useState(false);

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

    return (
        <div className="flex items-center gap-1.5 md:gap-2 shrink-0">
            {/* Collapsible Secondary Tools: Visible on >= 1024px */}
            <div className="hidden lg:flex items-center gap-1.5 md:gap-2 shrink-0">
                {/* Antigravity Quick Clean button */}
                <button
                    type="button"
                    onClick={() => setIsCleanModalOpen(true)}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer text-gray-700 dark:text-gray-300"
                    title={t('nav.quick_clean', 'Antigravity Cache & Retention Clean')}
                    aria-label="Quick Clean"
                >
                    <RotateCcw className="w-4 h-4 md:w-5 md:h-5" />
                </button>

                {/* Theme toggle button */}
                <button
                    type="button"
                    onClick={onThemeToggle}
                    className="w-9 h-9 md:w-10 md:h-10 rounded-full bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 flex items-center justify-center transition-all duration-150 ease-out shadow-xs cursor-pointer"
                    title={theme === 'light' ? t('nav.theme_to_dark') : t('nav.theme_to_light')}
                    aria-label="Toggle Theme"
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
                    onOpenCleanModal={() => setIsCleanModalOpen(true)}
                />
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
