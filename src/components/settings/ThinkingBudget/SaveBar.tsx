import { BudgetSectionProps } from "./BudgetSectionProps";
import { Check, Save } from "lucide-react";

export function SaveBar(props: BudgetSectionProps) {
    const { t, isSaving, isSaved, handleSave, onSave } = props;

    return (
        <>
                {onSave && (
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3.5 border-t border-gray-200 dark:border-base-200">
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                            {t("proxy.config.thinking_budget.save_hint", {
                                defaultValue: "修改思考预算设置后点击保存即可立即生效，配置将即时热更新至当前运行的反代服务。",
                            })}
                        </span>
                        <button
                            type="button"
                            disabled={isSaving}
                            onClick={handleSave}
                            className={`px-4 py-2 rounded-xl text-xs font-semibold flex items-center gap-1.5 shadow-sm transition-all active:scale-95 shrink-0 cursor-pointer ${
                                isSaved
                                    ? "bg-emerald-600 hover:bg-emerald-500 text-white"
                                    : "bg-blue-600 hover:bg-blue-500 text-white"
                            } ${isSaving ? "opacity-75 cursor-wait" : ""}`}
                        >
                            {isSaved ? (
                                <>
                                    <Check size={14} className="text-white" />
                                    <span>{t("common.saved", { defaultValue: "已保存生效" })}</span>
                                </>
                            ) : (
                                <>
                                    <Save size={14} className={isSaving ? "animate-spin" : ""} />
                                    <span>{isSaving ? t("common.saving", { defaultValue: "保存中..." }) : t("proxy.config.thinking_budget.save_btn", { defaultValue: "保存思考设置并热生效" })}</span>
                                </>
                            )}
                        </button>
                    </div>
                )}

        </>
    );
}
