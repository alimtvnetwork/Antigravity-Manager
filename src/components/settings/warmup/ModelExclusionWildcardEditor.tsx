import { useState } from 'react';
import { Filter, X, Plus } from 'lucide-react';
import { useWarmupStore } from '../../../stores/useWarmupStore';

export interface ModelExclusionWildcardEditorProps {
  patterns?: string[];
  onChange?: (patterns: string[]) => void;
  disabled?: boolean;
  className?: string;
}

/**
 * Checks whether a given model identifier matches any wildcard exclusion pattern.
 * Converts wildcard patterns (e.g. *-thinking-*) into regexes.
 */
export function isModelExcluded(modelId: string, patterns: string[]): boolean {
  if (!patterns || patterns.length === 0) return false;
  const target = modelId.toLowerCase().trim();

  return patterns.some((pattern) => {
    const trimmed = pattern.toLowerCase().trim();
    if (!trimmed) return false;

    // Convert glob pattern to regex: escape regex special chars except '*', then replace '*' with '.*'
    const escaped = trimmed
      .replace(/[.+?^${}()|[\]\\]/g, '\\$&')
      .replace(/\*/g, '.*');

    const regex = new RegExp(`^${escaped}$`, 'i');
    return regex.test(target);
  });
}

/**
 * ModelExclusionWildcardEditor provides interactive chip management for model
 * patterns to be excluded from automated warmup probes (e.g. thinking or experimental variants).
 */
export function ModelExclusionWildcardEditor({
  patterns,
  onChange,
  disabled = false,
  className = '',
}: ModelExclusionWildcardEditorProps) {
  const [inputValue, setInputValue] = useState('');
  const storePatterns = useWarmupStore((state) => state.config.excludedPatterns);
  const addStorePattern = useWarmupStore((state) => state.addExcludedPattern);
  const removeStorePattern = useWarmupStore((state) => state.removeExcludedPattern);
  const setStorePatterns = useWarmupStore((state) => state.setExcludedPatterns);

  const currentPatterns = patterns !== undefined ? patterns : storePatterns;

  const handleAdd = () => {
    const trimmed = inputValue.trim();
    if (!trimmed || disabled) return;

    if (onChange) {
      if (!currentPatterns.includes(trimmed)) {
        onChange([...currentPatterns, trimmed]);
      }
    } else {
      addStorePattern(trimmed);
    }
    setInputValue('');
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAdd();
    }
  };

  const handleRemove = (patternToRemove: string) => {
    if (disabled) return;
    if (onChange) {
      onChange(currentPatterns.filter((p) => p !== patternToRemove));
    } else {
      removeStorePattern(patternToRemove);
    }
  };

  const handleResetDefaults = () => {
    const defaults = ['*-thinking-*', '*-exp-*', '*-preview'];
    if (onChange) {
      onChange(defaults);
    } else {
      setStorePatterns(defaults);
    }
  };

  return (
    <div
      className={`rounded-xl border border-zinc-800 bg-zinc-900/60 p-4 transition-all duration-200 hover:border-zinc-700/80 ${className}`}
    >
      <div className="flex items-center justify-between gap-2 mb-2">
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-orange-400" />
          <span className="text-xs font-semibold text-zinc-200">Model Exclusion Wildcards</span>
        </div>
        <button
          type="button"
          onClick={handleResetDefaults}
          disabled={disabled}
          className="text-[10px] text-zinc-400 hover:text-zinc-200 underline disabled:opacity-50"
        >
          Reset Defaults
        </button>
      </div>

      <p className="text-[11px] text-zinc-400 mb-3 leading-relaxed">
        Exclude costly, slow, or thinking models from automated warmup using wildcard patterns (e.g. <code className="text-zinc-300 font-mono">*-thinking-*</code>).
      </p>

      {/* Chip list */}
      <div className="flex flex-wrap gap-1.5 p-2 bg-zinc-950/60 border border-zinc-800 rounded-lg mb-3 min-h-[38px] items-center">
        {currentPatterns.length === 0 ? (
          <span className="text-[11px] text-zinc-500 italic px-1">No patterns configured (All models allowed)</span>
        ) : (
          currentPatterns.map((pat) => (
            <span
              key={pat}
              className="bg-zinc-800 text-zinc-300 border border-zinc-700 text-xs px-2 py-0.5 rounded-full flex items-center gap-1 font-mono transition-colors hover:border-zinc-600"
            >
              <span>{pat}</span>
              <button
                type="button"
                onClick={() => handleRemove(pat)}
                disabled={disabled}
                className="hover:text-red-400 focus:outline-none disabled:opacity-50 ml-0.5"
                title={`Remove ${pat}`}
                aria-label={`Remove pattern ${pat}`}
              >
                <X className="w-3 h-3" />
              </button>
            </span>
          ))
        )}
      </div>

      {/* Add pattern input */}
      <div className="flex items-center gap-2">
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          placeholder="e.g. *-thinking-*, *-ultra"
          aria-label="Add exclusion pattern"
          className="flex-1 bg-zinc-800/80 border border-zinc-700 rounded-lg px-3 py-1.5 text-xs font-mono text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-orange-500 transition-colors disabled:opacity-50"
        />
        <button
          type="button"
          onClick={handleAdd}
          disabled={disabled || !inputValue.trim()}
          className="bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 hover:border-zinc-600 text-xs px-3 py-1.5 rounded-lg flex items-center gap-1 font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Add</span>
        </button>
      </div>
    </div>
  );
}
