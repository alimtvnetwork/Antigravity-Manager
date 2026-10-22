import { useEffect, RefObject } from 'react';

/**
 * Custom hook that listens for mousedown or touchstart events outside the referenced element(s).
 */
export function useClickOutsideListener(
  refOrRefs: RefObject<HTMLElement | null> | RefObject<HTMLElement | null>[],
  onClickOutside: () => void,
  enabled: boolean = true
) {
  useEffect(() => {
    if (!enabled) return;

    const handleEvent = (event: MouseEvent | TouchEvent) => {
      const target = event.target as Node | null;
      if (!target) return;

      const refs = Array.isArray(refOrRefs) ? refOrRefs : [refOrRefs];

      // If any of the refs contain the click target, do not trigger outside callback
      const isInside = refs.some((r) => r.current && r.current.contains(target));
      if (!isInside) {
        onClickOutside();
      }
    };

    document.addEventListener('mousedown', handleEvent, true);
    document.addEventListener('touchstart', handleEvent, true);

    return () => {
      document.removeEventListener('mousedown', handleEvent, true);
      document.removeEventListener('touchstart', handleEvent, true);
    };
  }, [refOrRefs, onClickOutside, enabled]);
}
