import { useState, useRef, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Users, Laptop, Settings, Menu, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClickOutside } from './NavDropdowns';
import { isActive, type NavItem } from './constants';
import { useConfigStore } from '../../stores/useConfigStore';
import { cn } from '../../utils/cn';

interface NavMenuProps {
    navItems: NavItem[];
}

export function NavMenu({ navItems }: NavMenuProps) {
    const location = useLocation();
    const { t } = useTranslation();
    const { isMenuItemHidden } = useConfigStore();
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    useClickOutside(menuRef, () => setIsMenuOpen(false));

    useEffect(() => {
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'nav-menu') {
                setIsMenuOpen(false);
            }
        };
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsMenuOpen(false);
            }
        };
        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        document.addEventListener('keydown', handleKeyDown);
        return () => {
            window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, []);

    // Filter hidden menu items
    const visibleNavItems = navItems.filter(item => !isMenuItemHidden(item.path));

    // Direct items: Accounts, Instances, Settings
    const accountsItem = visibleNavItems.find(item => item.path === '/accounts');
    const instancesItem = visibleNavItems.find(item => item.path === '/instances');
    const settingsItem = visibleNavItems.find(item => item.path === '/settings');

    const directPaths = new Set(['/accounts', '/instances', '/settings']);
    const dropdownNavItems = visibleNavItems.filter(item => !directPaths.has(item.path));

    const isAccountsActive = isActive(location.pathname, '/accounts');
    const isInstancesActive = isActive(location.pathname, '/instances');
    const isSettingsActive = isActive(location.pathname, '/settings');
    const activeDropdownItem = dropdownNavItems.find(item => isActive(location.pathname, item.path));
    const isDropdownRouteActive = Boolean(activeDropdownItem);

    return (
        <div className="flex items-center rounded-full bg-gray-100 dark:bg-[#0c2438] border border-gray-200/60 dark:border-[#15334d] p-0.5 shadow-xs">
            {/* 1. Accounts direct button */}
            {accountsItem && (
                <Link
                    to="/accounts"
                    draggable="false"
                    className={cn(
                        "flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold transition-all duration-150 cursor-pointer select-none",
                        isAccountsActive
                            ? "bg-blue-600 text-white shadow-xs"
                            : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                    )}
                    title={accountsItem.label}
                >
                    <Users className={cn("w-3.5 h-3.5 shrink-0", isAccountsActive ? "text-white" : "text-blue-600 dark:text-blue-400")} />
                    <span>{accountsItem.label}</span>
                </Link>
            )}

            {/* 2. Instances direct button */}
            {instancesItem && (
                <Link
                    to="/instances"
                    draggable="false"
                    className={cn(
                        "flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold transition-all duration-150 cursor-pointer select-none",
                        isInstancesActive
                            ? "bg-blue-600 text-white shadow-xs"
                            : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                    )}
                    title={instancesItem.label}
                >
                    <Laptop className={cn("w-3.5 h-3.5 shrink-0", isInstancesActive ? "text-white" : "text-teal-600 dark:text-teal-400")} />
                    <span>{instancesItem.label}</span>
                </Link>
            )}

            {/* 3. Settings direct button */}
            {settingsItem && (
                <Link
                    to="/settings"
                    draggable="false"
                    className={cn(
                        "flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold transition-all duration-150 cursor-pointer select-none",
                        isSettingsActive
                            ? "bg-blue-600 text-white shadow-xs"
                            : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                    )}
                    title={settingsItem.label}
                >
                    <Settings className={cn("w-3.5 h-3.5 shrink-0", isSettingsActive ? "text-white" : "text-indigo-600 dark:text-indigo-400")} />
                    <span>{settingsItem.label}</span>
                </Link>
            )}

            {/* 4. Compact Hamburger Dropdown for remaining views */}
            <div className="relative" ref={menuRef}>
                <button
                    type="button"
                    onClick={() => {
                        const next = !isMenuOpen;
                        setIsMenuOpen(next);
                        if (next) {
                            window.dispatchEvent(new CustomEvent('agm:dropdown-open', { detail: { source: 'nav-menu' } }));
                        }
                    }}
                    className={cn(
                        "w-7 h-7 rounded-full flex items-center justify-center transition-all duration-150 cursor-pointer ml-0.5",
                        isDropdownRouteActive || isMenuOpen
                            ? "bg-blue-600 text-white shadow-xs"
                            : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                    )}
                    title={activeDropdownItem ? `${activeDropdownItem.label} (More)` : t('common.menu', 'More Views')}
                    aria-label="More Navigation"
                    aria-expanded={isMenuOpen}
                >
                    <Menu className="w-3.5 h-3.5 shrink-0" />
                </button>

                {/* Dropdown Menu */}
                {isMenuOpen && (
                    <div className="absolute left-1/2 -translate-x-1/2 mt-2 w-60 max-w-[calc(100vw-32px)] bg-white dark:bg-slate-900 rounded-xl shadow-xl border border-gray-200 dark:border-slate-800 py-1.5 z-[9999] animate-in fade-in zoom-in-95 duration-150 origin-top">
                        <div className="flex items-center justify-between px-3 py-1 border-b border-gray-100 dark:border-slate-800 text-[10px] font-bold text-gray-400 uppercase tracking-wider mb-1">
                            <span>{t('common.more_views', 'More Views')}</span>
                            {activeDropdownItem && (
                                <span className="px-1.5 py-0.5 rounded-full text-[9px] font-bold bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/30">
                                    {activeDropdownItem.label}
                                </span>
                            )}
                        </div>
                        <div className="max-h-80 overflow-y-auto py-0.5">
                            {dropdownNavItems.map((item) => {
                                const isSelected = isActive(location.pathname, item.path);
                                return (
                                    <Link
                                        key={item.path}
                                        to={item.path}
                                        draggable="false"
                                        onClick={() => setIsMenuOpen(false)}
                                        className={cn(
                                            "w-full px-3 py-2 text-left text-xs flex items-center justify-between hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors cursor-pointer",
                                            isSelected
                                                ? "text-blue-600 dark:text-blue-400 font-semibold bg-blue-50/60 dark:bg-blue-900/20"
                                                : "text-gray-700 dark:text-gray-300"
                                        )}
                                    >
                                        <div className="flex items-center gap-2.5">
                                            <item.icon className="w-4 h-4 shrink-0" />
                                            <span>{item.label}</span>
                                        </div>
                                        {isSelected && <Check className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400 shrink-0" />}
                                    </Link>
                                );
                            })}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
