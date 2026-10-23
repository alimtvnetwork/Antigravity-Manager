import { BudgetSectionProps } from "./BudgetSectionProps";
import { PresetButtons } from "./PresetButtons";

export function FlashConfig(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, handleFlashModeChange } = props;

    return (
        <>
                        <div className="border border-gray-200 dark:border-base-200 rounded-xl p-4 bg-white dark:bg-base-100 space-y-3.5 shadow-2xs">
                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-gray-100 dark:border-base-200 pb-2.5">
                                <div>
                                    <span className="text-xs font-bold text-gray-900 dark:text-white">
                                        {t("proxy.config.thinking_budget.flash_family_title", {
                                            defaultValue: "Gemini Flash ≥ 3.0 series (such as 3-flash, 3.7-flash, 3.8-flash, tiered, etc.)",
                                        })}
                                    </span>
                                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                        {t("proxy.config.thinking_budget.flash_family_desc", {
                                            defaultValue: "Control the four budget levels of the Flash thinking model of Gemini ≥ 3.0 (< 3.0 such as 2.5-flash is a traditional non-thinking model, and the gateway automatically strips thinking)",
                                        })}
                                    </p>
                                </div>
                                <div className="flex items-center gap-3">
                                    <label className="inline-flex items-center gap-1.5 cursor-pointer text-xs font-medium text-gray-700 dark:text-gray-300">
                                        <input
                                            type="radio"
                                            name="flash_mode"
                                            checked={currentConfig.flash_mode === "default"}
                                            onChange={() => handleFlashModeChange("default")}
                                            className="text-blue-600 focus:ring-blue-500"
                                        />
                                        {t("proxy.config.thinking_budget.mode_default", {
                                            defaultValue: "Default mode (official adaptive)",
                                        })}
                                    </label>
                                    <label className="inline-flex items-center gap-1.5 cursor-pointer text-xs font-medium text-gray-700 dark:text-gray-300">
                                        <input
                                            type="radio"
                                            name="flash_mode"
                                            checked={currentConfig.flash_mode !== "default"}
                                            onChange={() => handleFlashModeChange("custom")}
                                            className="text-blue-600 focus:ring-blue-500"
                                        />
                                        {t("proxy.config.thinking_budget.mode_custom", {
                                            defaultValue: "Customized thinking budget model (recommended)",
                                        })}
                                    </label>
                                </div>
                            </div>

                            {currentConfig.flash_mode !== "default" ? (
                                <div className="space-y-3">
                                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                                        <div>
                                            <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                                {t("proxy.config.thinking_budget.tier_low", {
                                                    defaultValue: "Low gear",
                                                })}
                                            </label>
                                            <input
                                                type="text"
                                                inputMode="numeric"
                                                placeholder="1024"
                                                value={inputValues.flash_low ?? ""}
                                                onChange={(e) =>
                                                    handleInputChange("flash_low", e.target.value)
                                                }
                                                className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500"
                                            />
                                            <PresetButtons
                                            field="flash_low"
                                            presets={[1024, 2048]}
                                            inputValues={inputValues}
                                            currentConfig={currentConfig}
                                            setInputValues={setInputValues}
                                            onChange={onChange}
                                            getPresetTooltip={getPresetTooltip}
                                        />
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                {t("proxy.config.thinking_budget.flash_low_hint", {
                                                    defaultValue: "Very low inference (default 1024)",
                                                })}
                                            </p>
                                        </div>
                                        <div>
                                            <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                                {t("proxy.config.thinking_budget.tier_medium", {
                                                    defaultValue: "Medium gear",
                                                })}
                                            </label>
                                            <input
                                                type="text"
                                                inputMode="numeric"
                                                placeholder="4096"
                                                value={inputValues.flash_medium ?? ""}
                                                onChange={(e) =>
                                                    handleInputChange("flash_medium", e.target.value)
                                                }
                                                className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500"
                                            />
                                            <PresetButtons
                                            field="flash_medium"
                                            presets={[4096, 8192]}
                                            inputValues={inputValues}
                                            currentConfig={currentConfig}
                                            setInputValues={setInputValues}
                                            onChange={onChange}
                                            getPresetTooltip={getPresetTooltip}
                                        />
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                {t("proxy.config.thinking_budget.flash_medium_hint", {
                                                    defaultValue: "Standard balance file (default 4096)",
                                                })}
                                            </p>
                                        </div>
                                        <div>
                                            <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                                {t("proxy.config.thinking_budget.tier_high", {
                                                    defaultValue: "High gear",
                                                })}
                                            </label>
                                            <input
                                                type="text"
                                                inputMode="numeric"
                                                placeholder="16384"
                                                value={inputValues.flash_high ?? ""}
                                                onChange={(e) =>
                                                    handleInputChange("flash_high", e.target.value)
                                                }
                                                className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500"
                                            />
                                            <PresetButtons
                                            field="flash_high"
                                            presets={[16384, 32768]}
                                            inputValues={inputValues}
                                            currentConfig={currentConfig}
                                            setInputValues={setInputValues}
                                            onChange={onChange}
                                            getPresetTooltip={getPresetTooltip}
                                        />
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                {t("proxy.config.thinking_budget.flash_high_hint", {
                                                    defaultValue: "Deep inference file (default 16384)",
                                                })}
                                            </p>
                                        </div>
                                        <div>
                                            <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                                {t("proxy.config.thinking_budget.tier_tiered", {
                                                    defaultValue: "Tiered (free model)",
                                                })}
                                            </label>
                                            <input
                                                type="text"
                                                inputMode="numeric"
                                                placeholder="-1"
                                                value={inputValues.flash_tiered ?? ""}
                                                onChange={(e) =>
                                                    handleInputChange("flash_tiered", e.target.value)
                                                }
                                                className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500"
                                            />
                                            <PresetButtons
                                            field="flash_tiered"
                                            presets={[-1, 2048, 8192, 32768]}
                                            inputValues={inputValues}
                                            currentConfig={currentConfig}
                                            setInputValues={setInputValues}
                                            onChange={onChange}
                                            getPresetTooltip={getPresetTooltip}
                                        />
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                {t("proxy.config.thinking_budget.flash_tiered_hint", {
                                                    defaultValue: "Adaptive/Free Strength (default -1)",
                                                })}
                                            </p>
                                        </div>
                                    </div>
                                    <div className="p-3 bg-blue-50/70 dark:bg-blue-900/20 border border-blue-200/80 dark:border-blue-800/40 rounded-xl text-xs text-blue-800 dark:text-blue-200 leading-relaxed">
                                        {t("proxy.config.thinking_budget.tiered_desc", {
                                            defaultValue:
                                                "💡 Tiered Free Model Description: This model is specially built for Google Dynamic Adaptation. When set to -1, the model automatically allocates the amount of thinking according to the difficulty of the problem; if you specify a specific value (such as 30000+), you can break through the High gear to unlock Max extreme thinking.",
                                        })}
                                    </div>
                                </div>
                            ) : (
                                <p className="text-xs text-gray-500 dark:text-gray-400 italic">
                                    {t("proxy.config.thinking_budget.flash_default_hint", {
                                        defaultValue:
                                            "The official default mode is enabled: the official model ID is completely transparent, no thinkingBudget is injected, and the thinking behavior is completely determined by Google's upstream model.",
                                    })}
                                </p>
                            )}
                        </div>

        </>
    );
}
