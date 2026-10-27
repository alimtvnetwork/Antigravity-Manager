/**
 * Prompt Origin Classification & Tracking Types
 * Supports 4-tier prompt origin classification + Warmup Probe tier (Improvement 04).
 */

export type PromptOriginTier =
  | 'USER_PROMPT'          // Sky badge, User icon
  | 'SUBAGENT_INSTRUCTION'  // Purple badge, Bot icon
  | 'SYSTEM_MESSAGE'       // Zinc badge, Terminal icon
  | 'TOOL_OUTPUT'          // Amber badge, Wrench icon
  | 'WARMUP_PROBE';        // Orange badge, Flame/Sparkles icon

export interface PromptOriginMetadata {
  tier: PromptOriginTier;
  label?: string;
  sourceInstanceId?: string;
  subagentId?: string;
  timestamp?: number;
  probeModel?: string;
}

export interface PromptTreeNode {
  id: string;
  parentId?: string | null;
  origin: PromptOriginTier;
  content: string;
  createdAt: number;
  children?: PromptTreeNode[];
}
