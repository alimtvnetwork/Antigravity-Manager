import { BudgetSectionProps } from "./BudgetSectionProps";
import { PresetButtons } from "./PresetButtons";
import { ChevronDown } from "lucide-react";

export function ClaudeConfig(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, showClaudeAdvanced, setShowClaudeAdvanced, handleClaudeModeChange } = props;

    return (
        <>
                        <div className="border border-gray-200 dark:border-base-200 rounded-xl p-4 bg-white dark:bg-base-100 space-y-3.5 shadow-2xs">
                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-gray-100 dark:border-base-200 pb-2.5">
                                <div>
                                    <span className="text-xs font-bold text-gray-900 dark:text-white">
                                        {t("proxy.config.thinking_budget.claude_family_title", {
                                            defaultValue: "Claude 系列思考模型 (如 claude-3-7-sonnet-thinking 等)",
                                        })}
                                    </span>
                                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                        {t("proxy.config.thinking_budget.claude_family_desc", {
                                            defaultValue:
                                                "调控 Claude 思考模型的 Token 预算（官方模型列表仅区分 -thinking 与普通模型，此项专用于思考模型）",
                                        })}
                                    </p>
                                </div>
                                <div className="flex items-center gap-3">
                                    <label className="inline-flex items-center gap-1.5 cursor-pointer text-xs font-medium text-gray-700 dark:text-gray-300">
                                        <input
                                            type="radio"
                                            name="claude_mode"
                                            checked={currentConfig.claude_mode === "default"}
                                            onChange={() => handleClaudeModeChange("default")}
                                            className="text-blue-600 focus:ring-blue-500"
                                        />
                                        {t("proxy.config.thinking_budget.mode_default", {
                                            defaultValue: "默认模式 (官方规范)",
                                        })}
                                    </label>
                                    <label className="inline-flex items-center gap-1.5 cursor-pointer text-xs font-medium text-gray-700 dark:text-gray-300">
                                        <input
                                            type="radio"
                                            name="claude_mode"
                                            checked={currentConfig.claude_mode !== "default"}
                                            onChange={() => handleClaudeModeChange("custom")}
                                            className="text-blue-600 focus:ring-blue-500"
                                        />
                                        {t("proxy.config.thinking_budget.mode_custom", {
                                            defaultValue: "自定义思考预算模式 (推荐)",
                                        })}
                                    </label>
                                </div>
                            </div>

                            {currentConfig.claude_mode !== "default" ? (
                                <div className="space-y-3.5">
                                    {/* 核心统一预算输入 */}
                                    <div className="p-3.5 bg-purple-50/70 dark:bg-purple-900/20 border border-purple-200/80 dark:border-purple-800/40 rounded-xl space-y-2.5 shadow-2xs">
                                        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                                            <div>
                                                <label className="block text-xs font-bold text-gray-900 dark:text-white">
                                                    {t("proxy.config.thinking_budget.claude_budget_title", {
                                                        defaultValue: "Claude 思考模型预算 (Tokens)",
                                                    })}
                                                </label>
                                                <p className="text-xs text-gray-600 dark:text-gray-300 mt-0.5">
                                                    {t("proxy.config.thinking_budget.claude_budget_desc", {
                                                        defaultValue: "客户端调用思考模型（如 claude-3-7-sonnet-thinking）时的核心 Token 预算（填 -1 则由官方自适应）",
                                                    })}
                                                </p>
                                            </div>
                                            <div className="flex items-center gap-1.5 flex-wrap">
                                                {[1024, 2048, 4096, 8192, 16384, 32768, 65536, -1].map((val) => {
                                                    const currentBudgetNum = inputValues.claude_budget !== "" && inputValues.claude_budget !== "-"
                                                        ? parseInt(inputValues.claude_budget, 10)
                                                        : currentConfig.claude_budget;
                                                    return (
                                                        <button
                                                            key={val}
                                                            type="button"
                                                            title={getPresetTooltip(val)}
                                                            onClick={() => {
                                                                setInputValues((prev) => ({ ...prev, claude_budget: String(val) }));
                                                                onChange({ ...currentConfig, claude_budget: val });
                                                            }}
                                                            className={`px-2.5 py-1 rounded-lg text-xs font-mono font-semibold transition-all cursor-pointer ${
                                                                currentBudgetNum === val
                                                                    ? "bg-purple-600 hover:bg-purple-500 text-white shadow-xs"
                                                                    : "bg-white dark:bg-base-200 border border-gray-300 dark:border-base-300 text-gray-700 dark:text-gray-200 hover:border-purple-400 hover:text-purple-600 dark:hover:text-purple-300"
                                                            }`}
                                                        >
                                                            {val === -1 ? t("proxy.config.thinking_budget.preset_adaptive", { defaultValue: "自适应 (-1)" }) : `${val.toLocaleString()}`}
                                                        </button>
                                                    );
                                                })}
                                            </div>
                                        </div>

                                        <div className="flex items-center gap-3">
                                            <input
                                                type="text"
                                                inputMode="numeric"
                                                placeholder="16384"
                                                value={inputValues.claude_budget ?? ""}
                                                onChange={(e) =>
                                                    handleInputChange("claude_budget", e.target.value)
                                                }
                                                className="w-48 px-3 py-1.5 border border-purple-300 dark:border-purple-800/80 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-bold text-purple-700 dark:text-purple-300 focus:ring-2 focus:ring-purple-500/30 focus:border-purple-500 shadow-2xs"
                                            />
                                            <span className="text-xs font-medium text-gray-600 dark:text-gray-300">
                                                {t("proxy.config.thinking_budget.claude_budget_current", {
                                                    defaultValue: "推荐默认值: 16,384 Tokens (兼顾思考深度与响应速率)",
                                                })}
                                            </span>
                                        </div>
                                    </div>

                                    {/* 高级：客户端 Effort 档位映射 */}
                                    <div>
                                        <button
                                            type="button"
                                            onClick={() => setShowClaudeAdvanced((prev) => !prev)}
                                            className="inline-flex items-center gap-1.5 text-xs font-semibold text-purple-600 dark:text-purple-400 hover:underline cursor-pointer select-none"
                                        >
                                            <ChevronDown size={14} className={`transition-transform duration-200 ${showClaudeAdvanced ? "" : "-rotate-90"}`} />
                                            <span>
                                                {showClaudeAdvanced
                                                    ? t("proxy.config.thinking_budget.claude_hide_advanced", { defaultValue: "收起客户端 Effort 档位映射" })
                                                    : t("proxy.config.thinking_budget.claude_show_advanced", { defaultValue: "展开高级设置：客户端 Effort 档位映射 (low / medium / high)" })}
                                            </span>
                                        </button>

                                        {showClaudeAdvanced && (
                                            <div className="mt-2.5 p-3.5 bg-gray-50 dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-xl space-y-2.5 shadow-2xs">
                                                <p className="text-xs text-gray-600 dark:text-gray-300 leading-relaxed">
                                                    {t("proxy.config.thinking_budget.claude_effort_mapping_desc", {
                                                        defaultValue: "当部分商业客户端（如 Cline, Roo 等）在请求头或参数中附带 reasoning_effort 时，按以下预算映射：",
                                                    })}
                                                </p>
                                                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                                                    <div>
                                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-200 mb-1">
                                                            {t("proxy.config.thinking_budget.tier_low", { defaultValue: "Low 档位" })}
                                                        </label>
                                                        <input
                                                            type="text"
                                                            inputMode="numeric"
                                                            placeholder="1024"
                                                            value={inputValues.claude_low ?? ""}
                                                            onChange={(e) => handleInputChange("claude_low", e.target.value)}
                                                            className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500/30 focus:border-purple-500"
                                                        />
                                                        <PresetButtons field="claude_low" presets={[1024, 2048]} color="purple" inputValues={inputValues} currentConfig={currentConfig} setInputValues={setInputValues} onChange={onChange} getPresetTooltip={getPresetTooltip} />
                                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                            {t("proxy.config.thinking_budget.claude_low_hint", { defaultValue: "轻量思考 (默认 1024)" })}
                                                        </p>
                                                    </div>
                                                    <div>
                                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-200 mb-1">
                                                            {t("proxy.config.thinking_budget.tier_medium", { defaultValue: "Medium 档位" })}
                                                        </label>
                                                        <input
                                                            type="text"
                                                            inputMode="numeric"
                                                            placeholder="4096"
                                                            value={inputValues.claude_medium ?? ""}
                                                            onChange={(e) => handleInputChange("claude_medium", e.target.value)}
                                                            className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500/30 focus:border-purple-500"
                                                        />
                                                        <PresetButtons field="claude_medium" presets={[4096, 8192]} color="purple" inputValues={inputValues} currentConfig={currentConfig} setInputValues={setInputValues} onChange={onChange} getPresetTooltip={getPresetTooltip} />
                                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                            {t("proxy.config.thinking_budget.claude_medium_hint", { defaultValue: "日常平衡档 (默认 4096)" })}
                                                        </p>
                                                    </div>
                                                    <div>
                                                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-200 mb-1">
                                                            {t("proxy.config.thinking_budget.tier_high", { defaultValue: "High 档位" })}
                                                        </label>
                                                        <input
                                                            type="text"
                                                            inputMode="numeric"
                                                            placeholder="16384"
                                                            value={inputValues.claude_high ?? ""}
                                                            onChange={(e) => handleInputChange("claude_high", e.target.value)}
                                                            className="w-full px-3 py-1.5 border border-gray-300 dark:border-base-300 rounded-lg bg-white dark:bg-base-200 text-xs font-mono font-semibold text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500/30 focus:border-purple-500"
                                                        />
                                                        <PresetButtons field="claude_high" presets={[16384, 32768]} color="purple" inputValues={inputValues} currentConfig={currentConfig} setInputValues={setInputValues} onChange={onChange} getPresetTooltip={getPresetTooltip} />
                                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                                            {t("proxy.config.thinking_budget.claude_high_hint", { defaultValue: "深度思考档 (默认 16384)" })}
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>
                                        )}
                                    </div>

                                    {/* 全宽背景说明条 */}
                                    <div className="w-full p-3 bg-purple-50/70 dark:bg-purple-900/20 border border-purple-200/80 dark:border-purple-800/40 rounded-xl text-xs text-purple-800 dark:text-purple-200 leading-relaxed">
                                        {t("proxy.config.thinking_budget.claude_thinking_note", {
                                            defaultValue:
                                                "💡 Claude 思考模型说明：Claude 系列在模型列表中仅提供思考版模型（如 claude-3-7-sonnet-thinking）与普通版，未拆分 low/medium/high 后缀。此处设置的值将作为调用 Claude 思考模型时的统一深度思考预算。",
                                        })}
                                    </div>
                                </div>
                            ) : (
                                <p className="text-xs text-gray-500 dark:text-gray-400 italic">
                                    {t("proxy.config.thinking_budget.claude_default_hint", {
                                        defaultValue:
                                            "已启用官方默认模式：完全按官方协议规范透传，不注入额外自定义预算限制。",
                                    })}
                                </p>
                            )}
                        </div>
        </>
    );
}
