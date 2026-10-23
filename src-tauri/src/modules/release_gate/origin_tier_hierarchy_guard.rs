//! 4-Tier Prompt Origin Classification & Hierarchy Guard.
//!
//! Enforces prompt tree views and inspectors discipline per AGENTS.md:
//! Prompt tree views and inspectors must discriminate among 4 distinct origin tiers:
//! 1. `USER_PROMPT` (Sky badge, User icon)
//! 2. `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon)
//! 3. `SYSTEM_MESSAGE` (Zinc badge, Terminal icon)
//! 4. `TOOL_OUTPUT` (Amber badge, Wrench icon)
//! Subagents must nest hierarchically beneath their parent user prompt with branch glyphs (`↳`, `└──`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OriginTier {
    UserPrompt,
    SubagentInstruction,
    SystemMessage,
    ToolOutput,
}

impl OriginTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            OriginTier::UserPrompt => "USER_PROMPT",
            OriginTier::SubagentInstruction => "SUBAGENT_INSTRUCTION",
            OriginTier::SystemMessage => "SYSTEM_MESSAGE",
            OriginTier::ToolOutput => "TOOL_OUTPUT",
        }
    }

    pub fn badge_color_family(&self) -> &'static str {
        match self {
            OriginTier::UserPrompt => "sky",
            OriginTier::SubagentInstruction => "purple",
            OriginTier::SystemMessage => "zinc",
            OriginTier::ToolOutput => "amber",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            OriginTier::UserPrompt => "user",
            OriginTier::SubagentInstruction => "bot",
            OriginTier::SystemMessage => "terminal",
            OriginTier::ToolOutput => "wrench",
        }
    }

    pub fn default_label(&self) -> &'static str {
        match self {
            OriginTier::UserPrompt => "User",
            OriginTier::SubagentInstruction => "Subagent",
            OriginTier::SystemMessage => "System",
            OriginTier::ToolOutput => "Tool",
        }
    }
}

/// Classifies an origin tier from a record type and subagent flag.
pub fn classify_origin_tier(record_type: &str, is_subagent: bool) -> OriginTier {
    if is_subagent {
        return OriginTier::SubagentInstruction;
    }

    let upper = record_type.trim().to_ascii_uppercase();

    if upper.contains("TOOL") || upper.contains("OUTPUT") || upper.contains("WRENCH") {
        OriginTier::ToolOutput
    } else if upper.contains("SUBAGENT") || upper.contains("WORKER") || upper.contains("BOT") {
        OriginTier::SubagentInstruction
    } else if upper.contains("SYSTEM") || upper.contains("TERMINAL") || upper.contains("INIT") {
        OriginTier::SystemMessage
    } else {
        OriginTier::UserPrompt
    }
}

/// Formats a hierarchical tree node line with standard branch glyphs.
pub fn format_hierarchy_tree_node(
    tier: OriginTier,
    depth: usize,
    is_last_sibling: bool,
    label: &str,
) -> String {
    let indent = "    ".repeat(depth.saturating_sub(1));
    let glyph = if depth == 0 {
        ""
    } else if is_last_sibling {
        "└── ↳ "
    } else {
        "├── ↳ "
    };

    let badge = format!("[{}]", tier.as_str());
    if depth == 0 {
        format!("{} {}", badge, label)
    } else {
        format!("{}{}{} {}", indent, glyph, badge, label)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNodeInput {
    pub id: String,
    pub parent_id: Option<String>,
    pub tier: OriginTier,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HierarchyValidationReport {
    pub is_valid: bool,
    pub total_nodes: usize,
    pub user_prompts_count: usize,
    pub subagents_count: usize,
    pub system_messages_count: usize,
    pub tool_outputs_count: usize,
    pub unparented_subagents: Vec<String>,
    pub violation_reasons: Vec<String>,
}

/// Validates that subagents are properly parented under a USER_PROMPT root node.
pub fn validate_tree_hierarchy(nodes: &[TreeNodeInput]) -> HierarchyValidationReport {
    let mut user_prompts_count = 0;
    let mut subagents_count = 0;
    let mut system_messages_count = 0;
    let mut tool_outputs_count = 0;
    let mut unparented_subagents = Vec::new();
    let mut violation_reasons = Vec::new();

    let node_map: std::collections::HashMap<&str, &TreeNodeInput> =
        nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    for node in nodes {
        match node.tier {
            OriginTier::UserPrompt => user_prompts_count += 1,
            OriginTier::SubagentInstruction => {
                subagents_count += 1;
                // Subagents MUST have a parent_id that exists in the tree
                match &node.parent_id {
                    Some(pid) => {
                        if !node_map.contains_key(pid.as_str()) {
                            unparented_subagents.push(node.id.clone());
                            violation_reasons.push(format!(
                                "Subagent '{}' references missing parent '{}'",
                                node.id, pid
                            ));
                        }
                    }
                    None => {
                        unparented_subagents.push(node.id.clone());
                        violation_reasons.push(format!(
                            "Subagent '{}' has no parent USER_PROMPT; must nest hierarchically",
                            node.id
                        ));
                    }
                }
            }
            OriginTier::SystemMessage => system_messages_count += 1,
            OriginTier::ToolOutput => tool_outputs_count += 1,
        }
    }

    let is_valid = violation_reasons.is_empty();

    HierarchyValidationReport {
        is_valid,
        total_nodes: nodes.len(),
        user_prompts_count,
        subagents_count,
        system_messages_count,
        tool_outputs_count,
        unparented_subagents,
        violation_reasons,
    }
}
