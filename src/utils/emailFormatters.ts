import { invoke } from '@tauri-apps/api/core';
import { isTauri } from './env';
import { showToast } from '../components/common/ToastContainer';
import type { EmailAccount, EmailAccountInput } from '../services/emailService';

/**
 * Universal fields for single or bulk mailbox accounts
 */
export interface MailboxDataPayload {
    alias: string;
    email: string;
    password?: string;
    smtp_host: string;
    smtp_port: number;
    imap_host: string;
    imap_port: number;
    encryption_type: string;
    is_default: boolean;
    is_active: boolean;
}

/**
 * Clean account object for export, omitting internal IDs and timestamps
 */
export function cleanAccountForExport(acc: Partial<EmailAccount> | Partial<EmailAccountInput> | MailboxDataPayload | Record<string, any>): MailboxDataPayload {
    return {
        alias: acc.alias || 'Primary Mailbox',
        email: acc.email || '',
        ...((acc as any).password ? { password: (acc as any).password } : {}),
        smtp_host: acc.smtp_host || 'smtp.gmail.com',
        smtp_port: Number(acc.smtp_port) || 587,
        imap_host: acc.imap_host || 'imap.gmail.com',
        imap_port: Number(acc.imap_port) || 993,
        encryption_type: acc.encryption_type || 'TLS',
        is_default: Boolean(acc.is_default),
        is_active: acc.is_active !== false,
    };
}

/**
 * Convert single account to JSON
 */
export function accountToJson(account: Partial<EmailAccount> | MailboxDataPayload, pretty = true): string {
    const cleaned = cleanAccountForExport(account);
    return JSON.stringify(cleaned, null, pretty ? 2 : undefined);
}

/**
 * Convert multiple accounts to JSON
 */
export function accountsToJson(accounts: EmailAccount[], pretty = true): string {
    const cleaned = accounts.map(cleanAccountForExport);
    return JSON.stringify(cleaned, null, pretty ? 2 : undefined);
}

/**
 * Helper to escape YAML scalar values
 */
function escapeYamlValue(val: any): string {
    if (val === null || val === undefined) return '""';
    if (typeof val === 'boolean' || typeof val === 'number') return String(val);
    const str = String(val);
    if (str === '' || str.includes(':') || str.includes('#') || str.includes('\n') || str.includes('"') || str.startsWith('@')) {
        return JSON.stringify(str);
    }
    return str;
}

/**
 * Convert single account to YAML
 */
export function accountToYaml(account: Partial<EmailAccount> | MailboxDataPayload): string {
    const cleaned = cleanAccountForExport(account);
    return Object.entries(cleaned)
        .map(([key, val]) => `${key}: ${escapeYamlValue(val)}`)
        .join('\n');
}

/**
 * Convert multiple accounts to YAML list
 */
export function accountsToYaml(accounts: EmailAccount[]): string {
    if (accounts.length === 0) return '[]';
    return accounts
        .map((acc) => {
            const cleaned = cleanAccountForExport(acc);
            const lines = Object.entries(cleaned).map(([key, val], idx) => {
                const prefix = idx === 0 ? '- ' : '  ';
                return `${prefix}${key}: ${escapeYamlValue(val)}`;
            });
            return lines.join('\n');
        })
        .join('\n\n');
}

/**
 * CSV Headers for mailboxes
 */
const CSV_HEADERS = [
    'alias',
    'email',
    'password',
    'smtp_host',
    'smtp_port',
    'imap_host',
    'imap_port',
    'encryption_type',
    'is_default',
    'is_active',
];

function escapeCsvField(val: any): string {
    if (val === null || val === undefined) return '';
    const str = String(val);
    if (str.includes(',') || str.includes('"') || str.includes('\n')) {
        return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
}

/**
 * Convert single account to CSV
 */
export function accountToCsv(account: Partial<EmailAccount> | MailboxDataPayload): string {
    const cleaned = cleanAccountForExport(account);
    const row = [
        escapeCsvField(cleaned.alias),
        escapeCsvField(cleaned.email),
        escapeCsvField(cleaned.password || ''),
        escapeCsvField(cleaned.smtp_host),
        escapeCsvField(cleaned.smtp_port),
        escapeCsvField(cleaned.imap_host),
        escapeCsvField(cleaned.imap_port),
        escapeCsvField(cleaned.encryption_type),
        escapeCsvField(cleaned.is_default),
        escapeCsvField(cleaned.is_active),
    ];
    return `${CSV_HEADERS.join(',')}\n${row.join(',')}`;
}

/**
 * Convert multiple accounts to CSV
 */
export function accountsToCsv(accounts: EmailAccount[]): string {
    const headerLine = CSV_HEADERS.join(',');
    const rows = accounts.map((acc) => {
        const cleaned = cleanAccountForExport(acc);
        return [
            escapeCsvField(cleaned.alias),
            escapeCsvField(cleaned.email),
            escapeCsvField(cleaned.password || ''),
            escapeCsvField(cleaned.smtp_host),
            escapeCsvField(cleaned.smtp_port),
            escapeCsvField(cleaned.imap_host),
            escapeCsvField(cleaned.imap_port),
            escapeCsvField(cleaned.encryption_type),
            escapeCsvField(cleaned.is_default),
            escapeCsvField(cleaned.is_active),
        ].join(',');
    });
    return [headerLine, ...rows].join('\n');
}

/**
 * Robust parser that ingests JSON, YAML, or CSV text and extracts mailbox account records
 */
export function parseAccountsFromText(text: string): {
    accounts: MailboxDataPayload[];
    format: 'json' | 'yaml' | 'csv' | 'unknown';
    error?: string;
} {
    const trimmed = text.trim();
    if (!trimmed) {
        return { accounts: [], format: 'unknown', error: 'Empty content' };
    }

    // 1. Try parsing JSON
    if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
        try {
            const parsed = JSON.parse(trimmed);
            const list = Array.isArray(parsed) ? parsed : [parsed];
            const accounts = list.map((item) => cleanAccountForExport(item));
            return { accounts, format: 'json' };
        } catch (jsonErr: any) {
            // Not valid JSON, continue to other formats
        }
    }

    // 2. Try parsing CSV
    const lines = trimmed.split(/\r?\n/).filter((l) => l.trim().length > 0);
    if (lines.length > 0 && lines[0].includes(',')) {
        const firstLine = lines[0].toLowerCase();
        if (firstLine.includes('email') || firstLine.includes('smtp') || firstLine.includes('alias')) {
            const headers = lines[0].split(',').map((h) => h.trim().replace(/^["']|["']$/g, ''));
            const accounts: MailboxDataPayload[] = [];

            for (let i = 1; i < lines.length; i++) {
                const cols = parseCsvLine(lines[i]);
                if (cols.length === 0) continue;
                const obj: any = {};
                headers.forEach((h, idx) => {
                    obj[h] = cols[idx] ?? '';
                });
                if (obj.email || obj.alias) {
                    accounts.push(cleanAccountForExport(obj));
                }
            }

            if (accounts.length > 0) {
                return { accounts, format: 'csv' };
            }
        }
    }

    // 3. Try parsing YAML
    try {
        const yamlAccounts = parseSimpleYaml(trimmed);
        if (yamlAccounts.length > 0) {
            return { accounts: yamlAccounts.map(cleanAccountForExport), format: 'yaml' };
        }
    } catch {
        // Continue
    }

    return { accounts: [], format: 'unknown', error: 'Could not recognize format. Please check JSON, YAML, or CSV syntax.' };
}

/**
 * Basic CSV line splitter respecting quotes
 */
function parseCsvLine(line: string): string[] {
    const result: string[] = [];
    let current = '';
    let inQuotes = false;

    for (let i = 0; i < line.length; i++) {
        const char = line[i];
        if (char === '"') {
            if (inQuotes && line[i + 1] === '"') {
                current += '"';
                i++;
            } else {
                inQuotes = !inQuotes;
            }
        } else if (char === ',' && !inQuotes) {
            result.push(current.trim());
            current = '';
        } else {
            current += char;
        }
    }
    result.push(current.trim());
    return result;
}

/**
 * Lightweight YAML parser for single and list mailbox objects
 */
function parseSimpleYaml(yamlText: string): any[] {
    const lines = yamlText.split(/\r?\n/);
    const results: any[] = [];
    let currentObj: any = null;

    for (const rawLine of lines) {
        const line = rawLine.trim();
        if (!line || line.startsWith('#')) continue;

        if (line.startsWith('- ')) {
            if (currentObj && Object.keys(currentObj).length > 0) {
                results.push(currentObj);
            }
            currentObj = {};
            const rest = line.substring(2).trim();
            const colonIdx = rest.indexOf(':');
            if (colonIdx > 0) {
                const key = rest.substring(0, colonIdx).trim();
                const val = rest.substring(colonIdx + 1).trim().replace(/^["']|["']$/g, '');
                currentObj[key] = val;
            }
            continue;
        }

        const colonIdx = line.indexOf(':');
        if (colonIdx > 0) {
            const key = line.substring(0, colonIdx).trim();
            const val = line.substring(colonIdx + 1).trim().replace(/^["']|["']$/g, '');
            if (!currentObj) {
                currentObj = {};
            }
            currentObj[key] = val;
        }
    }

    if (currentObj && Object.keys(currentObj).length > 0) {
        results.push(currentObj);
    }

    return results;
}

/**
 * Download or save file with full native Tauri file save dialog support
 */
export async function downloadOrSaveFile(
    fileName: string,
    content: string,
    extension: 'json' | 'yaml' | 'yml' | 'csv' | 'txt' | 'xlsx'
): Promise<boolean> {
    if (isTauri()) {
        try {
            const { save } = await import('@tauri-apps/plugin-dialog');
            const targetPath = await save({
                defaultPath: fileName,
                filters: [
                    {
                        name: extension.toUpperCase(),
                        extensions:
                            extension === 'yaml' || extension === 'yml'
                                ? ['yaml', 'yml']
                                : extension === 'xlsx'
                                  ? ['xlsx', 'xls']
                                  : [extension],
                    },
                    {
                        name: 'All Files',
                        extensions: ['*'],
                    },
                ],
            });

            if (!targetPath) {
                return false; // User cancelled
            }

            await invoke('save_text_file', {
                path: targetPath,
                content,
            });

            showToast(`Saved successfully: ${targetPath}`, 'success');
            return true;
        } catch (e: any) {
            console.error('Tauri save dialog error:', e);
            showToast(`Save failed: ${e?.message || e}`, 'error');
            return false;
        }
    } else {
        // Web fallback
        const mimeTypes: Record<string, string> = {
            json: 'application/json',
            yaml: 'text/yaml',
            yml: 'text/yaml',
            csv: 'text/csv',
            txt: 'text/plain',
            xlsx: 'application/vnd.ms-excel',
        };
        const blob = new Blob([content], { type: mimeTypes[extension] || 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = fileName;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        showToast(`Downloaded ${fileName}`, 'success');
        return true;
    }
}
