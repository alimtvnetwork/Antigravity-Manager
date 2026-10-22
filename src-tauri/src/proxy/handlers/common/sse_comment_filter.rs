//! SSE Comment & Event Stream Stripper.
//! Filters out internal telemetry comments (`: ...\n`) while preserving required `data:` and `event:` blocks.

/// Strips lines beginning with `:` (SSE comment lines) from raw SSE stream chunks.
pub fn filter_sse_comments(chunk: &[u8]) -> Vec<u8> {
    if !chunk.contains(&b':') {
        return chunk.to_vec();
    }

    let text = match std::str::from_utf8(chunk) {
        Ok(s) => s,
        Err(_) => return chunk.to_vec(),
    };

    let mut filtered_lines = Vec::new();
    for line in text.split('\n') {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(':') {
            filtered_lines.push(line);
        }
    }

    filtered_lines.join("\n").into_bytes()
}

/// Evaluates if an SSE chunk contains comment frames.
pub fn contains_sse_comment(chunk: &[u8]) -> bool {
    if let Ok(text) = std::str::from_utf8(chunk) {
        text.split('\n').any(|l| l.trim_start().starts_with(':'))
    } else {
        false
    }
}
