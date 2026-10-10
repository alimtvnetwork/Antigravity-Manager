import { AlertTriangle } from "lucide-react";
import { BudgetSectionProps } from "./BudgetSectionProps";

export function ClearConfirmModal(props: BudgetSectionProps) {
    const { t, isClearingThinking, showClearThinkingConfirm, setShowClearThinkingConfirm, handleClearThinkingStore } = props;

    return (
        <>
                {showClearThinkingConfirm && (
                    <div className="modal modal-open">
                        <div className="modal-box max-w-md bg-white dark:bg-base-100 border border-base-300 shadow-2xl p-5">
                            <div className="flex items-start gap-3">
                                <div className="p-2 rounded-full bg-error/10 text-error shrink-0 mt-0.5">
                                    <AlertTriangle size={20} />
                                </div>
                                <div className="space-y-2 min-w-0">
                                    <h3 className="text-sm font-bold text-gray-900 dark:text-white">
                                        {t("proxy.config.thinking_budget.clear_modal_title", {
                                            defaultValue: "确认清空思考块存储？",
                                        })}
                                    </h3>
                                    <p className="text-xs text-gray-600 dark:text-gray-300 leading-relaxed whitespace-pre-line">
                                        {t("proxy.config.thinking_budget.clear_modal_desc", {
                                            defaultValue:
                                                "此操作将彻底清空内存常驻轮次 (RAM) 与本地 SQLite 数据库中所有的历史思维链和工具签名记录。\n\n⚠️ 注意：此操作仅清理思考块数据，绝不删除任何反向代理请求日志。",
                                        })}
                                    </p>
                                    <div className="p-2.5 rounded-lg bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800/50 text-[11px] text-amber-800 dark:text-amber-300 leading-relaxed">
                                        {t("proxy.config.thinking_budget.clear_modal_warning", {
                                            defaultValue:
                                                "建议仅在出现缓存命中异常、版本升级或开发者明确要求时执行。清空后新请求将重新建立干净的前缀索引。",
                                        })}
                                    </div>
                                </div>
                            </div>

                            <div className="modal-action mt-5 flex justify-end gap-2">
                                <button
                                    type="button"
                                    className="btn btn-sm btn-ghost"
                                    onClick={() => setShowClearThinkingConfirm(false)}
                                    disabled={isClearingThinking}
                                >
                                    {t("common.cancel", { defaultValue: "取消" })}
                                </button>
                                <button
                                    type="button"
                                    className="btn btn-sm btn-error text-white gap-1.5"
                                    onClick={handleClearThinkingStore}
                                    disabled={isClearingThinking}
                                >
                                    {isClearingThinking ? (
                                        <span className="loading loading-spinner loading-xs" />
                                    ) : (
                                        <Trash2 size={13} />
                                    )}
                                    {t("proxy.config.thinking_budget.clear_confirm_btn", {
                                        defaultValue: "确认清空",
                                    })}
                                </button>
                            </div>
                        </div>
                        <div
                            className="modal-backdrop bg-black/40"
                            onClick={() => !isClearingThinking && setShowClearThinkingConfirm(false)}
                        />
                    </div>
                )}
        </>
    );
}
