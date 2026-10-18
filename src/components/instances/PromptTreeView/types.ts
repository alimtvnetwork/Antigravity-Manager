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
    prompts?: AgmPromptTurnNode[];
    selected_turn_number?: number | null;
}

export interface AgmPromptTurnNode {
    turn_number: number;
    step_index: number;
    prompt_text: string;
    preview_200w: string;
    status: 'RUNNING' | 'QUEUED' | 'COMPLETED' | string;
    is_running: boolean;
    is_queued: boolean;
    timestamp: string;
    created_at: number;
    media_paths: string[];
    media_thumbnails: string[];
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
    renameConversation?: (conversationId: string, newTitle: string) => Promise<void> | void;
}

export interface TreePanelProps {
    renameConversation?: (conversationId: string, newTitle: string) => Promise<void> | void;
    onRenameConversation?: (conversationId: string, newTitle: string) => Promise<void> | void;
    onResendPrompts?: (count: number | 'all') => void;
    onEnqueue?: () => void;
    onRerunConversation?: (convoId: string, nudgeType: string) => void;
}


export enum PromptOriginTier {
    UserPrompt = 'USER_PROMPT',
    SubagentInstruction = 'SUBAGENT_INSTRUCTION',
    SystemMessage = 'SYSTEM_MESSAGE',
    ToolOutput = 'TOOL_OUTPUT',
}

export enum PromptViewMode {
    Preview = 'preview',
    Raw = 'raw',
    Edit = 'edit',
}

export enum CopyFormat {
    RichMedia = 'rich_media',
    CleanText = 'clean_text',
    Json = 'json',
}

export type ViewMode = 'preview' | 'raw' | 'edit';

export type PromptTier = PromptOriginTier | 'USER_PROMPT' | 'SUBAGENT_INSTRUCTION' | 'SYSTEM_MESSAGE' | 'TOOL_OUTPUT';

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

export interface ExtractedPromptImage {
    alt: string;
    dataUri: string;
    mimeType: string;
    byteLength: number;
    hash: string;
}

export interface PromptRewriteResult {
    rewrittenPrompt: string;
    hasImages: boolean;
    images: ExtractedPromptImage[];
}


