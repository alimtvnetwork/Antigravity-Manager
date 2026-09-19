import type { CapturedError } from '../stores/error-store';
import versionData from '../../version.json';

interface SuggestedFixMap {
  [errorCode: string]: string[];
}

const rawVersion = versionData.version || versionData.Version || '4.28.0';
const formattedVersion = rawVersion.startsWith('v') ? rawVersion : `v${rawVersion}`;

const APP_INFO = {
  name: 'AGM by Alim',
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
