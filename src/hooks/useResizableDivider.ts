import { useState, useEffect, useCallback, useRef } from 'react';

export interface ResizableOptions {
  initialWidth: number;
  minWidth?: number;
  maxWidth?: number;
  storageKey?: string;
}

/**
 * Hook for dragging and resizing sidebars, trees, or split view panels.
 */
export function useResizableDivider({
  initialWidth,
  minWidth = 240,
  maxWidth = 800,
  storageKey,
}: ResizableOptions) {
  const [width, setWidth] = useState<number>(() => {
    if (storageKey && typeof window !== 'undefined') {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        const parsed = parseInt(saved, 10);
        if (!isNaN(parsed) && parsed >= minWidth && parsed <= maxWidth) {
          return parsed;
        }
      }
    }
    return initialWidth;
  });

  const isDraggingRef = useRef(false);
  const startXRef = useRef(0);
  const startWidthRef = useRef(width);

  const startResizing = useCallback(
    (clientX: number) => {
      isDraggingRef.current = true;
      startXRef.current = clientX;
      startWidthRef.current = width;
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    },
    [width]
  );

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      startResizing(e.clientX);
    },
    [startResizing]
  );

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (e.touches.length > 0) {
        startResizing(e.touches[0].clientX);
      }
    },
    [startResizing]
  );

  useEffect(() => {
    const handleMove = (clientX: number) => {
      if (!isDraggingRef.current) return;
      const deltaX = clientX - startXRef.current;
      const newWidth = Math.min(maxWidth, Math.max(minWidth, startWidthRef.current + deltaX));
      setWidth(newWidth);
    };

    const handleMouseMove = (e: MouseEvent) => handleMove(e.clientX);
    const handleTouchMove = (e: TouchEvent) => {
      if (e.touches.length > 0) handleMove(e.touches[0].clientX);
    };

    const stopResizing = () => {
      if (!isDraggingRef.current) return;
      isDraggingRef.current = false;
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      if (storageKey) {
        localStorage.setItem(storageKey, String(width));
      }
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', stopResizing);
    window.addEventListener('touchmove', handleTouchMove);
    window.addEventListener('touchend', stopResizing);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', stopResizing);
      window.removeEventListener('touchmove', handleTouchMove);
      window.removeEventListener('touchend', stopResizing);
    };
  }, [width, minWidth, maxWidth, storageKey]);

  return {
    width,
    setWidth,
    handleMouseDown,
    handleTouchStart,
    isDragging: isDraggingRef.current,
  };
}
