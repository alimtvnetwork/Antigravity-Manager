/**
 * Detect if the app is running in a Tauri environment
 */
export const isTauri = () => {
    return typeof window !== 'undefined' &&
        (!!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI__);
};

/**
 * Detect if running on Linux
 */
export const isLinux = () => {
    return typeof navigator !== 'undefined' && navigator.userAgent.toLowerCase().includes('linux');
};

/**
 * Detect if running on macOS
 */
export const isMacOS = () => {
    if (typeof navigator === 'undefined') return false;
    const ua = navigator.userAgent.toLowerCase();
    return ua.includes('macintosh') || ua.includes('mac os');
};

/**
 * Detect if running on Windows
 */
export const isWindows = () => {
    if (typeof navigator === 'undefined') return false;
    return navigator.userAgent.toLowerCase().includes('windows');
};

