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
                    defaultValue: `已成功清空思考块存储 (共清理 ${count} 条历史记录)`,
                }),
                "success"
            );
            setShowClearThinkingConfirm(false);
        } catch (err: any) {
            showToast(
                t("proxy.config.thinking_budget.clear_error", {
                    error: String(err),
                    defaultValue: `清空思考块失败: ${String(err)}`,
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

    // 预算输入框本地编辑文本状态，允许用户清空退格为 "" 或输入负号 "-"
    const [inputValues, setInputValues] = useState<Record<string, string>>(() => {
        const init: Record<string, string> = {};
        for (const [key, defaultVal] of Object.entries(BUDGET_DEFAULTS)) {
            const val = (config as any)?.[key];
            init[key] = val !== undefined && val !== null ? String(val) : String(defaultVal);
        }
        return init;
    });

    // 外部配置实质性更新时同步（保留正在编辑的空值状态）
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

    // 输入框变更处理：允许清空为 ""，允许 "-"，不自动补 -1
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

        // 如果是合法完整数字，实时同步给配置对象；空值时不写入，留待最后保存阶段回填默认值
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

    // 最后保存配置阶段：对处于清空/非法状态的预算输入框自动回填为默认值
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

        // 回填到输入框界面显示
        setInputValues(nextInputs);
        // 同步给父组件配置
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
                defaultValue: "32,768 Tokens：适合复杂编程与深度架构任务",
            });
        }
        if (val === 16384 || val === 16000) {
            return t("proxy.config.thinking_budget.tooltip_16384", {
                defaultValue: "1.6w 档位 (16,384 Tokens)：适合复杂 Agent 任务",
            });
        }
        if (val === 8192) {
            return t("proxy.config.thinking_budget.tooltip_8192", {
                defaultValue: "8,192 Tokens：适合中等复杂度多步推理与代码排错",
            });
        }
        if (val === 4096) {
            return t("proxy.config.thinking_budget.tooltip_4096", {
                defaultValue: "4,096 Tokens：标准平衡档，兼顾思考质量与响应速度",
            });
        }
        if (val === 2048) {
            return t("proxy.config.thinking_budget.tooltip_2048", {
                defaultValue: "2,048 Tokens：日常轻量推理档",
            });
        }
        if (val === 1024) {
            return t("proxy.config.thinking_budget.tooltip_1024", {
                defaultValue: "1,024 Tokens：轻量极速思考，消耗极低",
            });
        }
        if (val === 65536) {
            return t("proxy.config.thinking_budget.tooltip_65536", {
                defaultValue: "65,536 Tokens：超长深度思考链极限档",
            });
        }
        if (val === 1001) {
            return t("proxy.config.thinking_budget.tooltip_1001", {
                defaultValue: "1,001 Tokens：Google 官方 3.1 Pro 低思考基准档",
            });
        }
        if (val === 10001) {
            return t("proxy.config.thinking_budget.tooltip_10001", {
                defaultValue: "10,001 Tokens：Google 官方 3.1 Pro 深度推理主力档",
            });
        }
        if (val === -1) {
            return t("proxy.config.thinking_budget.tooltip_adaptive", {
                defaultValue: "自适应 (-1)：完全由上游模型根据问题难度自动分配",
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
