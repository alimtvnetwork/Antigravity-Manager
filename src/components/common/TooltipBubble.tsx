import React, { useState, useRef, useId } from 'react';

export type TooltipPlacement = 'top' | 'bottom' | 'left' | 'right';

export interface TooltipBubbleProps {
  content: React.ReactNode;
  children: React.ReactNode;
  placement?: TooltipPlacement;
  delayMs?: number;
  className?: string;
}

export const TooltipBubble: React.FC<TooltipBubbleProps> = ({
  content,
  children,
  placement = 'top',
  delayMs = 150,
  className = '',
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const tooltipId = useId();

  const show = () => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setIsVisible(true), delayMs);
  };

  const hide = () => {
    if (timerRef.current) clearTimeout(timerRef.current);
    setIsVisible(false);
  };

  const placementStyles: Record<TooltipPlacement, string> = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
  };

  return (
    <div
      className="relative inline-flex items-center"
      aria-describedby={isVisible ? tooltipId : undefined}
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
    >
      {children}
      {isVisible && content && (
        <div
          id={tooltipId}
          role="tooltip"
          className={`absolute z-50 pointer-events-none px-2.5 py-1 text-xs font-normal text-slate-100 bg-slate-900/95 border border-slate-700/80 backdrop-blur-md rounded-lg shadow-xl whitespace-nowrap animate-in fade-in zoom-in-95 duration-100 ${placementStyles[placement]} ${className}`}
        >
          {content}
        </div>
      )}
    </div>
  );
};
