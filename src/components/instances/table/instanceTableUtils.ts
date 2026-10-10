import type { InstanceStatus } from '../../../services/instanceService';
import type { InstanceActionType, InstanceStatusWithDataDir } from './instanceTableTypes';

export function formatShortPath(fullPath: string): string {
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

export function resolveDataDir(inst: InstanceStatus): string {
    const withDir = inst as InstanceStatusWithDataDir;
    return withDir.data_dir || inst.config.data_dir || inst.config.executable_path || '';
}
