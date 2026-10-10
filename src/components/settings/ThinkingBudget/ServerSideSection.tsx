import { BudgetSectionProps } from "./BudgetSectionProps";

export function ServerSideSection(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, controlSource, thinkingStoreEnabled, onThinkingStoreChange, thinkingMaxMemoryTurns, onThinkingMaxMemoryTurnsChange, thinkingRetentionDays, onThinkingRetentionDaysChange, isSaving, isSaved, handleSave, onSave, showClaudeAdvanced, setShowClaudeAdvanced, isClearingThinking, showClearThinkingConfirm, setShowClearThinkingConfirm, handleClearThinkingStore } = props;

    return (
        <>
                {onThinkingStoreChange && (
                    <div className="p-3.5 bg-purple-50/70 dark:bg-purple-900/20 border border-purple-200/80 dark:border-purple-800/40 rounded-xl flex items-center justify-between gap-4 shadow-2xs">
                        <div className="space-y-0.5">
                            <div className="flex items-center gap-2">
                                <span className="text-xs font-bold text-gray-900 dark:text-white">
                                    {t("proxy.config.thinking_budget.store_enabled", { defaultValue: "服务端思考块与签名回填" })}
                                </span>
                                <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300 border border-purple-200 dark:border-purple-800/60">
                                    {t("proxy.config.thinking_budget.default_on_tag", { defaultValue: "默认开启" })}
                                </span>
                            </div>
                            <p className="text-xs text-gray-600 dark:text-gray-300 max-w-xl leading-relaxed">
                                {t("proxy.config.thinking_budget.store_enabled_desc", {
                                    defaultValue: "自动截获并持久化思考过程与加密签名；当商业 Agent（如 Cline, Roo, Claude Code 等）多轮对话未带回思考时由服务端自动补齐，杜绝上游 400 报错或上下文丢失。"
                                })}
                            </p>
                        </div>
                        <input
                            type="checkbox"
                            className="toggle toggle-sm toggle-primary shrink-0"
                            checked={thinkingStoreEnabled}
                            onChange={(e) => onThinkingStoreChange(e.target.checked)}
                        />
                    </div>
                )}

        </>
    );
}
