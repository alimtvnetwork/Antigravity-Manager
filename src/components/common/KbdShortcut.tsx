import React, { useMemo } from 'react';

export type KbdSize = 'xs' | 'sm' | 'md';
export type KbdVariant = 'default' | 'subtle' | 'emerald' | 'purple';

export interface KbdShortcutProps {
    /**
     * Shortcut definition string (e.g. "Mod+K", "Ctrl+Shift+P", "Enter")
     * or explicit array of key identifiers (e.g. ["Ctrl", "K"]).
     */
    shortcut?: string;
    keys?: string[];
    /**
     * Size scale: 'xs' (9px), 'sm' (10px, default), 'md' (11px).
     */
    size?: KbdSize;
    /**
     * Visual variant for borders and background tints.
     */
    variant?: KbdVariant;
    /**
     * Optional custom separator between key capsules (default: "+").
     */
    separator?: string;
    /**
     * Additional Tailwind class names.
     */
    className?: string;
    /**
     * Whether to render native OS symbols on macOS (e.g. ⌘, ⌥, ⇧) or text (default: true).
     */
    useMacSymbols?: boolean;
}

function isApplePlatform(): boolean {
    if (typeof window === 'undefined' || typeof navigator === 'undefined') {
        return false;
    }
    const platform = (navigator as { userAgentData?: { platform?: string } }).userAgentData?.platform
        || navigator.platform
        || navigator.userAgent;
    return /mac|iphone|ipad|ipod/i.test(platform);
}

function normalizeKeyLabel(rawKey: string, isMac: boolean, useMacSymbols: boolean): string {
    const key = rawKey.trim();
    const lower = key.toLowerCase();

    if (lower === 'mod' || lower === 'cmd' || lower === 'command') {
        if (isMac) {
            return useMacSymbols ? '⌘' : 'Cmd';
        }
        return 'Ctrl';
    }

    if (lower === 'ctrl' || lower === 'control') {
        if (isMac && useMacSymbols) {
            return '⌃';
        }
        return isMac ? 'Control' : 'Ctrl';
    }

    if (lower === 'alt' || lower === 'opt' || lower === 'option') {
        if (isMac) {
            return useMacSymbols ? '⌥' : 'Option';
        }
        return 'Alt';
    }

    if (lower === 'shift') {
        return isMac && useMacSymbols ? '⇧' : 'Shift';
    }

    if (lower === 'enter' || lower === 'return') {
        return isMac && useMacSymbols ? '↵' : 'Enter';
    }

    if (lower === 'esc' || lower === 'escape') {
        return 'Esc';
    }

    if (lower === 'backspace') {
        return isMac && useMacSymbols ? '⌫' : 'Backspace';
    }

    if (lower === 'tab') {
        return isMac && useMacSymbols ? '⇥' : 'Tab';
    }

    if (lower === 'space') {
        return 'Space';
    }

    // Single character capitalization
    if (key.length === 1) {
        return key.toUpperCase();
    }

    return key;
}

export const KbdShortcut: React.FC<KbdShortcutProps> = ({
    shortcut,
    keys: explicitKeys,
    size = 'sm',
    variant = 'default',
    separator,
    className = '',
    useMacSymbols = true,
}) => {
    const isMac = useMemo(() => isApplePlatform(), []);

    const parsedKeys = useMemo(() => {
        if (explicitKeys && explicitKeys.length > 0) {
            return explicitKeys;
        }
        if (shortcut) {
            return shortcut.split('+').map(k => k.trim()).filter(Boolean);
        }
        return [];
    }, [shortcut, explicitKeys]);

    const formattedKeys = useMemo(() => {
        return parsedKeys.map(k => normalizeKeyLabel(k, isMac, useMacSymbols));
    }, [parsedKeys, isMac, useMacSymbols]);

    if (formattedKeys.length === 0) {
        return null;
    }

    const sizeClasses = {
        xs: 'px-1 py-0.5 text-[9px] min-w-[16px]',
        sm: 'px-1.5 py-0.5 text-[10px] min-w-[18px]',
        md: 'px-2 py-0.5 text-[11px] min-w-[20px]',
    }[size];

    const variantClasses = {
        default: 'bg-slate-100 dark:bg-slate-800/90 text-slate-700 dark:text-slate-200 border-slate-300 dark:border-slate-700/70 shadow-xs',
        subtle: 'bg-transparent text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-800',
        emerald: 'bg-emerald-500/10 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border-emerald-300/40 dark:border-emerald-700/50',
        purple: 'bg-purple-500/10 dark:bg-purple-950/40 text-purple-700 dark:text-purple-300 border-purple-300/40 dark:border-purple-700/50',
    }[variant];

    const ariaLabel = parsedKeys.join(' plus ');

    return (
        <span
            className={`inline-flex items-center gap-1 font-mono font-medium select-none ${className}`}
            aria-label={ariaLabel}
        >
            {formattedKeys.map((keyLabel, idx) => (
                <React.Fragment key={`${keyLabel}-${idx}`}>
                    <kbd
                        className={`inline-flex items-center justify-center rounded border text-center font-bold leading-none tracking-tight transition-colors ${sizeClasses} ${variantClasses}`}
                    >
                        {keyLabel}
                    </kbd>
                    {separator && idx < formattedKeys.length - 1 && (
                        <span className="text-slate-400 dark:text-slate-600 text-[10px] font-sans font-light">
                            {separator}
                        </span>
                    )}
                </React.Fragment>
            ))}
        </span>
    );
};

export default KbdShortcut;
