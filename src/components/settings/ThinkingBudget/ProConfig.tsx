import { BudgetSectionProps } from "./BudgetSectionProps";
import { PresetButtons } from "./PresetButtons";

export function ProConfig(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, controlSource, thinkingStoreEnabled, onThinkingStoreChange, thinkingMaxMemoryTurns, onThinkingMaxMemoryTurnsChange, thinkingRetentionDays, onThinkingRetentionDaysChange, isSaving, isSaved, handleSave, onSave, showClaudeAdvanced, setShowClaudeAdvanced, isClearingThinking, showClearThinkingConfirm, setShowClearThinkingConfirm, handleClearThinkingStore } = props;

    return (
        <>
                        <div className="border border-gray-200 dark:border-base-200 rounded-xl p-4 bg-white dark:bg-base-100 space-y-3.5 shadow-2xs">
                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-gray-100 dark:border-base-200 pb-2.5">
                                <div>
                                    <span className="text-xs font-bold text-gray-900 dark:text-white">
                                        {t("proxy.config.thinking_budget.pro_family_title", {
                                            defaultValue: "Gemini Pro ≥ 3.0 系列 (如 3-pro, 3.1-pro 等)",
                                        })}
                                    </span>
                                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                        {t("proxy.config.thinking_budget.pro_family_desc", {
                                            defaultValue:
                                                "Google 官方 Gemini ≥ 3.0 Pro 体系仅提供 Low 与 High 两个档位。未指定 effort 或 medium 将自动走 High 保证 Pro 深度思考。",
                                        })}
                                    </p>
                                </div>
                                <div className="flex items-center gap-3">
                                    <label className="inline-flex items-center gap-1.5 cursor-pointer text-xs font-medium text-gray-700 dark:text-gray-300">
                                        <input
                                            type="radio"
                                            name="pro_mode"
                                            checked={currentConfig.pro_mode === "default"}
                                            onChange={() => handleProModeChange("default")}
                                            className="text-blue-600 focus:ring-blue-500"
                                        />
                                        {t("proxy.config.thinking_budget.mode_default", {
                                            defaultValue: "默认模式 (官方自适应)",
                                        })}
                                    </label>
                                    <label className="inline-flex items-center gap-1.5 cursor-pointer text-xs font-medium text-gray-700 dark:text-gray-300">
                                        <input
                                            type="radio"
                                            name="pro_mode"
                                            checked={currentConfig.pro_mode !== "default"}
                                            onChange={() => handleProModeChange("custom")}
                                            className="text-blue-600 focus:ring-blue-500"
                                        />
                                        {t("proxy.config.thinking_budget.mode_custom", {
                                            defaultValue: "自定义思考预算模式 (推荐)",
                                        })}
                                    </label>
                                </div>
                            </div>

                            {currentConfig.pro_mode !== "default" ? (
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                    <div>
                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                            {t("proxy.config.thinking_budget.tier_low", {
                                                defaultValue: "Low 档位",
                                            })}
                                        </label>
                                        <input
                                            type="text"
                                            inputMode="numeric"
                                            placeholder="1001"
                                            value={inputValues.pro_low ?? ""}
                                            onChange={(e) =>
                                                handleInputChange("pro_low", e.target.value)
                                            }
                                            className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500"
                                        />
                                        <PresetButtons
                                            field="pro_low"
                                            presets={[1001, 1024, 2048, 4096]}
                                            inputValues={inputValues}
                                            currentConfig={currentConfig}
                                            setInputValues={setInputValues}
                                            onChange={onChange}
                                            getPresetTooltip={getPresetTooltip}
                                        />
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                            {t("proxy.config.thinking_budget.pro_low_hint", {
                                                defaultValue: "低思考档位 (默认 1001，填 -1 则走官方自适应)",
                                            })}
                                        </p>
                                    </div>
                                    <div>
                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                            {t("proxy.config.thinking_budget.tier_pro_high", {
                                                defaultValue: "High 档位 (默认/主力档位)",
                                            })}
                                        </label>
                                        <input
                                            type="text"
                                            inputMode="numeric"
                                            placeholder="10001"
                                            value={inputValues.pro_high ?? ""}
                                            onChange={(e) =>
                                                handleInputChange("pro_high", e.target.value)
                                            }
                                            className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500"
                                        />
                                        <PresetButtons
                                            field="pro_high"
                                            presets={[10001, 16384, 32768]}
                                            inputValues={inputValues}
                                            currentConfig={currentConfig}
                                            setInputValues={setInputValues}
                                            onChange={onChange}
                                            getPresetTooltip={getPresetTooltip}
                                        />
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                            {t("proxy.config.thinking_budget.pro_high_hint", {
                                                defaultValue: "深度思考档位 (默认 10001，填 -1 则走官方自适应)",
                                            })}
                                        </p>
                                    </div>
                                </div>
                            ) : (
                                <p className="text-xs text-gray-500 dark:text-gray-400 italic">
                                    {t("proxy.config.thinking_budget.pro_default_hint", {
                                        defaultValue:
                                            "已启用官方默认模式：Gemini Pro 系列将由 Google 官方动态确定思考强度。",
                                    })}
                                </p>
                            )}
                        </div>

        </>
    );
}
