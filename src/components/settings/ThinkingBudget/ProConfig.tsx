import { BudgetSectionProps } from "./BudgetSectionProps";
import { PresetButtons } from "./PresetButtons";

export function ProConfig(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, handleProModeChange } = props;

    return (
        <>
                        <div className="border border-gray-200 dark:border-base-200 rounded-xl p-4 bg-white dark:bg-base-100 space-y-3.5 shadow-2xs">
                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-gray-100 dark:border-base-200 pb-2.5">
                                <div>
                                    <span className="text-xs font-bold text-gray-900 dark:text-white">
                                        {t("proxy.config.thinking_budget.pro_family_title", {
                                            defaultValue: "Gemini Pro ≥ 3.0 series (such as 3-pro, 3.1-pro, etc.)",
                                        })}
                                    </span>
                                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                        {t("proxy.config.thinking_budget.pro_family_desc", {
                                            defaultValue:
                                                "Google's official Gemini ≥ 3.0 Pro system only provides two gears: Low and High. Not specifying effort or medium will automatically go to High to ensure Pro deep thinking.",
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
                                            defaultValue: "Default mode (official adaptive)",
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
                                            defaultValue: "Customized thinking budget model (recommended)",
                                        })}
                                    </label>
                                </div>
                            </div>

                            {currentConfig.pro_mode !== "default" ? (
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                    <div>
                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                            {t("proxy.config.thinking_budget.tier_low", {
                                                defaultValue: "Low gear",
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
                                                defaultValue: "Low thinking gear (default 1001, fill in -1 to use official adaptive)",
                                            })}
                                        </p>
                                    </div>
                                    <div>
                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">
                                            {t("proxy.config.thinking_budget.tier_pro_high", {
                                                defaultValue: "High gear (default/main gear)",
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
                                                defaultValue: "Deep thinking gear (default 10001, fill in -1 to use official adaptive)",
                                            })}
                                        </p>
                                    </div>
                                </div>
                            ) : (
                                <p className="text-xs text-gray-500 dark:text-gray-400 italic">
                                    {t("proxy.config.thinking_budget.pro_default_hint", {
                                        defaultValue:
                                            "Official default mode enabled: The Gemini Pro series will have its thinking intensity determined by Google official dynamics.",
                                    })}
                                </p>
                            )}
                        </div>

        </>
    );
}
