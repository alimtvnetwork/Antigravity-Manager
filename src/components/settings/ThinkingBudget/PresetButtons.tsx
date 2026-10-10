import { useTranslation } from "react-i18next";
import type { ThinkingBudgetConfig } from "../../types/config";
import type { BudgetFieldKey } from "./budgetTypes";

interface PresetButtonsProps {
    field: BudgetFieldKey;
    presets: number[];
    color?: "blue" | "purple";
    inputValues: Record<string, string>;
    currentConfig: ThinkingBudgetConfig;
    setInputValues: React.Dispatch<React.SetStateAction<Record<string, string>>>;
    onChange: (config: ThinkingBudgetConfig) => void;
    getPresetTooltip: (val: number) => string;
}

export function PresetButtons({
    field,
    presets,
    color = "blue",
    inputValues,
    currentConfig,
    setInputValues,
    onChange,
    getPresetTooltip,
}: PresetButtonsProps) {
    const { t } = useTranslation();
    const currentRaw = inputValues[field];
    const currentVal =
        currentRaw !== "" && currentRaw !== "-"
            ? parseInt(currentRaw, 10)
            : (currentConfig as Record<string, number>)[field];

    const activeBg =
        color === "purple"
            ? "bg-purple-600 hover:bg-purple-500"
            : "bg-blue-600 hover:bg-blue-500";
    const hoverText =
        color === "purple"
            ? "hover:text-purple-600 hover:border-purple-400 dark:hover:text-purple-300"
            : "hover:text-blue-600 hover:border-blue-400 dark:hover:text-blue-300";

    return (
        <div className="flex items-center gap-1 mt-1.5 flex-wrap">
            {presets.map((val) => (
                <button
                    key={val}
                    type="button"
                    title={getPresetTooltip(val)}
                    onClick={() => {
                        setInputValues((prev) => ({ ...prev, [field]: String(val) }));
                        onChange({ ...currentConfig, [field]: val });
                    }}
                    className={`px-1.5 py-0.5 rounded text-[10px] font-mono font-semibold transition-all cursor-pointer ${
                        currentVal === val
                            ? `${activeBg} text-white shadow-xs`
                            : `bg-gray-100 dark:bg-base-300/80 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 ${hoverText}`
                    }`}
                >
                    {val === -1
                        ? t("proxy.config.thinking_budget.preset_adaptive", {
                              defaultValue: "自适应 (-1)",
                          })
                        : val.toLocaleString()}
                </button>
            ))}
        </div>
    );
}
