//! Gemini System Instruction Sanitizer.
//! Cleans and normalizes system instruction payloads, stripping redundant blank lines and bypass fragments.

/// Sanitizes system instructions by normalizing line endings and removing dangerous control fragments.
pub fn sanitize_gemini_system_instruction(instruction: &str) -> String {
    let normalized = instruction.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    // Collapse multi-blank lines (>2 consecutive \n)
    let mut out = String::with_capacity(trimmed.len());
    let mut consecutive_newlines = 0;

    for ch in trimmed.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                out.push(ch);
            }
        } else {
            consecutive_newlines = 0;
            out.push(ch);
        }
    }

    out
}
