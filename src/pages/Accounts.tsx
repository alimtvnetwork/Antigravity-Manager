import { Search, List, LayoutGrid, Trash2, ToggleLeft, ToggleRight, LocateFixed, RefreshCw, ChevronDown, Download, Loader2, Sparkles, Upload } from "lucide-react";
import AddAccountDialog from "../components/accounts/AddAccountDialog";
import { useAccountsState } from "./accounts/useAccountsState";
import { useAccountsHandlers } from "./accounts/useAccountsHandlers";
import { AccountsContent } from "./accounts/AccountsContent";
import { AccountsDialogs } from "./accounts/AccountsDialogs";
import { cn } from "../utils/cn";
import Pagination from "../components/common/Pagination";
import { showToast } from "../components/common/ToastContainer";
import type { FilterType } from "./accounts/types";

export default function Accounts() {
  const state = useAccountsState();
  const handlers = useAccountsHandlers(state);
  const {
    t, showAllQuotas, toggleShowAllQuotas, updateInfo, installUpdate, isInstalling,
    searchQuery, setSearchQuery, filter, setFilter, currentPage, setCurrentPage,
    activeFocusTargetAccount, isSearchExpanded, setIsSearchExpanded, viewMode, setViewMode, selectedIds,
    setIsWarmupConfirmOpen, isWarmuping, setIsBackupModalOpen, setBackupModalTab, setLocalPageSize, ITEMS_PER_PAGE,
    filterCounts, filteredAccounts, isRefreshing, searchInputRef, currentAccount
  } = state;
  const {
    handlePageChange, handleAddAccount, handleBatchDelete, handleBatchToggleProxy, handleRefreshClick, handleFocusActiveAccount
  } = handlers;
  return (
    <div className="h-full flex flex-col px-2.5 sm:px-4 pt-1.5 pb-4 gap-2.5 max-w-[1920px] mx-auto w-full min-w-0">
    {/* Top action bar: search, filters, and action buttons */}
    <div className="flex-none flex flex-wrap 2xl:flex-nowrap items-center justify-between gap-1.5 min-w-0 w-full">
      {/* Left controls: search, window, view mode, quota filter */}
      <div className="flex items-center gap-1 flex-wrap sm:flex-nowrap overflow-x-auto lg:overflow-x-visible scrollbar-none max-w-full py-0.5 min-w-0 shrink-0">
        {/* Search box - responsive */}
        <div className="hidden lg:block flex-none w-36 relative transition-all focus-within:w-44">
          <Search className="absolute left-2.5 top-1/2 transform -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
          <input
            type="text"
            placeholder={t('accounts.search_placeholder')}
            className="w-full h-8 pl-8 pr-3 bg-gray-100/50 dark:bg-white/[0.04] text-xs text-gray-900 dark:text-base-content border border-transparent hover:border-gray-200/50 dark:hover:border-white/5 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500/50 placeholder:text-gray-400 dark:placeholder:text-gray-500 transition-all"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        {/* Search button - small screens */}
        <div className="lg:hidden relative">
          {!isSearchExpanded ? (
            <button
              onClick={() => {
                setIsSearchExpanded(true);
                setTimeout(() => searchInputRef.current?.focus(), 100);
              }}
              className="h-8 w-8 inline-flex items-center justify-center bg-gray-100/40 dark:bg-white/[0.04] hover:bg-gray-200/60 dark:hover:bg-white/[0.08] rounded-lg transition-colors"
              title={t('accounts.search_placeholder')}
            >
              <Search className="w-3.5 h-3.5 text-gray-600 dark:text-gray-300" />
            </button>
          ) : (
            <div className="absolute left-0 top-0 z-10 w-64 flex items-center gap-1">
              <div className="flex-1 relative">
                <Search className="absolute left-2.5 top-1/2 transform -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
                <input
                  ref={searchInputRef}
                  type="text"
                  placeholder={t('accounts.search_placeholder')}
                  className="w-full h-8 pl-8 pr-3 bg-white dark:bg-slate-900 text-xs text-gray-900 dark:text-base-content border border-transparent rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500/50 placeholder:text-gray-400 dark:placeholder:text-gray-500 shadow-lg"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onBlur={() => setIsSearchExpanded(false)}
                />
              </div>
            </div>
          )}
        </div>

        {/* View mode switcher - Flat continuous grouping */}
        <div className="h-8 inline-flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-100/40 dark:bg-white/[0.04] shrink-0">
          <button
            className={cn(
              "h-7 w-7 inline-flex items-center justify-center rounded-md transition-all",
              viewMode === "list"
                ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs"
                : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content",
            )}
            onClick={() => setViewMode("list")}
            title={t("accounts.views.list")}
          >
            <List className="w-4 h-4" />
          </button>
          <button
            className={cn(
              "h-7 w-7 inline-flex items-center justify-center rounded-md transition-all",
              viewMode === "grid"
                ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs"
                : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content",
            )}
            onClick={() => setViewMode("grid")}
            title={t("accounts.views.grid")}
          >
            <LayoutGrid className="w-4 h-4" />
          </button>
        </div>

        <label className="h-8 inline-flex items-center shrink-0">
          <select
            className="h-8 rounded-lg border border-gray-200 bg-white px-2 text-xs font-semibold text-slate-950 dark:border-slate-700 dark:bg-[#070b10] dark:text-[#f5d76e]"
            value={filter}
            onChange={(event) => setFilter(event.target.value as FilterType)}
            aria-label="Account tier"
          >
            <option value="all">{t('accounts.all')} ({filterCounts.all})</option>
            <option value="pro">{t('accounts.pro')} ({filterCounts.pro})</option>
            <option value="ultra">{t('accounts.ultra')} ({filterCounts.ultra})</option>
            <option value="free">{t('accounts.free')} ({filterCounts.free})</option>
          </select>
        </label>
      </div>

      {/* Action buttons group */}
      <div className="flex items-center gap-2 shrink-0 ml-auto">
        {/* Batch operations if items selected */}
        {selectedIds.size > 0 && (
          <div className="flex items-center rounded-md bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
            <button
              className="px-2.5 py-1 text-red-600 dark:text-red-400 text-xs font-semibold rounded-l-md hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors flex items-center gap-1.5 cursor-pointer"
              onClick={handleBatchDelete}
              title={t("accounts.delete_selected", { count: selectedIds.size })}
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span className="hidden xl:inline">{t("accounts.delete_selected", { count: selectedIds.size })}</span>
            </button>
            <button
              className="px-2.5 py-1 text-orange-600 dark:text-orange-400 text-xs font-semibold hover:bg-orange-50 dark:hover:bg-orange-950/30 transition-colors flex items-center gap-1.5 cursor-pointer"
              onClick={() => handleBatchToggleProxy(false)}
              title={t("accounts.disable_proxy_selected", { count: selectedIds.size })}
            >
              <ToggleLeft className="w-3.5 h-3.5" />
              <span className="hidden xl:inline">{t("accounts.disable_proxy_selected", { count: selectedIds.size })}</span>
            </button>
            <button
              className="px-2.5 py-1 text-cyan-600 dark:text-cyan-400 text-xs font-semibold rounded-r-md hover:bg-cyan-50 dark:hover:bg-cyan-950/30 transition-colors flex items-center gap-1.5 cursor-pointer"
              onClick={() => handleBatchToggleProxy(true)}
              title={t("accounts.enable_proxy_selected", { count: selectedIds.size })}
            >
              <ToggleRight className="w-3.5 h-3.5" />
              <span className="hidden xl:inline">{t("accounts.enable_proxy_selected", { count: selectedIds.size })}</span>
            </button>
          </div>
        )}

        {/* Segmented Pill Capsule: Focus, +, Refresh, Warm, Show All Quotas */}
        <div className="flex items-center rounded-md bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
          {/* Focus Active/Picked Account Button */}
          <button
            type="button"
            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-l-md text-amber-700 dark:text-amber-300 hover:bg-amber-100/70 dark:hover:bg-amber-950/40 transition-colors cursor-pointer"
            onClick={handleFocusActiveAccount}
            title={
              activeFocusTargetAccount
                ? `Focus onto active account: ${activeFocusTargetAccount.email}`
                : currentAccount
                ? `Focus onto active account: ${currentAccount.email}`
                : "Focus active/picked account"
            }
          >
            <LocateFixed className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
            <span className="hidden sm:inline">Focus</span>
          </button>

          {/* Add Account Dialog */}
          <AddAccountDialog
            onAdd={handleAddAccount}
            showText={false}
            triggerClassName="flex items-center justify-center px-2.5 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
          />

          {/* Refresh */}
          <button
            className={cn(
              "flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-blue-600 dark:text-cyan-300 hover:bg-blue-50 dark:hover:bg-[#15334d] transition-colors cursor-pointer",
              isRefreshing && "opacity-70 cursor-not-allowed"
            )}
            onClick={handleRefreshClick}
            disabled={isRefreshing}
            title={
              selectedIds.size > 0
                ? t("accounts.refresh_selected", { count: selectedIds.size })
                : t("accounts.refresh_all")
            }
          >
            <RefreshCw
              className={cn("w-3.5 h-3.5", isRefreshing && "animate-spin")}
            />
            <span className="hidden xl:inline">
              {isRefreshing
                ? t("common.loading")
                : selectedIds.size > 0
                  ? t("accounts.refresh_selected", { count: selectedIds.size })
                  : t("accounts.refresh_all")}
            </span>
          </button>

          {/* Warmup */}
          <button
            className={cn(
              "flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-[#15334d] transition-colors cursor-pointer",
              isWarmuping && "opacity-70 cursor-not-allowed"
            )}
            onClick={() => setIsWarmupConfirmOpen(true)}
            disabled={isWarmuping}
            title={
              selectedIds.size > 0
                ? t("accounts.warmup_selected", { count: selectedIds.size })
                : t("accounts.warmup_all", "Warm up all accounts")
            }
          >
            <Sparkles
              className={cn("w-3.5 h-3.5", isWarmuping && "animate-pulse")}
            />
            <span className="hidden xl:inline">
              {isWarmuping
                ? t("common.loading")
                : selectedIds.size > 0
                  ? t("accounts.warmup_selected", { count: selectedIds.size })
                  : t("accounts.warmup_all", "Warm up all")}
            </span>
          </button>

          {/* Show All Emails */}
          <label
            className="flex items-center gap-2 cursor-pointer select-none px-3 py-1 rounded-r-md text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors"
            title={t("accounts.show_all_emails")}
          >
            <span className="text-xs font-semibold hidden xl:inline">
              {t("accounts.show_all_emails")}
            </span>
            <input
              type="checkbox"
              className="toggle toggle-xs toggle-primary"
              checked={showAllQuotas}
              onChange={toggleShowAllQuotas}
            />
          </label>
        </div>
        <div className="w-px h-4 bg-gray-200 dark:bg-gray-700 self-center mx-1 shrink-0"></div>

        <div className="relative group">
          <button
            type="button"
            className="px-2.5 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 text-xs font-medium rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors flex items-center gap-1.5 cursor-pointer"
            onClick={() => {
              setBackupModalTab("export");
              setIsBackupModalOpen(true);
            }}
            title={`${t("accounts.import_json")} / ${t("common.export")}`}
          >
            <Download className="w-3.5 h-3.5" />
            <span className="hidden lg:inline">Import / Export</span>
            <ChevronDown className="w-3 h-3 text-gray-400" />
          </button>
          <div className="invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-all duration-150 absolute right-0 top-full pt-1 w-44 z-[9999]">
            <div className="bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-xl py-1 text-xs">
              <button
                type="button"
                onClick={() => {
                  setBackupModalTab("import");
                  setIsBackupModalOpen(true);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
              >
                <Upload className="w-3.5 h-3.5 text-cyan-600 dark:text-cyan-400" />
                <span>{t("accounts.import_json")}</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  setBackupModalTab("export");
                  setIsBackupModalOpen(true);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
              >
                <Download className="w-3.5 h-3.5 text-blue-500" />
                <span>{t("common.export")}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

      <AccountsContent state={state} handlers={handlers} />

    {/* Minimalist floating pagination */}
    {filteredAccounts.length > 0 && (
      <div className="flex-none">
        <Pagination
          currentPage={currentPage}
          totalPages={Math.ceil(filteredAccounts.length / ITEMS_PER_PAGE)}
          onPageChange={handlePageChange}
          totalItems={filteredAccounts.length}
          itemsPerPage={ITEMS_PER_PAGE}
          onPageSizeChange={(newSize) => {
            setLocalPageSize(newSize);
            setCurrentPage(1); // Reset to first page
          }}
          pageSizeOptions={[50, 100, 150, 200]}
          centerContent={
            updateInfo?.has_update ? (
              <button
                type="button"
                onClick={async () => {
                  showToast(t("update_notification.installing_desc", "Launching official updater in background..."), "info");
                  await installUpdate();
                }}
                disabled={isInstalling}
                className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-gradient-to-r from-blue-500/10 to-purple-500/10 hover:from-blue-500/20 hover:to-purple-500/20 text-blue-600 dark:text-blue-400 border border-blue-500/20 shadow-xs transition-all duration-200 cursor-pointer active:scale-95 group disabled:opacity-75 disabled:cursor-not-allowed"
                title="A newer version is available. Click to launch installer."
              >
                {isInstalling ? (
                  <Loader2 className="w-3.5 h-3.5 text-blue-500 animate-spin" />
                ) : (
                  <Sparkles className="w-3.5 h-3.5 text-blue-500 group-hover:rotate-12 transition-transform" />
                )}
                <span>Update Available: v{updateInfo.latest_version}</span>
                <span className="text-[10px] bg-blue-600 hover:bg-blue-500 text-white px-1.5 py-0.5 rounded font-semibold ml-0.5 flex items-center gap-1">
                  {isInstalling ? "Installing..." : "Install Now"}
                </span>
              </button>
            ) : null
          }
        />
      </div>
    )}

      <AccountsDialogs state={state} handlers={handlers} />
    </div>
  );
}
