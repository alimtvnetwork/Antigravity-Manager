import React, { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { cn } from '../../utils/cn';

export type TooltipPlacement = 'top' | 'bottom' | 'left' | 'right';

export interface TooltipTriggerProps {
    onMouseEnter?: React.MouseEventHandler<HTMLElement>;
    onMouseLeave?: React.MouseEventHandler<HTMLElement>;
    onFocus?: React.FocusEventHandler<HTMLElement>;
    onBlur?: React.FocusEventHandler<HTMLElement>;
    ref?: React.Ref<HTMLElement>;
}

export interface PortalTooltipProps {
    content: React.ReactNode;
    placement?: TooltipPlacement;
    delayMs?: number;
    children: React.ReactElement<TooltipTriggerProps>;
    className?: string;
    disabled?: boolean;
}

export function PortalTooltip({
    content,
    placement = 'top',
    delayMs = 150,
    children,
    className,
    disabled = false,
}: PortalTooltipProps) {
    const [isVisible, setIsVisible] = useState(false);
    const [coords, setCoords] = useState<{ top: number; left: number }>({ top: 0, left: 0 });
    const triggerRef = useRef<HTMLElement | null>(null);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const updatePosition = () => {
        if (!triggerRef.current) return;
        const rect = triggerRef.current.getBoundingClientRect();
        const gap = 8;
        let top = 0;
        let left = 0;

        switch (placement) {
            case 'top':
                top = rect.top - gap;
                left = rect.left + rect.width / 2;
                break;
            case 'bottom':
                top = rect.bottom + gap;
                left = rect.left + rect.width / 2;
                break;
            case 'left':
                top = rect.top + rect.height / 2;
                left = rect.left - gap;
                break;
            case 'right':
                top = rect.top + rect.height / 2;
                left = rect.right + gap;
                break;
        }

        // Viewport clamping
        const padding = 12;
        left = Math.max(padding, Math.min(window.innerWidth - padding, left));
        top = Math.max(padding, Math.min(window.innerHeight - padding, top));

        setCoords({ top, left });
    };

    const handleMouseEnter = () => {
        if (disabled || !content) return;
        timerRef.current = setTimeout(() => {
            updatePosition();
            setIsVisible(true);
        }, delayMs);
    };

    const handleMouseLeave = () => {
        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
        setIsVisible(false);
    };

    useEffect(() => {
        return () => {
            if (timerRef.current) {
                clearTimeout(timerRef.current);
            }
        };
    }, []);

    const placementTransform: Record<TooltipPlacement, string> = {
        top: '-translate-x-1/2 -translate-y-full',
        bottom: '-translate-x-1/2',
        left: '-translate-x-full -translate-y-1/2',
        right: '-translate-y-1/2',
    };

    return (
        <>
            {React.cloneElement(children, {
                ref: triggerRef,
                onMouseEnter: handleMouseEnter,
                onMouseLeave: handleMouseLeave,
                onFocus: handleMouseEnter,
                onBlur: handleMouseLeave,
            })}

            {isVisible &&
                !disabled &&
                content &&
                typeof document !== 'undefined' &&
                createPortal(
                    <div
                        role="tooltip"
                        style={{ top: `${coords.top}px`, left: `${coords.left}px` }}
                        className={cn(
                            'fixed z-50 pointer-events-none px-2.5 py-1.5 text-xs font-medium rounded-lg',
                            'bg-slate-900/95 dark:bg-slate-950/95 backdrop-blur-md text-slate-100',
                            'border border-slate-700/60 shadow-xl max-w-xs break-words',
                            'animate-in fade-in zoom-in-95 duration-100 transition-opacity',
                            placementTransform[placement],
                            className
                        )}
                    >
                        {content}
                    </div>,
                    document.body
                )}
        </>
    );
}

export default PortalTooltip;
