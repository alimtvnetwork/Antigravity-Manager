import { startTransition } from 'react';
import { LayoutDashboard, Users, Network, Activity, BarChart3, Settings, Lock, KeyRound, Laptop, Mail } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useConfigStore } from '../../stores/useConfigStore';
import { useErrorStore } from '../../stores/error-store';
import { isLinux, isTauri } from '../../utils/env';
import { NavLogo } from './NavLogo';
import { NavMenu } from './NavMenu';
import { NavSettings } from './NavSettings';
import { InstanceSelector } from './InstanceSelector';
import { ErrorQueueBadge } from '../errors/error-queue-badge';
import type { NavItem } from './constants';

/**
 * Navbar main component
 *
 * Responsibility: Layout and state management, responsive handling delegated to subcomponents
 */
function Navbar() {
    const { t, i18n } = useTranslation();
    const { config, saveConfig } = useConfigStore();

    // Create navigation items with translated labels
    const navItems: NavItem[] = [
        { path: '/accounts', label: t('nav.accounts'), icon: Users, priority: 'high' },
        { path: '/dashboard', label: t('nav.dashboard'), icon: LayoutDashboard, priority: 'high' },
        { path: '/instances', label: t('nav.instances', 'Instances'), icon: Laptop, priority: 'high' },
        { path: '/api-proxy', label: t('nav.proxy'), icon: Network, priority: 'high' },
        { path: '/apikey-fun', label: t('nav.apikey_fun', 'Relay Hub'), icon: KeyRound, priority: 'high' },
        { path: '/monitor', label: t('nav.call_records'), icon: Activity, priority: 'medium' },
        { path: '/token-stats', label: t('nav.token_stats', 'Token Stats'), icon: BarChart3, priority: 'low' },
        { path: '/user-token', label: t('nav.user_token', 'User Tokens'), icon: Users, priority: 'low' },
        { path: '/security', label: t('nav.security'), icon: Lock, priority: 'low' },
        { path: '/email', label: t('nav.email', 'Email & Alerts'), icon: Mail, priority: 'high' },
        { path: '/settings', label: t('nav.settings'), icon: Settings, priority: 'high' },
    ];


    // Window dragging and double click maximize handlers
    const handleMouseDown = (e: React.MouseEvent<HTMLElement>) => {
        if (!isTauri()) return;
        const target = e.target as HTMLElement;
        if (target.closest('.no-drag')) return;
        if (target.closest('button')) return;
        if (target.closest('a')) return;
        if (target.closest('input')) return;
        if (target.closest('select')) return;
        if (target.closest('textarea')) return;
        try {
            getCurrentWindow().startDragging();
        } catch (err) {
            console.warn('Failed to start dragging window:', err);
        }
    };

    const handleDoubleClick = async (e: React.MouseEvent<HTMLElement>) => {
        if (!isTauri()) return;
        const target = e.target as HTMLElement;
        if (target.closest('.no-drag')) return;
        if (target.closest('button')) return;
        if (target.closest('a')) return;
        if (target.closest('input')) return;
        if (target.closest('select')) return;
        if (target.closest('textarea')) return;
        try {
            await getCurrentWindow().toggleMaximize();
        } catch (err) {
            console.error('Failed to toggle maximize window:', err);
            useErrorStore.getState().captureError(err, {
                source: 'Navbar.tsx',
                triggerAction: 'handleDoubleClick.toggleMaximize',
            });
        }
    };

    // Theme toggle logic (with View Transition animation)
    const toggleTheme = async (event: React.MouseEvent<HTMLButtonElement>) => {
        if (!config) return;

        const newTheme = config.theme === 'light' ? 'dark' : 'light';

        try {
            // Use View Transition API if supported, but skip on Linux (may cause crash)
            if (!isLinux()) {
                if ('startViewTransition' in document) {
                    const x = event.clientX;
                    const y = event.clientY;
                    const endRadius = Math.hypot(
                        Math.max(x, window.innerWidth - x),
                        Math.max(y, window.innerHeight - y)
                    );

                    // @ts-ignore
                    const transition = document.startViewTransition(async () => {
                        saveConfig({
                            ...config,
                            theme: newTheme,
                            language: config.language
                        }, true);
                    });

                    transition.ready.then(() => {
                        const isDarkMode = newTheme === 'dark';
                        const clipPath = isDarkMode
                            ? [`circle(${endRadius}px at ${x}px ${y}px)`, `circle(0px at ${x}px ${y}px)`]
                            : [`circle(0px at ${x}px ${y}px)`, `circle(${endRadius}px at ${x}px ${y}px)`];

                        document.documentElement.animate(
                            {
                                clipPath: clipPath
                            },
                            {
                                duration: 500,
                                easing: 'ease-in-out',
                                fill: 'forwards',
                                pseudoElement: isDarkMode ? '::view-transition-old(root)' : '::view-transition-new(root)'
                            }
                        );
                    });
                    return;
                }
            }

            // Fallback: direct switch (Linux or browsers without View Transition)
            await saveConfig({
                ...config,
                theme: newTheme,
                language: config.language
            }, true);
        } catch (err) {
            console.error('Failed to toggle theme:', err);
            const captured = useErrorStore.getState().captureError(err, {
                source: 'Navbar.tsx',
                triggerComponent: 'NavSettings.ThemeToggle',
                triggerAction: 'toggleTheme',
            });
            useErrorStore.getState().openErrorModal(captured);
        }
    };

    // Language change logic (instant response + non-blocking smooth transition + error reporting)
    const handleLanguageChange = (langCode: string) => {
        if (!config) return;

        // 1. Immediately set RTL / LTR layout direction
        document.documentElement.dir = langCode === 'ar' ? 'rtl' : 'ltr';

        // 2. Use startTransition to trigger non-blocking progressive re-render
        startTransition(() => {
            i18n.changeLanguage(langCode);
        });

        // 3. Asynchronously persist config without blocking UI
        saveConfig({
            ...config,
            language: langCode,
            theme: config.theme
        }, true).catch(err => {
            console.error('Failed to persist language config:', err);
            const captured = useErrorStore.getState().captureError(err, {
                source: 'Navbar.tsx',
                triggerComponent: 'NavSettings.LanguageDropdown',
                triggerAction: 'handleLanguageChange',
                context: { targetLanguage: langCode }
            });
            useErrorStore.getState().openErrorModal(captured);
        });
    };

    return (
        <nav
            onMouseDown={handleMouseDown}
            onDoubleClick={handleDoubleClick}
            style={{ position: 'sticky', top: 0, zIndex: 100, isolation: 'isolate' }}
            className="py-1.5 transition-colors duration-200 bg-[#FAFBFC] dark:bg-slate-900 border-b border-gray-200/50 dark:border-slate-800/80 select-none"
        >
            <div className="max-w-7xl mx-auto px-2 sm:px-3 md:px-5 relative w-full" style={{ zIndex: 10 }}>
                {/* Flexbox layout */}
                <div className="flex items-center justify-between h-14 gap-1 sm:gap-2 md:gap-3 min-w-0">
                    {/* Logo & Error Manager Badge */}
                    <div
                        className="no-drag shrink-0 flex items-center gap-1 sm:gap-1.5 min-w-0"
                        onMouseDown={(e) => e.stopPropagation()}
                        onDoubleClick={(e) => e.stopPropagation()}
                    >
                        <NavLogo />
                        <ErrorQueueBadge />
                    </div>

                    {/* Center draggable spacer */}
                    <div className="flex-1 h-full min-w-1" data-tauri-drag-region />

                    {/* Compact nav menu */}
                    <div
                        className="no-drag shrink-0 flex justify-center min-w-0 px-0.5 sm:px-1"
                        onMouseDown={(e) => e.stopPropagation()}
                        onDoubleClick={(e) => e.stopPropagation()}
                    >
                        <NavMenu navItems={navItems} />
                    </div>

                    {/* Center draggable spacer */}
                    <div className="flex-1 h-full min-w-1" data-tauri-drag-region />

                    {/* Instance selector and settings (docked to far right) */}
                    <div
                        className="no-drag flex items-center gap-1 sm:gap-1.5 md:gap-2 shrink-0 z-50 ml-auto min-w-0"
                        onMouseDown={(e) => e.stopPropagation()}
                        onDoubleClick={(e) => e.stopPropagation()}
                    >
                        <InstanceSelector />
                        <NavSettings
                            theme={(config?.theme as 'light' | 'dark') || 'light'}
                            currentLanguage={config?.language || 'en'}
                            onThemeToggle={toggleTheme}
                            onLanguageChange={handleLanguageChange}
                        />
                    </div>
                </div>
            </div>
        </nav>
    );
}

export default Navbar;
