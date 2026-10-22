import React, { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';

interface ModalPortalLayerProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
  ariaLabel?: string;
  closeOnBackdropClick?: boolean;
  closeOnEscape?: boolean;
  className?: string;
}

export const ModalPortalLayer: React.FC<ModalPortalLayerProps> = ({
  isOpen,
  onClose,
  children,
  ariaLabel = 'Modal Dialog',
  closeOnBackdropClick = true,
  closeOnEscape = true,
  className = '',
}) => {
  const backdropRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen || !closeOnEscape) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, closeOnEscape, onClose]);

  if (!isOpen) return null;

  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (closeOnBackdropClick && e.target === backdropRef.current) {
      onClose();
    }
  };

  const portalContent = (
    <div
      ref={backdropRef}
      role="dialog"
      aria-modal="true"
      aria-label={ariaLabel}
      onClick={handleBackdropClick}
      className={`fixed inset-0 z-[99999] flex items-center justify-center bg-black/70 backdrop-blur-sm transition-opacity duration-200 p-4 ${className}`}
    >
      <div className="relative max-h-[90vh] overflow-y-auto w-full flex items-center justify-center">
        {children}
      </div>
    </div>
  );

  return createPortal(portalContent, document.body);
};
