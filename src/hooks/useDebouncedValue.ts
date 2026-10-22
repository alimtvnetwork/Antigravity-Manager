import { useState, useEffect, useRef, useCallback } from 'react';

/**
 * Returns a debounced version of the input value with control handles.
 *
 * @param value The value to debounce.
 * @param delayMs Delay in milliseconds (default 250ms).
 */
export function useDebouncedValue<T>(value: T, delayMs = 250): [T, { flush: () => void; cancel: () => void }] {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const latestValueRef = useRef<T>(value);
  latestValueRef.current = value;

  const cancel = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const flush = useCallback(() => {
    cancel();
    setDebouncedValue(latestValueRef.current);
  }, [cancel]);

  useEffect(() => {
    timerRef.current = setTimeout(() => {
      setDebouncedValue(value);
    }, delayMs);

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [value, delayMs]);

  return [debouncedValue, { flush, cancel }];
}
