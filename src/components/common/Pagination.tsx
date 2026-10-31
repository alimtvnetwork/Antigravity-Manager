import React from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface PaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
    totalItems: number;
    itemsPerPage: number;
    onPageSizeChange?: (pageSize: number) => void;
    pageSizeOptions?: number[];
    centerContent?: React.ReactNode;
}

function Pagination({
    currentPage,
    totalPages,
    onPageChange,
    totalItems: _totalItems,
    itemsPerPage,
    onPageSizeChange,
    pageSizeOptions = [50, 100, 150, 200],
    centerContent
}: PaginationProps) {
    const { t } = useTranslation();

    if (totalPages <= 1 && !onPageSizeChange) return null;

    // Calculate displayed page range (up to 5 pages)
    let startPage = Math.max(1, currentPage - 2);
    let endPage = Math.min(totalPages, startPage + 4);

    if (endPage - startPage < 4) {
        startPage = Math.max(1, endPage - 4);
    }

    const pages = [];
    for (let i = startPage; i <= endPage; i++) {
        pages.push(i);
    }

    return (
        <div className="flex items-center justify-between px-4 py-2" role="region" aria-label="Pagination Navigation">
            {/* Mobile View */}
            <nav className="flex flex-1 justify-between sm:hidden" aria-label="Mobile Pagination">
                <button
                    type="button"
                    onClick={() => onPageChange(currentPage - 1)}
                    disabled={currentPage === 1}
                    aria-label={t('common.prev_page') || 'Previous page'}
                    aria-disabled={currentPage === 1}
                    className={`relative inline-flex items-center rounded-md border border-gray-300 dark:border-gray-600 px-4 py-2 text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 ${currentPage === 1
                        ? 'bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed opacity-60'
                        : 'bg-white dark:bg-base-100 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-base-200'
                        }`}
                >
                    {t('common.prev_page')}
                </button>
                <button
                    type="button"
                    onClick={() => onPageChange(currentPage + 1)}
                    disabled={currentPage === totalPages}
                    aria-label={t('common.next_page') || 'Next page'}
                    aria-disabled={currentPage === totalPages}
                    className={`relative ml-3 inline-flex items-center rounded-md border border-gray-300 dark:border-gray-600 px-4 py-2 text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 ${currentPage === totalPages
                        ? 'bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed opacity-60'
                        : 'bg-white dark:bg-base-100 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-base-200'
                        }`}
                >
                    {t('common.next_page')}
                </button>
            </nav>

            {/* Desktop View */}
            <div className="hidden sm:grid sm:grid-cols-[1fr_auto_1fr] sm:items-center w-full gap-3">
                <div className="flex items-center gap-2 justify-self-start">
                    {onPageSizeChange && (
                        <>
                            <label htmlFor="pagination-page-size-select" className="text-sm text-gray-600 dark:text-gray-400">
                                {t('common.per_page')}
                            </label>
                            <select
                                id="pagination-page-size-select"
                                aria-label={t('common.per_page') || 'Items per page'}
                                value={pageSizeOptions.includes(itemsPerPage) ? itemsPerPage : 150}
                                onChange={(e) => onPageSizeChange(parseInt(e.target.value))}
                                className="px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-base-100 text-gray-900 dark:text-base-content focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60"
                            >
                                {pageSizeOptions.map(size => (
                                    <option key={size} value={size}>{size} {t('common.items')}</option>
                                ))}
                            </select>
                        </>
                    )}
                </div>

                <div className="flex items-center justify-center gap-3">
                    {centerContent}
                    <nav className="isolate inline-flex -space-x-px rounded-md shadow-sm" aria-label="Pagination">
                        <button
                            type="button"
                            onClick={() => onPageChange(currentPage - 1)}
                            disabled={currentPage === 1}
                            aria-label={t('common.prev_page') || 'Previous page'}
                            aria-disabled={currentPage === 1}
                            className={`relative inline-flex items-center rounded-l-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-base-200 focus:z-20 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 ${currentPage === 1 ? 'cursor-not-allowed opacity-50' : ''
                                }`}
                        >
                            <span className="sr-only">{t('common.prev_page')}</span>
                            <ChevronLeft className="h-4 w-4" aria-hidden="true" />
                        </button>

                        {/* First Page Link if needed */}
                        {startPage > 1 && (
                            <>
                                <button
                                    type="button"
                                    onClick={() => onPageChange(1)}
                                    aria-label="Page 1"
                                    className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 dark:text-gray-200 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-base-200 focus:z-20 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60"
                                >
                                    1
                                </button>
                                {startPage > 2 && (
                                    <span aria-hidden="true" className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-700 dark:text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 focus:outline-offset-0 select-none">
                                        ...
                                    </span>
                                )}
                            </>
                        )}

                        {pages.map(page => (
                            <button
                                key={page}
                                type="button"
                                onClick={() => onPageChange(page)}
                                aria-label={`Page ${page}`}
                                aria-current={page === currentPage ? 'page' : undefined}
                                className={`relative inline-flex items-center px-4 py-2 text-sm font-semibold focus:z-20 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 ${page === currentPage
                                    ? 'z-10 bg-sky-600 text-white'
                                    : 'text-gray-900 dark:text-gray-200 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-base-200'
                                    }`}
                            >
                                {page}
                            </button>
                        ))}

                        {/* Last Page Link if needed */}
                        {endPage < totalPages && (
                            <>
                                {endPage < totalPages - 1 && (
                                    <span aria-hidden="true" className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-700 dark:text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 focus:outline-offset-0 select-none">
                                        ...
                                    </span>
                                )}
                                <button
                                    type="button"
                                    onClick={() => onPageChange(totalPages)}
                                    aria-label={`Page ${totalPages}`}
                                    className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 dark:text-gray-200 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-base-200 focus:z-20 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60"
                                >
                                    {totalPages}
                                </button>
                            </>
                        )}

                        <button
                            type="button"
                            onClick={() => onPageChange(currentPage + 1)}
                            disabled={currentPage === totalPages}
                            aria-label={t('common.next_page') || 'Next page'}
                            aria-disabled={currentPage === totalPages}
                            className={`relative inline-flex items-center rounded-r-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-base-200 focus:z-20 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 ${currentPage === totalPages ? 'cursor-not-allowed opacity-50' : ''
                                }`}
                        >
                            <span className="sr-only">{t('common.next_page')}</span>
                            <ChevronRight className="h-4 w-4" aria-hidden="true" />
                        </button>
                    </nav>
                </div>
                <div />
            </div>
        </div>
    );
}

export default Pagination;
