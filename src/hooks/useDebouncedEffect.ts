import { useEffect, useRef } from 'react';

/**
 * Executes a callback after a specified debounce delay, automatically cancelling
 * previous timers when dependencies update or the component unmounts.
 */
export function useDebouncedEffect(
  effect: () => void | (() => void),
  deps: React.DependencyList,
  delayMs: number = 300
): void {
  const cleanupRef = useRef<void | (() => void)>(undefined);

  useEffect(() => {
    const handler = setTimeout(() => {
      cleanupRef.current = effect();
    }, delayMs);

    return () => {
      clearTimeout(handler);
      if (typeof cleanupRef.current === 'function') {
        cleanupRef.current();
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, delayMs]);
}
