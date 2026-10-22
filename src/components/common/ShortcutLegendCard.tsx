import React from 'react';
import { Keyboard, X } from 'lucide-react';

export interface ShortcutItem {
  keys: string[];
  description: string;
  category?: string;
}

export interface ShortcutLegendCardProps {
  shortcuts?: ShortcutItem[];
  onClose?: () => void;
  className?: string;
}

const DEFAULT_SHORTCUTS: ShortcutItem[] = [
  { keys: ['⌘', 'K'], description: 'Focus search & filter input', category: 'Navigation' },
  { keys: ['Esc'], description: 'Close active modal or dropdown', category: 'Navigation' },
  { keys: ['N'], description: 'Focus prompt dispatch text box', category: 'Prompt Actions' },
  { keys: ['Tab'], description: 'Cycle through interactive controls', category: 'Accessibility' },
  { keys: ['⌘', 'C'], description: 'Copy selected prompt or code block', category: 'Clipboard' },
];

export const ShortcutLegendCard: React.FC<ShortcutLegendCardProps> = ({
  shortcuts = DEFAULT_SHORTCUTS,
  onClose,
  className = '',
}) => {
  return (
    <div
      className={`p-4 rounded-2xl bg-slate-900/90 dark:bg-slate-950/95 border border-slate-700/80 shadow-2xl backdrop-blur-xl max-w-md w-full ${className}`}
    >
      <div className="flex items-center justify-between pb-3 border-b border-slate-800">
        <div className="flex items-center gap-2 text-slate-200 font-semibold text-sm md:text-base">
          <Keyboard className="w-4 h-4 text-sky-400" />
          <span>Keyboard Shortcuts</span>
        </div>
        {onClose && (
          <button
            type="button"
            onClick={onClose}
            className="p-1 text-slate-400 hover:text-slate-200 rounded-lg hover:bg-slate-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        )}
      </div>

      <div className="mt-3 space-y-2.5">
        {shortcuts.map((sc, idx) => (
          <div
            key={`sc-${idx}-${sc.description}`}
            className="flex items-center justify-between text-xs py-1"
          >
            <span className="text-slate-300 font-medium">{sc.description}</span>
            <div className="flex items-center gap-1 shrink-0">
              {sc.keys.map((k, kIdx) => (
                <kbd
                  key={`key-${kIdx}-${k}`}
                  className="px-2 py-0.5 font-mono text-[11px] font-semibold text-slate-200 bg-slate-800 border border-slate-700 rounded-md shadow-sm"
                >
                  {k}
                </kbd>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
