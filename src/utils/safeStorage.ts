/**
 * Safe LocalStorage Cache Wrapper
 * Handles QuotaExceeded errors, JSON serialization issues, and falls back to an in-memory map.
 */

const memoryStore = new Map<string, string>();

export function safeGetItem<T>(key: string, fallback: T): T {
  try {
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem(key) : memoryStore.get(key);
    if (raw === null || raw === undefined) {
      return fallback;
    }
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export function safeSetItem<T>(key: string, value: T): boolean {
  try {
    const serialized = JSON.stringify(value);
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(key, serialized);
    }
    memoryStore.set(key, serialized);
    return true;
  } catch {
    // Quota exceeded or storage disabled fallback to memory map only
    try {
      memoryStore.set(key, JSON.stringify(value));
      return true;
    } catch {
      return false;
    }
  }
}

export function safeRemoveItem(key: string): void {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(key);
    }
    memoryStore.delete(key);
  } catch {
    memoryStore.delete(key);
  }
}
