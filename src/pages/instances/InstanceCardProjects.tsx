import { Folder } from 'lucide-react';
import type { InstanceStatus } from '../../services/instanceService';
import type { AgmProjectTreeNode } from '../../components/instances/PromptTreeViewModal';
import { isNodeOwnedByInstance } from './instancePageUtils';
import type { InstancePageApi } from './instancePageTypes';

export function InstanceCardProjects({ api, inst }: { api: InstancePageApi; inst: InstanceStatus }) {
    const { config, projectTreeNodes, setPromptTreeInstance } = api;

    const instanceProjects = projectTreeNodes.filter((node: AgmProjectTreeNode) =>
        isNodeOwnedByInstance(node, inst.config)
    );

    const sortedProjects = [...instanceProjects].sort((a, b) => {
        const isARunning = Boolean(inst.is_running) && Boolean(a.is_running);
        const isBRunning = Boolean(inst.is_running) && Boolean(b.is_running);
        if (isARunning && !isBRunning) return -1;
        if (isBRunning && !isARunning) return 1;
        const aLatest = Math.max(0, ...(a.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        const bLatest = Math.max(0, ...(b.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        if (bLatest !== aLatest) return bLatest - aLatest;
        return a.repo_name.localeCompare(b.repo_name);
    });

    const maxP = (config as { instance_card_max_projects?: number })?.instance_card_max_projects || 3;
    const displayedProjects = sortedProjects.slice(0, maxP);

    return (
        <div className="mt-2 p-2 rounded-[5px] bg-gray-50/80 dark:bg-[#0c2438]/80 border border-gray-200/70 dark:border-[#15334d] text-xs">
            <div className="flex items-center justify-between text-[11px] font-semibold text-slate-700 dark:text-slate-300 mb-1.5 pb-1 border-b border-gray-200/60 dark:border-[#15334d]/60">
                <span className="flex items-center gap-1">
                    <Folder className="w-3 h-3 text-blue-500" />
                    <span>Active / Recent Projects</span>
                </span>
                <span className="px-1.5 py-0.2 rounded-[4px] text-[10px] font-mono bg-gray-200/70 dark:bg-[#15334d] text-slate-600 dark:text-slate-300">
                    {instanceProjects.length}
                </span>
            </div>

            {displayedProjects.length > 0 ? (
                <div className="space-y-1">
                    {displayedProjects.map((proj) => {
                        const isProjRunning = Boolean(inst.is_running) && Boolean(proj.is_running);
                        const totalTurns =
                            proj.conversations?.reduce((sum, c) => sum + Math.max(c.step_count || 1, 1), 0) || 0;

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
                                className="flex items-center justify-between gap-1.5 px-1.5 py-1 rounded-[5px] bg-white dark:bg-[#081a2b] hover:bg-blue-50 dark:hover:bg-[#15334d] border border-gray-200/50 dark:border-[#15334d]/60 transition-colors cursor-pointer group/proj"
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
                                        <span className="px-1 py-0.2 rounded-[4px] text-[9px] font-bold bg-[#1af18d]/15 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 flex items-center gap-0.5 shadow-2xs">
                                            <span className="w-1 h-1 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse" />
                                            RUNNING
                                        </span>
                                    )}
                                    <span className="px-1.5 py-0.2 rounded-[4px] text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-gray-100 dark:bg-[#0c2438] border border-gray-200/50 dark:border-[#15334d]">
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
