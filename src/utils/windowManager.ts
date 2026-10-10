import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize, PhysicalSize, PhysicalPosition } from '@tauri-apps/api/dpi';
import { isTauri } from './env';
import { useViewStore } from '../stores/useViewStore';
import { useErrorStore } from '../stores/error-store';

let priorBounds: { x: number; y: number; width: number; height: number } | null = null;

/**
 * Enter mini view mode
 * @param contentHeight The height of the content to fit
 * @param shouldCenter Whether to center the window (default: false)
 */
export const enterMiniMode = async (contentHeight: number, shouldCenter: boolean = false) => {
    if (!isTauri()) return;
    try {
        const win = getCurrentWindow();

        // Save current window bounds before shrinking to mini mode
        const pos = await win.outerPosition().catch(() => null);
        const size = await win.outerSize().catch(() => null);
        if (pos && size && size.width > 350 && size.height > 250) {
            priorBounds = { x: pos.x, y: pos.y, width: size.width, height: size.height };
            useViewStore.getState().setSavedBounds(priorBounds);
        }

        // Hide window decorations (title bar) first to ensure accurate sizing
        await win.setDecorations(false);

        // Set window size: width 300, height = content height
        await win.setSize(new LogicalSize(300, contentHeight + 2));

        await win.setAlwaysOnTop(true);
        // Enable window shadow
        await win.setShadow(true);
        // Disable resizing in mini mode
        await win.setResizable(false);

        // Center window only if requested (usually on first load)
        if (shouldCenter) {
            await win.center();
        }
    } catch (error) {
        console.error('Failed to enter mini mode:', error);
        // Tracked in the error module; window keeps current state, user can toggle manually.
        useErrorStore.getState().trackWarning(error, {
          source: 'windowManager.enterMiniMode',
          triggerAction: 'enter_mini_mode',
        });
    }
};

/**
 * Exit mini view mode and restore prior window state
 */
export const exitMiniMode = async () => {
    if (!isTauri()) return;
    try {
        const win = getCurrentWindow();
        const storedBounds = priorBounds || useViewStore.getState().savedBounds;

        if (storedBounds && storedBounds.width > 500 && storedBounds.height > 400) {
            await win.setSize(new PhysicalSize(storedBounds.width, storedBounds.height));
            await win.setPosition(new PhysicalPosition(storedBounds.x, storedBounds.y));
        } else {
            // Restore to a reasonable default size if prior bounds unavailable
            await win.setSize(new LogicalSize(1200, 800));
            await win.center();
        }

        await win.setAlwaysOnTop(false);
        // Keep custom title bar decorations disabled
        await win.setDecorations(false);
        // Re-enable resizing
        await win.setResizable(true);
    } catch (error) {
        console.error('Failed to exit mini mode:', error);
        // Tracked in the error module; window keeps mini-mode state, user can toggle manually.
        useErrorStore.getState().trackWarning(error, {
          source: 'windowManager.exitMiniMode',
          triggerAction: 'exit_mini_mode',
        });
    }
};

/**
 * Ensure window is in valid full view state (Self-healing)
 * Used on app startup to recover from improper shutdown in mini mode
 */
export const ensureFullViewState = async () => {
    if (!isTauri()) return;
    try {
        const win = getCurrentWindow();
        const isMin = await win.isMinimized().catch(() => false);
        if (isMin) {
            await win.unminimize().catch((e) => {
                // Best-effort startup self-heal; tracked in the error module, remaining checks still run.
                useErrorStore.getState().trackWarning(e, { source: 'windowManager.ensureFullViewState', triggerAction: 'unminimize_window' });
            });
        }
        const pos = await win.outerPosition().catch(() => null);
        if (pos) {
            const isOffscreenX = pos.x < -1000;
            const isOffscreenY = pos.y < -1000;
            if (isOffscreenX || isOffscreenY) {
                await win.center().catch((e) => {
                    // Best-effort startup self-heal; tracked in the error module, remaining checks still run.
                    useErrorStore.getState().trackWarning(e, { source: 'windowManager.ensureFullViewState', triggerAction: 'center_window' });
                });
            }
        }
        const size = await win.outerSize().catch(() => null);
        if (size) {
            const isTooNarrow = size.width < 500;
            const isTooShort = size.height < 400;
            if (isTooNarrow || isTooShort) {
                await win.setSize(new LogicalSize(1200, 800)).catch((e) => {
                    // Best-effort window op; tracked in the error module, window keeps current size.
                    useErrorStore.getState().trackWarning(e, { source: 'windowManager.ensureSaneBounds', triggerAction: 'set_window_size' });
                });
                await win.center().catch((e) => {
                    // Best-effort window op; tracked in the error module.
                    useErrorStore.getState().trackWarning(e, { source: 'windowManager.ensureSaneBounds', triggerAction: 'center_window' });
                });
            }
        }
        // Enforce custom title bar (frameless) for Full View
        await win.setDecorations(false);
        await win.setResizable(true);
        await win.setAlwaysOnTop(false);
    } catch (error) {
        console.error('Failed to ensure full view state:', error);
        // Tracked in the error module; startup self-heal is best-effort, app continues regardless.
        useErrorStore.getState().trackWarning(error, {
          source: 'windowManager.ensureFullViewState',
          triggerAction: 'ensure_full_view_state',
        });
    }
};

/**
 * Unminimize, heal off-screen position and focus window safely
 */
export const unminimizeAndFocusWindow = async () => {
    if (!isTauri()) return;
    try {
        const win = getCurrentWindow();
        await win.show().catch((e) => {
            // Best-effort window op; tracked in the error module, remaining ops still attempted.
            useErrorStore.getState().trackWarning(e, { source: 'windowManager.unminimizeAndFocusWindow', triggerAction: 'show_window' });
        });
        await win.unminimize().catch((e) => {
            // Best-effort window op; tracked in the error module, remaining ops still attempted.
            useErrorStore.getState().trackWarning(e, { source: 'windowManager.unminimizeAndFocusWindow', triggerAction: 'unminimize_window' });
        });
        const pos = await win.outerPosition().catch(() => null);
        if (pos) {
            const isOffscreenX = pos.x < -1000;
            const isOffscreenY = pos.y < -1000;
            if (isOffscreenX || isOffscreenY) {
                await win.center().catch((e) => {
                    // Best-effort window op; tracked in the error module, focus still attempted.
                    useErrorStore.getState().trackWarning(e, { source: 'windowManager.unminimizeAndFocusWindow', triggerAction: 'center_window' });
                });
            }
        }
        await win.setFocus().catch((e) => {
            // Best-effort window op; tracked in the error module.
            useErrorStore.getState().trackWarning(e, { source: 'windowManager.unminimizeAndFocusWindow', triggerAction: 'focus_window' });
        });
    } catch (error) {
        console.error('Failed to unminimize and focus window:', error);
        // Tracked in the error module; window restore is best-effort, user can focus the window manually.
        useErrorStore.getState().trackWarning(error, {
          source: 'windowManager.unminimizeAndFocusWindow',
          triggerAction: 'unminimize_and_focus',
        });
    }
};
