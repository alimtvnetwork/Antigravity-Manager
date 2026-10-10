export interface AgmConversationNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    conversation_id: string;
    short_id: string;
    title: string;
    status: string;
    is_running: boolean;
    step_count: number;
    instance_id: string;
    instance_seq_num?: number;
    instance_name?: string;
    instance_exe_name?: string;
    prompt_preview_200w: string;
    prompt_tail_snippet?: string;
    prompt_word_count: number;
    last_modified: string;
    byte_size?: number;
    repeat_count?: number;
    repeat_badge?: string;
    sub_runs?: AgmConversationNode[];
    prompt_category?: string;
    is_queued?: boolean;
    full_prompt_text?: string;
    latest_step_summary?: string;
    latest_response?: string;
    execution_results?: string;
    tool_calls_summary?: string;
}


export interface AgmProjectTreeNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    project_id: string;
    repo_name: string;
    repo_path: string;
    instance_id: string;
    instance_seq_num?: number;
    instance_name: string;
    instance_exe_name?: string;
    bound_email?: string;
    is_running: boolean;
    conversations: AgmConversationNode[];
    running_count?: number;
    queued_count?: number;
    byte_size?: number;
    repeat_count?: number;
    repeat_badge?: string;
}

export interface PromptTreeViewModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId: string;
    instanceName: string;
    sequenceNumber?: number;
    executablePath?: string;
    initialSelectedProjectId?: string;
}

export type ViewMode = 'preview' | 'raw' | 'edit';

export type PromptTier = 'USER_PROMPT' | 'SUBAGENT_INSTRUCTION' | 'SYSTEM_MESSAGE' | 'TOOL_OUTPUT';

export interface TierClassificationResult {
    tier: PromptTier;
    category: PromptTier;
    label: string;
    roleBadge: string;
    badgeStyle: string;
    iconName: 'user' | 'bot' | 'terminal' | 'wrench';
    isSubagent: boolean;
    isNonPrompt: boolean;
    confidence: number;
    matchedPattern?: string;
    subagentRole?: string;
    description: string;
}

export type PromptCategory = PromptTier;
export type PromptCategoryResult = TierClassificationResult;


export interface HierarchicalConversationNode {
    primaryNode: AgmConversationNode;
    classification: TierClassificationResult;
    subagents: HierarchicalConversationNode[];
    isExpanded: boolean;
}

