import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { request } from "../../utils/request";
import { showToast } from "../common/ToastContainer";
import type { ThinkingBudgetConfig, ThinkingControlSource, ThinkingBudgetMode } from "../../types/config";
import {
    ThinkingBudgetProps,
    DEFAULT_CONFIG,
    BudgetFieldKey,
    BUDGET_DEFAULTS,
} from "./ThinkingBudget/budgetTypes";
import { BudgetHeader } from "./ThinkingBudget/BudgetHeader";
import { ServerSideSection } from "./ThinkingBudget/ServerSideSection";
import { MemorySection } from "./ThinkingBudget/MemorySection";
import { ControlSourceSection } from "./ThinkingBudget/ControlSourceSection";
import { FlashConfig } from "./ThinkingBudget/FlashConfig";
import { ProConfig } from "./ThinkingBudget/ProConfig";
import { ClaudeConfig } from "./ThinkingBudget/ClaudeConfig";
import { SaveBar } from "./ThinkingBudget/SaveBar";
import { ClearConfirmModal } from "./ThinkingBudget/ClearConfirmModal";

export default function ThinkingBudget({
    config = DEFAULT_CONFIG,
    onChange,
    onSave,
    thinkingStoreEnabled = true,
    onThinkingStoreChange,
    thinkingMaxMemoryTurns = 600,
    onThinkingMaxMemoryTurnsChange,
    thinkingRetentionDays = 15,
    onThinkingRetentionDaysChange,
}: ThinkingBudgetProps) {
    const { t } = useTranslation();
    const [isSaving, setIsSaving] = useState(false);
    const [isSaved, setIsSaved] = useState(false);
    const [showClaudeAdvanced, setShowClaudeAdvanced] = useState(false);
    const [isClearingThinking, setIsClearingThinking] = useState(false);
    const [showClearThinkingConfirm, setShowClearThinkingConfirm] = useState(false);

    const handleClearThinkingStore = async () => {
        setIsClearingThinking(true);
        try {
            const res = await request<number | { deleted?: number }>("clear_thinking_store");
            const count = typeof res === "number" ? res : res?.deleted ?? 0;
            showToast(
                t("proxy.config.thinking_budget.clear_success", {
                    count,
                    defaultValue: `Successfully cleared thinking store (purged  records)`,
                }),
                "success"
            );
            setShowClearThinkingConfirm(false);
        } catch (err: any) {
            showToast(
                t("proxy.config.thinking_budget.clear_error", {
                    error: String(err),
                    defaultValue: `Failed to clear thinking store: ${String(err)}`,
                }),
                "error"
            );
        } finally {
            setIsClearingThinking(false);
        }
    };

    const currentConfig: ThinkingBudgetConfig = {
        ...DEFAULT_CONFIG,
        ...config,
    };

    // The local edit text state of the budget input box allows the user to clear the backspace to "" or enter a negative sign "-"
    const [inputValues, setInputValues] = useState<Record<string, string>>(() => {
        const init: Record<string, string> = {};
        for (const [key, defaultVal] of Object.entries(BUDGET_DEFAULTS)) {
            const val = (config as any)?.[key];
            init[key] = val !== undefined && val !== null ? String(val) : String(defaultVal);
        }
        return init;
    });

    // Synchronize when the external configuration is substantially updated (retain the null value status being edited)
    const lastConfigRef = useRef(config);
    useEffect(() => {
        if (config && config !== lastConfigRef.current) {
            lastConfigRef.current = config;
            setInputValues((prev) => {
                const next = { ...prev };
                for (const key of Object.keys(BUDGET_DEFAULTS)) {
                    const val = (config as any)?.[key];
                    if (val !== undefined && val !== null) {
                        const parsed = parseInt(prev[key], 10);
                        if (parsed !== val && prev[key] !== "" && prev[key] !== "-") {
                            next[key] = String(val);
                        }
                    }
                }
                return next;
            });
        }
    }, [config]);

    const handleControlSourceChange = (source: ThinkingControlSource) => {
        onChange({
            ...currentConfig,
            control_source: source,
        });
    };

    const handleFlashModeChange = (mode: ThinkingBudgetMode) => {
        onChange({
            ...currentConfig,
            flash_mode: mode,
        });
    };

    const handleProModeChange = (mode: ThinkingBudgetMode) => {
        onChange({
            ...currentConfig,
            pro_mode: mode,
        });
    };

    const handleClaudeModeChange = (mode: ThinkingBudgetMode) => {
        onChange({
            ...currentConfig,
            claude_mode: mode,
        });
    };

    // Input box change processing: allow clearing to "", allow "-", do not automatically add -1
    const handleInputChange = (field: BudgetFieldKey, rawVal: string) => {
        if (rawVal !== "" && rawVal !== "-") {
            if (!/^-?\d+$/.test(rawVal)) {
                return;
            }
        }
        setInputValues((prev) => ({
            ...prev,
            [field]: rawVal,
        }));

        // If it is a legal complete number, it will be synchronized to the configuration object in real time; if it is empty, it will not be written and will be left to backfill the default value in the final save stage.
        const trimmed = rawVal.trim();
        if (trimmed !== "" && trimmed !== "-") {
            const parsed = parseInt(trimmed, 10);
            if (!isNaN(parsed)) {
                onChange({
                    ...currentConfig,
                    [field]: parsed,
                });
            }
        }
    };

    // Final save configuration phase: automatically backfill the budget input box in the empty/illegal state with the default value
    const handleSave = async () => {
        const nextInputs = { ...inputValues };
        const nextConfig: ThinkingBudgetConfig = { ...currentConfig };

        for (const [key, defaultVal] of Object.entries(BUDGET_DEFAULTS)) {
            const valStr = (nextInputs[key] ?? "").trim();
            let finalVal: number;
            if (valStr === "" || valStr === "-" || isNaN(parseInt(valStr, 10))) {
                finalVal = defaultVal;
                nextInputs[key] = String(defaultVal);
            } else {
                finalVal = parseInt(valStr, 10);
                nextInputs[key] = String(finalVal);
            }
            (nextConfig as any)[key] = finalVal;
        }

        // Backfill to input box interface display
        setInputValues(nextInputs);
        // Synchronize configuration to parent component
        onChange(nextConfig);

        if (onSave) {
            setIsSaving(true);
            try {
                await onSave();
                setIsSaved(true);
                setTimeout(() => setIsSaved(false), 2000);
            } finally {
                setIsSaving(false);
            }
        }
    };

    const getPresetTooltip = (val: number): string => {
        if (val === 32768) {
            return t("proxy.config.thinking_budget.tooltip_32768", {
                defaultValue: "32,768 Tokens: Suitable for complex programming and deep architecture tasks",
            });
        }
        if (val === 16384 || val === 16000) {
            return t("proxy.config.thinking_budget.tooltip_16384", {
                defaultValue: "1.6w level (16,384 Tokens): suitable for complex Agent tasks",
            });
        }
        if (val === 8192) {
            return t("proxy.config.thinking_budget.tooltip_8192", {
                defaultValue: "8,192 Tokens: suitable for medium-complexity multi-step reasoning and code debugging",
            });
        }
        if (val === 4096) {
            return t("proxy.config.thinking_budget.tooltip_4096", {
                defaultValue: "4,096 Tokens: Standard balanced file, taking into account quality of thinking and speed of response",
            });
        }
        if (val === 2048) {
            return t("proxy.config.thinking_budget.tooltip_2048", {
                defaultValue: "2,048 Tokens: Daily lightweight reasoning file",
            });
        }
        if (val === 1024) {
            return t("proxy.config.thinking_budget.tooltip_1024", {
                defaultValue: "1,024 Tokens: lightweight, fast thinking, extremely low consumption",
            });
        }
        if (val === 65536) {
            return t("proxy.config.thinking_budget.tooltip_65536", {
                defaultValue: "65,536 Tokens: Extremely long deep thinking chain limit",
            });
        }
        if (val === 1001) {
            return t("proxy.config.thinking_budget.tooltip_1001", {
                defaultValue: "1,001 Tokens: Google official 3.1 Pro low-thinking benchmark file",
            });
        }
        if (val === 10001) {
            return t("proxy.config.thinking_budget.tooltip_10001", {
                defaultValue: "10,001 Tokens: Google’s official 3.1 Pro deep inference main file",
            });
        }
        if (val === -1) {
            return t("proxy.config.thinking_budget.tooltip_adaptive", {
                defaultValue: "Adaptive (-1): Completely automatically assigned by the upstream model based on problem difficulty",
            });
        }
        return `${val.toLocaleString()} Tokens`;
    };


    const controlSource = currentConfig.control_source || "gateway";

    const sectionProps = {
        t,
        currentConfig,
        onChange,
        inputValues,
        setInputValues,
        handleInputChange,
        getPresetTooltip,
        handleControlSourceChange,
        handleFlashModeChange,
        handleProModeChange,
        handleClaudeModeChange,
        controlSource,
        thinkingStoreEnabled,
        onThinkingStoreChange,
        thinkingMaxMemoryTurns,
        onThinkingMaxMemoryTurnsChange,
        thinkingRetentionDays,
        onThinkingRetentionDaysChange,
        isSaving,
        isSaved,
        handleSave,
        onSave,
        showClaudeAdvanced,
        setShowClaudeAdvanced,
        isClearingThinking,
        showClearThinkingConfirm,
        setShowClearThinkingConfirm,
        handleClearThinkingStore,
    };

    return (
        <div className="space-y-4">
            <BudgetHeader {...sectionProps} />
            <ServerSideSection {...sectionProps} />
            <MemorySection {...sectionProps} />
            <ControlSourceSection {...sectionProps} />
            {controlSource === "gateway" && (
                <>
                    <FlashConfig {...sectionProps} />
                    <ProConfig {...sectionProps} />
                    <ClaudeConfig {...sectionProps} />
                </>
            )}
            <SaveBar {...sectionProps} />
            <ClearConfirmModal {...sectionProps} />
        </div>
    );
}
