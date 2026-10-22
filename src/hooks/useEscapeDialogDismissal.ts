import { useEffect, useRef } from 'react';

interface UseEscapeDialogDismissalOptions {
  isOpen: boolean;
  onDismiss: () => void;
  stopPropagation?: boolean;
  disabled?: boolean;
}

/**
 * Custom hook to handle Escape key dismissal for dialogs, modals, and dropdown sheets.
 * Automatically manages listener lifecycle and prevents event bubbling to parent layers.
 */
export function useEscapeDialogDismissal({
  isOpen,
  onDismiss,
  stopPropagation = true,
  disabled = false,
}: UseEscapeDialogDismissalOptions) {
  const onDismissRef = useRef(onDismiss);
  onDismissRef.current = onDismiss;

  useEffect(() => {
    if (!isOpen || disabled) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        if (stopPropagation) {
          event.stopPropagation();
          event.preventDefault();
        }
        onDismissRef.current();
      }
    };

    window.addEventListener('keydown', handleKeyDown, true); // Capture phase to preempt nested handlers
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [isOpen, disabled, stopPropagation]);
}
