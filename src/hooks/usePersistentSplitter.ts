/**
 * Splitter Rail localStorage Persistence Hook
 * Manages draggable split-pane widths with persistent local storage caching and bound constraints.
 */

import { useState, useEffect, useCallback, useRef } from 'react';

export interface PersistentSplitterOptions {
  storageKey: string;
  defaultWidth: number;
  minWidth?: number;
  maxWidth?: number;
  direction?: 'horizontal' | 'vertical';
}

export function usePersistentSplitter({
  storageKey,
  defaultWidth,
  minWidth = 240,
  maxWidth = 760,
  direction = 'horizontal',
}: PersistentSplitterOptions) {
  const [width, setWidth] = useState<number>(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        const parsed = parseFloat(saved);
        if (!isNaN(parsed)) {
          return Math.min(Math.max(parsed, minWidth), maxWidth);
        }
      }
    } catch {
      // LocalStorage access failure fallback
    }
    return defaultWidth;
  });

  const [isDragging, setIsDragging] = useState(false);
  const startPosRef = useRef(0);
  const startWidthRef = useRef(0);

  const startDragging = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    startPosRef.current = direction === 'horizontal' ? e.clientX : e.clientY;
    startWidthRef.current = width;
  }, [width, direction]);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const currentPos = direction === 'horizontal' ? e.clientX : e.clientY;
      const delta = currentPos - startPosRef.current;
      const nextWidth = Math.min(Math.max(startWidthRef.current + delta, minWidth), maxWidth);
      setWidth(nextWidth);
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, direction, minWidth, maxWidth]);

  // Persist width when dragging completes
  useEffect(() => {
    if (!isDragging) {
      try {
        localStorage.setItem(storageKey, width.toString());
      } catch {
        // Ignored
      }
    }
  }, [width, isDragging, storageKey]);

  const resetWidth = useCallback(() => {
    setWidth(defaultWidth);
    try {
      localStorage.setItem(storageKey, defaultWidth.toString());
    } catch {
      // Ignored
    }
  }, [defaultWidth, storageKey]);

  return {
    width,
    setWidth,
    isDragging,
    startDragging,
    resetWidth,
  };
}
