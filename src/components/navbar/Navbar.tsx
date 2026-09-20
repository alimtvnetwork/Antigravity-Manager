import { LayoutDashboard, Users, Network, Activity, BarChart3, Settings, Lock, KeyRound, Laptop, Mail } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useConfigStore } from '../../stores/useConfigStore';
import { isLinux } from '../../utils/env';
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
    const { t } = useTranslation();
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


    // Theme toggle logic (with View Transition animation)
    const toggleTheme = async (event: React.MouseEvent<HTMLButtonElement>) => {
        if (!config) return;

        const newTheme = config.theme === 'light' ? 'dark' : 'light';

        // Use View Transition API if supported, but skip on Linux (may cause crash)
        if ('startViewTransition' in document && !isLinux()) {
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
        } else {
            // Fallback: direct switch (Linux or browsers without View Transition)
            await saveConfig({
                ...config,
                theme: newTheme,
                language: config.language
            }, true);
        }
    };

    // Language change logic
    const handleLanguageChange = async (langCode: string) => {
        if (!config) return;

        await saveConfig({
            ...config,
            language: langCode,
            theme: config.theme
        }, true);
    };

    return (
        <nav
            style={{ position: 'sticky', top: 0, zIndex: 100, isolation: 'isolate' }}
            className="py-1.5 transition-colors duration-200 bg-[#FAFBFC] dark:bg-slate-900 border-b border-gray-200/50 dark:border-slate-800/80"
        >

            <div className="max-w-7xl mx-auto px-3 md:px-5 relative w-full" style={{ zIndex: 10 }}>
                {/* Flexbox layout */}
                <div className="flex items-center justify-between h-14 gap-2 md:gap-3">
                    {/* Logo & Error Manager Badge */}
                    <div className="shrink-0 flex items-center gap-1.5 min-w-0">
                        <NavLogo />
                        <ErrorQueueBadge />
                    </div>

                    {/* Compact nav menu */}
                    <div className="flex-1 flex justify-center min-w-0 px-1">
                        <NavMenu navItems={navItems} />
                    </div>

                    {/* Instance selector and settings (docked) */}
                    <div className="flex items-center gap-1.5 md:gap-2 shrink-0">
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
