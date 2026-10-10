import { useAccountStore } from '../../stores/useAccountStore';
import { useConfigStore } from '../../stores/useConfigStore';
import { useInstanceStore } from '../../stores/useInstanceStore';
import type { InstanceStatus } from '../../services/instanceService';
import { findQuotaModel } from '../../config/modelConfig';
import { isGhostConversation } from '../../components/instances/PromptTreeViewModal';
import type { AgmProjectTreeNode } from '../../components/instances/PromptTreeViewModal';
import { isNodeOwnedByInstance } from './instancePageUtils';
import type { InstanceActionType } from './instancePageUtils';
import type { Account } from '../../types/account';

export interface InstanceCardData {
    boundAccount: Account | null;
    displayEmail: string | null;
    geminiModel: { display_name?: string; name?: string; percentage: number; reset_time?: string } | null;
    weeklyQuota: { percentage: number; resetTime?: string } | null;
    hasActiveTask: boolean;
    effectiveExePath: string | null;
    currentAction: InstanceActionType;
    isBusy: boolean;
}

export function useInstanceCardData(
    inst: InstanceStatus,
    runningTreeNodes: AgmProjectTreeNode[],
    actionState: Record<string, InstanceActionType>
): InstanceCardData {
    const { accounts, currentAccount } = useAccountStore();
    const { config } = useConfigStore();
    const { activeInstanceId } = useInstanceStore();

    const isActive = inst.config.id === activeInstanceId;

    const boundAccount = accounts.find((a) => {
        if (inst.config.bound_account_id) {
            return a.id === inst.config.bound_account_id;
        }
        if (inst.config.bound_email) {
            return a.email.toLowerCase() === (inst.config.bound_email || '').toLowerCase();
        }
        if (isActive) {
            const cur = currentAccount as { email?: string } | null;
            if (cur?.email) {
                return a.email.toLowerCase() === cur.email.toLowerCase();
            }
        }
        return false;
    }) || null;

    const curEmail = (currentAccount as { email?: string } | null)?.email;
    const displayEmail =
        inst.config.bound_email || boundAccount?.email || (isActive && curEmail ? curEmail : null);

    const geminiPro = findQuotaModel(boundAccount?.quota?.models, 'gemini-pro');
    const geminiFlash = findQuotaModel(boundAccount?.quota?.models, 'gemini-flash');
    const geminiModel = geminiPro || geminiFlash || null;

    const weeklyBucket = (boundAccount?.quota?.quota_groups || [])
        .find((item) => {
            const name = (item.display_name || '').toLowerCase();
            return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
        })
        ?.buckets?.find(
            (item) =>
                (item.window || '').toLowerCase().includes('week') ||
                (item.bucket_id || '').toLowerCase().includes('week')
        );

    const weeklyQuota = weeklyBucket
        ? {
              percentage: Math.round((weeklyBucket.remaining_fraction || 0) * 100),
              resetTime: weeklyBucket.reset_time,
          }
        : ((boundAccount?.quota as { weekly?: { percentage: number; resetTime?: string } | null })?.weekly || null);

    const hasActiveTask =
        Boolean(inst.is_running) &&
        runningTreeNodes.some((node) => {
            const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
            const hasGenuineRunningConv = (node.conversations || []).some(
                (c) =>
                    Boolean(c.is_running) &&
                    !isGhostConversation(c) &&
                    !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            return isInstanceMatch && Boolean(node.is_running) && hasGenuineRunningConv;
        });

    const cfg = config as { antigravity_ide_executable?: string; antigravity_executable?: string } | null;
    const defaultExePath =
        cfg?.antigravity_ide_executable || cfg?.antigravity_executable || 'Antigravity IDE (Default System Path)';
    const effectiveExePath = inst.config.executable_path || (inst.config.is_default ? defaultExePath : null);

    const currentAction = actionState[inst.config.id] || null;
    const isBusy = Boolean(currentAction);

    return {
        boundAccount,
        displayEmail,
        geminiModel,
        weeklyQuota,
        hasActiveTask,
        effectiveExePath,
        currentAction,
        isBusy,
    };
}
