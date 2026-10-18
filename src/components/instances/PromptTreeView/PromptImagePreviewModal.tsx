import { useState, useEffect, useCallback, useRef, type WheelEvent, type MouseEvent } from 'react';
import { ZoomIn, ZoomOut, RotateCcw, Copy, Check, Download, X, Image as ImageIcon } from 'lucide-react';
import { writeNativeImageToClipboard } from './clipboard';
import { useErrorStore } from '../../../stores/error-store';

export interface PromptImagePreviewModalProps {
    isOpen: boolean;
    imageSrc: string | null;
    altText?: string;
    onClose: () => void;
}

export function PromptImagePreviewModal({
    isOpen,
    imageSrc,
    altText,
    onClose,
}: PromptImagePreviewModalProps) {
    const [zoomScale, setZoomScale] = useState(1);
    const [panPosition, setPanPosition] = useState({ x: 0, y: 0 });
    const [isPanning, setIsPanning] = useState(false);
    const [isCopied, setIsCopied] = useState(false);
    const dragStartRef = useRef({ x: 0, y: 0, initialPanX: 0, initialPanY: 0 });

    const handleResetZoom = useCallback(() => {
        setZoomScale(1);
        setPanPosition({ x: 0, y: 0 });
    }, []);

    const handleZoomIn = useCallback(() => {
        setZoomScale((prev) => Math.min(5, Math.round((prev + 0.25) * 100) / 100));
    }, []);

    const handleZoomOut = useCallback(() => {
        setZoomScale((prev) => Math.max(0.25, Math.round((prev - 0.25) * 100) / 100));
    }, []);

    // Reset view whenever a new image opens
    useEffect(() => {
        if (isOpen) {
            handleResetZoom();
            setIsCopied(false);
        }
    }, [isOpen, imageSrc, handleResetZoom]);

    // Handle Escape key and keyboard shortcuts
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            } else if (e.key === '+' || e.key === '=') {
                e.preventDefault();
                handleZoomIn();
            } else if (e.key === '-') {
                e.preventDefault();
                handleZoomOut();
            } else if (e.key === '0') {
                e.preventDefault();
                handleResetZoom();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose, handleZoomIn, handleZoomOut, handleResetZoom]);

    // Wheel zoom
    const handleWheel = useCallback((e: WheelEvent) => {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 0.15 : -0.15;
        setZoomScale((prev) => Math.min(5, Math.max(0.25, Math.round((prev + delta) * 100) / 100)));
    }, []);

    // Pan / Drag handlers
    const handleMouseDown = useCallback((e: MouseEvent) => {
        if (e.button !== 0) return;
        setIsPanning(true);
        dragStartRef.current = {
            x: e.clientX,
            y: e.clientY,
            initialPanX: panPosition.x,
            initialPanY: panPosition.y,
        };
    }, [panPosition]);

    const handleMouseMove = useCallback((e: MouseEvent) => {
        if (!isPanning) return;
        const dx = e.clientX - dragStartRef.current.x;
        const dy = e.clientY - dragStartRef.current.y;
        setPanPosition({
            x: dragStartRef.current.initialPanX + dx,
            y: dragStartRef.current.initialPanY + dy,
        });
    }, [isPanning]);

    const handleMouseUp = useCallback(() => {
        setIsPanning(false);
    }, []);

    const handleCopyNative = useCallback(async () => {
        if (!imageSrc) return;
        try {
            const isSuccess = await writeNativeImageToClipboard(
                imageSrc,
                () => {
                    setIsCopied(true);
                    setTimeout(() => setIsCopied(false), 2200);
                },
                (err) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'PromptImagePreviewModal.handleCopyNative',
                        triggerAction: 'write_native_image_preview',
                    });
                }
            );
            if (isSuccess) {
                setIsCopied(true);
                setTimeout(() => setIsCopied(false), 2200);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImagePreviewModal.handleCopyNative.outer',
                triggerAction: 'copy_native_image_outer',
            });
        }
    }, [imageSrc]);

    const handleDownload = useCallback(() => {
        if (!imageSrc) return;
        try {
            const a = document.createElement('a');
            a.href = imageSrc;
            const extMatch = imageSrc.match(/\.(png|jpe?g|webp|gif|svg)(?:\?.*)?$/i);
            const ext = extMatch ? extMatch[1].toLowerCase() : 'png';
            a.download = `prompt-attachment-${Date.now()}.${ext}`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImagePreviewModal.handleDownload',
                triggerAction: 'download_image',
            });
        }
    }, [imageSrc]);

    if (!isOpen || !imageSrc) return null;

    const isZoomed = zoomScale > 1;

    return (
        <div
            className="fixed inset-0 z-[100050] flex flex-col items-center justify-between bg-black/85 backdrop-blur-md select-none transition-opacity duration-200"
            onClick={onClose}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
        >
            {/* Top Toolbar / Header Capsule */}
            <div
                className="w-full flex items-center justify-between px-6 py-4 pointer-events-auto"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center gap-2 text-slate-300 text-xs font-medium">
                    <ImageIcon className="w-4 h-4 text-purple-400" />
                    <span className="truncate max-w-sm text-slate-200">
                        {altText || 'Prompt Image Attachment'}
                    </span>
                </div>

                {/* Segmented Capsule Actions */}
                <div className="inline-flex items-center rounded-lg border border-slate-700/60 bg-slate-900/90 shadow-lg px-2 py-1 gap-1 text-slate-300 text-xs">
                    <button
                        type="button"
                        onClick={handleZoomOut}
                        disabled={zoomScale <= 0.25}
                        className="p-1.5 rounded-md hover:bg-slate-800 disabled:opacity-40 transition-colors cursor-pointer"
                        title="Zoom Out (-)"
                    >
                        <ZoomOut className="w-3.5 h-3.5" />
                    </button>
                    <span className="px-1.5 font-mono text-[11px] min-w-[48px] text-center text-purple-300">
                        {Math.round(zoomScale * 100)}%
                    </span>
                    <button
                        type="button"
                        onClick={handleZoomIn}
                        disabled={zoomScale >= 5}
                        className="p-1.5 rounded-md hover:bg-slate-800 disabled:opacity-40 transition-colors cursor-pointer"
                        title="Zoom In (+)"
                    >
                        <ZoomIn className="w-3.5 h-3.5" />
                    </button>
                    <div className="w-[1px] h-4 bg-slate-700 mx-0.5" />
                    <button
                        type="button"
                        onClick={handleResetZoom}
                        className="p-1.5 rounded-md hover:bg-slate-800 transition-colors cursor-pointer"
                        title="Reset Zoom (0)"
                    >
                        <RotateCcw className="w-3.5 h-3.5" />
                    </button>
                    <div className="w-[1px] h-4 bg-slate-700 mx-0.5" />
                    <button
                        type="button"
                        onClick={handleCopyNative}
                        className="flex items-center gap-1.5 px-3 py-1 rounded-md bg-purple-600/30 hover:bg-purple-600/50 text-purple-200 transition-colors cursor-pointer font-medium"
                        title="Copy native PNG image binary to clipboard for pasting into Gemini Web, Photoshop, Paint"
                    >
                        {isCopied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                        <span>{isCopied ? 'Copied PNG!' : 'Copy PNG'}</span>
                    </button>
                    <button
                        type="button"
                        onClick={handleDownload}
                        className="p-1.5 rounded-md hover:bg-slate-800 transition-colors cursor-pointer"
                        title="Download image file"
                    >
                        <Download className="w-3.5 h-3.5" />
                    </button>
                    <div className="w-[1px] h-4 bg-slate-700 mx-0.5" />
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-md hover:bg-red-500/20 hover:text-red-300 transition-colors cursor-pointer text-slate-400"
                        title="Close (Esc)"
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Central Canvas Viewport */}
            <div
                className="relative flex-1 w-full flex items-center justify-center overflow-hidden cursor-grab active:cursor-grabbing p-4"
                onWheel={handleWheel}
                onMouseDown={handleMouseDown}
                onClick={(e) => e.stopPropagation()}
            >
                <img
                    src={imageSrc}
                    alt={altText || 'Preview'}
                    draggable={false}
                    className="max-h-[82vh] max-w-[90vw] object-contain rounded-lg shadow-2xl transition-transform duration-75 ease-out select-none"
                    style={{
                        transform: `translate(${panPosition.x}px, ${panPosition.y}px) scale(${zoomScale})`,
                        cursor: isZoomed ? (isPanning ? 'grabbing' : 'grab') : 'default',
                    }}
                />
            </div>

            {/* Footer Hint */}
            <div className="py-2 text-[11px] text-slate-500 font-mono pointer-events-none">
                Scroll to zoom · Drag to pan · Double click or press 0 to reset · Esc to close
            </div>
        </div>
    );
}
