import type { CapturedError } from '../stores/error-store';
import versionData from '../../version.json';

interface SuggestedFixMap {
  [errorCode: string]: string[];
}

const rawVersion = versionData.version || versionData.Version || '4.29.0';
const formattedVersion = rawVersion.startsWith('v') ? rawVersion : `v${rawVersion}`;

const APP_INFO = {
  name: 'Agm Tool By Alim',
  version: formattedVersion,
};


const SUGGESTED_FIXES: SuggestedFixMap = {
  E1001: [
    'Check local network connectivity and firewall settings',
    'Verify proxy service is started and listening',
    'Check if proxy target upstream host is reachable',
  ],
  E1002: [
    'Network request timed out to upstream server',
    'Check upstream latency or verify VPN / proxy tunnel status',
    'Retry request with longer timeout',
  ],
  E1003: [
    'Rate limit exceeded (HTTP 429) on active model/account',
    'Allow cooldown timer to elapse before retrying',
    'Switch active instance to an alternate account profile',
  ],
  E2001: [
    'OAuth token expired or authorization code rejected',
    'Re-authenticate account via Google OAuth login flow',
    'Verify OAuth client credentials in settings',
  ],
  E3001: [
    'SQLite database error or file lock contention',
    'Verify read/write permissions on application data directory',
    'Check disk storage space on the system drive',
  ],
  E4001: [
    'Filesystem I/O error occurred during file operation',
    'Ensure target directory exists and is writable',
    'Verify antivirus is not locking file descriptors',
  ],
  E5001: [
    'Configuration validation error in settings payload',
    'Review settings for syntax or missing required fields',
    'Reset configuration to defaults if corrupted',
  ],
  E6001: [
    'Account management operation failed',
    'Verify account refresh token is valid in SQLite storage',
    'Run account sync or quota refresh',
  ],
  E7001: [
    'Proxy service or port binding conflict',
    'Check if local proxy port is already occupied by another process',
    'Restart proxy service from the Proxy tab',
  ],
  E8001: [
    'Tauri IPC bridge failure',
    'Ensure application window runtime is responsive',
    'Restart desktop application',
  ],
  E9001: [
    'Unexpected application error',
    'Inspect backend diagnostic logs and stack trace below',
    'Share this formatted report directly with AI assistance',
  ],
};

const DEFAULT_FIXES = [
  'Inspect diagnostic stack trace for root cause',
  'Review trigger context and recent interactions',
  'Share this diagnostic report directly with AI for automated resolution',
];

export function getSuggestedFixes(code: string): string[] {
  return SUGGESTED_FIXES[code] ?? DEFAULT_FIXES;
}

export function generateCompactReport(error: CapturedError): string {
  const sections: string[] = [];

  sections.push(`## Compact Error Report`);
  sections.push(`**App:** ${APP_INFO.name} ${APP_INFO.version}`);
  sections.push(`**Code:** ${error.code}`);
  sections.push(`**Level:** ${error.level}`);
  sections.push(`**Timestamp:** ${error.createdAt}`);

  if (error.route) {
    sections.push(`### Page\n\`${error.route}\``);
  }

  if (error.uiClickPathArrow) {
    sections.push(`### User Interaction\n\`\`\`\n${error.uiClickPathArrow}\n\`\`\``);
  }

  const triggerLines = buildTriggerContextLines(error);
  if (triggerLines.length > 0) {
    sections.push(`### Trigger Context\n${triggerLines.join('\n\n')}`);
  }

  sections.push(`### Message\n${error.message}`);

  if (error.details) {
    sections.push(`### Details\n${error.details}`);
  }

  if (error.endpoint || error.method) {
    const statusPart = error.responseStatus ? `\n**Status:** ${error.responseStatus}` : '';
    sections.push(`### Request\n**${error.method || 'INVOKE'}** ${error.endpoint || 'unknown'}${statusPart}`);
  }

  const backendSection = buildBackendDiagnosticSection(error);
  if (backendSection) {
    sections.push(`### Backend Diagnostics\n\`\`\`\n${backendSection}\n\`\`\``);
  }

  if (error.stackTrace) {
    sections.push(`### Frontend Stack Trace\n\`\`\`\n${error.stackTrace.trim()}\n\`\`\``);
  }

  const contextJson = buildContextJson(error);
  if (contextJson) {
    sections.push(`### Context\n\`\`\`json\n${contextJson}\n\`\`\``);
  }

  const fixes = getSuggestedFixes(error.code);
  if (fixes.length > 0) {
    sections.push(`### Suggested Troubleshooting Steps\n${fixes.map((f) => `- ${f}`).join('\n')}`);
  }

  return sections.join('\n\n');
}

function buildTriggerContextLines(error: CapturedError): string[] {
  const lines: string[] = [];
  if (error.triggerComponent) {
    lines.push(`**Component:** ${error.triggerComponent}`);
  }
  if (error.triggerAction) {
    lines.push(`**Action:** ${error.triggerAction}`);
  }
  if (error.context?.source) {
    lines.push(`**Source:** ${error.context.source}`);
  }
  return lines;
}

function buildBackendDiagnosticSection(error: CapturedError): string | null {
  if (error.backendStackTrace) {
    return error.backendStackTrace.trim();
  }
  if (error.envelopeErrors?.Backend && error.envelopeErrors.Backend.length > 0) {
    return error.envelopeErrors.Backend.join('\n');
  }
  if (error.envelopeErrors?.BackendMessage) {
    return error.envelopeErrors.BackendMessage;
  }
  return null;
}

function buildContextJson(error: CapturedError): string | null {
  if (!error.context) {
    return null;
  }
  const keys = Object.keys(error.context).filter((k) => error.context?.[k] !== undefined);
  if (keys.length === 0) {
    return null;
  }
  return JSON.stringify(error.context, null, 2);
}

export function generateJsonReport(error: CapturedError): string {
  return JSON.stringify(error, null, 2);
}

export function generateAllErrorsMarkdownReport(errors: CapturedError[]): string {
  const sections: string[] = [];
  sections.push(`# Error Manager Diagnostics History Report`);
  sections.push(`**Application:** ${APP_INFO.name} (${APP_INFO.version})`);
  sections.push(`**Generated At:** ${new Date().toLocaleString()}`);
  sections.push(`**Total Captured Errors:** ${errors.length}`);
  sections.push(`\n---\n`);

  if (errors.length === 0) {
    sections.push(`*No errors currently recorded in history.*`);
    return sections.join('\n\n');
  }

  errors.forEach((err, idx) => {
    sections.push(`## [Error #${idx + 1}] ${err.code} — ${err.message}`);
    sections.push(`- **Timestamp:** ${err.createdAt}`);
    sections.push(`- **Severity Level:** ${err.level}`);
    if (err.endpoint) {
      sections.push(`- **Endpoint:** \`${err.method || 'INVOKE'}\` ${err.endpoint}`);
    }
    if (err.route) {
      sections.push(`- **Route / Page:** ${err.route}`);
    }
    if (err.details) {
      sections.push(`- **Details:** ${err.details}`);
    }
    if (err.triggerAction || err.triggerComponent) {
      sections.push(`- **Trigger:** ${err.triggerComponent || ''} ${err.triggerAction ? `(${err.triggerAction})` : ''}`);
    }

    const backendSection = buildBackendDiagnosticSection(err);
    if (backendSection) {
      sections.push(`### Backend Diagnostics\n\`\`\`\n${backendSection}\n\`\`\``);
    }
    if (err.stackTrace) {
      sections.push(`### Stack Trace\n\`\`\`\n${err.stackTrace.trim()}\n\`\`\``);
    }
    const fixes = getSuggestedFixes(err.code);
    if (fixes.length > 0) {
      sections.push(`### Suggested Troubleshooting Steps\n${fixes.map((f) => `- ${f}`).join('\n')}`);
    }
    sections.push(`\n---\n`);
  });

  return sections.join('\n\n');
}

export function generateComprehensiveAllDataReport(error: CapturedError): string {
  const sections: string[] = [];

  sections.push(`# Comprehensive Error Diagnostic Report`);
  sections.push(`**Application:** ${APP_INFO.name} (${APP_INFO.version})`);
  sections.push(`**Error ID:** ${error.id}`);
  sections.push(`**Code:** ${error.code}`);
  sections.push(`**Severity Level:** ${error.level.toUpperCase()}`);
  sections.push(`**Captured At:** ${error.createdAt}`);
  if (error.requestedAt) {
    sections.push(`**Requested At:** ${error.requestedAt}`);
  }

  sections.push(`\n---\n`);
  sections.push(`## 1. Error Message & Details`);
  sections.push(`**Message:**\n${error.message}`);
  if (error.details) {
    sections.push(`**Details:**\n${error.details}`);
  }

  sections.push(`\n## 2. Trigger & Location`);
  if (error.route) {
    sections.push(`- **Route / Page:** \`${error.route}\``);
  }
  if (error.routeComponent) {
    sections.push(`- **Route Component:** \`${error.routeComponent}\``);
  }
  if (error.triggerComponent) {
    sections.push(`- **Trigger Component:** \`${error.triggerComponent}\``);
  }
  if (error.triggerAction) {
    sections.push(`- **Trigger Action:** \`${error.triggerAction}\``);
  }
  if (error.context?.source) {
    sections.push(`- **Source:** \`${String(error.context.source)}\``);
  }

  if (error.endpoint || error.method) {
    sections.push(`\n## 3. Network / IPC Request`);
    sections.push(`- **Method:** \`${error.method || 'INVOKE'}\``);
    sections.push(`- **Endpoint:** \`${error.endpoint || 'N/A'}\``);
    if (error.responseStatus) {
      sections.push(`- **Response Status:** ${error.responseStatus}`);
    }
    if (error.requestBody) {
      sections.push(`- **Request Body:**\n\`\`\`json\n${error.requestBody}\n\`\`\``);
    }
  }

  const hasClicks = Boolean(error.uiClickPath && error.uiClickPath.length > 0);
  if (error.uiClickPathArrow || hasClicks) {
    sections.push(`\n## 4. User Interaction Flow`);
    if (error.uiClickPathArrow) {
      sections.push(`\`\`\`\n${error.uiClickPathArrow}\n\`\`\``);
    }
    if (hasClicks && error.uiClickPath) {
      const clickLines = error.uiClickPath.map(
        (c, idx) =>
          `${idx + 1}. [${new Date(c.timestamp).toLocaleTimeString()}] ${c.action} on \`${c.element}\`${c.text ? ` ("${c.text}")` : ''}${c.componentName ? ` in <${c.componentName}>` : ''}`
      );
      sections.push(clickLines.join('\n'));
    }
  }

  const backendDiag = buildBackendDiagnosticSection(error);
  if (backendDiag) {
    sections.push(`\n## 5. Backend Diagnostics & Rust Stack`);
    sections.push(`\`\`\`\n${backendDiag}\n\`\`\``);
  }

  if (error.envelopeErrors) {
    sections.push(`\n## 6. Structured Envelope Errors`);
    if (error.envelopeErrors.BackendMessage) {
      sections.push(`- **Backend Message:** ${error.envelopeErrors.BackendMessage}`);
    }
    const hasBackendEnvelope = Boolean(error.envelopeErrors.Backend && error.envelopeErrors.Backend.length > 0);
    if (hasBackendEnvelope && error.envelopeErrors.Backend) {
      sections.push(`- **Backend Errors:**\n\`\`\`\n${error.envelopeErrors.Backend.join('\n')}\n\`\`\``);
    }
    const hasFrontendEnvelope = Boolean(error.envelopeErrors.Frontend && error.envelopeErrors.Frontend.length > 0);
    if (hasFrontendEnvelope && error.envelopeErrors.Frontend) {
      sections.push(`- **Frontend Errors:**\n\`\`\`\n${error.envelopeErrors.Frontend.join('\n')}\n\`\`\``);
    }
    const hasDelegatedEnvelope = Boolean(error.envelopeErrors.DelegatedServiceErrorStack && error.envelopeErrors.DelegatedServiceErrorStack.length > 0);
    if (hasDelegatedEnvelope && error.envelopeErrors.DelegatedServiceErrorStack) {
      sections.push(`- **Delegated Service Errors:**\n\`\`\`\n${error.envelopeErrors.DelegatedServiceErrorStack.join('\n')}\n\`\`\``);
    }
  }

  const hasFrames = Boolean(error.parsedFrames && error.parsedFrames.length > 0);
  if (error.stackTrace || hasFrames) {
    sections.push(`\n## 7. Frontend Stack Trace`);
    if (hasFrames && error.parsedFrames) {
      const frameLines = error.parsedFrames.map(
        (f) => `- \`${f.function}\` at ${f.file}:${f.line}${f.column ? `:${f.column}` : ''} ${f.isInternal ? '(internal)' : ''}`
      );
      sections.push(frameLines.join('\n'));
    }
    if (error.stackTrace) {
      sections.push(`\n**Raw Stack:**\n\`\`\`\n${error.stackTrace.trim()}\n\`\`\``);
    }
  }

  const hasInvocation = Boolean(error.invocationChain && error.invocationChain.length > 0);
  if (hasInvocation && error.invocationChain) {
    sections.push(`\n## 8. Invocation Chain`);
    sections.push(error.invocationChain.map((fn, i) => `${i + 1}. \`${fn}\``).join('\n'));
  }

  const contextJson = buildContextJson(error);
  if (contextJson) {
    sections.push(`\n## 9. Full Context Payload`);
    sections.push(`\`\`\`json\n${contextJson}\n\`\`\``);
  }

  sections.push(`\n## 10. Raw Error Object (JSON)`);
  sections.push(`\`\`\`json\n${JSON.stringify(error, null, 2)}\n\`\`\``);

  const fixes = getSuggestedFixes(error.code);
  if (fixes.length > 0) {
    sections.push(`\n## 11. Recommended Fixes`);
    sections.push(fixes.map((f) => `- ${f}`).join('\n'));
  }

  return sections.join('\n\n');
}


