import { useState, useEffect, useCallback } from 'react';
import { useErrorStore } from '../stores/error-store';

export function usePersistentState<T>(key: string, initialValue: T): [T, (value: T | ((prev: T) => T)) => void] {
  const [state, setState] = useState<T>(() => {
    try {
      const stored = localStorage.getItem(key);
      if (stored !== null) {
        return JSON.parse(stored) as T;
      }
    } catch (e) {
      useErrorStore.getState().trackWarning(e, {
        source: 'usePersistentState.init',
        triggerAction: `read_${key}`,
      });
    }
    return initialValue;
  });

  const setPersistentState = useCallback(
    (value: T | ((prev: T) => T)) => {
      setState((prev) => {
        const nextValue = typeof value === 'function' ? (value as (prev: T) => T)(prev) : value;
        try {
          localStorage.setItem(key, JSON.stringify(nextValue));
        } catch (e) {
          useErrorStore.getState().trackWarning(e, {
            source: 'usePersistentState.save',
            triggerAction: `write_${key}`,
          });
        }
        return nextValue;
      });
    },
    [key]
  );

  useEffect(() => {
    const handleStorageEvent = (e: StorageEvent) => {
      if (e.key === key && e.newValue !== null) {
        try {
          setState(JSON.parse(e.newValue) as T);
        } catch {
          // Ignore external JSON parse failure
        }
      }
    };

    window.addEventListener('storage', handleStorageEvent);
    return () => window.removeEventListener('storage', handleStorageEvent);
  }, [key]);

  return [state, setPersistentState];
}
