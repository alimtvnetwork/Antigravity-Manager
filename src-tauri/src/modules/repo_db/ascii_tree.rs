//! Prompt Tree ASCII Tree Visualizer.
//! Generates monospace hierarchical tree representations with terminal branch glyphs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiTreeNode {
    pub label: String,
    pub badge: Option<String>,
    pub children: Vec<AsciiTreeNode>,
}

/// Renders a tree structure into a clean ASCII/Unicode branch hierarchy string.
pub fn render_ascii_tree(root: &AsciiTreeNode) -> String {
    let mut out = String::new();
    let root_label = match &root.badge {
        Some(b) => format!("{} [{}]", root.label, b),
        None => root.label.clone(),
    };
    out.push_str(&root_label);
    out.push('\n');

    render_children(&root.children, "", &mut out);
    out
}

fn render_children(children: &[AsciiTreeNode], prefix: &str, out: &mut String) {
    let count = children.len();
    for (i, child) in children.iter().enumerate() {
        let is_last = i + 1 == count;
        let connector = if is_last { "└── " } else { "├── " };
        let node_line = match &child.badge {
            Some(b) => format!("{}{}{} [{}]\n", prefix, connector, child.label, b),
            None => format!("{}{}{}\n", prefix, connector, child.label),
        };
        out.push_str(&node_line);

        let next_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        render_children(&child.children, &next_prefix, out);
    }
}
