import { useEffect, useState, type ReactElement } from 'react';
import { createPortal } from 'react-dom';
import { Sparkles, ShieldCheck, Loader2, X } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';

export interface WarmupConfirmModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => Promise<void> | void;
  selectedCount: number;
  isWarmuping?: boolean;
}

function resolveTargetDescription(count: number): string {
  const target = count > 0 ? `${count} selected accounts` : 'all active accounts';
  return `This will probe ${target} to initialize their 7-day rolling quota windows.`;
}

function renderShieldBadge(): ReactElement {
  return (
    <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/10 border border-emerald-500/25 text-emerald-400 text-xs font-medium">
      <ShieldCheck className="w-4 h-4 shrink-0 text-emerald-400" aria-hidden="true" />
      <span>Zero excess token consumption guaranteed (Shield active).</span>
    </div>
  );
}

function renderIconHeader(): ReactElement {
  return (
    <div
      aria-hidden="true"
      className="w-12 h-12 rounded-full flex items-center justify-center mb-3 bg-orange-500/15 border border-orange-500/25 text-orange-400 shadow-sm shrink-0"
    >
      <Sparkles className="w-6 h-6 text-orange-400" />
    </div>
  );
}

function renderActionFooter(
  isProcessing: boolean,
  onClose: () => void,
  onConfirm: () => void
): ReactElement {
  return (
    <div className="flex gap-3 w-full pt-4 mt-auto border-t border-zinc-800">
      <button
        type="button"
        disabled={isProcessing}
        onClick={onClose}
        className="flex-1 px-4 py-2.5 text-sm font-medium rounded-xl text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/70 transition-colors disabled:opacity-50 cursor-pointer"
      >
        Cancel
      </button>
      <button
        type="button"
        disabled={isProcessing}
        onClick={onConfirm}
        className="flex-1 px-4 py-2.5 text-sm font-semibold rounded-xl bg-orange-600 hover:bg-orange-500 text-white shadow-lg shadow-orange-950/40 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
      >
        {isProcessing && <Loader2 className="w-4 h-4 animate-spin text-white" />}
        <span>Start Warmup</span>
      </button>
    </div>
  );
}

export function WarmupConfirmModal({
  isOpen,
  onClose,
  onConfirm,
  selectedCount,
  isWarmuping = false,
}: WarmupConfirmModalProps): ReactElement | null {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const isProcessing = isWarmuping || isSubmitting;

  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape' && !isProcessing) onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, isProcessing, onClose]);

  if (!isOpen) return null;

  const handleExecute = async (): Promise<void> => {
    setIsSubmitting(true);
    try {
      await onConfirm();
    } catch (error) {
      useErrorStore.getState().trackWarning(error, {
        source: 'WarmupConfirmModal.handleExecute',
        triggerAction: 'start_warmup',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return createPortal(
    <div className="modal modal-open z-[99999] fixed inset-0 items-center justify-center p-4 overflow-y-auto">
      <div data-tauri-drag-region className="fixed top-0 left-0 right-0 h-8 z-[100000]" />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="warmup-dialog-title"
        className="modal-box relative max-w-sm w-full bg-slate-900 border border-slate-800 shadow-2xl rounded-2xl p-6 flex flex-col items-center text-center overflow-hidden transform transition-all animate-in fade-in zoom-in-95 duration-200 z-10"
      >
        <button
          type="button"
          disabled={isProcessing}
          onClick={onClose}
          className="absolute top-3.5 right-3.5 p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/70 transition-colors cursor-pointer z-20"
          aria-label="Close"
        >
          <X className="w-4 h-4" />
        </button>

        {renderIconHeader()}
        <h3 id="warmup-dialog-title" className="text-lg font-bold text-zinc-100 mb-2 shrink-0">
          Confirm Account Warmup
        </h3>
        <p className="text-zinc-400 text-xs mb-4 leading-relaxed px-1">
          {resolveTargetDescription(selectedCount)}
        </p>
        <div className="mb-2 w-full flex justify-center">
          {renderShieldBadge()}
        </div>
        {renderActionFooter(isProcessing, onClose, handleExecute)}
      </div>
      <div
        className="modal-backdrop bg-black/60 backdrop-blur-sm fixed inset-0 z-[-1] pointer-events-auto"
        onClick={isProcessing ? undefined : onClose}
      />
    </div>,
    document.body
  );
}

export default WarmupConfirmModal;
