import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useTranslation } from 'react-i18next';
import { isTauri, isMacOS } from '../../utils/env';
import versionData from '../../../version.json';

export default function TitleBar() {
    const { t } = useTranslation();
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

    if (!isTauri()) {
        return null;
    }

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

    const isMac = isMacOS();

    return (
        <header
            data-tauri-drag-region
            onMouseDown={(e) => {
                if ((e.target as HTMLElement).closest('button')) return;
                getCurrentWindow().startDragging();
            }}
            onDoubleClick={async (e) => {
                if ((e.target as HTMLElement).closest('button')) return;
                await handleToggleMaximize();
            }}
            className="w-full h-8 flex items-center justify-between select-none bg-[#FAFBFC] dark:bg-base-300 border-b border-gray-200/50 dark:border-base-200/60 relative z-50 shrink-0 transition-colors duration-150"
        >
            {isMac ? (
                /* macOS Traffic Lights Style */
                <>
                    <div className="flex items-center gap-2 pl-3 h-full select-none">
                        <button
                            type="button"
                            onClick={handleClose}
                            className="w-3 h-3 rounded-full bg-[#FF5F56] hover:brightness-90 active:brightness-75 transition-all flex items-center justify-center group cursor-pointer"
                            title={t('common.close', 'Close')}
                            aria-label="Close"
                        >
                            <span className="opacity-0 group-hover:opacity-100 text-[#4C0002] text-[8px] font-bold leading-none">✕</span>
                        </button>
                        <button
                            type="button"
                            onClick={handleMinimize}
                            className="w-3 h-3 rounded-full bg-[#FFBD2E] hover:brightness-90 active:brightness-75 transition-all flex items-center justify-center group cursor-pointer"
                            title={t('common.minimize', 'Minimize')}
                            aria-label="Minimize"
                        >
                            <span className="opacity-0 group-hover:opacity-100 text-[#5E3F00] text-[8px] font-bold leading-none">−</span>
                        </button>
                        <button
                            type="button"
                            onClick={handleToggleMaximize}
                            className="w-3 h-3 rounded-full bg-[#27C93F] hover:brightness-90 active:brightness-75 transition-all flex items-center justify-center group cursor-pointer"
                            title={isMaximized ? t('common.restore', 'Restore') : t('common.maximize', 'Maximize')}
                            aria-label={isMaximized ? "Restore" : "Maximize"}
                        >
                            <span className="opacity-0 group-hover:opacity-100 text-[#004D11] text-[8px] font-bold leading-none">＋</span>
                        </button>
                    </div>
                    <div className="flex-1 h-full flex items-center justify-center" data-tauri-drag-region>
                        <span className="text-[11px] font-medium text-gray-500 dark:text-gray-400" title="Antigravity Manager Tools By Alim">Agm Tool By Alim</span>
                    </div>
                    <div className="w-16 h-full" data-tauri-drag-region />
                </>
            ) : (
                /* Windows & Linux Fluent Caption Controls */
                <>
                    {/* Left: App Branding Pill */}
                    <div className="flex items-center gap-2 px-3 h-full pointer-events-none" data-tauri-drag-region title="Antigravity Manager Tools By Alim">
                        <div className="w-3.5 h-3.5 rounded-sm bg-gradient-to-tr from-amber-500 to-amber-300 flex items-center justify-center text-[9px] font-black text-white shadow-xs">
                            A
                        </div>
                        <span className="text-[11px] font-semibold text-gray-700 dark:text-gray-300 tracking-tight">
                            Agm Tool By Alim
                        </span>
                        <span className="text-[9px] font-mono px-1 py-0.2 rounded bg-gray-200/60 dark:bg-base-200 text-gray-500 font-medium">
                            v{versionData.version || versionData.Version || '4.31.0'}
                        </span>
                    </div>

                    {/* Center: Draggable Spacer */}
                    <div className="flex-1 h-full" data-tauri-drag-region />

                    {/* Right: Custom Fluid Window Controls */}
                    <div className="flex items-center h-full">
                        {/* Minimize Button */}
                        <button
                            type="button"
                            onClick={handleMinimize}
                            className="w-11 h-8 flex items-center justify-center text-gray-600 dark:text-gray-400 hover:bg-gray-200/80 dark:hover:bg-white/10 active:bg-gray-300/80 dark:active:bg-white/15 transition-colors duration-150 cursor-pointer"
                            title={t('common.minimize', 'Minimize')}
                            aria-label="Minimize"
                        >
                            <svg className="w-3 h-3" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M2 8h12v1.5H2z" />
                            </svg>
                        </button>

                        {/* Maximize / Restore Button */}
                        <button
                            type="button"
                            onClick={handleToggleMaximize}
                            className="w-11 h-8 flex items-center justify-center text-gray-600 dark:text-gray-400 hover:bg-gray-200/80 dark:hover:bg-white/10 active:bg-gray-300/80 dark:active:bg-white/15 transition-colors duration-150 cursor-pointer"
                            title={isMaximized ? t('common.restore', 'Restore') : t('common.maximize', 'Maximize')}
                            aria-label={isMaximized ? "Restore" : "Maximize"}
                        >
                            {isMaximized ? (
                                <svg className="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                    <rect x="4.5" y="1.5" width="10" height="10" rx="1" />
                                    <path d="M1.5 5.5v9h9" />
                                </svg>
                            ) : (
                                <svg className="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                                    <rect x="2" y="2" width="12" height="12" rx="1.5" />
                                </svg>
                            )}
                        </button>

                        {/* Close Button */}
                        <button
                            type="button"
                            onClick={handleClose}
                            className="w-11 h-8 flex items-center justify-center text-gray-600 dark:text-gray-400 hover:bg-[#E81123] hover:text-white dark:hover:bg-[#E81123] dark:hover:text-white active:bg-[#C40E1E] transition-colors duration-150 cursor-pointer"
                            title={t('common.close', 'Close')}
                            aria-label="Close"
                        >
                            <svg className="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                                <path d="M3 3l10 10M13 3L3 13" />
                            </svg>
                        </button>
                    </div>
                </>
            )}
        </header>
    );
}
