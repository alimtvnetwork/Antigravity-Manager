import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize, PhysicalSize, PhysicalPosition } from '@tauri-apps/api/dpi';
import { isTauri } from './env';
import { useViewStore } from '../stores/useViewStore';

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
        const size = await win.outerSize().catch(() => null);
        if (size) {
            const isTooNarrow = size.width < 500;
            const isTooShort = size.height < 400;
            if (isTooNarrow || isTooShort) {
                await win.setSize(new LogicalSize(1200, 800)).catch(() => {});
                await win.center().catch(() => {});
            }
        }
        // Enforce custom title bar (frameless) for Full View
        await win.setDecorations(false);
        await win.setResizable(true);
        await win.setAlwaysOnTop(false);
    } catch (error) {
        console.error('Failed to ensure full view state:', error);
    }
};

/**
 * Unminimize, heal off-screen position and focus window safely
 */
export const unminimizeAndFocusWindow = async () => {
    if (!isTauri()) return;
    try {
        const win = getCurrentWindow();
        await win.show().catch(() => {});
        await win.unminimize().catch(() => {});
        const pos = await win.outerPosition().catch(() => null);
        if (pos) {
            const isOffscreenX = pos.x < -1000;
            const isOffscreenY = pos.y < -1000;
            if (isOffscreenX || isOffscreenY) {
                await win.center().catch(() => {});
            }
        }
        await win.setFocus().catch(() => {});
    } catch (error) {
        console.error('Failed to unminimize and focus window:', error);
    }
};
