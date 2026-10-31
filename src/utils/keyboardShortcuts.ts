/**
 * Imp-89: Centralized Keyboard Shortcut Registry
 * Handles global keybindings with active input protection (skips execution when inside input, textarea, contenteditable).
 */

export interface KeyboardShortcut {
    key: string;
    ctrlOrMeta?: boolean;
    shift?: boolean;
    alt?: boolean;
    description: string;
    action: () => void;
}

export function isInputElement(element: EventTarget | null): boolean {
    if (!element || !(element instanceof HTMLElement)) {
        return false;
    }
    const tag = element.tagName.toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') {
        return true;
    }
    return element.isContentEditable;
}

export function registerGlobalShortcut(shortcut: KeyboardShortcut): () => void {
    const handler = (event: KeyboardEvent) => {
        if (isInputElement(event.target)) {
            return;
        }

        const matchesKey = event.key.toLowerCase() === shortcut.key.toLowerCase();
        const matchesCtrl = shortcut.ctrlOrMeta ? (event.ctrlKey || event.metaKey) : (!event.ctrlKey && !event.metaKey);
        const matchesShift = shortcut.shift ? event.shiftKey : !event.shiftKey;
        const matchesAlt = shortcut.alt ? event.altKey : !event.altKey;

        if (matchesKey && matchesCtrl && matchesShift && matchesAlt) {
            event.preventDefault();
            shortcut.action();
        }
    };

    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
}
