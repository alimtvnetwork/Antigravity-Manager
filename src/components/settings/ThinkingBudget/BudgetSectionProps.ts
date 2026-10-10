import type * as React from "react";
import type { ThinkingBudgetConfig } from "../../types/config";
import type { BudgetFieldKey } from "./budgetTypes";

export interface BudgetSectionProps {
    t: (key: string, opts?: Record<string, unknown>) => string;
    currentConfig: ThinkingBudgetConfig;
    onChange: (config: ThinkingBudgetConfig) => void;
    inputValues: Record<string, string>;
    setInputValues: React.Dispatch<React.SetStateAction<Record<string, string>>>;
    handleInputChange: (field: BudgetFieldKey, rawVal: string) => void;
    getPresetTooltip: (val: number) => string;
    controlSource: string;
    // Thinking store
    thinkingStoreEnabled?: boolean;
    onThinkingStoreChange?: (enabled: boolean) => void;
    thinkingMaxMemoryTurns?: number;
    onThinkingMaxMemoryTurnsChange?: (turns: number) => void;
    thinkingRetentionDays?: number;
    onThinkingRetentionDaysChange?: (days: number) => void;
    // UI state
    isSaving: boolean;
    isSaved: boolean;
    handleSave: () => Promise<void>;
    onSave?: () => Promise<void> | void;
    showClaudeAdvanced: boolean;
    setShowClaudeAdvanced: React.Dispatch<React.SetStateAction<boolean>>;
    isClearingThinking: boolean;
    showClearThinkingConfirm: boolean;
    setShowClearThinkingConfirm: React.Dispatch<React.SetStateAction<boolean>>;
    handleClearThinkingStore: () => Promise<void>;
}
