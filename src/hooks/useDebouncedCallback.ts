import { useRef, useEffect, useCallback } from 'react';

export interface DebouncedOptions {
    leading?: boolean;
    trailing?: boolean;
}

export interface DebouncedCallback<T extends (...args: any[]) => any> {
    (...args: Parameters<T>): void;
    cancel: () => void;
    flush: () => void;
}

/**
 * useDebouncedCallback - Creates a debounced version of a callback function.
 *
 * Automatically clears pending timers on component unmount to avoid memory leaks.
 */
export function useDebouncedCallback<T extends (...args: any[]) => any>(
    callback: T,
    delay: number,
    options: DebouncedOptions = {}
): DebouncedCallback<T> {
    const { leading = false, trailing = true } = options;

    const callbackRef = useRef(callback);
    callbackRef.current = callback;

    const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const lastArgsRef = useRef<Parameters<T> | null>(null);
    const leadingCalledRef = useRef(false);

    const cancel = useCallback(() => {
        if (timeoutRef.current !== null) {
            clearTimeout(timeoutRef.current);
            timeoutRef.current = null;
        }
        lastArgsRef.current = null;
        leadingCalledRef.current = false;
    }, []);

    const flush = useCallback(() => {
        if (timeoutRef.current !== null && lastArgsRef.current !== null) {
            callbackRef.current(...lastArgsRef.current);
            cancel();
        }
    }, [cancel]);

    useEffect(() => {
        return () => {
            cancel();
        };
    }, [cancel]);

    const debounced = useCallback((...args: Parameters<T>) => {
        lastArgsRef.current = args;

        if (leading && !leadingCalledRef.current) {
            callbackRef.current(...args);
            leadingCalledRef.current = true;
        }

        if (timeoutRef.current !== null) {
            clearTimeout(timeoutRef.current);
        }

        timeoutRef.current = setTimeout(() => {
            if (trailing && (!leading || lastArgsRef.current !== null)) {
                if (lastArgsRef.current !== null) {
                    callbackRef.current(...lastArgsRef.current);
                }
            }
            timeoutRef.current = null;
            lastArgsRef.current = null;
            leadingCalledRef.current = false;
        }, delay);
    }, [delay, leading, trailing]);

    const result = debounced as DebouncedCallback<T>;
    result.cancel = cancel;
    result.flush = flush;

    return result;
}

export default useDebouncedCallback;
