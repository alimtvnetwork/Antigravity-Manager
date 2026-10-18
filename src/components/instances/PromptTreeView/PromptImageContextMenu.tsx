import { useEffect, useRef, useCallback, useState } from 'react';
import { Copy, Eye, Download, Link2, Check, Layers, FileText } from 'lucide-react';
import { writeNativeImageToClipboard, copyAllImagesAsRichHtml, copyImageAsMarkdownText } from './clipboard';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';

export interface PromptImageContextMenuProps {
    isOpen: boolean;
    position: { x: number; y: number };
    imageSrc: string;
    altText?: string;
    allImages?: string[];
    onClose: () => void;
    onOpenPreview: (src: string, alt?: string) => void;
}

export function PromptImageContextMenu({
    isOpen,
    position,
    imageSrc,
    altText,
    allImages,
    onClose,
    onOpenPreview,
}: PromptImageContextMenuProps) {
    const menuRef = useRef<HTMLDivElement>(null);
    const [isImageCopied, setIsImageCopied] = useState(false);
    const [isAllImagesCopied, setIsAllImagesCopied] = useState(false);
    const [isMarkdownCopied, setIsMarkdownCopied] = useState(false);
    const [isLinkCopied, setIsLinkCopied] = useState(false);

    // Close on click outside or escape key
    useEffect(() => {
        if (!isOpen) return;

        const mountTime = Date.now();

        const handleClickOutside = (e: MouseEvent) => {
            if (Date.now() - mountTime < 100) return;
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                onClose();
            }
        };

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }
        };

        const handleScroll = (e: Event) => {
            if (Date.now() - mountTime < 150) return;
            if (menuRef.current && menuRef.current.contains(e.target as Node)) return;
            onClose();
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleKeyDown);
        window.addEventListener('scroll', handleScroll, true);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('scroll', handleScroll, true);
        };
    }, [isOpen, onClose]);

    const handleCopyImageBinary = useCallback(async () => {
        try {
            const isSuccess = await writeNativeImageToClipboard(
                imageSrc,
                () => {
                    setIsImageCopied(true);
                    setTimeout(() => {
                        setIsImageCopied(false);
                        onClose();
                    }, 1200);
                },
                (err) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'PromptImageContextMenu.handleCopyImageBinary',
                        triggerAction: 'context_menu_copy_image',
                    });
                }
            );
            if (isSuccess) {
                setIsImageCopied(true);
                setTimeout(() => {
                    setIsImageCopied(false);
                    onClose();
                }, 1200);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImageContextMenu.handleCopyImageBinary.outer',
                triggerAction: 'context_menu_copy_image_outer',
            });
        }
    }, [imageSrc, onClose]);

    const handleCopyAllImages = useCallback(async () => {
        try {
            const imagesToCopy = allImages && allImages.length > 0 ? allImages : [imageSrc];
            const isSuccess = await copyAllImagesAsRichHtml(
                imagesToCopy,
                () => {
                    setIsAllImagesCopied(true);
                    setTimeout(() => {
                        setIsAllImagesCopied(false);
                        onClose();
                    }, 1200);
                },
                (err) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'PromptImageContextMenu.handleCopyAllImages',
                        triggerAction: 'context_menu_copy_all_images',
                    });
                }
            );
            if (isSuccess) {
                setIsAllImagesCopied(true);
                setTimeout(() => {
                    setIsAllImagesCopied(false);
                    onClose();
                }, 1200);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImageContextMenu.handleCopyAllImages.outer',
                triggerAction: 'context_menu_copy_all_images_outer',
            });
        }
    }, [allImages, imageSrc, onClose]);

    const handleCopyImageAsText = useCallback(async () => {
        try {
            const isSuccess = await copyImageAsMarkdownText(
                imageSrc,
                altText || 'Image',
                () => {
                    setIsMarkdownCopied(true);
                    setTimeout(() => {
                        setIsMarkdownCopied(false);
                        onClose();
                    }, 1200);
                },
                (err) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'PromptImageContextMenu.handleCopyImageAsText',
                        triggerAction: 'context_menu_copy_image_as_text',
                    });
                }
            );
            if (isSuccess) {
                setIsMarkdownCopied(true);
                setTimeout(() => {
                    setIsMarkdownCopied(false);
                    onClose();
                }, 1200);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImageContextMenu.handleCopyImageAsText.outer',
                triggerAction: 'context_menu_copy_image_as_text_outer',
            });
        }
    }, [altText, imageSrc, onClose]);

    const handleCopyLink = useCallback(async () => {
        try {
            await copyToClipboard(imageSrc);
            setIsLinkCopied(true);
            setTimeout(() => {
                setIsLinkCopied(false);
                onClose();
            }, 1200);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImageContextMenu.handleCopyLink',
                triggerAction: 'context_menu_copy_link',
            });
        }
    }, [imageSrc, onClose]);

    const handleDownload = useCallback(() => {
        try {
            const a = document.createElement('a');
            a.href = imageSrc;
            const extMatch = imageSrc.match(/\.(png|jpe?g|webp|gif|svg)(?:\?.*)?$/i);
            const ext = extMatch ? extMatch[1].toLowerCase() : 'png';
            a.download = `prompt-attachment-${Date.now()}.${ext}`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            onClose();
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PromptImageContextMenu.handleDownload',
                triggerAction: 'context_menu_download',
            });
        }
    }, [imageSrc, onClose]);

    if (!isOpen) return null;

    // Viewport boundary clamping
    const menuWidth = 230;
    const menuHeight = 210;
    const x = Math.min(position.x, window.innerWidth - menuWidth - 12);
    const y = Math.min(position.y, window.innerHeight - menuHeight - 12);

    return (
        <div
            ref={menuRef}
            className="fixed z-[100050] min-w-[210px] rounded-xl border border-slate-700/60 bg-slate-900/95 backdrop-blur-md shadow-2xl p-1.5 text-xs text-slate-200 animate-in fade-in zoom-in-95 duration-100"
            style={{ left: Math.max(12, x), top: Math.max(12, y) }}
            onClick={(e) => e.stopPropagation()}
        >
            <div className="px-2.5 py-1 text-[10px] font-semibold text-slate-400 uppercase tracking-wider border-b border-slate-800/80 mb-1">
                Image Attachment
            </div>

            <button
                type="button"
                onClick={handleCopyImageBinary}
                className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left text-slate-200 hover:bg-purple-600/25 hover:text-purple-200 transition-colors cursor-pointer"
            >
                {isImageCopied ? (
                    <Check className="w-3.5 h-3.5 text-emerald-400" />
                ) : (
                    <Copy className="w-3.5 h-3.5 text-purple-400" />
                )}
                <span className="font-medium">
                    {isImageCopied ? 'Copied Single Image!' : 'Copy Single Image (PNG)'}
                </span>
            </button>

            <button
                type="button"
                onClick={handleCopyAllImages}
                className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left text-slate-200 hover:bg-purple-600/25 hover:text-purple-200 transition-colors cursor-pointer"
            >
                {isAllImagesCopied ? (
                    <Check className="w-3.5 h-3.5 text-emerald-400" />
                ) : (
                    <Layers className="w-3.5 h-3.5 text-purple-400" />
                )}
                <span className="font-medium">
                    {isAllImagesCopied ? 'Copied All Images!' : 'Copy All Attached Images'}
                </span>
            </button>

            <button
                type="button"
                onClick={handleCopyImageAsText}
                className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left text-slate-200 hover:bg-purple-600/25 hover:text-purple-200 transition-colors cursor-pointer"
            >
                {isMarkdownCopied ? (
                    <Check className="w-3.5 h-3.5 text-emerald-400" />
                ) : (
                    <FileText className="w-3.5 h-3.5 text-sky-400" />
                )}
                <span className="font-medium">
                    {isMarkdownCopied ? 'Copied Image as Text!' : 'Copy Image as Text (Markdown)'}
                </span>
            </button>

            <button
                type="button"
                onClick={() => {
                    onClose();
                    onOpenPreview(imageSrc, altText);
                }}
                className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left text-slate-200 hover:bg-slate-800 transition-colors cursor-pointer"
            >
                <Eye className="w-3.5 h-3.5 text-sky-400" />
                <span>View Fullscreen Lightbox</span>
            </button>

            <button
                type="button"
                onClick={handleCopyLink}
                className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left text-slate-200 hover:bg-slate-800 transition-colors cursor-pointer"
            >
                {isLinkCopied ? (
                    <Check className="w-3.5 h-3.5 text-emerald-400" />
                ) : (
                    <Link2 className="w-3.5 h-3.5 text-slate-400" />
                )}
                <span>{isLinkCopied ? 'Copied Data/URL!' : 'Copy Image URL / Data'}</span>
            </button>

            <div className="my-1 border-t border-slate-800" />

            <button
                type="button"
                onClick={handleDownload}
                className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left text-slate-200 hover:bg-slate-800 transition-colors cursor-pointer"
            >
                <Download className="w-3.5 h-3.5 text-emerald-400" />
                <span>Download Image File</span>
            </button>
        </div>
    );
}
