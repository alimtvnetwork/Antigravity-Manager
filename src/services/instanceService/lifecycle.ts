import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../stores/error-store';
import type { InstanceStatus } from './types';

export async function launchInstance(instanceId: string): Promise<void> {
    try {
        return await invoke('launch_instance', { instanceId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.launchInstance',
            endpoint: 'launch_instance',
            triggerAction: 'launch_instance',
            context: { instanceId },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function focusOrLaunchInstance(instanceId: string, workspacePath?: string): Promise<boolean> {
    try {
        return await invoke('focus_or_launch_instance', { instanceId, workspacePath });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.focusOrLaunchInstance',
            endpoint: 'focus_or_launch_instance',
            triggerAction: 'focus_or_launch_instance',
            context: { instanceId, workspacePath },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function focusInstanceWorkspace(instanceId: string, repoPath: string, repoName: string): Promise<boolean> {
    try {
        return await invoke('focus_instance_workspace', { instanceId, repoPath, repoName });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.focusInstanceWorkspace',
            endpoint: 'focus_instance_workspace',
            triggerAction: 'focus_instance_workspace',
            context: { instanceId, repoPath, repoName },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function sendPromptNow(
    instanceId: string,
    repoPath: string,
    promptContent: string,
    conversationId?: string
): Promise<any> {
    try {
        return await invoke('send_prompt_now', { instanceId, repoPath, promptContent, conversationId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.sendPromptNow',
            endpoint: 'send_prompt_now',
            triggerAction: 'send_prompt_now',
            context: { instanceId, repoPath },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function enqueuePrompt(
    instanceId: string,
    repoPath: string,
    promptContent: string,
    conversationId?: string
): Promise<any> {
    try {
        return await invoke('enqueue_prompt', { instanceId, repoPath, promptContent, conversationId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.enqueuePrompt',
            endpoint: 'enqueue_prompt',
            triggerAction: 'enqueue_prompt',
            context: { instanceId, repoPath },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function getRunningInstancesProcessCount(): Promise<number> {
    try {
        return await invoke('get_running_instances_process_count');
    } catch (e) {
        useErrorStore.getState().trackWarning(e, {
            source: 'instanceService.getRunningInstancesProcessCount',
            triggerAction: 'get_running_instances_process_count',
        });
        return 0;
    }
}

export async function cloneInstanceExecutable(instanceId: string): Promise<string> {
    try {
        return await invoke('clone_instance_executable', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.cloneInstanceExecutable',
            endpoint: 'clone_instance_executable',
            triggerAction: 'clone_instance_executable',
        });
        throw e;
    }
}

export async function setInstanceExecutable(instanceId: string, executablePath?: string): Promise<void> {
    try {
        return await invoke('set_instance_executable', { instanceId, executablePath });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.setInstanceExecutable',
            endpoint: 'set_instance_executable',
            triggerAction: 'set_instance_executable',
        });
        throw e;
    }
}

export async function closeInstance(instanceId: string): Promise<void> {
    return await invoke('close_instance', { instanceId });
}

export async function stopInstance(instanceId: string): Promise<void> {
    return await invoke('stop_instance', { instanceId });
}

export async function restartInstance(instanceId: string): Promise<InstanceStatus> {
    try {
        return await invoke('restart_instance', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.restartInstance',
            endpoint: 'restart_instance',
            triggerAction: 'restart_instance',
        });
        throw e;
    }
}

export async function fastForwardInstance(instanceId: string): Promise<string> {
    try {
        return await invoke('fast_forward_instance', { instanceId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.fastForwardInstance',
            endpoint: 'fast_forward_instance',
            triggerAction: 'fast_forward_instance',
            context: { instanceId },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}

export async function cleanAndRestartWorkspace(): Promise<string> {
    try {
        return await invoke('clean_and_restart_workspace');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.cleanAndRestartWorkspace',
            endpoint: 'clean_and_restart_workspace',
            triggerAction: 'clean_and_restart_workspace',
        });
        throw e;
    }
}

export async function getActiveInstance(): Promise<string> {
    return await invoke('get_active_instance');
}

export async function setActiveInstance(instanceId: string): Promise<void> {
    return await invoke('set_active_instance', { instanceId });
}

export async function setDefaultInstance(instanceId: string): Promise<void> {
    return await invoke('set_default_instance', { instanceId });
}

export async function switchAccountToInstance(accountId: string, instanceId?: string): Promise<void> {
    try {
        return await invoke('switch_account_to_instance', { accountId, instanceId });
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.switchAccountToInstance',
            endpoint: 'switch_account_to_instance',
            triggerAction: 'switch_account_to_instance',
            context: { accountId, instanceId },
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}
