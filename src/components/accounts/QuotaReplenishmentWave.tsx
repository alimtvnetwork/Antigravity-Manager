import { useEffect, useRef, useState } from 'react';
import { cn } from '../../utils/cn';

export interface QuotaReplenishmentWaveProps {
    trigger: boolean;
    onComplete?: () => void;
    className?: string;
}

function buildWaveGradient(
    ctx: CanvasRenderingContext2D,
    width: number
): CanvasGradient {
    const gradient = ctx.createLinearGradient(0, 0, width, 0);
    gradient.addColorStop(0, 'rgba(249, 115, 22, 0)');
    gradient.addColorStop(0.5, 'rgba(52, 211, 153, 0.4)');
    gradient.addColorStop(1, 'rgba(249, 115, 22, 0)');
    return gradient;
}

function drawWaveSlice(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    progress: number
): void {
    const waveHeadX = progress * (width + 120) - 60;
    const waveHeight = height * 0.45;

    ctx.beginPath();
    ctx.moveTo(0, height);
    for (let x = 0; x <= width; x += 4) {
        const distFromHead = x - waveHeadX;
        const gaussian = Math.exp(-Math.pow(distFromHead / 70, 2));
        const yOffset = Math.sin(x * 0.04 + progress * Math.PI * 2) * waveHeight * gaussian;
        ctx.lineTo(x, height * 0.5 - yOffset);
    }
    ctx.lineTo(width, height);
    ctx.closePath();
}

function renderWaveFrame(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    progress: number
): void {
    ctx.clearRect(0, 0, width, height);
    drawWaveSlice(ctx, width, height, progress);
    ctx.fillStyle = buildWaveGradient(ctx, width);
    ctx.fill();
}

function syncCanvasDimensions(canvas: HTMLCanvasElement): { width: number; height: number } {
    const rect = canvas.getBoundingClientRect();
    const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
    const width = Math.max(1, Math.floor(rect.width * dpr));
    const height = Math.max(1, Math.floor(rect.height * dpr));

    if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
    }
    return { width, height };
}

export function QuotaReplenishmentWave({
    trigger,
    onComplete,
    className,
}: QuotaReplenishmentWaveProps) {
    const canvasRef = useRef<HTMLCanvasElement | null>(null);
    const [isAnimating, setIsAnimating] = useState(false);
    const onCompleteRef = useRef(onComplete);
    onCompleteRef.current = onComplete;

    useEffect(() => {
        if (trigger) {
            setIsAnimating(true);
        }
    }, [trigger]);

    useEffect(() => {
        if (!isAnimating) return;
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const { width, height } = syncCanvasDimensions(canvas);
        const startTime = performance.now();
        let animFrameId = 0;

        const step = (now: number) => {
            const elapsed = now - startTime;
            const progress = Math.min(1, elapsed / 1200);
            renderWaveFrame(ctx, width, height, progress);

            if (progress < 1) {
                animFrameId = requestAnimationFrame(step);
            } else {
                ctx.clearRect(0, 0, width, height);
                setIsAnimating(false);
                onCompleteRef.current?.();
            }
        };

        animFrameId = requestAnimationFrame(step);
        return () => {
            cancelAnimationFrame(animFrameId);
        };
    }, [isAnimating]);

    if (!isAnimating && !trigger) {
        return null;
    }

    return (
        <div
            className={cn(
                'absolute inset-0 pointer-events-none overflow-hidden rounded-[inherit] z-10',
                className
            )}
            aria-hidden="true"
        >
            <canvas ref={canvasRef} className="w-full h-full block" />
        </div>
    );
}
