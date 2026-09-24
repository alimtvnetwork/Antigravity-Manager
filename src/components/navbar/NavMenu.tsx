import { useState, useRef, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { ChevronDown, Menu, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClickOutside } from './NavDropdowns';
import { isActive, type NavItem } from './constants';
import { useConfigStore } from '../../stores/useConfigStore';

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
        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        return () => window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
    }, []);

    // Filter hidden menu items
    const visibleNavItems = navItems.filter(item => !isMenuItemHidden(item.path));

    // Find currently active navigation item
    const currentItem = visibleNavItems.find(item => isActive(location.pathname, item.path)) || visibleNavItems[0];
    const CurrentIcon = currentItem ? currentItem.icon : Menu;

    return (
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
                className="flex items-center gap-2 px-3.5 py-1.5 rounded-full text-xs md:text-sm font-medium bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 text-gray-800 dark:text-gray-200 transition-colors border border-gray-200/60 dark:border-slate-700 shadow-xs cursor-pointer"
                title={t('common.menu', 'Menu')}
            >
                <CurrentIcon className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400 shrink-0" />
                <span className="whitespace-nowrap font-medium">
                    {currentItem?.label || t('common.menu', 'Menu')}
                </span>
                <ChevronDown className={`w-3.5 h-3.5 text-gray-500 shrink-0 transition-transform duration-200 ${isMenuOpen ? 'rotate-180' : ''}`} />
            </button>

            {/* Navigation dropdown menu */}
            {isMenuOpen && (
                <div className="absolute left-1/2 -translate-x-1/2 mt-2 w-56 max-w-[calc(100vw-32px)] bg-white dark:bg-slate-900 rounded-xl shadow-xl border border-gray-200 dark:border-slate-800 py-2 z-[9999] animate-in fade-in zoom-in-95 duration-150 origin-top">
                    <div className="px-3 py-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                        {t('common.navigation', 'Navigation')}
                    </div>
                    <div className="max-h-80 overflow-y-auto py-1">
                        {visibleNavItems.map((item) => {
                            const isSelected = isActive(location.pathname, item.path);
                            return (
                                <Link
                                    key={item.path}
                                    to={item.path}
                                    draggable="false"
                                    onClick={() => setIsMenuOpen(false)}
                                    className={`
                                        w-full px-3 py-2 text-left text-xs flex items-center justify-between hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors
                                        ${isSelected
                                            ? 'text-blue-600 dark:text-blue-400 font-medium bg-blue-50/60 dark:bg-blue-900/20'
                                            : 'text-gray-700 dark:text-gray-300'
                                        }
                                    `}
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
    );
}
