import React, { useRef, useEffect, ReactNode } from 'react';
import { useClickOutside } from '../../hooks/useClickOutside';

export type PopoverPosition = 'bottom-left' | 'bottom-right' | 'top-left' | 'top-right';

export interface DarkGlassPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  trigger: ReactNode;
  children: ReactNode;
  position?: PopoverPosition;
  className?: string;
  closeOnEscape?: boolean;
}

export const DarkGlassPopover: React.FC<DarkGlassPopoverProps> = ({
  isOpen,
  onClose,
  trigger,
  children,
  position = 'bottom-right',
  className = '',
  closeOnEscape = true,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const popoverRef = useRef<HTMLDivElement | null>(null);

  useClickOutside(containerRef, () => {
    if (isOpen) onClose();
  }, isOpen);

  useEffect(() => {
    if (!isOpen) return;
    popoverRef.current?.focus();

    if (!closeOnEscape) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown, true);
    return () => window.removeEventListener('keydown', handleKeyDown, true);
  }, [isOpen, closeOnEscape, onClose]);

  const getPositionClasses = () => {
    switch (position) {
      case 'bottom-left':
        return 'top-full left-0 mt-2';
      case 'top-left':
        return 'bottom-full left-0 mb-2';
      case 'top-right':
        return 'bottom-full right-0 mb-2';
      case 'bottom-right':
      default:
        return 'top-full right-0 mt-2';
    }
  };

  return (
    <div ref={containerRef} className="relative inline-block">
      {trigger}

      {isOpen && (
        <div
          ref={popoverRef}
          role="dialog"
          aria-modal="true"
          tabIndex={-1}
          aria-label="Dialog popover"
          className={`absolute z-50 min-w-[200px] p-3 bg-slate-900/95 dark:bg-slate-950/95 border border-slate-700/80 rounded-2xl shadow-2xl backdrop-blur-xl focus:outline-none animate-in fade-in-0 zoom-in-95 ${getPositionClasses()} ${className}`}
        >
          {children}
        </div>
      )}
    </div>
  );
};
