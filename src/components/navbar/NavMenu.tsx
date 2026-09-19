import { useState, useRef } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { ChevronDown, Menu, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { NavigationDropdown, useClickOutside } from './NavDropdowns';
import { isActive, getCurrentNavItem, type NavItem } from './constants';
import { useConfigStore } from '../../stores/useConfigStore';

interface NavMenuProps {
    navItems: NavItem[];
}

const PRIMARY_PATHS = ['/', '/accounts', '/instances'];

/**
 * 导航菜单组件 - 紧凑菜单栏
 *
 * 布局策略:
 * - 常规屏幕 (≥ 640px): 常用页面药丸标签 (Dashboard, Accounts, Instances) + 紧凑 Menu 按钮下拉其它功能
 * - 小屏 (< 640px): 单个全局导航下拉按钮
 */
export function NavMenu({ navItems }: NavMenuProps) {
    const location = useLocation();
    const { t } = useTranslation();
    const { isMenuItemHidden } = useConfigStore();
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    useClickOutside(menuRef, () => setIsMenuOpen(false));

    // 过滤隐藏的菜单项
    const visibleNavItems = navItems.filter(item => !isMenuItemHidden(item.path));

    // 分离核心高频导航与扩展菜单导航
    const primaryItems = visibleNavItems.filter(item => PRIMARY_PATHS.includes(item.path));
    const secondaryItems = visibleNavItems.filter(item => !PRIMARY_PATHS.includes(item.path));

    // 检测当前是否有扩展项处于激活状态
    const activeSecondaryItem = secondaryItems.find(item => isActive(location.pathname, item.path));
    const hasSecondaryActive = Boolean(activeSecondaryItem);

    return (
        <>
            {/* 常规视图 (≥ 640px): 核心药丸 + 紧凑 Menu 按钮 */}
            <nav className="max-[639px]:hidden flex items-center gap-1 bg-gray-100 dark:bg-base-200 rounded-full p-1 shadow-xs">
                {primaryItems.map((item) => (
                    <Link
                        key={item.path}
                        to={item.path}
                        draggable="false"
                        className={`
                            px-3 md:px-4
                            py-1.5
                            rounded-full
                            text-xs md:text-sm
                            font-medium
                            transition-all
                            whitespace-nowrap
                            ${isActive(location.pathname, item.path)
                                ? 'bg-gray-900 text-white shadow-xs dark:bg-white dark:text-gray-900'
                                : 'text-gray-700 hover:text-gray-900 hover:bg-gray-200/80 dark:text-gray-400 dark:hover:text-base-content dark:hover:bg-base-100'
                            }
                        `}
                    >
                        {item.label}
                    </Link>
                ))}

                {/* 扩展功能 Menu 按钮 */}
                {secondaryItems.length > 0 && (
                    <div className="relative" ref={menuRef}>
                        <button
                            type="button"
                            onClick={() => setIsMenuOpen(!isMenuOpen)}
                            className={`
                                flex items-center gap-1.5
                                px-3 md:px-4
                                py-1.5
                                rounded-full
                                text-xs md:text-sm
                                font-medium
                                transition-all
                                whitespace-nowrap
                                ${hasSecondaryActive
                                    ? 'bg-gray-900 text-white shadow-xs dark:bg-white dark:text-gray-900'
                                    : 'text-gray-700 hover:text-gray-900 hover:bg-gray-200/80 dark:text-gray-400 dark:hover:text-base-content dark:hover:bg-base-100'
                                }
                            `}
                            title={t('common.menu', 'Menu')}
                        >
                            {hasSecondaryActive && activeSecondaryItem ? (
                                <>
                                    <activeSecondaryItem.icon className="w-3.5 h-3.5" />
                                    <span>{activeSecondaryItem.label}</span>
                                </>
                            ) : (
                                <>
                                    <Menu className="w-3.5 h-3.5" />
                                    <span>{t('common.menu', 'Menu')}</span>
                                </>
                            )}
                            <ChevronDown className={`w-3 h-3 transition-transform duration-200 ${isMenuOpen ? 'rotate-180' : ''}`} />
                        </button>

                        {/* 下拉菜单 */}
                        {isMenuOpen && (
                            <div className="absolute left-1/2 -translate-x-1/2 mt-2 w-52 bg-white dark:bg-base-200 rounded-xl shadow-xl border border-gray-200 dark:border-base-100 py-1.5 z-50 animate-in fade-in zoom-in-95 duration-150 origin-top">
                                <div className="px-3 py-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                                    {t('common.menu', 'Menu')}
                                </div>
                                {secondaryItems.map((item) => {
                                    const active = isActive(location.pathname, item.path);
                                    return (
                                        <Link
                                            key={item.path}
                                            to={item.path}
                                            draggable="false"
                                            onClick={() => setIsMenuOpen(false)}
                                            className={`
                                                w-full px-3 py-2 text-left text-xs flex items-center justify-between hover:bg-gray-50 dark:hover:bg-base-100 transition-colors
                                                ${active
                                                    ? 'text-blue-600 dark:text-blue-400 font-medium bg-blue-50/50 dark:bg-blue-900/20'
                                                    : 'text-gray-700 dark:text-gray-300'
                                                }
                                            `}
                                        >
                                            <div className="flex items-center gap-2.5">
                                                <item.icon className="w-4 h-4" />
                                                <span>{item.label}</span>
                                            </div>
                                            {active && <Check className="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" />}
                                        </Link>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                )}
            </nav>

            {/* 紧凑视图 (< 640px): 纯下拉 */}
            <div className="min-[640px]:hidden">
                <NavigationDropdown
                    navItems={visibleNavItems}
                    isActive={(path) => isActive(location.pathname, path)}
                    getCurrentNavItem={() => getCurrentNavItem(location.pathname, visibleNavItems)}
                    onNavigate={() => { }}
                    showLabel={true}
                />
            </div>
        </>
    );
}
