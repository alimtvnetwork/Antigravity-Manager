/**
 * Conversation Turn Token Counter Estimator
 * Heuristic token estimator for user prompts, code snippets, and multimodal image attachments.
 */

export interface TokenEstimation {
  estimatedTokens: number;
  charCount: number;
  wordCount: number;
  imageTokens: number;
  estimatedCostUsd: number;
}

export function estimatePromptTokens(text: string, imageCount = 0): TokenEstimation {
  const clean = text || '';
  const charCount = clean.length;
  const words = clean.trim().split(/\s+/).filter(Boolean);
  const wordCount = words.length;

  // Split code blocks vs standard text
  let codeChars = 0;
  let textChars = 0;

  const codeBlockRegex = /```[\s\S]*?```/g;
  let match: RegExpExecArray | null;
  let lastIndex = 0;

  while ((match = codeBlockRegex.exec(clean)) !== null) {
    textChars += match.index - lastIndex;
    codeChars += match[0].length;
    lastIndex = match.index + match[0].length;
  }
  textChars += clean.length - lastIndex;

  // Heuristic: ~4 chars per token in prose, ~3.2 chars per token in code
  const textTokens = Math.ceil(textChars / 4.0);
  const codeTokens = Math.ceil(codeChars / 3.2);

  // Vision token estimation: ~258 tokens per standard 1024x1024 tile
  const imageTokens = imageCount * 258;

  const totalTokens = textTokens + codeTokens + imageTokens;

  // Gemini Flash pricing baseline ($0.075 / 1M tokens)
  const estimatedCostUsd = (totalTokens / 1_000_000.0) * 0.075;

  return {
    estimatedTokens: totalTokens,
    charCount,
    wordCount,
    imageTokens,
    estimatedCostUsd: Math.round(estimatedCostUsd * 100_000) / 100_000,
  };
}
