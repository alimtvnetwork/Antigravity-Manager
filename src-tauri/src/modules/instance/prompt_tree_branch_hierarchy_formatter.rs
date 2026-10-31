//! Prompt Tree Branch Hierarchy Formatter (`Imp-257`).
//!
//! Enforces AGENTS.md 4-Tier Prompt Origin Classification & Hierarchy:
//! "Prompt tree views and inspectors must discriminate among 4 distinct origin tiers:
//! `USER_PROMPT` (Sky badge, User icon), `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon),
//! `SYSTEM_MESSAGE` (Zinc badge, Terminal icon), and `TOOL_OUTPUT` (Amber badge, Wrench icon).
//! Subagents must nest hierarchically beneath their parent user prompt with branch glyphs (`↳`, `└──`)."

use serde::{Deserialize, Serialize};

/// The 4-tier prompt origin classification hierarchy required by AGENTS.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromptOriginTier {
    UserPrompt,
    SubagentInstruction,
    SystemMessage,
    ToolOutput,
}

/// Visual branch glyph used to represent hierarchical nesting in tree views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BranchGlyph {
    Root,
    ChildBranch,
    TerminalBranch,
}

impl BranchGlyph {
    /// Returns the static branch glyph text.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "",
            Self::ChildBranch => "↳ ",
            Self::TerminalBranch => "└── ",
        }
    }

    /// Resolves the visual branch glyph based on tree depth and child positioning.
    pub fn from_depth_and_position(depth: usize, is_last_child: bool) -> Self {
        if depth == 0 {
            Self::Root
        } else if is_last_child {
            Self::TerminalBranch
        } else {
            Self::ChildBranch
        }
    }
}

/// Input descriptor representing a node in the prompt tree hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNodeDescriptor {
    pub id: String,
    pub parent_id: Option<String>,
    pub origin: PromptOriginTier,
    pub depth: usize,
    pub is_last_child: bool,
    pub title: String,
}

impl TreeNodeDescriptor {
    /// Constructs a new tree node descriptor with explicit properties.
    pub fn new(
        id: impl Into<String>,
        parent_id: Option<impl Into<String>>,
        origin: PromptOriginTier,
        depth: usize,
        is_last_child: bool,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.map(Into::into),
            origin,
            depth,
            is_last_child,
            title: title.into(),
        }
    }
}

/// Formatted row ready for tree view or CLI display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormattedTreeRow {
    pub id: String,
    pub glyph_prefix: String,
    pub badge_label: &'static str,
    pub badge_color: &'static str,
    pub formatted_line: String,
}

/// Returns the badge label and Tailwind color class for a given prompt origin tier.
pub fn origin_badge_props(origin: PromptOriginTier) -> (&'static str, &'static str) {
    match origin {
        PromptOriginTier::UserPrompt => ("USER_PROMPT", "sky"),
        PromptOriginTier::SubagentInstruction => ("SUBAGENT_INSTRUCTION", "purple"),
        PromptOriginTier::SystemMessage => ("SYSTEM_MESSAGE", "zinc"),
        PromptOriginTier::ToolOutput => ("TOOL_OUTPUT", "amber"),
    }
}

/// Returns the iconic symbol name matching the 4-tier origin classification in AGENTS.md.
pub fn origin_icon_name(origin: PromptOriginTier) -> &'static str {
    match origin {
        PromptOriginTier::UserPrompt => "User",
        PromptOriginTier::SubagentInstruction => "Bot",
        PromptOriginTier::SystemMessage => "Terminal",
        PromptOriginTier::ToolOutput => "Wrench",
    }
}

/// Computes the branch glyph prefix string based on node depth and sibling position.
pub fn compute_glyph_prefix(depth: usize, is_last_child: bool) -> String {
    if depth == 0 {
        String::new()
    } else if is_last_child {
        let indent = "  ".repeat(depth - 1);
        format!("{indent}└── ")
    } else {
        let indent = "  ".repeat(depth - 1);
        format!("{indent}↳ ")
    }
}

/// Formats an individual tree node into a display row with branch glyph and tier badge.
pub fn format_tree_node(node: &TreeNodeDescriptor) -> FormattedTreeRow {
    let glyph_prefix = compute_glyph_prefix(node.depth, node.is_last_child);
    let (badge_label, badge_color) = origin_badge_props(node.origin);
    let formatted_line = format!("{glyph_prefix}[{badge_label}] {}", node.title);

    FormattedTreeRow {
        id: node.id.clone(),
        glyph_prefix,
        badge_label,
        badge_color,
        formatted_line,
    }
}

/// Formats an entire slice of tree nodes into formatted display rows.
pub fn format_prompt_tree(nodes: &[TreeNodeDescriptor]) -> Vec<FormattedTreeRow> {
    nodes.iter().map(format_tree_node).collect()
}
