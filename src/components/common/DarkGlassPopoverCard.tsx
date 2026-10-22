import React, { useRef, useEffect } from 'react';
import { X } from 'lucide-react';

interface DarkGlassPopoverCardProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  anchorPosition?: 'bottom-left' | 'bottom-right' | 'top-left' | 'top-right';
  className?: string;
  showCloseButton?: boolean;
}

export const DarkGlassPopoverCard: React.FC<DarkGlassPopoverCardProps> = ({
  isOpen,
  onClose,
  title,
  children,
  anchorPosition = 'bottom-left',
  className = '',
  showCloseButton = true,
}) => {
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  let posClasses = 'top-full left-0 mt-1.5';
  if (anchorPosition === 'bottom-right') posClasses = 'top-full right-0 mt-1.5';
  if (anchorPosition === 'top-left') posClasses = 'bottom-full left-0 mb-1.5';
  if (anchorPosition === 'top-right') posClasses = 'bottom-full right-0 mb-1.5';

  return (
    <div
      ref={cardRef}
      role="dialog"
      aria-modal="false"
      className={`absolute ${posClasses} z-50 min-w-[240px] max-w-sm rounded-xl bg-slate-900/95 border border-slate-700/80 shadow-2xl backdrop-blur-xl p-3.5 text-slate-200 animate-in fade-in zoom-in-95 duration-100 ${className}`}
    >
      {(title || showCloseButton) && (
        <div className="flex items-center justify-between gap-2 pb-2 mb-2 border-b border-slate-800/80">
          {title && <span className="text-xs font-semibold text-slate-100 tracking-wide">{title}</span>}
          {showCloseButton && (
            <button
              type="button"
              onClick={onClose}
              className="p-1 rounded-md text-slate-400 hover:text-slate-100 hover:bg-slate-800/60 transition-colors ml-auto"
              aria-label="Close popover"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      )}
      <div className="text-xs text-slate-300 leading-relaxed">{children}</div>
    </div>
  );
};
