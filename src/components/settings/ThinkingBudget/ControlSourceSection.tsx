import { BudgetSectionProps } from "./BudgetSectionProps";

export function ControlSourceSection(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, controlSource, thinkingStoreEnabled, onThinkingStoreChange, thinkingMaxMemoryTurns, onThinkingMaxMemoryTurnsChange, thinkingRetentionDays, onThinkingRetentionDaysChange, isSaving, isSaved, handleSave, onSave, showClaudeAdvanced, setShowClaudeAdvanced, isClearingThinking, showClearThinkingConfirm, setShowClearThinkingConfirm, handleClearThinkingStore } = props;

    return (
        <>
                <div className="space-y-2">
                    <label className="text-xs font-semibold text-gray-800 dark:text-gray-200">
                        {t("proxy.config.thinking_budget.control_source_label", {
                            defaultValue: "控制权归属 (Top-Level Authority)",
                        })}
                    </label>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        {/* 网关权威控制 */}
                        <button
                            type="button"
                            onClick={() => handleControlSourceChange("gateway")}
                            className={`text-left p-3.5 rounded-xl border transition-all cursor-pointer relative ${
                                controlSource === "gateway"
                                    ? "border-blue-500 bg-blue-50/70 dark:bg-blue-900/20 text-gray-900 dark:text-white shadow-xs ring-1 ring-blue-500/40"
                                    : "border-gray-200 dark:border-base-300 bg-white dark:bg-base-200 text-gray-700 dark:text-gray-300 hover:border-gray-300 dark:hover:border-base-300"
                            }`}
                        >
                            <div className="flex items-center justify-between mb-1.5">
                                <span className="text-xs font-bold flex items-center gap-1.5 text-gray-900 dark:text-white">
                                    <span
                                        className={`w-2.5 h-2.5 rounded-full ${
                                            controlSource === "gateway"
                                                ? "bg-blue-500"
                                                : "bg-gray-300 dark:bg-base-300"
                                        }`}
                                    />
                                    {t("proxy.config.thinking_budget.source_gateway", {
                                        defaultValue: "网关权威控制",
                                    })}
                                </span>
                                <span className="text-[10px] px-2 py-0.5 rounded-full font-bold bg-emerald-100 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800/60">
                                    {t("proxy.config.thinking_budget.recommended_tag", {
                                        defaultValue: "首选 / 推荐",
                                    })}
                                </span>
                            </div>
                            <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                                {t("proxy.config.thinking_budget.source_gateway_desc", {
                                    defaultValue:
                                        "网关层统一掌控思维链预算与启发式档位，消除不同客户端参数差异，杜绝由于超额预算或格式不当引发的官方 400 报错。",
                                })}
                            </p>
                        </button>

                        {/* 客户端直接控制 */}
                        <button
                            type="button"
                            onClick={() => handleControlSourceChange("client")}
                            className={`text-left p-3.5 rounded-xl border transition-all cursor-pointer relative ${
                                controlSource === "client"
                                    ? "border-amber-500 bg-amber-50/70 dark:bg-amber-900/20 text-gray-900 dark:text-white shadow-xs ring-1 ring-amber-500/40"
                                    : "border-gray-200 dark:border-base-300 bg-white dark:bg-base-200 text-gray-700 dark:text-gray-300 hover:border-gray-300 dark:hover:border-base-300"
                            }`}
                        >
                            <div className="flex items-center justify-between mb-1.5">
                                <span className="text-xs font-bold flex items-center gap-1.5 text-gray-900 dark:text-white">
                                    <span
                                        className={`w-2.5 h-2.5 rounded-full ${
                                            controlSource === "client"
                                                ? "bg-amber-500"
                                                : "bg-gray-300 dark:bg-base-300"
                                        }`}
                                    />
                                    {t("proxy.config.thinking_budget.source_client", {
                                        defaultValue: "客户端直接控制",
                                    })}
                                </span>
                                <span className="text-[10px] px-2 py-0.5 rounded-full font-bold bg-rose-100 dark:bg-rose-950/60 text-rose-700 dark:text-rose-300 border border-rose-200 dark:border-rose-800/60">
                                    {t("proxy.config.thinking_budget.danger_tag", {
                                        defaultValue: "危险 / 不推荐",
                                    })}
                                </span>
                            </div>
                            <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                                {t("proxy.config.thinking_budget.source_client_desc", {
                                    defaultValue:
                                        "归一化后直接提取客户端上送的预算并透传。若客户端未传则由上游自适应。不恰当的预算可能直接导致 Google 报错。",
                                })}
                            </p>
                        </button>
                    </div>
                </div>

                {/* 客户端控制警示框 */}
                {controlSource === "client" && (
                    <div className="p-3.5 bg-rose-50/80 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800/40 rounded-xl text-xs text-rose-800 dark:text-rose-200 flex items-start gap-2.5">
                        <svg
                            className="w-4 h-4 text-rose-500 shrink-0 mt-0.5"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                            />
                        </svg>
                        <div className="space-y-1 text-[11px] leading-relaxed">
                            <span className="font-semibold">
                                {t("proxy.config.thinking_budget.client_warning_title", {
                                    defaultValue: "危险警告：已切换为客户端直接控制",
                                })}
                            </span>
                            <p>
                                {t("proxy.config.thinking_budget.client_warning_desc", {
                                    defaultValue:
                                        "不同客户端插件（如 Cline, Roo, Cherry 等）上送的 thinking.budget_tokens 或 effort 各不相同。若客户端指定了超过 Google API 支持的数值（如 Flash 超过 24576、或 Pro 传入非标准数值），Google 将直接返回 400 Bad Request 错误中断生成。",
                                })}
                            </p>
                        </div>
                    </div>
                )}

        </>
    );
}
