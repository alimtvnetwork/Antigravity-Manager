import { useRef, useState, useEffect, type ReactElement } from 'react';
import { Check, Sparkles, X, Loader2 } from 'lucide-react';
import { useClickOutside } from '../../hooks/useClickOutside';
import { useErrorStore } from '../../stores/error-store';
import { cn } from '../../utils/cn';

export interface ModelOption {
  id: string;
  label: string;
  category: string;
  isDefault?: boolean;
}

export interface BatchWarmupModelPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  selectedCount: number;
  onApplyModels: (models: string[]) => Promise<void> | void;
  className?: string;
}

export const AVAILABLE_WARMUP_MODELS: ModelOption[] = [
  { id: 'gemini-2.5-flash', label: 'Gemini 2.5 Flash', category: 'High Speed', isDefault: true },
  { id: 'gemini-2.5-pro', label: 'Gemini 2.5 Pro', category: 'Reasoning', isDefault: true },
  { id: 'gemini-2.5-flash-thinking', label: 'Gemini 2.5 Flash Thinking', category: 'Deep Thinking' },
  { id: 'gemini-2.0-flash', label: 'Gemini 2.0 Flash', category: 'Standard' },
];

function toggleModelSelection(current: string[], targetId: string): string[] {
  if (current.includes(targetId)) {
    return current.filter((item) => item !== targetId);
  }
  return [...current, targetId];
}

function renderModelPill(
  model: ModelOption,
  isSelected: boolean,
  onToggle: (id: string) => void
): ReactElement {
  return (
    <button
      key={model.id}
      type="button"
      onClick={() => onToggle(model.id)}
      className={cn(
        'inline-flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-full border transition-all font-medium cursor-pointer select-none',
        isSelected
          ? 'bg-orange-500/15 text-orange-300 border-orange-500/40 shadow-xs'
          : 'bg-zinc-800/80 text-zinc-400 border-zinc-700/70 hover:border-zinc-500 hover:text-zinc-200'
      )}
    >
      <Check
        className={cn('w-3 h-3 transition-opacity', isSelected ? 'opacity-100 text-orange-400' : 'opacity-25')}
        aria-hidden="true"
      />
      <span>{model.label}</span>
    </button>
  );
}

function renderPopoverHeader(selectedCount: number, onClose: () => void): ReactElement {
  const countLabel = selectedCount === 1 ? '1 account' : `${selectedCount} accounts`;
  return (
    <div className="flex items-center justify-between pb-2 mb-2 border-b border-zinc-800">
      <div>
        <h4 className="text-xs font-semibold text-zinc-200">Warmup Target Models</h4>
        <p className="text-[11px] text-zinc-400">Assign models for {countLabel}</p>
      </div>
      <button
        type="button"
        onClick={onClose}
        className="p-1 text-zinc-400 hover:text-zinc-200 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
        aria-label="Close"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}

function renderApplyButton(
  selectedCount: number,
  isSubmitting: boolean,
  isDisabled: boolean,
  onApply: () => void
): ReactElement {
  const countLabel = selectedCount === 1 ? '1 account' : `${selectedCount} accounts`;
  return (
    <button
      type="button"
      disabled={isDisabled || isSubmitting}
      onClick={onApply}
      className="w-full mt-3 px-3 py-1.5 text-xs font-semibold rounded-xl bg-orange-600 hover:bg-orange-500 text-white shadow-md shadow-orange-950/40 transition-all flex items-center justify-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
    >
      {isSubmitting ? (
        <Loader2 className="w-3.5 h-3.5 animate-spin text-white" />
      ) : (
        <Sparkles className="w-3.5 h-3.5 text-orange-200" />
      )}
      <span>Apply to Selected ({countLabel})</span>
    </button>
  );
}

export function BatchWarmupModelPopover({
  isOpen,
  onClose,
  selectedCount,
  onApplyModels,
  className,
}: BatchWarmupModelPopoverProps): ReactElement | null {
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedModels, setSelectedModels] = useState<string[]>(['gemini-2.5-flash', 'gemini-2.5-pro']);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useClickOutside(containerRef, onClose, isOpen);

  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent): void => {
      if (e.key === 'Escape' && !isSubmitting) onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, isSubmitting, onClose]);

  if (!isOpen) return null;

  const handleToggle = (id: string): void => {
    setSelectedModels((prev) => toggleModelSelection(prev, id));
  };

  const handleApply = async (): Promise<void> => {
    setIsSubmitting(true);
    try {
      await onApplyModels(selectedModels);
      onClose();
    } catch (error) {
      useErrorStore.getState().trackWarning(error, {
        source: 'BatchWarmupModelPopover.handleApply',
        triggerAction: 'apply_warmup_models',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div
      ref={containerRef}
      role="dialog"
      aria-modal="false"
      className={cn(
        'absolute right-0 top-full mt-1.5 z-50 w-72 rounded-2xl bg-slate-900/95 border border-slate-700/80 shadow-2xl backdrop-blur-xl p-3 text-slate-200 animate-in fade-in zoom-in-95 duration-100',
        className
      )}
    >
      {renderPopoverHeader(selectedCount, onClose)}
      <div className="flex flex-wrap gap-1.5 py-1">
        {AVAILABLE_WARMUP_MODELS.map((model) =>
          renderModelPill(model, selectedModels.includes(model.id), handleToggle)
        )}
      </div>
      {renderApplyButton(
        selectedCount,
        isSubmitting,
        selectedModels.length === 0,
        handleApply
      )}
    </div>
  );
}

export default BatchWarmupModelPopover;
