// Regex / identity patterns (split from prompt_sanitizer.rs)
use once_cell::sync::Lazy;
use regex::Regex;

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

/// Code block protection regex: isolates multiline fenced blocks ```...``` and inline code `...`
pub(crate) static RE_CODE_BLOCK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?ms)(```[\s\S]*?```|`[^`\r\n]+`)").unwrap());

/// Regex matching high-risk client pseudo-headers that trigger upstream Google WAF (applied only to non-code blocks):
/// 1) Generalized matching of client-injected `*-billing*` pseudo-header lines (`x-anthropic-billing-header:`, `x-billing:`, `x-client-billing:`, `anthropic-billing-header:`, etc.)
/// 2) Matching any `x-` header carrying Claude Code CLI billing signatures (`cc_version`, `cc_entrypoint`, `cch=`)
/// 3) Matching mid-paragraph embedded `x-anthropic-billing-header:` and subsequent declarations
/// 4) Matching Claude Agent SDK leading fingerprint declarations (`You are a Claude agent, built on Anthropic's Claude Agent SDK.`)
/// Strictly excludes any `session` keywords to preserve user queries and conversation tracking.
pub(crate) static RE_WAF_TRIGGER_HEADERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?im)^\s*(?:x-[a-z0-9_-]*billing[a-z0-9_-]*|[a-z0-9_-]+-billing-(?:header|metadata|token|info)):\s*[^\r\n]*(\r?\n)?",
        r"|^\s*x-[a-z0-9_-]+:\s*[^\r\n]*(?:cc_version|cc_entrypoint|cch=)[^\r\n]*(\r?\n)?",
        r"|(?i)x-anthropic-billing-header:\s*[^\r\n]*",
        r"|(?i)You are a Claude agent, built on Anthropic's Claude Agent SDK\.(\r?\n)?"
    ))
    .unwrap()
});

/// Regex to collapse redundant empty lines (collapsing 3+ consecutive newlines to 2 to maintain paragraph structure)
pub(crate) static RE_MULTI_NEWLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

// ============================ System Track: Identity Declaration Normalization ============================
//
// What triggers upstream WAF pseudo-rate-limiting (false 429 RESOURCE_EXHAUSTED) is identity attribution declarations,
// rather than model capabilities or request size. Upstream applies stricter rules when identifying vendor / product /
// competitor model fingerprints in self-introductions, rejecting requests and falsely flagging accounts as exhausted.
// (See issues #3507 / #3506, and Codex `based on GPT-x` in #3444 / #3489).
//
// Client frameworks cannot be exhaustively enumerated, so normalization must be broad-spectrum:
// any statement containing opening + noun phrase + attribution/version is normalized to neutral identity.

/// Identity normalization scan window: scans only the first 4 sentences of each system prompt block.
/// Identity declarations appear at the start of prompts; restricting the window ensures
/// normal occurrences of 'created by ...' in body text/code are 100% unaffected.
pub(crate) const IDENTITY_SCAN_MAX_SENTENCES: usize = 4;

/// Neutral identity declaration after normalization (stripping vendor/product/competitor fingerprints)
pub(crate) const NEUTRAL_IDENTITY: &str = "You are an AI Agent.";

/// Identity declaration normalization regex (broad match, active only in system prompt head window).
///
/// Form = A Identity opening + B Identity noun phrase + C Attribution/version + D Subject + E Punctuation:
/// ```text
/// You are Hermes Agent, an intelligent AI assistant created by Nous Research.   → You are an AI Agent.
/// You are Codex, an advanced coding agent based on GPT-6.                       → You are an AI Agent.
/// You are an AI agent created by Example Corp.                                  → You are an AI Agent.
/// You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team … → You are an AI Agent.
/// ```
/// Defense rules against false positives:
/// 1. Must have both identity noun and attribution declaration: 'You are a helpful assistant.', 'You are Claude Code, Anthropic's
///    official CLI for Claude.', 'You are JeikCode AI coding Agent by Jeik.' (no attribution verb) are preserved intact;
/// 2. Attribution subject bounded by punctuation ('.' ',' newline) and must never cross XML tags - structurally eliminating
///    cross-line consumption bugs while preserving Zero Delimiter Touch;
///    ensuring subsequent prompt instructions remain completely intact;
/// 3. Replaces only the declaration sentence itself: content following attribution (including comma-continued instructions) is verbatim preserved;
/// 4. Transition window <= 8 chars: bridge text between noun and verb must be very short to prevent clause-crossing false positives;
///    (e.g., '... agent, and the config was created by X' in body text must not be normalized).
pub(crate) static RE_IDENTITY_DECLARATION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        // A. Identity opening (word boundary required to avoid matching inside phrases)
        r"(?i:\b(?:you\s+are|you're|i\s+am|i'm)\b[\s,，:：]*)",
        // B. Identity noun phrase: optional article + <=8 modifiers + optional AI + identity noun
        r"(?i:(?:an?\s+)?(?:[\w'\-]+[\s,，]+){0,8}(?:ai[\s,，]+)?(?:agent\b|assistant\b|ai\b))",
        // C. Attribution / version declaration (verb + preposition, or based on)
        //    Transition window permits comma (covering 'You are a Claude agent, built on Anthropic\'s ...'),
        //    but forbids crossing sentence ends, newlines, and XML tags, with max length of 8 characters.
        //    Narrowing the window prevents accidentally deleting cross-clause sentences.
        r"(?i:[^.!?。！？\r\n<>]{0,8}?\b(?:created|built|made|developed|designed|trained|powered|maintained|published|released|provided)\s+(?:by|on|upon|at)\s+",
        r"|[^.!?。！？\r\n<>]{0,8}?\bbased\s+on\s+)",
        // D. Attribution subject: matches up to punctuation boundary
        r"[^.,，。\r\n<>]{1,80}",
        // E. Trailing punctuation, used to determine replacement punctuation (comma-continued -> preserve comma)
        r"([.,，。]?)"
    ))
    .unwrap()
});

/// Fast pre-filter markers for identity declaration (matches any before running full regex on large prompts)
pub(crate) const IDENTITY_FAST_MARKERS: [&str; 13] = [
    "created by",
    "built by",
    "built on",
    "made by",
    "developed by",
    "designed by",
    "trained by",
    "powered by",
    "maintained by",
    "published by",
    "released by",
    "provided by",
    "based on",
];

/// Claude Agent SDK injected standalone identity block (exact match)
pub(crate) const CLAUDE_AGENT_SDK_IDENTITY: &str =
    "You are a Claude agent, built on Anthropic's Claude Agent SDK.";
/// Normalization target: Claude Code CLI identity. Deliberately preserved as mapping to known client identity -
/// upstream is more stable with this identity.
pub(crate) const CLAUDE_CODE_CLI_IDENTITY: &str =
    "You are Claude Code, Anthropic's official CLI for Claude.";

/// Claude Desktop injected single-line billing metadata prefix (issue #3452: causes upstream 429 when combined with many tools).
/// Model-agnostic: any client-injected risky metadata must be cleaned.
pub(crate) const BILLING_METADATA_PREFIX: &str = "x-anthropic-billing-header:";

/// Pipeline system prompt boundary markers (stripped if mixed in, avoiding upstream pollution)
pub(crate) const SYSTEM_PROMPT_END_MARKERS: [&str; 2] =
    ["--- [SYSTEM_PROMPT_END] ---", "[SYSTEM_PROMPT_END]"];

/// Pipeline official identity text (stripped if mixed in, avoiding duplicate identity declarations)
pub(crate) const SELF_IDENTITY_BLOCK: &str = "You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team working on Advanced Agentic Coding.\nYou are pair programming with a USER to solve their coding task. The task may require creating a new codebase, modifying or debugging an existing codebase, or simply answering a question.\n**Absolute paths only**\n**Proactiveness**";
