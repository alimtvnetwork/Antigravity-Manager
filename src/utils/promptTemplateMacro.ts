/**
 * Rich Prompt Template Macro Engine
 * Interpolates contextual dynamic tokens into prompt templates and nudge messages.
 */

export interface PromptMacroContext {
  workspace?: string;
  account?: string;
  branch?: string;
  timestamp?: string;
  promptId?: string;
  round?: number;
  total?: number;
  customVars?: Record<string, string>;
}

export function interpolateTemplateMacros(template: string, ctx: PromptMacroContext): string {
  if (!template) return '';

  const now = ctx.timestamp || new Date().toISOString();
  const workspace = ctx.workspace || 'default-workspace';
  const account = ctx.account || 'default-account';
  const branch = ctx.branch || 'main';
  const promptId = ctx.promptId || 'P000';
  const round = ctx.round !== undefined ? String(ctx.round) : '1';
  const total = ctx.total !== undefined ? String(ctx.total) : '1';

  let result = template
    .replace(/\{\{\s*workspace\s*\}\}/gi, workspace)
    .replace(/\{\{\s*account\s*\}\}/gi, account)
    .replace(/\{\{\s*branch\s*\}\}/gi, branch)
    .replace(/\{\{\s*timestamp\s*\}\}/gi, now)
    .replace(/\{\{\s*prompt_id\s*\}\}/gi, promptId)
    .replace(/\{\{\s*round\s*\}\}/gi, round)
    .replace(/\{\{\s*total\s*\}\}/gi, total);

  if (ctx.customVars) {
    for (const [k, v] of Object.entries(ctx.customVars)) {
      const reg = new RegExp(`\\{\\{\\s*${k}\\s*\\}\\}`, 'gi');
      result = result.replace(reg, v);
    }
  }

  return result;
}
