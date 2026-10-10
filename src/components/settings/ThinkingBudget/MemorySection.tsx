import { BudgetSectionProps } from "./BudgetSectionProps";
import { HardDrive, HelpCircle, Layers, Trash2 } from "lucide-react";

export function MemorySection(props: BudgetSectionProps) {
    const { t, currentConfig, onChange, inputValues, setInputValues, handleInputChange, getPresetTooltip, controlSource, thinkingStoreEnabled, onThinkingStoreChange, thinkingMaxMemoryTurns, onThinkingMaxMemoryTurnsChange, thinkingRetentionDays, onThinkingRetentionDaysChange, isSaving, isSaved, handleSave, onSave, showClaudeAdvanced, setShowClaudeAdvanced, isClearingThinking, showClearThinkingConfirm, setShowClearThinkingConfirm, handleClearThinkingStore } = props;

    return (
        <>
                {(onThinkingMaxMemoryTurnsChange || onThinkingRetentionDaysChange) && (
                    <div className="p-3.5 bg-blue-50/70 dark:bg-blue-900/20 border border-blue-200/80 dark:border-blue-800/40 rounded-xl space-y-3 shadow-2xs">
                        <div className="flex items-center justify-between flex-wrap gap-2">
                            <div className="flex items-center gap-2">
                                <span className="text-xs font-bold text-gray-900 dark:text-white flex items-center gap-1.5">
                                    <Layers size={14} className="text-blue-600 dark:text-blue-400" />
                                    {t("proxy.config.thinking_budget.window_settings_title", {
                                        defaultValue: "思考块双层滑动窗口与容量配置 (RAM + SQLite)",
                                    })}
                                </span>
                                <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-blue-800/60">
                                    {t("proxy.config.thinking_budget.dual_layer_tag", {
                                        defaultValue: "双层索引穿透",
                                    })}
                                </span>
                            </div>

                            {/* 清空思考块按钮与小字提示 */}
                            <div className="flex items-center gap-2">
                                <span className="text-[10px] text-gray-500 dark:text-gray-400 hidden sm:inline-block">
                                    {t("proxy.config.thinking_budget.clear_tip", {
                                        defaultValue: "仅当缓存命中异常、版本更新或开发者要求时才删除",
                                    })}
                                </span>
                                <button
                                    type="button"
                                    onClick={() => setShowClearThinkingConfirm(true)}
                                    disabled={isClearingThinking}
                                    title={t("proxy.config.thinking_budget.clear_btn_tooltip", {
                                        defaultValue: "清空所有思考块缓存与数据库（不影响请求日志）",
                                    })}
                                    className="btn btn-xs btn-outline btn-error gap-1.5 h-6 min-h-6 px-2.5 text-[11px] font-medium rounded-lg shadow-2xs hover:shadow-xs transition-all"
                                >
                                    <Trash2 size={12} className={isClearingThinking ? "animate-spin" : ""} />
                                    {t("proxy.config.thinking_budget.clear_btn", {
                                        defaultValue: "清空思考块",
                                    })}
                                </button>
                            </div>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                            {/* 1. 内存常驻思考轮次 (L1 RAM) */}
                            {onThinkingMaxMemoryTurnsChange && (
                                <div className="p-3 bg-white/90 dark:bg-base-100 rounded-lg border border-blue-200/70 dark:border-base-200 shadow-2xs flex items-center justify-between gap-3">
                                    <div className="space-y-0.5 min-w-0">
                                        <div className="flex items-center gap-1.5">
                                            <Layers size={13} className="text-blue-600 dark:text-blue-400 shrink-0" />
                                            <span className="text-xs font-bold text-gray-800 dark:text-gray-200 truncate">
                                                {t("proxy.config.thinking_budget.max_memory_turns_label", {
                                                    defaultValue: "内存常驻轮次 (RAM 窗口)",
                                                })}
                                            </span>
                                            <span className="text-[10px] px-1.5 py-0.2 rounded font-bold bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 shrink-0">
                                                {t("proxy.config.thinking_budget.default_600_tag", {
                                                    defaultValue: "默认 600",
                                                })}
                                            </span>
                                        </div>
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 leading-tight">
                                            {t("proxy.config.thinking_budget.max_memory_turns_subdesc", {
                                                defaultValue: "单轮思考约 2 KB；600 轮 ≈ 1.2 MB / 会话。",
                                            })}
                                        </p>
                                    </div>
                                    <div className="flex items-center gap-1.5 shrink-0">
                                        <input
                                            type="number"
                                            min={10}
                                            max={10000}
                                            step={50}
                                            className="input input-xs input-bordered w-20 text-center font-mono font-bold bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-white"
                                            value={thinkingMaxMemoryTurns}
                                            onChange={(e) => {
                                                const val = parseInt(e.target.value, 10);
                                                if (!isNaN(val)) {
                                                    onThinkingMaxMemoryTurnsChange(Math.max(10, Math.min(10000, val)));
                                                }
                                            }}
                                        />
                                        <span className="text-xs font-medium text-gray-500">
                                            {t("proxy.config.thinking_budget.turns_unit", { defaultValue: "轮" })}
                                        </span>
                                    </div>
                                </div>
                            )}

                            {/* 2. SQLite 思考库保留周期 (L2 Disk) */}
                            {onThinkingRetentionDaysChange && (
                                <div className="p-3 bg-white/90 dark:bg-base-100 rounded-lg border border-purple-200/70 dark:border-base-200 shadow-2xs flex items-center justify-between gap-3">
                                    <div className="space-y-0.5 min-w-0">
                                        <div className="flex items-center gap-1.5">
                                            <HardDrive size={13} className="text-purple-600 dark:text-purple-400 shrink-0" />
                                            <span className="text-xs font-bold text-gray-800 dark:text-gray-200 truncate">
                                                {t("proxy.config.experimental.thinking_retention_days_label", {
                                                    defaultValue: "思考库保留周期 (SQLite)",
                                                })}
                                            </span>
                                            <span className="text-[10px] px-1.5 py-0.2 rounded font-bold bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300 shrink-0">
                                                {t("proxy.config.thinking_budget.default_15_days_tag", {
                                                    defaultValue: "默认 15天",
                                                })}
                                            </span>
                                        </div>
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 leading-tight">
                                            {t("proxy.config.thinking_budget.retention_days_subdesc", {
                                                defaultValue: "活跃会话每次请求自动顺延，无请求才过期。",
                                            })}
                                        </p>
                                    </div>
                                    <div className="flex items-center gap-1.5 shrink-0">
                                        <input
                                            type="number"
                                            min={1}
                                            max={365}
                                            className="input input-xs input-bordered w-20 text-center font-mono font-bold bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-white"
                                            value={thinkingRetentionDays}
                                            onChange={(e) => {
                                                const val = parseInt(e.target.value, 10);
                                                if (!isNaN(val)) {
                                                    onThinkingRetentionDaysChange(Math.max(1, Math.min(365, val)));
                                                }
                                            }}
                                        />
                                        <span className="text-xs font-medium text-gray-500">
                                            {t("proxy.config.thinking_budget.days_unit", { defaultValue: "天" })}
                                        </span>
                                    </div>
                                </div>
                            )}
                        </div>

                        {/* 滑动窗口机制与服务器内存配置建议指南 */}
                        <div className="p-3 rounded-lg bg-white/80 dark:bg-base-200/80 border border-blue-100 dark:border-blue-900/40 text-xs space-y-2 text-gray-600 dark:text-gray-300">
                            <div className="flex items-start gap-1.5 font-semibold text-blue-950 dark:text-blue-200">
                                <HelpCircle size={14} className="text-blue-500 shrink-0 mt-0.5" />
                                <span>
                                    {t("proxy.config.thinking_budget.window_guide_title", {
                                        defaultValue: "滑动窗口淘汰机制与各并发规模选型建议",
                                    })}
                                    :
                                </span>
                            </div>
                            <p className="leading-relaxed pl-5 text-[11px] text-gray-500 dark:text-gray-400">
                                {t("proxy.config.thinking_budget.window_guide_desc", {
                                    defaultValue:
                                        "超长会话超过此设定轮次时，系统自动执行滑动窗口先进先出（FIFO）淘汰；被淘汰轮次绝不降级为破坏缓存的占位符，而是由本地 SQLite 专属索引（primary_tool_id）在纳秒级精准穿透回捞，保证 Prompt Cache 100% 字节级严格对齐且绝不 OOM。",
                                })}
                            </p>
                            <div className="pl-5 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 text-[11px] pt-1">
                                {CONCURRENCY_GUIDE_PRESETS.map((preset) => (
                                    <div
                                        key={preset.key}
                                        className="p-2 rounded bg-blue-50/40 dark:bg-base-300/40 border border-blue-100/60 dark:border-base-300"
                                    >
                                        <span className="font-bold text-gray-800 dark:text-gray-200 block">
                                            {preset.icon} {t(preset.titleKey, { defaultValue: preset.titleDefault })}
                                        </span>
                                        <span className="text-gray-500 dark:text-gray-400">
                                            <Trans
                                                i18nKey={preset.descKey}
                                                defaults={preset.descDefault}
                                                components={{ 1: <strong className="text-blue-600 dark:text-blue-400" /> }}
                                            />
                                        </span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                )}

        </>
    );
}
