/**
 * Formatted Audit Data Exporter
 * Converts audit log records into CSV, TSV, and Markdown tables with browser download triggers.
 */

export function exportAuditRecordsToCsv(records: Record<string, unknown>[]): string {
  if (records.length === 0) return '';

  const headers = Object.keys(records[0]);
  const lines: string[] = [headers.map(h => `"${h.replace(/"/g, '""')}"`).join(',')];

  for (const row of records) {
    const values = headers.map(h => {
      const val = row[h];
      if (val === null || val === undefined) return '""';
      const str = typeof val === 'object' ? JSON.stringify(val) : String(val);
      return `"${str.replace(/"/g, '""')}"`;
    });
    lines.push(values.join(','));
  }

  return lines.join('\n');
}

export function exportAuditRecordsToMarkdown(
  records: Record<string, unknown>[],
  columns?: string[]
): string {
  if (records.length === 0) return '*No audit records found.*';

  const cols = columns && columns.length > 0 ? columns : Object.keys(records[0]);

  const headerLine = `| ${cols.join(' | ')} |`;
  const separatorLine = `| ${cols.map(() => ':---').join(' | ')} |`;

  const rows = records.map(row => {
    const cellVals = cols.map(c => {
      const val = row[c];
      if (val === null || val === undefined) return '';
      const str = typeof val === 'object' ? JSON.stringify(val) : String(val);
      return str.replace(/\|/g, '\\|').replace(/\n/g, ' ');
    });
    return `| ${cellVals.join(' | ')} |`;
  });

  return [headerLine, separatorLine, ...rows].join('\n');
}

export function downloadExportFile(content: string, filename: string, mimeType = 'text/csv;charset=utf-8;'): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}
