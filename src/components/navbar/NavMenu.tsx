import { useState, useRef, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Users, Laptop, Settings, ChevronDown, Check, Download, Search, X, MessageSquare } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClickOutside } from './NavDropdowns';
import { isActive, type NavItem } from './constants';
import { useConfigStore } from '../../stores/useConfigStore';
import { useUpdateStore } from '../../stores/use-update-store';
import { showToast } from '../common/ToastContainer';
import { cn } from '../../utils/cn';
import { TelegramGuideModal } from '../telegram/TelegramGuideModal';

interface NavMenuProps {
    navItems: NavItem[];
}

export function NavMenu({ navItems }: NavMenuProps) {
    const location = useLocation();
    const { t } = useTranslation();
    const { isMenuItemHidden } = useConfigStore();
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const [isTelegramGuideOpen, setIsTelegramGuideOpen] = useState(false);
    const [dropdownSearch, setDropdownSearch] = useState('');
    const menuRef = useRef<HTMLDivElement>(null);

    useClickOutside(menuRef, () => {
        setIsMenuOpen(false);
        setDropdownSearch('');
    });

    useEffect(() => {
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'nav-menu') {
                setIsMenuOpen(false);
                setDropdownSearch('');
            }
        };
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsMenuOpen(false);
                setDropdownSearch('');
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

    const isDropdownSearching = dropdownSearch.trim().length > 0;
    const filteredDropdownNavItems = dropdownNavItems.filter((item) => {
        if (!isDropdownSearching) return true;
        const query = dropdownSearch.trim().toLowerCase();
        return item.label.toLowerCase().includes(query) || item.path.toLowerCase().includes(query);
    });

    const isAccountsActive = isActive(location.pathname, '/accounts');
    const isInstancesActive = isActive(location.pathname, '/instances');
    const isSettingsActive = isActive(location.pathname, '/settings');
    const activeDropdownItem = dropdownNavItems.find(item => isActive(location.pathname, item.path));
    const isDropdownRouteActive = Boolean(activeDropdownItem);

    const checkForUpdates = useUpdateStore((s) => s.checkForUpdates);
    const handleCheckUpdates = async () => {
        setIsMenuOpen(false);
        showToast(t('nav.checking_updates', 'Checking for updates...'), 'info');
        try {
            const info = await checkForUpdates(true);
            if (info?.has_update) {
                // The UpdateNotification card appears automatically via the store.
                return;
            }
            if (info) {
                showToast(t('nav.update_up_to_date', 'No updates available (latest version installed)'), 'success');
            } else {
                showToast(t('nav.no_updates', 'No updates available (latest version installed)'), 'info');
            }
        } catch {
            showToast(t('nav.no_updates', 'No updates available (latest version installed)'), 'info');
        }
    };

    return (
        <div className="flex items-center rounded-full bg-gray-100 dark:bg-slate-900/90 border border-gray-200/60 dark:border-slate-800 p-0.5 shadow-xs divide-x divide-gray-200/70 dark:divide-slate-800/80">
            {/* 1. Accounts direct button (Icon-only with tooltip) */}
            {accountsItem && (
                <div className="relative group">
                    <Link
                        to="/accounts"
                        draggable="false"
                        className={cn(
                            "flex items-center justify-center px-2.5 py-1 text-xs font-semibold transition-all duration-150 cursor-pointer select-none rounded-full",
                            isAccountsActive
                                ? "bg-blue-600 text-white shadow-xs"
                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-gray-200/80 dark:hover:bg-slate-800/60"
                        )}
                        aria-label="Accounts (Profile Vault)"
                        title="Accounts (Profile Vault)"
                    >
                        <Users className={cn("w-3.5 h-3.5 shrink-0", isAccountsActive ? "text-white" : "text-blue-600 dark:text-blue-400")} />
                    </Link>
                    <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
                        Accounts (Profile Vault)
                    </div>
                </div>
            )}

            {/* 2. Instances direct button (Icon-only with tooltip) */}
            {instancesItem && (
                <div className="relative group">
                    <Link
                        to="/instances"
                        draggable="false"
                        className={cn(
                            "flex items-center justify-center px-2.5 py-1 text-xs font-semibold transition-all duration-150 cursor-pointer select-none rounded-full",
                            isInstancesActive
                                ? "bg-blue-600 text-white shadow-xs"
                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-gray-200/80 dark:hover:bg-slate-800/60"
                        )}
                        aria-label="Instances (Engine Runner)"
                        title="Instances (Engine Runner)"
                    >
                        <Laptop className={cn("w-3.5 h-3.5 shrink-0", isInstancesActive ? "text-white" : "text-teal-600 dark:text-teal-400")} />
                    </Link>
                    <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
                        Instances (Engine Runner)
                    </div>
                </div>
            )}

            {/* Standalone Telegram Button (Beside instances, opens TelegramGuideModal) */}
            <div className="relative group">
                <button
                    type="button"
                    onClick={() => setIsTelegramGuideOpen(true)}
                    className={cn(
                        "flex items-center justify-center px-2.5 py-1 text-xs font-semibold transition-all duration-150 cursor-pointer select-none rounded-full",
                        isActive(location.pathname, '/telegram')
                            ? "bg-blue-600 text-white shadow-xs"
                            : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-gray-200/80 dark:hover:bg-slate-800/60"
                    )}
                    aria-label="Telegram Remote Control Guide"
                    title="Telegram Remote Control Guide"
                >
                    <MessageSquare className={cn("w-3.5 h-3.5 shrink-0", isActive(location.pathname, '/telegram') ? "text-white" : "text-sky-500 dark:text-sky-400")} />
                </button>
                <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
                    Telegram Remote Guide
                </div>
            </div>

            {/* 3. Settings direct button (Icon-only with tooltip) */}
            {settingsItem && (
                <div className="relative group">
                    <Link
                        to="/settings"
                        draggable="false"
                        className={cn(
                            "flex items-center justify-center px-2.5 py-1 text-xs font-semibold transition-all duration-150 cursor-pointer select-none rounded-full",
                            isSettingsActive
                                ? "bg-blue-600 text-white shadow-xs"
                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-gray-200/80 dark:hover:bg-slate-800/60"
                        )}
                        aria-label="Settings"
                        title="Settings"
                    >
                        <Settings className={cn("w-3.5 h-3.5 shrink-0", isSettingsActive ? "text-white" : "text-indigo-600 dark:text-indigo-400")} />
                    </Link>
                    <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-white text-[10px] font-medium rounded-md shadow-lg border border-slate-700/60 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-150 z-[9999] whitespace-nowrap">
                        Settings
                    </div>
                </div>
            )}

            {/* 4. Compact Triangle Chevron Dropdown for remaining views */}
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
                        "h-7 px-2 flex items-center justify-center transition-all duration-150 cursor-pointer rounded-full",
                        isDropdownRouteActive || isMenuOpen
                            ? "bg-blue-600 text-white shadow-xs"
                            : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-gray-200/80 dark:hover:bg-slate-800/60"
                    )}
                    title={activeDropdownItem ? `${activeDropdownItem.label} (More)` : t('common.menu', 'More Views')}
                    aria-label="More Navigation"
                    aria-expanded={isMenuOpen}
                >
                    <ChevronDown className={cn("w-3.5 h-3.5 shrink-0 transition-transform duration-200", isMenuOpen && "rotate-180")} />
                </button>

                {/* Dropdown Menu */}
                {isMenuOpen && (
                    <div className="absolute left-1/2 -translate-x-1/2 mt-2 w-64 max-w-[calc(100vw-32px)] bg-white dark:bg-slate-900 rounded-xl shadow-xl border border-gray-200 dark:border-slate-800 py-1.5 z-[9999] animate-in fade-in zoom-in-95 duration-150 origin-top">
                        {/* Update section - Prominent at the top */}
                        <div className="px-1.5 pb-1.5 mb-1 border-b border-gray-100 dark:border-slate-800">
                            <button
                                type="button"
                                onClick={handleCheckUpdates}
                                className="w-full px-3 py-2 text-left text-xs flex items-center gap-2.5 rounded-lg hover:bg-blue-50/80 dark:hover:bg-blue-900/25 transition-colors cursor-pointer text-gray-700 dark:text-gray-300 group"
                                title={t('nav.check_for_updates_title', 'Check for a new version now')}
                            >
                                <Download className="w-4 h-4 shrink-0 text-blue-600 dark:text-blue-400 transition-transform duration-300 ease-out group-hover:-translate-y-0.5 group-hover:scale-110" />
                                <span className="font-semibold text-blue-600 dark:text-blue-400">{t('nav.check_for_updates', 'Check for Updates')}</span>
                            </button>
                        </div>

                        {/* Telegram Guide quick access */}
                        <div className="px-1.5 pb-1 mb-1 border-b border-gray-100 dark:border-slate-800">
                            <button
                                type="button"
                                onClick={() => {
                                    setIsMenuOpen(false);
                                    setIsTelegramGuideOpen(true);
                                }}
                                className="w-full px-3 py-1.5 text-left text-xs flex items-center gap-2.5 rounded-lg hover:bg-sky-50/80 dark:hover:bg-sky-900/25 transition-colors cursor-pointer text-gray-700 dark:text-gray-300 group"
                                title="Open Telegram Setup Guide & CLI Reference"
                            >
                                <MessageSquare className="w-4 h-4 shrink-0 text-sky-500 dark:text-sky-400 transition-transform duration-300 ease-out group-hover:scale-110" />
                                <span className="font-semibold text-sky-600 dark:text-sky-400">Telegram Setup Guide</span>
                            </button>
                        </div>

                        {/* Real-time Search Input */}
                        <div className="px-2 py-1 border-b border-gray-100 dark:border-slate-800">
                            <div className="flex items-center gap-1.5 px-2 py-1 bg-gray-100 dark:bg-slate-800 rounded-lg text-xs">
                                <Search className="w-3.5 h-3.5 text-gray-400 shrink-0" />
                                <input
                                    type="text"
                                    value={dropdownSearch}
                                    onChange={(e) => setDropdownSearch(e.target.value)}
                                    placeholder={t('common.search_views', 'Search views...')}
                                    className="w-full bg-transparent border-0 focus:ring-0 focus:outline-none text-[11px] text-gray-800 dark:text-gray-200"
                                />
                                {isDropdownSearching && (
                                    <button
                                        type="button"
                                        onClick={() => setDropdownSearch('')}
                                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 cursor-pointer"
                                    >
                                        <X className="w-3 h-3" />
                                    </button>
                                )}
                            </div>
                        </div>

                        <div className="flex items-center justify-between px-3 py-1 border-b border-gray-100 dark:border-slate-800 text-[10px] font-bold text-gray-400 uppercase tracking-wider mb-1">
                            <span>{t('common.more_views', 'More Views')} ({filteredDropdownNavItems.length})</span>
                            {activeDropdownItem && (
                                <span className="px-1.5 py-0.5 rounded-full text-[9px] font-bold bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/30">
                                    {activeDropdownItem.label}
                                </span>
                            )}
                        </div>

                        <div className="max-h-80 overflow-y-auto py-0.5">
                            {filteredDropdownNavItems.length === 0 ? (
                                <div className="py-4 text-center text-xs text-gray-400">
                                    {t('common.no_views_found', 'No matching views found')}
                                </div>
                            ) : (
                                filteredDropdownNavItems.map((item) => {
                                    const isSelected = isActive(location.pathname, item.path);
                                    return (
                                        <Link
                                            key={item.path}
                                            to={item.path}
                                            draggable="false"
                                            onClick={() => {
                                                setIsMenuOpen(false);
                                                setDropdownSearch('');
                                            }}
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
                                })
                            )}
                        </div>
                    </div>
                )}
            </div>

            <TelegramGuideModal
                isOpen={isTelegramGuideOpen}
                onClose={() => setIsTelegramGuideOpen(false)}
            />
        </div>
    );
}
