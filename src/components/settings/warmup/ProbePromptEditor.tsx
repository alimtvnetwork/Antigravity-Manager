import React, { useState, useCallback } from 'react';
import { MessageSquareCode, RotateCcw, Check, Sparkles } from 'lucide-react';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';

export const DEFAULT_PROBE_PROMPTS = ['ping', 'Hi', 'OK', '1+1='] as const;
export type StandardProbePreset = (typeof DEFAULT_PROBE_PROMPTS)[number];

export interface ProbePromptEditorProps {
  probePrompt?: string;
  onChangePrompt?: (prompt: string) => void;
  className?: string;
}

interface PresetPillProps {
  preset: string;
  isSelected: boolean;
  onSelect: (preset: string) => void;
}

/**
 * Quick-select preset pill for canonical probe payloads.
 */
const PresetPill: React.FC<PresetPillProps> = ({ preset, isSelected, onSelect }) => {
  const handleClick = () => onSelect(preset);

  return (
    <button
      type="button"
      onClick={handleClick}
      className={cn(
        'text-[10px] px-2.5 py-1 rounded-full font-mono transition-colors cursor-pointer select-none border',
        isSelected
          ? 'bg-orange-500/20 text-orange-400 border-orange-500/40 font-semibold'
          : 'bg-zinc-800 text-zinc-300 border-transparent hover:bg-zinc-700 hover:text-zinc-100'
      )}
    >
      {isSelected && <Check className="w-2.5 h-2.5 inline mr-1" />}
      {preset}
    </button>
  );
};

/**
 * Custom Probe Prompt Editor.
 * Configures the micro-payload dispatched during quota warmup probes.
 */
export const ProbePromptEditor: React.FC<ProbePromptEditorProps> = ({
  probePrompt: propPrompt,
  onChangePrompt,
  className,
}) => {
  const [internalPrompt, setInternalPrompt] = useState<string>('ping');
  const activePrompt = propPrompt ?? internalPrompt;

  const handleUpdate = useCallback(
    (nextPrompt: string) => {
      try {
        setInternalPrompt(nextPrompt);
        if (onChangePrompt) {
          onChangePrompt(nextPrompt);
        }
      } catch (err) {
        useErrorStore.getState().trackWarning(err, {
          source: 'ProbePromptEditor',
          triggerAction: 'change_prompt',
        });
      }
    },
    [onChangePrompt]
  );

  const handleReset = useCallback(() => {
    handleUpdate('ping');
  }, [handleUpdate]);

  const isStandardPreset = DEFAULT_PROBE_PROMPTS.includes(
    activePrompt as StandardProbePreset
  );

  return (
    <div
      className={cn(
        'border border-zinc-800 rounded-xl bg-zinc-950/70 backdrop-blur-md p-4 flex flex-col gap-3',
        className
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <MessageSquareCode className="w-4 h-4 text-orange-400" />
          <h4 className="text-xs font-semibold text-zinc-100 uppercase tracking-wider">
            Custom Probe Prompt Editor
          </h4>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="text-[10px] text-zinc-500">Presets:</span>
          <div className="inline-flex p-0.5 bg-zinc-900 border border-zinc-800/80 rounded-full gap-1">
            {DEFAULT_PROBE_PROMPTS.map((preset) => (
              <PresetPill
                key={preset}
                preset={preset}
                isSelected={activePrompt === preset}
                onSelect={handleUpdate}
              />
            ))}
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <input
            type="text"
            value={activePrompt}
            onChange={(e) => handleUpdate(e.target.value)}
            placeholder="e.g. ping, Hi, 1+1="
            maxLength={64}
            className="w-full bg-zinc-900 border border-zinc-700/80 rounded-lg px-3 py-1.5 text-xs text-zinc-200 font-mono focus:border-orange-500 focus:outline-none focus:ring-1 focus:ring-orange-500"
          />
          <span className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] text-zinc-500 font-mono">
            {activePrompt.length}/64
          </span>
        </div>

        <button
          type="button"
          onClick={handleReset}
          title="Reset to default (ping)"
          className="p-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors cursor-pointer"
        >
          <RotateCcw className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="flex items-center justify-between text-[11px] text-zinc-400 bg-zinc-900/50 border border-zinc-800/60 rounded-lg px-3 py-1.5">
        <div className="flex items-center gap-1.5">
          <Sparkles className="w-3 h-3 text-orange-400/80 shrink-0" />
          <span>
            Dispatches micro-payload strictly capped at &le; 2 tokens to initialize 7-day quota clock.
          </span>
        </div>
        {!isStandardPreset && (
          <span className="text-[10px] font-mono text-amber-400 bg-amber-500/10 border border-amber-500/20 px-1.5 py-0.2 rounded">
            Custom Payload
          </span>
        )}
      </div>
    </div>
  );
};
