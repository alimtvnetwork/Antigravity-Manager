import type { InstanceActionType } from '../../components/instances/InstanceTable';
import type { AgmProjectTreeNode } from '../../components/instances/PromptTreeViewModal';

export { type InstanceActionType };

export function truncatePath(fullPath?: string | null): string {
    if (!fullPath) return '';
    const isWindows = fullPath.includes('\\') || /^[a-zA-Z]:/.test(fullPath);
    const sep = isWindows ? '\\' : '/';
    const parts = fullPath.split(/[\\/]/).filter(Boolean);
    if (parts.length === 0) return fullPath;
    return `...${sep}${parts[parts.length - 1]}`;
}

export function getActionLabel(action: InstanceActionType): string {
    switch (action) {
        case 'launch':
            return 'Launching...';
        case 'stop':
            return 'Stopping...';
        case 'restart':
            return 'Restarting...';
        case 'switch':
            return 'Switching...';
        case 'fast-forward':
            return 'Rotating...';
        case 'sync':
            return 'Syncing...';
        case 'wipe':
            return 'Wiping...';
        case 'delete':
            return 'Deleting...';
        default:
            return 'Processing...';
    }
}

export function isDefaultOwned(nodeInstId: string | undefined | null, cfgId: string): boolean {
    if (!nodeInstId || nodeInstId.trim() === '') {
        return false;
    }
    const isDefaultAlias = nodeInstId === 'default' || nodeInstId === '__default__';
    const isIdMatch = Boolean(cfgId) && nodeInstId === cfgId;
    return isDefaultAlias || isIdMatch;
}

export const isNodeOwnedByInstance = (
    node: AgmProjectTreeNode,
    instConfig: { id: string; is_default?: boolean; seq_num?: number }
): boolean => {
    if (!node.instance_id || node.instance_id.trim() === '') {
        return false;
    }
    const isDefaultInstance = instConfig.is_default || instConfig.id === 'default';
    if (isDefaultInstance) {
        return isDefaultOwned(node.instance_id, instConfig.id);
    }
    if (node.instance_id === 'default' || node.instance_id === '__default__') {
        return false;
    }
    if (node.instance_id === instConfig.id) {
        return true;
    }
    const hasSeqNum = typeof instConfig.seq_num === 'number' && instConfig.seq_num > 1;
    if (hasSeqNum && node.instance_seq_num === instConfig.seq_num) {
        return true;
    }
    return false;
};

export interface InstanceTheme {
    name: string;
    accentBar: string;
    badge: string;
    emailPill: string;
    dot: string;
}

export const INSTANCE_THEMES: InstanceTheme[] = [
    {
        name: 'Indigo',
        accentBar: 'from-indigo-500 via-blue-500 to-indigo-600',
        badge: 'bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border-indigo-400/30',
        emailPill:
            'bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800/60',
        dot: 'bg-indigo-500',
    },
    {
        name: 'Emerald',
        accentBar: 'from-emerald-500 via-teal-500 to-emerald-600',
        badge: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-400/30',
        emailPill:
            'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/60',
        dot: 'bg-emerald-500',
    },
    {
        name: 'Cyan',
        accentBar: 'from-cyan-500 via-sky-500 to-blue-500',
        badge: 'bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border-cyan-400/30',
        emailPill:
            'bg-cyan-50 dark:bg-cyan-950/40 text-cyan-700 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800/60',
        dot: 'bg-cyan-500',
    },
    {
        name: 'Amber',
        accentBar: 'from-amber-500 via-yellow-500 to-orange-500',
        badge: 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-400/30',
        emailPill:
            'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800/60',
        dot: 'bg-amber-500',
    },
    {
        name: 'Sky',
        accentBar: 'from-sky-500 via-blue-500 to-cyan-600',
        badge: 'bg-sky-500/10 text-sky-700 dark:text-sky-300 border-sky-400/30',
        emailPill:
            'bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 border-sky-200 dark:border-sky-800/60',
        dot: 'bg-sky-500',
    },
    {
        name: 'Teal',
        accentBar: 'from-teal-500 via-emerald-500 to-teal-600',
        badge: 'bg-teal-500/10 text-teal-700 dark:text-teal-300 border-teal-400/30',
        emailPill:
            'bg-teal-50 dark:bg-teal-950/40 text-teal-700 dark:text-teal-300 border-teal-200 dark:border-teal-800/60',
        dot: 'bg-teal-500',
    },
    {
        name: 'Slate',
        accentBar: 'from-slate-500 via-gray-500 to-slate-600',
        badge: 'bg-slate-500/10 text-slate-700 dark:text-slate-300 border-slate-400/30',
        emailPill:
            'bg-slate-100 dark:bg-slate-900/60 text-slate-700 dark:text-slate-300 border-slate-300 dark:border-slate-800',
        dot: 'bg-slate-500',
    },
];
