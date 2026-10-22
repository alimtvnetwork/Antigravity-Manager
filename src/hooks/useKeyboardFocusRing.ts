import { useState, useEffect } from 'react';

/**
 * Hook to detect whether the user is actively navigating via keyboard (Tab / Arrow keys).
 * Conditionally enables high-contrast keyboard focus rings while suppressing rings on mouse clicks.
 */
export function useKeyboardFocusRing(): boolean {
  const [isKeyboardUser, setIsKeyboardUser] = useState<boolean>(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Tab' || e.key.startsWith('Arrow')) {
        setIsKeyboardUser(true);
      }
    };

    const handlePointerDown = () => {
      setIsKeyboardUser(false);
    };

    window.addEventListener('keydown', handleKeyDown, true);
    window.addEventListener('pointerdown', handlePointerDown, true);

    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
      window.removeEventListener('pointerdown', handlePointerDown, true);
    };
  }, []);

  return isKeyboardUser;
}
