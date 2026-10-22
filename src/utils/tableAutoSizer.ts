/**
 * Dynamic Table Column Auto-Sizer.
 * Calculates optimal column width distributions based on container width, min/max constraints, and flex factors.
 */

export interface TableColumnConfig {
    id: string;
    minWidth?: number;
    maxWidth?: number;
    defaultWidth?: number;
    flex?: number; // relative share of surplus width (default 1)
}

export interface AutoSizerOptions {
    gap?: number;
    padding?: number;
}

/**
 * Calculates responsive column widths given a total container width and column configurations.
 */
export function calculateOptimalColumnWidths(
    columns: TableColumnConfig[],
    containerWidth: number,
    options: AutoSizerOptions = {}
): Record<string, number> {
    const { gap = 0, padding = 0 } = options;
    const availableWidth = Math.max(0, containerWidth - padding - (columns.length - 1) * gap);

    const widths: Record<string, number> = {};
    if (columns.length === 0) return widths;

    // 1. Assign minWidth or defaultWidth baseline
    let totalBaseWidth = 0;
    const flexibleCols: TableColumnConfig[] = [];

    for (const col of columns) {
        const base = col.minWidth || col.defaultWidth || 100;
        widths[col.id] = base;
        totalBaseWidth += base;
        if ((col.flex ?? 1) > 0) {
            flexibleCols.push(col);
        }
    }

    // 2. Distribute surplus width according to flex factors
    const surplus = availableWidth - totalBaseWidth;
    if (surplus > 0 && flexibleCols.length > 0) {
        const totalFlex = flexibleCols.reduce((sum, col) => sum + (col.flex ?? 1), 0);
        let distributed = 0;

        for (let i = 0; i < flexibleCols.length; i++) {
            const col = flexibleCols[i];
            const isLast = i === flexibleCols.length - 1;
            const flexShare = isLast
                ? surplus - distributed
                : Math.floor(surplus * ((col.flex ?? 1) / totalFlex));

            const calculated = widths[col.id] + flexShare;
            const capped = col.maxWidth ? Math.min(col.maxWidth, calculated) : calculated;
            widths[col.id] = capped;
            distributed += flexShare;
        }
    }

    return widths;
}
