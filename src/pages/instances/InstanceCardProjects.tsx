import { Folder } from 'lucide-react';
import type { InstanceStatus } from '../../services/instanceService';
import type { AgmProjectTreeNode } from '../../components/instances/PromptTreeViewModal';
import { isNodeOwnedByInstance, isConversationGenuinelyRunning, isConversationOwnedByInstance } from './instancePageUtils';
import type { InstancePageApi } from './instancePageTypes';

export function isProjectRunningForInstance(proj: AgmProjectTreeNode, inst: InstanceStatus): boolean {
    const isInstanceRunning = Boolean(inst.is_running);
    if (!isInstanceRunning) {
        return false;
    }
    const isProjectRunning = Boolean(proj.is_running);
    if (!isProjectRunning) {
        return false;
    }
    const conversations = proj.conversations || [];
    return conversations.some((c) =>
        isConversationGenuinelyRunning(c, inst.config, proj.instance_seq_num)
    );
}

export function InstanceCardProjects({ api, inst }: { api: InstancePageApi; inst: InstanceStatus }) {
    const { config, projectTreeNodes, setPromptTreeInstance } = api;

    const instanceProjects = projectTreeNodes.filter((node: AgmProjectTreeNode) =>
        isNodeOwnedByInstance(node, inst.config)
    );

    const sortedProjects = [...instanceProjects].sort((a, b) => {
        const isARunning = isProjectRunningForInstance(a, inst);
        const isBRunning = isProjectRunningForInstance(b, inst);
        if (isARunning !== isBRunning) {
            return isARunning ? -1 : 1;
        }
        const aLatest = Math.max(0, ...(a.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        const bLatest = Math.max(0, ...(b.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        if (bLatest !== aLatest) return bLatest - aLatest;
        return a.repo_name.localeCompare(b.repo_name);
    });

    const maxP = (config as { instance_card_max_projects?: number })?.instance_card_max_projects || 3;
    const displayedProjects = sortedProjects.slice(0, maxP);

    const isSettling =
        api.resumeIndicator?.instanceId === inst.config.id &&
        (api.resumeIndicator.status === 'scheduled' || api.resumeIndicator.status === 'restoring');

    return (
        <div className="relative mt-2 p-2 rounded-[5px] bg-gray-50/80 dark:bg-[#171025]/80 border border-gray-200/70 dark:border-[#40285e] text-xs">
            {isSettling && (
                <div className="absolute inset-0 bg-slate-900/70 dark:bg-[#090611]/85 backdrop-blur-[3px] z-10 flex flex-col items-center justify-center gap-1 rounded-[5px] p-2 text-center select-none border border-amber-500/30">
                    <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 animate-pulse font-mono">
                        <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping" />
                        <span>🔒 Masked ({api.resumeIndicator?.secondsLeft ?? 10}s)</span>
                    </span>
                    <span className="text-[10px] text-slate-300 dark:text-slate-400 font-medium">
                        Prompts masked during switch settle
                    </span>
                </div>
            )}
            <div className="flex items-center justify-between text-[11px] font-semibold text-slate-700 dark:text-slate-300 mb-1.5 pb-1 border-b border-gray-200/60 dark:border-[#40285e]/60">
                <span className="flex items-center gap-1">
                    <Folder className="w-3 h-3 text-blue-500" />
                    <span>Active / Recent Projects</span>
                </span>
                <span className="px-1.5 py-0.2 rounded-[4px] text-[10px] font-mono bg-gray-200/70 dark:bg-[#40285e] text-slate-600 dark:text-slate-300">
                    {instanceProjects.length}
                </span>
            </div>

            {displayedProjects.length > 0 ? (
                <div className="space-y-1">
                    {displayedProjects.map((proj) => {
                        const isProjRunning = isProjectRunningForInstance(proj, inst);
                        const instanceConvs = (proj.conversations || []).filter((c) =>
                            isConversationOwnedByInstance(c.instance_id, inst.config, c.instance_seq_num ?? proj.instance_seq_num)
                        );
                        const totalTurns =
                            instanceConvs.reduce((sum, c) => sum + Math.max(c.prompts?.length || 1, 1), 0);

                        return (
                            <div
                                key={proj.project_id}
                                onClick={() =>
                                    setPromptTreeInstance({
                                        id: inst.config.id,
                                        name: inst.config.name,
                                        projectId: proj.project_id,
                                        seqNum: inst.config.seq_num,
                                        executablePath: inst.config.executable_path,
                                    })
                                }
                                className="flex items-center justify-between gap-1.5 px-1.5 py-1 rounded-[5px] bg-white dark:bg-[#081a2b] hover:bg-blue-50 dark:hover:bg-[#40285e] border border-gray-200/50 dark:border-[#40285e]/60 transition-colors cursor-pointer group/proj"
                                title="Click to view prompts in Prompt Tree"
                            >
                                <div className="flex items-center gap-1.5 min-w-0 flex-1">
                                    <Folder className="w-3 h-3 shrink-0 text-slate-400 group-hover/proj:text-blue-500 transition-colors" />
                                    <span
                                        className="truncate font-medium text-[11px] text-slate-800 dark:text-slate-200"
                                        title={proj.repo_name}
                                    >
                                        {proj.repo_name}
                                    </span>
                                </div>
                                <div className="flex items-center gap-1 shrink-0">
                                    {isProjRunning && (
                                        <span className="px-1 py-0.2 rounded-[4px] text-[9px] font-bold bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border border-emerald-500/25 flex items-center gap-0.5 shadow-2xs">
                                            <span className="w-1 h-1 rounded-full bg-emerald-500 shadow-[0_0_2px_rgba(16,185,129,0.5)]" />
                                            RUNNING
                                        </span>
                                    )}
                                    <span className="px-1.5 py-0.2 rounded-[4px] text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-gray-100 dark:bg-[#171025] border border-gray-200/50 dark:border-[#40285e]">
                                        {totalTurns} turns
                                    </span>
                                </div>
                            </div>
                        );
                    })}
                </div>
            ) : (
                <div className="text-[10px] text-gray-400 dark:text-slate-500 italic py-1 text-center">
                    No recent projects
                </div>
            )}
        </div>
    );
}
