/**
 * Keyboard Shortcut Tooltip Stacker
 * Formats multi-platform keyboard shortcuts with native modifier glyphs and keys.
 */

export function getPlatformIsMac(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Mac|iPhone|iPod|iPad/i.test(navigator.platform || navigator.userAgent);
}

export function formatShortcut(shortcut: string, isMac = getPlatformIsMac()): string {
  if (!shortcut) return '';

  const parts = shortcut.split('+').map(p => p.trim());
  if (!isMac) {
    return parts.join('+');
  }

  const macGlyphs: Record<string, string> = {
    ctrl: '⌃',
    control: '⌃',
    alt: '⌥',
    option: '⌥',
    shift: '⇧',
    cmd: '⌘',
    command: '⌘',
    meta: '⌘',
    enter: '↵',
    return: '↵',
    backspace: '⌫',
    delete: '⌦',
    esc: '⎋',
    escape: '⎋',
  };

  return parts
    .map(p => {
      const lower = p.toLowerCase();
      return macGlyphs[lower] || p.toUpperCase();
    })
    .join('');
}

export function splitShortcutKeys(shortcut: string, isMac = getPlatformIsMac()): string[] {
  if (!shortcut) return [];
  const parts = shortcut.split('+').map(p => p.trim());
  if (!isMac) return parts;

  return parts.map(p => {
    const lower = p.toLowerCase();
    switch (lower) {
      case 'ctrl':
      case 'control':
        return '⌃';
      case 'alt':
      case 'option':
        return '⌥';
      case 'shift':
        return '⇧';
      case 'cmd':
      case 'command':
      case 'meta':
        return '⌘';
      default:
        return p.toUpperCase();
    }
  });
}
