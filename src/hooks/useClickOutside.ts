import { useEffect, type RefObject } from 'react';

type AnyRef = RefObject<HTMLElement | null>;

/**
 * Hook to detect clicks or touch taps outside of one or multiple referenced elements.
 * Useful for dismissing popovers, dropdowns, and modals.
 *
 * @param refs A single ref or an array of refs to treat as inside the container.
 * @param handler Callback invoked when a click or tap occurs outside all refs.
 * @param enabled Whether the outside click listener is active (defaults to true).
 */
export function useClickOutside(
  refs: AnyRef | AnyRef[],
  handler: (event: MouseEvent | TouchEvent) => void,
  enabled = true
): void {
  useEffect(() => {
    if (!enabled) return;

    const refList = Array.isArray(refs) ? refs : [refs];

    const listener = (event: MouseEvent | TouchEvent) => {
      const target = event.target as Node | null;
      if (!target) return;

      // Check if the click target is inside ANY of the provided refs
      const isInside = refList.some((ref) => {
        return ref.current && ref.current.contains(target);
      });

      if (!isInside) {
        handler(event);
      }
    };

    // Use mousedown and touchstart to capture the event before blur/focus shifts
    document.addEventListener('mousedown', listener, true);
    document.addEventListener('touchstart', listener, true);

    return () => {
      document.removeEventListener('mousedown', listener, true);
      document.removeEventListener('touchstart', listener, true);
    };
  }, [refs, handler, enabled]);
}
