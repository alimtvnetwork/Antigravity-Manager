
import { useEffect } from 'react';
import { useConfigStore } from '../../stores/useConfigStore';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { isLinux } from '../../utils/env';
import { findPalette, THEME_PALETTES } from './themePalettes';
import { useErrorStore } from '../../stores/error-store';

export default function ThemeManager() {
    const { config, loadConfig } = useConfigStore();

    // Load config on mount
    useEffect(() => {
        const init = async () => {
            await loadConfig();
            // Show window after a short delay to ensure React has painted
            setTimeout(async () => {
                if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
                    const win = getCurrentWindow();
                    const isMin = await win.isMinimized().catch(() => false);
                    if (isMin) {
                        await win.unminimize().catch(() => {});
                    }
                    const pos = await win.outerPosition().catch(() => null);
                    if (pos) {
                        const isOffscreenX = pos.x < -1000;
                        const isOffscreenY = pos.y < -1000;
                        if (isOffscreenX || isOffscreenY) {
                            await win.center().catch(() => {});
                        }
                    }
                    await win.show().catch(() => {});
                    await win.setFocus().catch(() => {});
                }
            }, 100);
        };
        init();
    }, [loadConfig]);

    // Apply theme when config changes
    useEffect(() => {
        if (!config) return;

        const applyTheme = async (theme: string) => {
            const root = document.documentElement;
            const palette = findPalette(theme);
            const isDark = palette.dark;

            for (const item of THEME_PALETTES) {
                root.classList.remove(`palette-${item.id}`);
            }
            root.classList.add(`palette-${palette.id}`);
            root.style.setProperty('--bg', palette.bg);
            root.style.setProperty('--surface', palette.surface);
            root.style.setProperty('--primary', palette.primary);
            root.style.setProperty('--fg', palette.fg);

            const surfaceHover = palette.surfaceHover || `color-mix(in srgb, ${palette.surface} 88%, white)`;
            const primaryHover = palette.primaryHover || `color-mix(in srgb, ${palette.primary} 85%, white)`;
            const borderHover = palette.borderHover || palette.primary;

            root.style.setProperty('--surface-hover', surfaceHover);
            root.style.setProperty('--primary-hover', primaryHover);
            root.style.setProperty('--border-hover', borderHover);

            // Additional semantic mappings for components
            root.style.setProperty('--app-bg', palette.bg);
            root.style.setProperty('--app-surface', palette.surface);
            root.style.setProperty('--app-primary', palette.primary);
            root.style.setProperty('--app-border', isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.08)');
            root.style.setProperty('--app-surface-hover', surfaceHover);
            root.style.setProperty('--app-primary-hover', primaryHover);
            root.style.setProperty('--app-border-hover', borderHover);

            // Set Tauri window background color
            // Skip on Linux due to crash with transparent windows + softbuffer
            try {
                if (!isLinux() && (window as any).__TAURI_INTERNALS__) {
                    getCurrentWindow().setBackgroundColor(palette.bg).catch(e => {
                        console.error('Failed to set window background color:', e);
                        // Tracked in the error module; cosmetic only, theme still applied to DOM.
                        useErrorStore.getState().trackWarning(e, {
                          source: 'ThemeManager.applyTheme',
                          triggerAction: 'set_window_background',
                        });
                    });

                    const { invoke } = await import('@tauri-apps/api/core');
                    invoke('set_window_theme', { theme: isDark ? 'dark' : 'light' }).catch((e) => {
                        // Ignore errors on non-Windows platforms; tracked in the error module, theme still applied to DOM.
                        useErrorStore.getState().trackWarning(e, {
                          source: 'ThemeManager.applyTheme',
                          triggerAction: 'set_window_theme',
                        });
                    });
                }
            } catch (e) {
                console.error('Window background sync failed:', e);
                // Tracked in the error module; theme still applied to DOM below.
                useErrorStore.getState().trackWarning(e, {
                  source: 'ThemeManager.applyTheme',
                  triggerAction: 'sync_window_background',
                });
            }

            root.setAttribute('data-theme', isDark ? 'dark' : 'light');
            root.style.backgroundColor = palette.bg;
            root.style.color = palette.fg;

            if (isDark) {
                root.classList.add('dark');
            } else {
                root.classList.remove('dark');
            }
        };

        const theme = config.theme || 'system';

        // Sync to localStorage for early boot check
        localStorage.setItem('app-theme-preference', theme);

        if (theme === 'system') {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

            const handleSystemChange = (e: MediaQueryListEvent | MediaQueryList) => {
                const systemTheme = e.matches ? 'dark' : 'light';
                applyTheme(systemTheme);
            };

            // Initial alignment
            handleSystemChange(mediaQuery);

            // Listen for changes
            mediaQuery.addEventListener('change', handleSystemChange);
            return () => mediaQuery.removeEventListener('change', handleSystemChange);
        } else {
            applyTheme(theme);
        }
    }, [config?.theme]);

    return null; // This component handles side effects only
}
